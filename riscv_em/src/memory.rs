use std::io::Write;
use std::io::{Bytes, Read};
use termion::async_stdin;

pub enum RTC {
    Mtime,
    Mtimecmp,
}

// #[derive(Debug)]
pub struct Memory {
    base_addr: u32,
    data: Vec<u8>,
    memory_size: u32,

    mtime: u32,
    mtimeh: u32,
    mtimecmp: u32,
    mtimecmph: u32,

    stdin: Bytes<termion::AsyncReader>,
    read_byte: u8,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            base_addr: super::RAM_OFFSET,
            data: vec![0; super::RAM_SIZE],
            memory_size: super::RAM_OFFSET + super::RAM_SIZE as u32,

            mtime: 0,
            mtimeh: 0,
            mtimecmp: 0,
            mtimecmph: 0,

            stdin: async_stdin().bytes(),
            read_byte: 0,
        }
    }
}

impl Memory {
    pub fn get_word(&self, addr: u32) -> Result<u32, u32> {
        // if addr & 0b11 > 0 {
        //     println!("{}", addr & 0b11);
        //     return Err(4);
        // }

        if addr < self.base_addr || addr > self.memory_size {
            return match addr {
                0x1100bffc => Ok(self.mtimeh),
                0x1100bff8 => Ok(self.mtime),
                0x11004004 => Ok(self.mtimecmph),
                0x11004000 => Ok(self.mtimecmp),
                _ => Ok(0),
            };
        }
        let address = (addr - self.base_addr) as usize;
        let d = self.data[address] as u32;
        let c = self.data[address + 1] as u32;
        let b = self.data[address + 2] as u32;
        let a = self.data[address + 3] as u32;
        Ok((a << 24) + (b << 16) + (c << 8) + d)
    }
    pub fn get_hword(&self, addr: u32) -> Result<u16, u32> {
        // if addr & 0b1 > 0 {
        //     return Err(4);
        // }
        let address = (addr - self.base_addr) as usize;
        let b = self.data[address] as u16;
        let a = self.data[address + 1] as u16;
        Ok((a << 8) + b)
    }
    pub fn get_byte(&mut self, addr: u32) -> Result<u8, u32> {
        if addr < self.base_addr || addr > self.memory_size {
            return match addr {
                // read uart byte
                0x10000000 => Ok(self.read_byte),
                // check if there is something to read
                0x10000005 => {
                    let mut bytes_to_read = 0;
                    if let Some(Ok(byte)) = self.stdin.next() {
                        self.read_byte = byte;
                        bytes_to_read = 1;
                    }
                    let ret = 0x60 | bytes_to_read;
                    Ok(ret)
                }
                _ => Ok(0),
            };
        }
        let address = (addr - self.base_addr) as usize;
        Ok(self.data[address])
    }

    pub fn insert_word(&mut self, addr: u32, data: u32) -> Result<u32, u32> {
        // if addr & 0b11 > 0 {
        //     return Err(6);
        // }

        if addr < self.base_addr {
            match addr {
                // syscon
                0x11100000 => {
                    return Ok(data);
                }
                0x1100bffc => {
                    self.mtimeh = data;
                }
                0x1100bff8 => {
                    self.mtime = data;
                }
                0x11004004 => {
                    self.mtimecmph = data;
                }
                0x11004000 => {
                    self.mtimecmp = data;
                }
                _ => {}
            };
            return Ok(0);
        }
        let address = (addr - self.base_addr) as usize;
        let mask: u32 = (1 << 8) - 1;
        let d: u8 = (data & mask) as u8;
        let c: u8 = ((data & mask << 8) >> 8) as u8;
        let b: u8 = ((data & mask << 16) >> 16) as u8;
        let a: u8 = ((data & mask << 24) >> 24) as u8;
        self.data[address] = d;
        self.data[address + 1] = c;
        self.data[address + 2] = b;
        self.data[address + 3] = a;
        Ok(0)
    }
    pub fn insert_hword(&mut self, addr: u32, data: u16) -> Result<(), u32> {
        // if addr & 0b1 > 0 {
        //     return Err(6);
        // }
        if addr < self.base_addr {
            return Err(7);
        }
        let address = (addr - self.base_addr) as usize;
        let mask: u16 = (2 << 8) - 1;
        let d: u8 = (data & mask) as u8;
        let c: u8 = ((data & mask << 8) >> 8) as u8;
        self.data[address] = d;
        self.data[address + 1] = c;
        Ok(())
    }
    pub fn insert_byte(&mut self, addr: u32, data: u8) -> Result<(), u32> {
        if addr < self.base_addr {
            match addr {
                0x10000000 => {
                    print!("{}", data as char);
                    std::io::stdout().flush();
                } // TODO: UART;
                0x11100000 => {} // TODO: SYSCON;
                _ => {}
            };
            return Ok(());
        }
        let address = (addr - self.base_addr) as usize;
        self.data[address] = data;
        Ok(())
    }

    pub fn csr_read(&self, csr: RTC) -> u64 {
        match csr {
            RTC::Mtime => {
                let mtimel = self.get_word(0x1100bff8).unwrap();
                let mtimeh = self.get_word(0x1100bffc).unwrap();
                ((mtimeh as u64) << 32) + mtimel as u64
            }
            RTC::Mtimecmp => {
                let mtimecmpl = self.get_word(0x11004000).unwrap();
                let mtimecmph = self.get_word(0x11004004).unwrap();
                ((mtimecmph as u64) << 32) + mtimecmpl as u64
            }
        }
    }
    pub fn csr_write(&mut self, csr: RTC, val: u64) -> Result<(), u32> {
        match csr {
            RTC::Mtime => {
                self.insert_word(0x1100bff8, val as u32)?;
                self.insert_word(0x1100bffc, (val >> 32) as u32)?;
            }
            RTC::Mtimecmp => {
                self.insert_word(0x11004000, val as u32)?;
                self.insert_word(0x11004004, (val >> 32) as u32)?;
            }
        };
        Ok(())
    }
}
