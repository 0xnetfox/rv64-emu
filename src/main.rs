extern crate core;

mod mmu;
mod emulator;
mod processor;

use elf_parser::parser::ElfParser;
use crate::emulator::Emu;

fn main() {
    let contents = std::fs::read("./out/rv64i-test").unwrap();
    let elf = ElfParser::parse(contents).unwrap();
    let mut emu = Emu::new(2 * 1024 * 1024, elf.headers.entry.0);

    emu.load_sections(elf.program_headers);
    emu.run().unwrap();
}
