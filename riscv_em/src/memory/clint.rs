use crate::core::{Core, csr};

pub struct Clint {
    base: usize,
    length: usize,

    pub mtime: u32,
    pub mtimeh: u32,

    mtimecmp: u32,
    mtimecmph: u32,
}

impl Default for Clint {
    fn default() -> Self {
        Clint {
            base: 0x2000000,
            length: 0xc0000,
            mtime: 0,
            mtimeh: 0,
            mtimecmp: 0,
            mtimecmph: 0,
        }
    }
}

impl Clint {
    pub fn claim(&self, addr: u32) -> bool {
        if addr as usize >= self.base && (addr as usize) < self.base + self.length {
            return true;
        }
        return false;
    }

    pub fn tick(&mut self, core: &mut Core) {
        let mtime = ((self.mtimeh as u64) << 32) + (self.mtime as u64);
        let mtimecmp = ((self.mtimecmph as u64) << 32) + (self.mtimecmp as u64);
        let mut mip = csr::read(csr::Csr::mip, core);
        if mtime > mtimecmp {
            mip |= 1 << 7;
            core.wfi = false;
        } else {
            mip &= !(1 << 7);
        }
        csr::write(csr::Csr::mip, mip, core);
    }

    pub fn read(&mut self, addr: u32) -> u32 {
        let addr = addr as usize - self.base;

        match addr {
            0xbffc => self.mtimeh,
            0xbff8 => self.mtime,
            0x4004 => self.mtimecmph,
            0x4000 => self.mtimecmp,
            _ => 0,
        }
    }
    pub fn write(&mut self, addr: u32, data: u32) {
        let addr = addr as usize - self.base;

        match addr {
            0xbffc => self.mtimeh = data,
            0xbff8 => self.mtime = data,
            0x4004 => self.mtimecmph = data,
            0x4000 => self.mtimecmp = data,
            _ => {}
        };
    }
}
