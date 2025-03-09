use std::collections::HashMap;

pub struct Memory {
    data: HashMap<u32, u8>,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            data: HashMap::new(),
        }
    }
}

impl Memory {
    pub fn new() -> Self {
        return Memory {
            ..Default::default()
        };
    }

    pub fn get_word(&self, mut address: u32) -> u32 {
        let d: u32 = self.data.get(&address).copied().unwrap_or(0).into();
        address += 1;
        let c: u32 = self.data.get(&address).copied().unwrap_or(0).into();
        address += 1;
        let b: u32 = self.data.get(&address).copied().unwrap_or(0).into();
        address += 1;
        let a: u32 = self.data.get(&address).copied().unwrap_or(0).into();
        (a << 24) + (b << 16) + (c << 8) + d
    }
    pub fn get_hword(&self, mut address: u32) -> u16 {
        let b: u16 = self.data.get(&address).copied().unwrap_or(0).into();
        address += 1;
        let a: u16 = self.data.get(&address).copied().unwrap_or(0).into();
        (a << 8) + b
    }
    pub fn get_byte(&self, address: u32) -> u8 {
        self.data.get(&address).copied().unwrap_or(0)
    }
    pub fn insert_word(&mut self, address: u32, data: u32) {
        let mut mask: u32 = (2 << 8) - 1;
        let d: u8 = (data & mask).try_into().unwrap();
        mask <<= 8;
        let c: u8 = ((data & mask) >> 8).try_into().unwrap();
        mask <<= 8;
        let b: u8 = ((data & mask) >> 16).try_into().unwrap();
        mask <<= 8;
        let a: u8 = ((data & mask) >> 24).try_into().unwrap();
        self.data.insert(address, d);
        self.data.insert(address + 1, c);
        self.data.insert(address + 2, b);
        self.data.insert(address + 3, a);
    }
    pub fn insert_hword(&mut self, address: u32, data: u16) {
        let mut mask: u16 = (2 << 8) - 1;
        let d: u8 = (data & mask).try_into().unwrap();
        mask <<= 8;
        let c: u8 = ((data & mask) >> 8).try_into().unwrap();
        self.data.insert(address, d);
        self.data.insert(address + 1, c);
    }
    pub fn insert_byte(&mut self, address: u32, data: u8) {
        self.data.insert(address, data);
    }
}
