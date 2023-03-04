use crate::processor::{Processor, Register};
use crate::mmu::{Mmu, Perm, VirtAddr};
use elf_parser::elf::phdr::{Elf64PHdr, PType, PTypeData, PF_EXEC, PF_READ, PF_WRITE};

pub struct Emu {
    pub memory: Mmu,
    pub processor: Processor,
}

impl Emu {
    pub fn new(mem_size: usize, entrypoint: u64) -> Self {
        Emu {
            memory:     Mmu::new(mem_size),
	    processor:  Processor::new(entrypoint),
        }
    }

    pub fn load_sections(&mut self, sections: Vec<Elf64PHdr>) {
	sections
	    .iter()
	    .filter(|s| s.p_type == PType::PtLoad)
	    .for_each(|ls| self.load_section(ls));
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

    pub fn fetch(&self, addr: VirtAddr) -> u32 {
	u32::from_le_bytes(self.memory.read::<4>(addr).unwrap())
    }

    pub fn tick(&mut self) -> Result<(), ()> {
	let tick_pc = self.processor.reg(Register::Pc);
	let instr_w = self.fetch(VirtAddr(tick_pc as usize));

	let Some(instr) = Processor::decode(instr_w) else {
	    unimplemented!("instruction for word {:#04b} not implemented", instr_w);
	};

	let next_pc = (instr.operation)(&mut self.processor, instr_w, tick_pc);

	if tick_pc == next_pc {
	    self.processor.inc_pc();
	}

	Ok(())
    }

    pub fn run(&mut self) -> Result<(), ()> {
	loop {
	    self.tick()?;
	}
    }
}

