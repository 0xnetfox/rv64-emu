extern crate core;

mod mmu;

use elf_parser::parser::ElfParser;
use crate::mmu::{Mmu};


struct Emu {
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

#[cfg(test)]
mod tests {
    use crate::mmu::{PERM_RAW, PERM_READ, PERM_WRITE, VirtAddr};
    use super::*;

    fn setup() -> Emu {
        Emu::new(2 * 1024)
    }

    #[test]
    fn write_from_base_state() {
        let mut emu = setup();

        emu.memory.write_from(b"hello", VirtAddr(0x0)).unwrap();
        assert_eq!(emu.memory.memory[0..5], [b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn raw_perms_after_write() {
        let mut emu = setup();

        assert!(emu.memory.permissions[0..5].iter().all(|b| b.0 == PERM_RAW | PERM_WRITE));
        emu.memory.write_from(b"hello", VirtAddr(0x0)).unwrap();
        assert!(emu.memory.permissions[0..5].iter().all(|b| b.0 == PERM_READ | PERM_WRITE));
    }

    #[test]
    fn read_from_raw_memory() {
        let mut emu = setup();

        let mut buff = [0u8; 5];
        match emu.memory.read_into(&mut buff, VirtAddr(0x0)) {
            Ok(_) => {assert!(false, "match should fail")}
            Err(_) => {assert!(true)}
        }
    }
}