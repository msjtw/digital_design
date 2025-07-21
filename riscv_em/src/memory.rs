use crate::core::{Core};
use std::io::Write;
use std::io::{Bytes, Read};
use termion::async_stdin;

// #[derive(Debug)]
pub struct Memory {
    base_addr: u32,
    data: Vec<u8>,
    memory_size: u32,

    stdin: Bytes<termion::AsyncReader>,
    read_byte: u8,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            base_addr: super::RAM_OFFSET,
            data: vec![0; super::RAM_SIZE],
            memory_size: super::RAM_OFFSET + super::RAM_SIZE as u32,

            stdin: async_stdin().bytes(),
            read_byte: 0,
        }
    }
}
pub fn read_word(addr: u32, core: &Core) -> Result<u32, u32> {
    let memory = &core.memory;
    // if addr & 0b11 > 0 {
    //     println!("{}", addr & 0b11);
    //     return Err(4);
    // }

    if addr < memory.base_addr || addr > memory.memory_size {
        return match addr {
            0x1100bffc => Ok((core.mtime >> 32) as u32),
            0x1100bff8 => Ok(core.mtime as u32),
            0x11004004 => Ok((core.mtimecmp >> 32) as u32),
            0x11004000 => Ok(core.mtimecmp as u32),
            _ => Ok(0),
        };
    }
    let address = (addr - memory.base_addr) as usize;
    let d = memory.data[address] as u32;
    let c = memory.data[address + 1] as u32;
    let b = memory.data[address + 2] as u32;
    let a = memory.data[address + 3] as u32;
    Ok((a << 24) + (b << 16) + (c << 8) + d)
}
pub fn fetch_word(addr: u32, core: &Core) -> Result<u32, u32> {
    let memory = &core.memory;
    // if addr & 0b11 > 0 {
    //     println!("{}", addr & 0b11);
    //     return Err(4);
    // }

    let address = (addr - memory.base_addr) as usize;
    let d = memory.data[address] as u32;
    let c = memory.data[address + 1] as u32;
    let b = memory.data[address + 2] as u32;
    let a = memory.data[address + 3] as u32;
    Ok((a << 24) + (b << 16) + (c << 8) + d)
}

pub fn read_hword(addr: u32, core: &Core) -> Result<u16, u32> {
    let memory = &core.memory;
    // if addr & 0b1 > 0 {
    //     return Err(4);
    // }
    let address = (addr - memory.base_addr) as usize;
    let b = memory.data[address] as u16;
    let a = memory.data[address + 1] as u16;
    Ok((a << 8) + b)
}
pub fn read_byte(addr: u32, core: &mut Core) -> Result<u8, u32> {
    let memory = &mut core.memory;
    if addr < memory.base_addr || addr > memory.memory_size {
        return match addr {
            // read uart byte
            0x10000000 => Ok(memory.read_byte),
            // check if there is something to read
            0x10000005 => {
                let mut bytes_to_read = 0;
                if let Some(Ok(byte)) = memory.stdin.next() {
                    memory.read_byte = byte;
                    bytes_to_read = 1;
                }
                let ret = 0x60 | bytes_to_read;
                Ok(ret)
            }
            _ => Ok(0),
        };
    }
    let address = (addr - memory.base_addr) as usize;
    Ok(memory.data[address])
}

pub fn write_word(addr: u32, data: u32, core: &mut Core) -> Result<u32, u32> {
    let memory = &mut core.memory;
    // if addr & 0b11 > 0 {
    //     return Err(6);
    // }

    if addr < memory.base_addr {
        match addr {
            // syscon
            0x11100000 => {
                return Ok(data);
            }
            0x1100bffc => {
                let mtimel = core.mtime as u32;
                core.mtime = ((data as u64) << 32) + mtimel as u64;
            }
            0x1100bff8 => {
                let mtimeh = (core.mtime >> 32) as u32;
                core.mtime = ((mtimeh as u64) << 32) + data as u64;
            }
            0x11004004 => {
                let mtimecmpl = core.mtimecmp as u32;
                core.mtimecmp = ((data as u64) << 32) + mtimecmpl as u64;
            }
            0x11004000 => {
                let mtimecmph = (core.mtimecmp >> 32) as u32;
                core.mtimecmp = ((mtimecmph as u64) << 32) + data as u64;
            }
            _ => {}
        };
        return Ok(0);
    }
    let address = (addr - memory.base_addr) as usize;
    // println!("{:x} {:x}", addr, addr - memory.base_addr);
    let mask: u32 = (1 << 8) - 1;
    let d: u8 = (data & mask) as u8;
    let c: u8 = ((data & mask << 8) >> 8) as u8;
    let b: u8 = ((data & mask << 16) >> 16) as u8;
    let a: u8 = ((data & mask << 24) >> 24) as u8;
    memory.data[address] = d;
    memory.data[address + 1] = c;
    memory.data[address + 2] = b;
    memory.data[address + 3] = a;
    Ok(0)
}
pub fn write_hword(addr: u32, data: u16, core: &mut Core) -> Result<(), u32> {
    let memory = &mut core.memory;
    // if addr & 0b1 > 0 {
    //     return Err(6);
    // }
    if addr < memory.base_addr {
        return Err(7);
    }
    let address = (addr - memory.base_addr) as usize;
    let mask: u16 = (2 << 8) - 1;
    let d: u8 = (data & mask) as u8;
    let c: u8 = ((data & mask << 8) >> 8) as u8;
    memory.data[address] = d;
    memory.data[address + 1] = c;
    Ok(())
}
pub fn write_byte(addr: u32, data: u8, core: &mut Core) -> Result<(), u32> {
    let memory = &mut core.memory;
    if addr < memory.base_addr {
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
    let address = (addr - memory.base_addr) as usize;
    memory.data[address] = data;
    Ok(())
}

fn phys_read(addr: u32, core: &Memory){

}

fn phys_write(addr: u32, data: u8, core: &mut Memory){

}
