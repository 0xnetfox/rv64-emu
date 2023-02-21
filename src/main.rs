mod mmu;

use elf_parser::parser::ElfParser;
use crate::mmu::Mmu;


struct Emu {
    memory: Mmu
}

fn main() {
    let contents = std::fs::read("./out/rv64i-test").unwrap();
    let _ = ElfParser::parse(contents);

    let mmu = Mmu::new(2 * 1024);
}
