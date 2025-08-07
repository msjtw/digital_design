mod pmp;
mod sv32;

use crate::core::{Core, exceptions};
use std::io::Write;
use std::io::{Bytes, Read};
use termion::async_stdin;

// #[derive(Debug)]
pub struct Memory {
    base_addr: u32,
    data: Vec<u8>,

    stdin: Bytes<termion::AsyncReader>,
    read_byte: u8,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            base_addr: super::RAM_OFFSET,
            data: vec![0; super::RAM_SIZE as usize],

            stdin: async_stdin().bytes(),
            read_byte: 0,
        }
    }
}

pub struct MemoryPermissions {
    r: bool,
    w: bool,
    x: bool,
}

pub fn read_word(addr: u32, core: &Core) -> Result<u32, exceptions::Exception> {
    match sv32::translate(addr, core) {
        Ok((phys_addr, perm)) => {
            if perm.r {
                return phys_read_word(phys_addr, core);
            }
            return Err(exceptions::Exception::Load_page_fault);
        }
        Err(x) => {
            match x {
                Some(x) => return Err(x),
                None => return Err(exceptions::Exception::Load_page_fault),
            };
        }
    };
}
pub fn fetch_word(addr: u32, core: &Core) -> Result<u32, exceptions::Exception> {
    match sv32::translate(addr, core) {
        Ok((phys_addr, perm)) => {
            if perm.x {
                return phys_fetch_word(phys_addr, core);
            }
            return Err(exceptions::Exception::Instruction_page_fault);
        }
        Err(x) => {
            match x {
                Some(x) => return Err(x),
                None => return Err(exceptions::Exception::Instruction_page_fault),
            };
        }
    };
}
pub fn read_hword(addr: u32, core: &Core) -> Result<u16, exceptions::Exception> {
    match sv32::translate(addr, core) {
        Ok((phys_addr, perm)) => {
            if perm.r {
                return phys_read_hword(phys_addr, core);
            }
            return Err(exceptions::Exception::Load_page_fault);
        }
        Err(x) => {
            match x {
                Some(x) => return Err(x),
                None => return Err(exceptions::Exception::Load_page_fault),
            };
        }
    };
}
pub fn read_byte(addr: u32, core: &mut Core) -> Result<u8, exceptions::Exception> {
    match sv32::translate(addr, core) {
        Ok((phys_addr, perm)) => {
            if perm.r {
                return phys_read_byte(phys_addr, core);
            }
            return Err(exceptions::Exception::Load_page_fault);
        }
        Err(x) => {
            match x {
                Some(x) => return Err(x),
                None => return Err(exceptions::Exception::Load_page_fault),
            };
        }
    };
}
pub fn write_word(addr: u32, data: u32, core: &mut Core) -> Result<u32, exceptions::Exception> {
    match sv32::translate(addr, core) {
        Ok((phys_addr, perm)) => {
            if perm.w {
                return phys_write_word(phys_addr, data, core);
            }
            return Err(exceptions::Exception::StoreAMO_page_fault);
        }
        Err(x) => {
            match x {
                Some(x) => return Err(x),
                None => return Err(exceptions::Exception::StoreAMO_page_fault),
            };
        }
    };
}
pub fn write_hword(addr: u32, data: u16, core: &mut Core) -> Result<(), exceptions::Exception> {
    match sv32::translate(addr, core) {
        Ok((phys_addr, perm)) => {
            if perm.w {
                return phys_write_hword(phys_addr, data, core);
            }
            return Err(exceptions::Exception::StoreAMO_page_fault);
        }
        Err(x) => {
            match x {
                Some(x) => return Err(x),
                None => return Err(exceptions::Exception::StoreAMO_page_fault),
            };
        }
    };
}
pub fn write_byte(addr: u32, data: u8, core: &mut Core) -> Result<(), exceptions::Exception> {
    match sv32::translate(addr, core) {
        Ok((phys_addr, perm)) => {
            if perm.w {
                return phys_write_byte(phys_addr, data, core);
            }
            return Err(exceptions::Exception::StoreAMO_page_fault);
        }
        Err(x) => {
            match x {
                Some(x) => return Err(x),
                None => return Err(exceptions::Exception::StoreAMO_page_fault),
            };
        }
    };
}

