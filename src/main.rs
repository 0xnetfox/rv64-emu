extern crate core;

mod mmu;

use elf_parser::parser::ElfParser;
use crate::mmu::{Mmu};


pub struct Emu {
    pub memory: Mmu
}

impl Emu {
    pub fn new(mem_size: usize) -> Self {
        Emu {
            memory: Mmu::new(mem_size)
        }
    }
}

fn main() {
    let contents = std::fs::read("./out/rv64i-test").unwrap();
    let _ = ElfParser::parse(contents);

    let _ = Emu::new(2 * 1024);
}