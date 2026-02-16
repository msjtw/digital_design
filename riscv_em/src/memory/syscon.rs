pub const SYSCON_POWEROFF: u32 = 1;
pub const SYSCON_REBOOT: u32 = 2;

// #[derive(Debug)]
pub struct Syscon {
    base: u32,
    length: u32,

    pub val: u32,
}

impl Default for Syscon {
    fn default() -> Self {
        Syscon {
            base: 0x1c00000 ,
            length: 0x1000,
            val: 0,
        }
    }
}

impl Syscon {
    pub fn claim(&self, addr: u32) -> bool {
        if addr >= self.base && addr < self.base + self.length {
            return true;
        }
        return false;
    }

    pub fn store_word(&mut self, addr: u32, data: u32) {
        println!("syscon store {data}");
        let addr = addr - self.base;
        if addr == 0 {
            self.val = data;
        }
    }
}