pub fn phys_read_word(addr: u32, core: &Core) -> Result<u32, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 4, core);
    if !perm.r {
        println!("1 Error! read:0x{:x}", addr);
        return Err(exceptions::Exception::Load_access_fault);
    }

    let memory = &core.memory;

    if addr < memory.base_addr {
        return match addr {
            0x1100bffc => Ok((core.mtime >> 32) as u32),
            0x1100bff8 => Ok(core.mtime as u32),
            0x11004004 => Ok((core.mtimecmp >> 32) as u32),
            0x11004000 => Ok(core.mtimecmp as u32),
            _ => {
                println!("2 Error! read:0x{:x}", addr);
                Ok(0)
            }
        };
    }
    let address = (addr - memory.base_addr) as usize;
    let d = memory.data[address] as u32;
    let c = memory.data[address + 1] as u32;
    let b = memory.data[address + 2] as u32;
    let a = memory.data[address + 3] as u32;
    Ok((a << 24) + (b << 16) + (c << 8) + d)
}
pub fn phys_fetch_word(addr: u32, core: &Core) -> Result<u32, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 4, core);
    if !perm.x {
        println!("3 Error! read:0x{:x}", addr);
        return Err(exceptions::Exception::Instruction_access_fault);
    }

    let memory = &core.memory;

    if addr > super::RAM_OFFSET + super::RAM_SIZE {
        // return Err(exceptions::Exception::Instruction_access_fault);
        println!("max: 0x{:x} < 0x{:x}",super::RAM_OFFSET + super::RAM_SIZE, addr);
    }
    let address = (addr - memory.base_addr) as usize;
    let d = memory.data[address] as u32;
    let c = memory.data[address + 1] as u32;
    let b = memory.data[address + 2] as u32;
    let a = memory.data[address + 3] as u32;
    Ok((a << 24) + (b << 16) + (c << 8) + d)
}

pub fn phys_read_hword(addr: u32, core: &Core) -> Result<u16, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 2, core);
    if !perm.r {
        println!("5 Error! read:0x{:x}", addr);
        return Err(exceptions::Exception::Load_access_fault);
    }

    let memory = &core.memory;

    if addr < memory.base_addr {
        println!("6 Error! read:0x{:x}", addr);
    }

    let address = (addr - memory.base_addr) as usize;
    let b = memory.data[address] as u16;
    let a = memory.data[address + 1] as u16;
    Ok((a << 8) + b)
}
pub fn phys_read_byte(addr: u32, core: &mut Core) -> Result<u8, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 1, core);
    if !perm.r {
        println!("7 Error! read:0x{:x}", addr);
        return Err(exceptions::Exception::Load_access_fault);
    }

    let memory = &mut core.memory;

    if addr < memory.base_addr {
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
            _ => {
                println!("8 Error! read:0x{:x}", addr);
                Ok(0)
            }
        };
    }
    let address = (addr - memory.base_addr) as usize;
    Ok(memory.data[address])
}

pub fn phys_write_word(
    addr: u32,
    data: u32,
    core: &mut Core,
) -> Result<u32, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 4, core);
    if !perm.w {
        println!("9 Error! write:0x{:x}", addr);
        return Err(exceptions::Exception::Load_access_fault);
    }

    let memory = &mut core.memory;

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
            _ => {
                println!("10 Error! write:0x{:x}", addr);
            }
        };
        return Ok(0);
    }
    let address = (addr - memory.base_addr) as usize;
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
pub fn phys_write_hword(
    addr: u32,
    data: u16,
    core: &mut Core,
) -> Result<(), exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 2, core);
    if !perm.w {
        println!("11 Error! write:0x{:x}", addr);
        return Err(exceptions::Exception::Load_access_fault);
    }
    let memory = &mut core.memory;

    if addr < memory.base_addr {
        println!("12 Error! write:0x{:x}", addr);
    }

    let address = (addr - memory.base_addr) as usize;
    let mask: u16 = (2 << 8) - 1;
    let d: u8 = (data & mask) as u8;
    let c: u8 = ((data & mask << 8) >> 8) as u8;
    memory.data[address] = d;
    memory.data[address + 1] = c;
    Ok(())
}
pub fn phys_write_byte(addr: u32, data: u8, core: &mut Core) -> Result<(), exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 1, core);
    if !perm.w {
        println!("13 Error! write:0x{:x}", addr);
        return Err(exceptions::Exception::Load_access_fault);
    }

    let memory = &mut core.memory;

    if addr < memory.base_addr {
        match addr {
            0x10000000 => {
                print!("{}", data as char);
                let _ = std::io::stdout().flush();
            } // TODO: UART;
            0x11100000 => {} // TODO: SYSCON;
            _ => {
                println!("14 Error! write:0x{:x}", addr);
            }
        };
        return Ok(());
    }
    let address = (addr - memory.base_addr) as usize;
    memory.data[address] = data;
    Ok(())
}
