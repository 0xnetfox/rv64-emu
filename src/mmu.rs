#[allow(dead_code)]
pub const PERM_EXEC: u8 = 0x1;
pub const PERM_WRITE: u8 = 0x2;
pub const PERM_READ: u8 = 0x4;
pub const PERM_RAW: u8 = 0x8;

/// Represents a address on this `Mmu` implementation
#[derive(Debug, Copy, Clone)]
pub struct VirtAddr(pub usize);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Perm(pub u8);

#[derive(Debug)]
pub struct Mmu {
    pub memory: Vec<u8>,
    pub permissions: Vec<Perm>,
}

#[allow(dead_code)]
impl Mmu {
    pub fn new(size: usize) -> Self {
        Mmu {
            memory: vec![0u8; size],
            permissions: vec![Perm(PERM_WRITE | PERM_RAW); size],
        }
    }

    pub fn read<const SIZE: usize>(&mut self, offset: VirtAddr) -> Result<[u8; SIZE], ()> {
	let mut buffer = [0u8; SIZE];
	self.read_into(&mut buffer, offset)?;
	Ok(buffer)
    }

    pub fn read_into(&mut self, buf: &mut [u8], off: VirtAddr) -> Result<(), ()> {
        let w_len = off.0 + buf.len();

        if self.permissions[off.0..w_len]
            .iter()
            .any(|b| (b.0 & PERM_READ) == 0)
        {
            return Err(());
        }

        buf.copy_from_slice(&self.memory[off.0..w_len]);
        Ok(())
    }

    pub fn write_from(&mut self, addr: VirtAddr, buf: &[u8]) -> Result<(), ()> {
        let w_len = addr.0 + buf.len();

        // check that all permissions are met and notify if there are `PERM_RAW` bytes
        // that need to be updated after the write
        let mut has_raw = false;
        if self.permissions[addr.0..w_len].iter().any(|b| {
            if b.0 & PERM_RAW != 0 {
                has_raw = true
            }

            (b.0 & PERM_WRITE) == 0
        }) {
            return Err(());
        }

        self.memory[addr.0..w_len].copy_from_slice(buf);

        // updates all the `PERM_RAW` bytes found in the section we copied
        if has_raw {
            self.permissions[addr.0..w_len]
                .iter_mut()
                .for_each(|b| *b = Perm((b.0 | PERM_READ) & !PERM_RAW))
        }

        Ok(())
    }

    pub fn set_perms(&mut self, addr: VirtAddr, size: usize, perm: Perm) -> Result<(), ()> {
        self.permissions
            .get_mut(addr.0..addr.0.checked_add(size).unwrap())
            .unwrap()
            .iter_mut()
            .for_each(|x| *x = perm);

        Ok(())
    }

    pub fn vprintln(&self, start: usize, end: usize, round: bool) -> Option<()> {
        let mut end = end;
        if round {
            end = (end + 0xf) & !0xf;
        }

        let memmap: Vec<(&u8, &Perm)> = self
            .memory
            .get(start..end)?
            .iter()
            .zip(self.permissions.get(start..end)?.iter())
            .collect();

        let chunk = 32;
        let mut base = VirtAddr(start);
        memmap.chunks(chunk).for_each(|byte_chunk| {
            // print the memory address reference
            print!("0x{:08x}:\t", base.0);

            for (byte, perm) in byte_chunk {
                let color = format!("\x1b[4{}m", perm.0 & 0x7f);
                print!("{} {:02x} \x1b[0m", color, byte);
            }
            println!();
            base.0 += chunk;
        });

        print!("\n");
        println!("\x1b[41m   \x1b[0m :: __E");
        println!("\x1b[42m   \x1b[0m :: _W_");
        println!("\x1b[43m   \x1b[0m :: _WE");
        println!("\x1b[44m   \x1b[0m :: R__");
        println!("\x1b[45m   \x1b[0m :: R_E");
        println!("\x1b[46m   \x1b[0m :: RW_");
        println!("\x1b[47m   \x1b[0m :: RWE\n");

        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Mmu {
        Mmu::new(2 * 1024)
    }

    #[test]
    fn write_from_base_state() {
        let mut mmu = setup();

        mmu.write_from(VirtAddr(0x0), b"hello").unwrap();
        assert_eq!(mmu.memory[0..5], [b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn raw_perms_after_write() {
        let mut mmu = setup();

        assert!(mmu.permissions[0..5]
            .iter()
            .all(|b| b.0 == PERM_RAW | PERM_WRITE));
        mmu.write_from(VirtAddr(0x0), b"hello").unwrap();
        assert!(mmu.permissions[0..5]
            .iter()
            .all(|b| b.0 == PERM_READ | PERM_WRITE));
    }

    #[test]
    fn read_from_raw_memory() {
        let mut mmu = setup();

        let mut buff = [0u8; 5];
        match mmu.read_into(&mut buff, VirtAddr(0x0)) {
            Ok(_) => {
                assert!(false, "match should fail")
            }
            Err(_) => {
                assert!(true)
            }
        }
    }
}
