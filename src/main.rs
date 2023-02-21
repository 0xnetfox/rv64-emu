extern crate core;

mod mmu;

use crate::mmu::{Mmu, Perm, VirtAddr};
use elf_parser::elf::phdr::{Elf64PHdr, PType, PTypeData, PF_EXEC, PF_READ, PF_WRITE};
use elf_parser::parser::ElfParser;

pub struct Emu {
    pub memory: Mmu,
    pub entry_point: VirtAddr,
}

impl Emu {
    pub fn new(mem_size: usize, entry_point: u64) -> Self {
        Emu {
            memory: Mmu::new(mem_size),
            entry_point: VirtAddr(entry_point as usize),
        }
    }

    pub fn load_section(&mut self, section: &Elf64PHdr) {
        let perms = (section.flags & (PF_EXEC | PF_WRITE | PF_READ)) as u8;

        match &section.section {
            PTypeData::PtLoadData(bytes) => {
                self.memory
                    .write_from(VirtAddr(section.vaddr.0 as usize), bytes)
                    .unwrap();
                self.memory
                    .set_perms(VirtAddr(section.vaddr.0 as usize), bytes.len(), Perm(perms))
                    .unwrap();
            }
            _ => unreachable!(),
        }

        self.memory.vprintln(
            section.vaddr.0 as usize,
            (section.vaddr.0 + section.memsz) as usize,
            true,
        );
    }
}

fn main() {
    let contents = std::fs::read("./out/rv64i-test").unwrap();
    let elf = ElfParser::parse(contents).unwrap();
    let mut emu = Emu::new(2 * 1024 * 1024, elf.headers.entry.0);

    elf.program_headers
        .iter()
        .filter(|s| s.p_type == PType::PtLoad)
        .for_each(|ls| emu.load_section(ls));
}
