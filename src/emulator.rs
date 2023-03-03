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

    pub fn run(&mut self) -> Result<(), ()> {
	'next_instr: loop {
	    let pc = VirtAddr(self.processor.reg(Register::Pc) as usize);
	    let instr_w: u32 = u32::from_le_bytes(self.memory.read::<4>(pc)?);

	    let Some(instr) = Processor::decode_instruction(instr_w) else {
		unimplemented!("instruction for word {:#04b} not implemented", instr_w);
	    };

	    (instr.operation)(instr_w);

	    // TODO this will cause problems when jumping around, SO MAKE SURE TO CHANGE IT
	    self.processor.inc_pc();

	    println!("{:#04x}", pc.0);
	    println!("{:#04x}", instr_w);
	};

	Ok(())
    }
}

