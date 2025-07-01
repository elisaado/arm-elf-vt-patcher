use clap::Parser;
use clap::ArgAction;
use clap_num::maybe_hex;
use goblin::elf::{Elf, SectionHeader};
use object::{Architecture, Object, ObjectSymbol};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file: String,

    #[arg(short, long)]
    output_file: String,

    #[arg(short = 'a', long, value_parser=maybe_hex::<u32>, help = "Address of the interrupt to add (prefix with 0x for hex)")]
    interrupt_address: u32,

    #[arg(short, long, value_parser=maybe_hex::<u32>, help = "Address of the vector table (prefix with 0x for hex) (defaults to 0x0)", default_value_t = 0)]
    vector_table_offset: u32,

    #[arg(short, long, value_parser=maybe_hex::<u64>, help = "Which entry *in the vector table* to write to (defaults to the first zero entry in the interrupts section of the vector table)")]
    n_th_entry: Option<usize>,

    #[arg(short, long, help = "Do not automatically correct interrupt address to account for ARM-thumb mode")]
    do_not_correct: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let path = Path::new(&args.input_file);
    let new_path = Path::new(&args.output_file);

    let mut buffer = read_file(path)?;

    let elf = Elf::parse(&buffer).expect("Failed to parse ELF");
    let obj = object::File::parse(&*buffer).expect("Failed to parse ELF");

    if elf.is_64 {
        // because our words are hardcoded to be 4 u8's
        return Err("ELF is 64 bit... Not implemented...".into());
    }

    if !elf.little_endian {
        // because we use to/from_le functions
        return Err("ELF is little endian... Not implemented...".into());
    }

    if obj.architecture() != Architecture::Arm {
        // because we manually do +1 for thumb mode
        return Err("ELF is not ARM architecture... Not implemented...".into());
    }
    
    if args.input_file == args.output_file {
        return Err("Cannot use input file as output file!".into());
    }

    // there are 16 other things in the vector table before the interrupts start
    // each "thing" is 4 bytes wide, so 4 * 16
    let symbol_address: u64 = args.vector_table_offset.into();

    let section = find_section(&elf, symbol_address).expect("Failed to find section");

    println!(
        "Symbol is in section {} at file offset 0x{:x}",
        elf.shdr_strtab.get_at(section.sh_name).unwrap(),
        section.sh_offset
    );

    let file_offset = section.sh_offset + (symbol_address - section.sh_addr);

    let offset = file_offset as usize;

    let mut entry = args.n_th_entry.unwrap_or(0);
    if args.n_th_entry.is_none() {
        // we start at 16 because that's where the interrupts start
        for i in (16..) {
            if i >= section.sh_size {
                return Err(Box::from(
                    "No zero-addresses found in interrupt list to overwrite. Try specifying an n-th entry to patch manually",
                ));
            }

            let i = i as usize;
            let word = &buffer[offset + i * 4..offset + i * 4 + 4];

            let word = u32::from_le_bytes(word.try_into().unwrap());

            if word == 0x0 {
                entry = i;
                break;
            }
        }
    }
    
    let mut interrupt_address = args.interrupt_address;

    let interrupt_address_from_sym = obj.symbols().find_map(|sym| {
        let addr = sym.address();
        if (addr == args.interrupt_address as u64 || addr == (args.interrupt_address + 1) as u64)
            && sym.size() > 0
        {
            Some(addr as u32)
        } else {
            None
        }
    });

    if interrupt_address_from_sym.is_none() {
        println!(
            "Provided interrupt address was not found as a symbol or its symbol size was 0! Proceed with caution!"
        );
    }
    
    if !args.do_not_correct {
        interrupt_address = interrupt_address_from_sym
            .unwrap_or(args.interrupt_address);

        if args.interrupt_address != interrupt_address {
            println!("Automatically corrected interrupt address to account for ARM-thumb mode. Pass -d to disable correction.");
        }
    }
    
    println!(
        "Using interrupt address 0x{:x}",
        (interrupt_address)
    );
    

    buffer[offset + entry * 4..offset + entry * 4 + 4].copy_from_slice(&interrupt_address.to_le_bytes());

    if let Err(err) = write_file(new_path, &buffer) {
        println!("Failed to write output file: {err}");
        return Err(err.into());
    }

    println!("Patched bytes at file offset 0x{file_offset:x}");

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

fn write_file(path: &Path, buffer: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(buffer)?;

    file.flush()?;

    Ok(())
}

fn find_section<'a>(elf: &'a Elf, symbol_address: u64) -> Option<&'a SectionHeader> {
    elf.section_headers
        .iter()
        .find(|sh| symbol_address >= sh.sh_addr && symbol_address < sh.sh_addr + sh.sh_size)
}
