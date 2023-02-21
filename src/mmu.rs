#[allow(dead_code)]
const PERM_READ:  u8 = 1;
#[allow(dead_code)]
const PERM_WRITE: u8 = 2;
#[allow(dead_code)]
const PERM_RAW:   u8 = 4;
#[allow(dead_code)]
const PERM_EXEC:  u8 = 8;

/// Represents a address on this `Mmu` implementation
pub struct VirtAddr(usize);

#[derive(Debug, Copy, Clone)]
pub struct Perm(pub u8);

#[derive(Debug)]
pub struct Mmu {
    pub memory: Vec<u8>,
    pub permissions: Vec<Perm>
}

#[allow(dead_code)]
impl Mmu {
    pub fn new(size: usize) -> Self {
        Mmu {
            memory: vec![0u8; size],
            permissions: vec![Perm(PERM_RAW); size]
        }
    }

    pub fn read_into(&mut self, buf: &mut [u8], off: VirtAddr) -> Result<(), ()> {
        if self.permissions[off.0..off.0 + buf.len()]
            .iter()
            .all(|b| (b.0 & PERM_READ) == 0) {
            return Err(())
        }

        buf.copy_from_slice(&self.memory[off.0..off.0 + buf.len()]);
        Ok(())
    }

    pub fn write_from(&mut self, buf: &[u8], off: VirtAddr) -> Result<(), ()> {
        if self.permissions[off.0..off.0 + buf.len()]
            .iter()
            .all(|b| (b.0 & PERM_WRITE) != 0) {
            return Err(())
        }

        self.memory[off.0..off.0 + buf.len()].copy_from_slice(buf);
        Ok(())
    }
}