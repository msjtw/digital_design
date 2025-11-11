mod pmp;
mod sv32;

use crate::SoC;
use crate::core::{Core, exceptions};
use std::collections::HashMap;
use sv32::AccessType;

#[derive(Debug, Clone, Copy)]
pub struct MemoryPermissions {
    r: bool,
    w: bool,
    x: bool,
}

// #[derive(Debug)]
pub struct Memory {
    base_addr: u32,
    data: Vec<u8>,

    tlb: HashMap<(u32, u32), (u32, MemoryPermissions)>,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            base_addr: super::RAM_OFFSET,
            data: vec![0; super::RAM_SIZE as usize],

            tlb: HashMap::new(),
        }
    }
}

pub fn tlb_flush(memory: &mut Memory) {
    memory.tlb.clear();
}

pub fn read_word(addr: u32, soc: &mut SoC) -> Result<u32, exceptions::Exception> {
    if addr & 0b11 > 0 {
        soc.core.trap_val = addr;
        return Err(exceptions::Exception::Load_address_misaligned);
    }
    match sv32::translate(addr, soc, AccessType::R) {
        Ok((phys_addr, perm)) => {
            if perm.r {
                return phys_read_word(phys_addr, soc);
            }
            soc.core.trap_val = addr;
            return Err(exceptions::Exception::Load_page_fault);
        }
        Err(x) => {
            // println!("mmu error 1 @0x{:08x}", addr);
            match x {
                Some(x) => return Err(x),
                None => {
                    soc.core.trap_val = addr;
                    return Err(exceptions::Exception::Load_page_fault);
                }
            };
        }
    };
}
pub fn fetch_word(addr: u32, soc: &mut SoC) -> Result<u32, exceptions::Exception> {
    match sv32::translate(addr, soc, AccessType::X) {
        Ok((phys_addr, perm)) => {
            if perm.x {
                return phys_fetch_word(phys_addr, soc);
            }
            soc.core.trap_val = addr;
            return Err(exceptions::Exception::Instruction_page_fault);
        }
        Err(x) => {
            // println!("mmu error 2");
            match x {
                Some(x) => return Err(x),
                None => {
                    soc.core.trap_val = addr;
                    return Err(exceptions::Exception::Instruction_page_fault);
                }
            };
        }
    };
}
pub fn read_hword(addr: u32, soc: &mut SoC) -> Result<u16, exceptions::Exception> {
    if addr & 0b1 > 0 {
        soc.core.trap_val = addr;
        return Err(exceptions::Exception::Load_address_misaligned);
    }
    match sv32::translate(addr, soc, AccessType::R) {
        Ok((phys_addr, perm)) => {
            if perm.r {
                return phys_read_hword(phys_addr, soc);
            }
            soc.core.trap_val = addr;
            return Err(exceptions::Exception::Load_page_fault);
        }
        Err(x) => {
            // println!("mmu error 3");
            match x {
                Some(x) => return Err(x),
                None => {
                    soc.core.trap_val = addr;
                    return Err(exceptions::Exception::Load_page_fault);
                }
            };
        }
    };
}
pub fn read_byte(addr: u32, soc: &mut SoC) -> Result<u8, exceptions::Exception> {
    match sv32::translate(addr, soc, AccessType::R) {
        Ok((phys_addr, perm)) => {
            if perm.r {
                return phys_read_byte(phys_addr, soc);
            }
            soc.core.trap_val = addr;
            return Err(exceptions::Exception::Load_page_fault);
        }
        Err(x) => {
            // println!("mmu error 4");
            match x {
                Some(x) => return Err(x),
                None => {
                    soc.core.trap_val = addr;
                    return Err(exceptions::Exception::Load_page_fault);
                }
            };
        }
    };
}
pub fn write_word(addr: u32, data: u32, soc: &mut SoC) -> Result<u32, exceptions::Exception> {
    if addr & 0b11 > 0 {
        soc.core.trap_val = addr;
        return Err(exceptions::Exception::StoreAMO_address_misaligned);
    }
    match sv32::translate(addr, soc, AccessType::W) {
        Ok((phys_addr, perm)) => {
            if perm.w {
                return phys_write_word(phys_addr, data, soc);
            }
            // println!("mmu error 51");
            soc.core.trap_val = addr;
            return Err(exceptions::Exception::StoreAMO_page_fault);
        }
        Err(x) => {
            // println!("mmu error 5");
            match x {
                Some(x) => return Err(x),
                None => {
                    soc.core.trap_val = addr;
                    return Err(exceptions::Exception::StoreAMO_page_fault);
                }
            };
        }
    };
}
pub fn write_hword(addr: u32, data: u16, soc: &mut SoC) -> Result<(), exceptions::Exception> {
    if addr & 0b1 > 0 {
        soc.core.trap_val = addr;
        return Err(exceptions::Exception::StoreAMO_address_misaligned);
    }
    match sv32::translate(addr, soc, AccessType::W) {
        Ok((phys_addr, perm)) => {
            if perm.w {
                return phys_write_hword(phys_addr, data, soc);
            }
            // println!("mmu error 61");
            soc.core.trap_val = addr;
            return Err(exceptions::Exception::StoreAMO_page_fault);
        }
        Err(x) => {
            // println!("mmu error 6");
            match x {
                Some(x) => return Err(x),
                None => {
                    soc.core.trap_val = addr;
                    return Err(exceptions::Exception::StoreAMO_page_fault);
                }
            };
        }
    };
}
pub fn write_byte(addr: u32, data: u8, soc: &mut SoC) -> Result<(), exceptions::Exception> {
    match sv32::translate(addr, soc, AccessType::W) {
        Ok((phys_addr, perm)) => {
            if perm.w {
                return phys_write_byte(phys_addr, data, soc);
            }
            // println!("mmu error 71");
            soc.core.trap_val = addr;
            return Err(exceptions::Exception::StoreAMO_page_fault);
        }
        Err(x) => {
            // println!("mmu error 7 {:?} 0x{:08x}", x, addr);
            match x {
                Some(x) => return Err(x),
                None => {
                    soc.core.trap_val = addr;
                    return Err(exceptions::Exception::StoreAMO_page_fault);
                }
            };
        }
    };
}

