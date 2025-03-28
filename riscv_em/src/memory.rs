pub enum Csr {
    Mtime,
    Mtimecmp,
}

#[derive(Debug)]
pub struct Memory {
    base_addr: u32,
    data: [u8; super::RAM_SIZE],

    mtime: u32,
    mtimeh: u32,
    mtimecmp: u32,
    mtimecmph: u32,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            base_addr: super::RAM_OFFSET,
            data: [0; super::RAM_SIZE],

            mtime: 0,
            mtimeh: 0,
            mtimecmp: 0,
            mtimecmph: 0,
        }
    }
}

impl Memory {
    pub fn get_word(&self, addr: u32) -> u32 {
        if addr < self.base_addr {
            return match addr {
                0x10000005 => 0, // TODO: UART
                0x10000000 => 0, // TODO: UART
                0x1100bffc => self.mtimeh,
                0x1100bff8 => self.mtime,
                _ => 0, // TODO: read error,
            };
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
            match addr {
                0x10000000 => {} // TODO: UART;
                0x11100000 => {} // TODO: SYSCON;
                0x11004004 => {
                    self.mtimecmph = data;
                }
                0x11004000 => {
                    self.mtimecmp = data;
                }
                _ => {} // TODO: write error
            };
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

    pub fn csr_read(&self, csr: Csr) -> u64 {
        match csr {
            Csr::Mtime => {
                let mtimel = self.get_word(0x1100bff8);
                let mtimeh = self.get_word(0x1100bffc);
                (mtimeh as u64) << 32 + mtimel as u64
            }
            Csr::Mtimecmp => {
                let mtimecmpl = self.get_word(0x11004000);
                let mtimecmph = self.get_word(0x11004004);
                (mtimecmph as u64) << 32 + mtimecmpl as u64
            }
        }
    }
    pub fn csr_write(&mut self, csr: Csr, val: u64) -> () {
        match csr {
            Csr::Mtime => {
                self.insert_word(0x1100bff8, val as u32);
                self.insert_word(0x1100bffc, (val >> 32) as u32);
            }
            Csr::Mtimecmp => {
                self.insert_word(0x11004000, val as u32);
                self.insert_word(0x11004004, (val >> 32) as u32);
            }
        }
    }
}
