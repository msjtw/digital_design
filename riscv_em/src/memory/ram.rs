use crate::{RAM_OFFSET, RAM_SIZE};

// #[derive(Debug)]
pub struct RAM {
    base: u32,
    length: u32,

    data: Vec<u8>,
}

impl Default for RAM {
    fn default() -> Self {
        RAM {
            base: RAM_OFFSET,
            length: RAM_SIZE,
            data: vec![0; RAM_SIZE as usize],
        }
    }
}

impl RAM {
    pub fn claim(&self, addr: u32) -> bool {
        if addr >= self.base && addr < self.base + self.length {
            return true;
        }
        return false;
    }

    pub fn load_word(&self, addr: u32) -> u32 {
        let address = (addr - self.base) as usize;
        if address > RAM_SIZE as usize {
            println!("0x{:08x}", addr);
        }
        let d = self.data[address] as u32;
        let c = self.data[address + 1] as u32;
        let b = self.data[address + 2] as u32;
        let a = self.data[address + 3] as u32;
        (a << 24) + (b << 16) + (c << 8) + d
    }

    pub fn load_hword(&self, addr: u32) -> u16 {
        let address = (addr - self.base) as usize;
        let b = self.data[address] as u16;
        let a = self.data[address + 1] as u16;
        (a << 8) + b
    }

    pub fn load_byte(&self, addr: u32) -> u8 {
        let address = (addr - self.base) as usize;
        self.data[address]
    }

    pub fn store_word(&mut self, addr: u32, data: u32) {
        let address = (addr - self.base) as usize;
        let mask: u32 = (1 << 8) - 1;
        let d: u8 = (data & mask) as u8;
        let c: u8 = ((data & mask << 8) >> 8) as u8;
        let b: u8 = ((data & mask << 16) >> 16) as u8;
        let a: u8 = ((data & mask << 24) >> 24) as u8;
        self.data[address] = d;
        self.data[address + 1] = c;
        self.data[address + 2] = b;
        self.data[address + 3] = a;
    }
    pub fn store_hword(&mut self, addr: u32, data: u16) {
        let address = (addr - self.base) as usize;
        let mask: u16 = (2 << 8) - 1;
        let d: u8 = (data & mask) as u8;
        let c: u8 = ((data & mask << 8) >> 8) as u8;
        self.data[address] = d;
        self.data[address + 1] = c;
    }
    pub fn store_byte(&mut self, addr: u32, data: u8) {
        let address = (addr - self.base) as usize;
        self.data[address] = data;
    }
}