pub fn phys_read_word(addr: u32, soc: &mut SoC) -> Result<u32, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 4, soc.core);
    if !perm.r {
        // println!("1 Error! read:0x{:x}", addr);
        soc.core.trap_val = addr;
        return Err(exceptions::Exception::Load_access_fault);
    }

    let memory = &soc.memory;

    if addr < memory.base_addr {
        if soc.plic.claim(addr) {
            return Ok(soc.plic.read(addr));
        } else {
            return match addr {
                0x200bffc => {
                    if soc.core.p_start {
                        eprintln!("mtime change 0x{:x}", soc.core.mtime);
                    }
                    Ok((soc.core.mtime >> 32) as u32)
                }
                0x200bff8 => {
                    if soc.core.p_start {
                        eprintln!("mtime change 0x{:x}", soc.core.mtime);
                    }
                    Ok(soc.core.mtime as u32)
                }
                _ => Ok(0),
            };
        }
    }
    let address = (addr - memory.base_addr) as usize;
    let d = memory.data[address] as u32;
    let c = memory.data[address + 1] as u32;
    let b = memory.data[address + 2] as u32;
    let a = memory.data[address + 3] as u32;
    Ok((a << 24) + (b << 16) + (c << 8) + d)
}
pub fn phys_fetch_word(addr: u32, soc: &mut SoC) -> Result<u32, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 4, soc.core);
    if !perm.x {
        // println!("3 Error! read:0x{:x}", addr);
        soc.core.trap_val = addr;
        return Err(exceptions::Exception::Instruction_access_fault);
    }

    let memory = &soc.memory;

    let address = (addr - memory.base_addr) as usize;
    let d = memory.data[address] as u32;
    let c = memory.data[address + 1] as u32;
    let b = memory.data[address + 2] as u32;
    let a = memory.data[address + 3] as u32;
    Ok((a << 24) + (b << 16) + (c << 8) + d)
}

