#[allow(dead_code)]
pub const PERM_READ: u8 = 1;
#[allow(dead_code)]
pub const PERM_WRITE: u8 = 2;
#[allow(dead_code)]
pub const PERM_RAW: u8 = 4;
#[allow(dead_code)]
pub const PERM_EXEC: u8 = 8;

/// Represents a address on this `Mmu` implementation
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

    pub fn write_from(&mut self, buf: &[u8], off: VirtAddr) -> Result<(), ()> {
        let w_len = off.0 + buf.len();

        // check that all permissions are met and notify if there are `PERM_RAW` bytes
        // that need to be updated after the write
        let mut has_raw = false;
        if self.permissions[off.0..w_len].iter().any(|b| {
            if b.0 & PERM_RAW != 0 {
                has_raw = true
            }

            (b.0 & PERM_WRITE) == 0
        }) {
            return Err(());
        }

        self.memory[off.0..w_len].copy_from_slice(buf);

        // updates all the `PERM_RAW` bytes found in the section we copied
        if has_raw {
            self.permissions[off.0..w_len]
                .iter_mut()
                .for_each(|b| *b = Perm((b.0 | PERM_READ) & !PERM_RAW))
        }

        Ok(())
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

        mmu.write_from(b"hello", VirtAddr(0x0)).unwrap();
        assert_eq!(mmu.memory[0..5], [b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn raw_perms_after_write() {
        let mut mmu = setup();

        assert!(mmu.permissions[0..5]
            .iter()
            .all(|b| b.0 == PERM_RAW | PERM_WRITE));
        mmu.write_from(b"hello", VirtAddr(0x0)).unwrap();
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
