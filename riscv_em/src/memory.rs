
 #[derive(Debug)]
pub struct Memory {
    base_addr: u32,
    data: [u8; super::RAM_SIZE],
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            base_addr: 0x80000000,
            data: [0; super::RAM_SIZE],
        }
    }
}

impl Memory {
    pub fn get_word(&self, addr: u32) -> u32 {
        if addr < self.base_addr {
            // TODO: CRS access
        }
        let mut address = (addr - self.base_addr) as usize;
        let d = self.data[address] as u32;
        address += 1;
        let c = self.data[address] as u32;
        address += 1;
        let b = self.data[address] as u32;
        address += 1;
        let a = self.data[address] as u32;
        (a << 24) + (b << 16) + (c << 8) + d
    }
    pub fn get_hword(&self, addr: u32) -> u16 {
        if addr < self.base_addr {
            // TODO: Error
        }
        let mut address = (addr - self.base_addr) as usize;
        let b = self.data[address] as u16;
        address += 1;
        let a = self.data[address] as u16;
        (a << 8) + b
    }
    pub fn get_byte(&self, addr: u32) -> u8 {
        if addr < self.base_addr {
            // TODO: Error
        }
        let address = (addr - self.base_addr) as usize;
        self.data[address]
    }
    
    pub fn insert_word(&mut self, addr: u32, data: u32) {
        if addr < self.base_addr {
            // TODO: CSR access
        }
        let address = (addr - self.base_addr) as usize;
        let mut mask: u32 = (1 << 8) - 1;
        let d: u8 = (data & mask).try_into().unwrap();
        mask <<= 8;
        let c: u8 = ((data & mask) >> 8).try_into().unwrap();
        mask <<= 8;
        let b: u8 = ((data & mask) >> 16).try_into().unwrap();
        mask <<= 8;
        let a: u8 = ((data & mask) >> 24).try_into().unwrap();
        self.data[address] = d;
        self.data[address + 1] = c;
        self.data[address + 2] = b;
        self.data[address + 3] = a;
    }
    pub fn insert_hword(&mut self, addr: u32, data: u16) {
        if addr < self.base_addr {
            // TODO: Error
        }
        let address = (addr - self.base_addr) as usize;
        let mut mask: u16 = (2 << 8) - 1;
        let d: u8 = (data & mask).try_into().unwrap();
        mask <<= 8;
        let c: u8 = ((data & mask) >> 8).try_into().unwrap();
        self.data[address] = d;
        self.data[address + 1] = c;
    }
    pub fn insert_byte(&mut self, addr: u32, data: u8) {
        if addr < self.base_addr {
            // TODO: Error
        }
        let address = (addr - self.base_addr) as usize;
        self.data[address] = data;
    }
}