pub fn phys_read_hword(addr: u32, soc: &mut SoC) -> Result<u16, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 2, soc.core);
    if !perm.r {
        // println!("5 Error! read:0x{:x}", addr);
        soc.core.trap_val = addr;
        return Err(exceptions::Exception::Load_access_fault);
    }

    let memory = &soc.memory;

    if addr < memory.base_addr {
        // println!("6 Error! read:0x{:x}", addr);
    }

    let address = (addr - memory.base_addr) as usize;
    let b = memory.data[address] as u16;
    let a = memory.data[address + 1] as u16;
    Ok((a << 8) + b)
}
pub fn phys_read_byte(addr: u32, soc: &mut SoC) -> Result<u8, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 1, soc.core);
    if !perm.r {
        // println!("7 Error! read:0x{:x}", addr);
        soc.core.trap_val = addr;
        return Err(exceptions::Exception::Load_access_fault);
    }

    let memory = &mut soc.memory;

    if addr < memory.base_addr {
        if soc.uart.claim(addr) {
            return Ok(soc.uart.read(addr));
        } else {
            return Ok(0);
        }
    }
    let address = (addr - memory.base_addr) as usize;
    Ok(memory.data[address])
}

pub fn phys_write_word(addr: u32, data: u32, soc: &mut SoC) -> Result<u32, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 4, soc.core);
    if !perm.w {
        // println!("9 Error! write:0x{:x}", addr);
        soc.core.trap_val = addr;
        return Err(exceptions::Exception::Load_access_fault);
    }

    let memory = &mut soc.memory;

    if addr < memory.base_addr {
        if soc.plic.claim(addr) {
            soc.plic.write(addr, data);
        } else {
            match addr {
                // syscon
                0x11100000 => {
                    return Ok(data);
                }
                0x2004004 => {
                    if soc.core.p_start {
                        eprintln!("mtime change 0x{:x}", soc.core.mtime);
                    }
                    let mtimecmpl = soc.core.mtimecmp as u32;
                    soc.core.mtimecmp = ((data as u64) << 32) + mtimecmpl as u64;
                }
                0x2004000 => {
                    if soc.core.p_start {
                        eprintln!("mtime change 0x{:x}", soc.core.mtime);
                    }
                    let mtimecmph = (soc.core.mtimecmp >> 32) as u32;
                    soc.core.mtimecmp = ((mtimecmph as u64) << 32) + data as u64;
                }
                _ => {
                    // println!("10 Error! write:0x{:x}", addr);
                }
            };
        }

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
pub fn phys_write_hword(addr: u32, data: u16, soc: &mut SoC) -> Result<(), exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 2, soc.core);
    if !perm.w {
        // println!("11 Error! write:0x{:x}", addr);
        soc.core.trap_val = addr;
        return Err(exceptions::Exception::Load_access_fault);
    }
    let memory = &mut soc.memory;

    if addr < memory.base_addr {
        // println!("12 Error! write:0x{:x}", addr);
    }

    let address = (addr - memory.base_addr) as usize;
    let mask: u16 = (2 << 8) - 1;
    let d: u8 = (data & mask) as u8;
    let c: u8 = ((data & mask << 8) >> 8) as u8;
    memory.data[address] = d;
    memory.data[address + 1] = c;
    Ok(())
}
pub fn phys_write_byte(addr: u32, data: u8, soc: &mut SoC) -> Result<(), exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 1, soc.core);
    if !perm.w {
        // println!("13 Error! write:0x{:x}", addr);
        soc.core.trap_val = addr;
        return Err(exceptions::Exception::Load_access_fault);
    }

    let memory = &mut soc.memory;

    if addr < memory.base_addr {
        if soc.uart.claim(addr) {
            soc.uart.write(addr, data);
        } else {
            match addr {
                0x11100000 => {} // TODO: SYSCON;
                _ => {
                    // println!("14 Error! write:0x{:x}", addr);
                }
            };
        }

        return Ok(());
    }
    let address = (addr - memory.base_addr) as usize;
    memory.data[address] = data;
    Ok(())
}
