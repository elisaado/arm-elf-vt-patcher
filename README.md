# arm-elf-vt-patcher

<sub>putting the i in isr_vector</sub>

This is a tool for adding and removing addresses in the interrupt vector table of ARM ELF binaries. It was built to make firmware fuzzing with [Hoedur](https://github.com/fuzzware-fuzzer/hoedur) easier, as Hoedur can act as an interrupt-based fuzzer.

## Installation

The tool has been tested on Linux & MacOS. Windows support is not guaranteed.

### Prerequisites
- Rust & Cargo

### Installation steps
1. Clone the repository

   `git clone https://github.com/elisaado/arm-elf-vt-patcher`
3. Change directory into the project

   `cd arm-elf-vt-patcher`
5. Use cargo install to build & install the binary

   `cargo install .`
7. Enjoy!

## Usage

`elf-vt-patcher --help`

### Examples

Adding an interrupt, which has address 0x22230 to the vector table which has an offset of 0x19000

`elf-vt-patcher -i input.elf -o output.elf -a 0x22230 -v 0x19000`

Removing the 16th entry of the vector table (a.k.a. the first interrupt) by overwriting it the address 0x0. `-d` here is used to tell the program *not* to look up the symbol at the corresponding address (`0x0` in this case).

`elf-vt-patcher -i input.elf -o output.elf -a 0x0 -v 0x19000 -d -n 16`
