mod pmp;
mod sv32;
use crate::{
    core::{Core, Hart, exceptions, virt_memory::sv32::AccessType},
    memory::*,
};

use super::MemoryPermissions;

pub fn virt_read_word(
    addr: u32,
    hart: &mut Hart,
    bus: &mut MemoryBus,
) -> Result<u32, exceptions::Exception> {
    if addr & 0b11 > 0 {
        hart.core.trap_val = addr;
        return Err(exceptions::Exception::Load_address_misaligned);
    }
    match sv32::translate(addr, hart, bus, AccessType::R) {
        Ok((phys_addr, perm)) => {
            if perm.r {
                return phys_read_word(phys_addr, hart, bus);
            }
            hart.core.trap_val = addr;
            return Err(exceptions::Exception::Load_page_fault);
        }
        Err(x) => {
            // println!("mmu error 1 @0x{:08x}", addr);
            match x {
                Some(x) => return Err(x),
                None => {
                    hart.core.trap_val = addr;
                    return Err(exceptions::Exception::Load_page_fault);
                }
            };
        }
    };
}
pub fn virt_fetch_word(
    addr: u32,
    hart: &mut Hart,
    bus: &mut MemoryBus,
) -> Result<u32, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 4, &hart.core);
    if !perm.x {
        // println!("3 Error! read:0x{:x}", addr);
        hart.core.trap_val = addr;
        return Err(exceptions::Exception::Instruction_access_fault);
    }
    match sv32::translate(addr, hart, bus, AccessType::X) {
        Ok((phys_addr, perm)) => {
            if perm.x {
                return phys_fetch_word(phys_addr, hart, bus);
            }
            hart.core.trap_val = addr;
            return Err(exceptions::Exception::Instruction_page_fault);
        }
        Err(x) => {
            // println!("mmu error 2");
            match x {
                Some(x) => return Err(x),
                None => {
                    hart.core.trap_val = addr;
                    return Err(exceptions::Exception::Instruction_page_fault);
                }
            };
        }
    };
}
pub fn virt_read_hword(
    addr: u32,
    hart: &mut Hart,
    bus: &mut MemoryBus,
) -> Result<u16, exceptions::Exception> {
    if addr & 0b1 > 0 {
        hart.core.trap_val = addr;
        return Err(exceptions::Exception::Load_address_misaligned);
    }
    match sv32::translate(addr, hart, bus, AccessType::R) {
        Ok((phys_addr, perm)) => {
            if perm.r {
                return phys_read_hword(phys_addr, hart, bus);
            }
            hart.core.trap_val = addr;
            return Err(exceptions::Exception::Load_page_fault);
        }
        Err(x) => {
            // println!("mmu error 3");
            match x {
                Some(x) => return Err(x),
                None => {
                    hart.core.trap_val = addr;
                    return Err(exceptions::Exception::Load_page_fault);
                }
            };
        }
    };
}
pub fn virt_read_byte(
    addr: u32,
    hart: &mut Hart,
    bus: &mut MemoryBus,
) -> Result<u8, exceptions::Exception> {
    match sv32::translate(addr, hart, bus, AccessType::R) {
        Ok((phys_addr, perm)) => {
            if perm.r {
                return phys_read_byte(phys_addr, hart, bus);
            }
            hart.core.trap_val = addr;
            return Err(exceptions::Exception::Load_page_fault);
        }
        Err(x) => {
            // println!("mmu error 4");
            match x {
                Some(x) => return Err(x),
                None => {
                    hart.core.trap_val = addr;
                    return Err(exceptions::Exception::Load_page_fault);
                }
            };
        }
    };
}
pub fn virt_write_word(
    addr: u32,
    data: u32,
    hart: &mut Hart,
    bus: &mut MemoryBus,
) -> Result<(), exceptions::Exception> {
    if addr & 0b11 > 0 {
        hart.core.trap_val = addr;
        return Err(exceptions::Exception::StoreAMO_address_misaligned);
    }
    match sv32::translate(addr, hart, bus, AccessType::W) {
        Ok((phys_addr, perm)) => {
            if perm.w {
                return phys_write_word(phys_addr, data, hart, bus);
            }
            // println!("mmu error 51");
            hart.core.trap_val = addr;
            return Err(exceptions::Exception::StoreAMO_page_fault);
        }
        Err(x) => {
            // println!("mmu error 5");
            match x {
                Some(x) => return Err(x),
                None => {
                    hart.core.trap_val = addr;
                    return Err(exceptions::Exception::StoreAMO_page_fault);
                }
            };
        }
    };
}
pub fn virt_write_hword(
    addr: u32,
    data: u16,
    hart: &mut Hart,
    bus: &mut MemoryBus,
) -> Result<(), exceptions::Exception> {
    if addr & 0b1 > 0 {
        hart.core.trap_val = addr;
        return Err(exceptions::Exception::StoreAMO_address_misaligned);
    }
    match sv32::translate(addr, hart, bus, AccessType::W) {
        Ok((phys_addr, perm)) => {
            if perm.w {
                return phys_write_hword(phys_addr, data, hart, bus);
            }
            // println!("mmu error 61");
            hart.core.trap_val = addr;
            return Err(exceptions::Exception::StoreAMO_page_fault);
        }
        Err(x) => {
            // println!("mmu error 6");
            match x {
                Some(x) => return Err(x),
                None => {
                    hart.core.trap_val = addr;
                    return Err(exceptions::Exception::StoreAMO_page_fault);
                }
            };
        }
    };
}
pub fn virt_write_byte(
    addr: u32,
    data: u8,
    hart: &mut Hart,
    bus: &mut MemoryBus,
) -> Result<(), exceptions::Exception> {
    match sv32::translate(addr, hart, bus, AccessType::W) {
        Ok((phys_addr, perm)) => {
            if perm.w {
                return phys_write_byte(phys_addr, data, hart, bus);
            }
            // println!("mmu error 71");
            hart.core.trap_val = addr;
            return Err(exceptions::Exception::StoreAMO_page_fault);
        }
        Err(x) => {
            // println!("mmu error 7 {:?} 0x{:08x}", x, addr);
            match x {
                Some(x) => return Err(x),
                None => {
                    hart.core.trap_val = addr;
                    return Err(exceptions::Exception::StoreAMO_page_fault);
                }
            };
        }
    };
}

pub fn phys_read_word(
    addr: u32,
    hart: &mut Hart,
    bus: &mut MemoryBus,
) -> Result<u32, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 4, &hart.core);
    if !perm.r {
        // println!("1 Error! read:0x{:x}", addr);
        hart.core.trap_val = addr;
        return Err(exceptions::Exception::Load_access_fault);
    }
    if hart.clint.claim(addr) {
        let val = hart.clint.read(addr);
        return Ok(val);
    }

    load_word(bus, addr)
}
pub fn phys_fetch_word(
    addr: u32,
    hart: &mut Hart,
    bus: &mut MemoryBus,
) -> Result<u32, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 4, &hart.core);
    if !perm.x {
        // println!("3 Error! read:0x{:x}", addr);
        hart.core.trap_val = addr;
        return Err(exceptions::Exception::Instruction_access_fault);
    }

    load_word(bus, addr)
}

pub fn phys_read_hword(
    addr: u32,
    hart: &mut Hart,
    bus: &mut MemoryBus,
) -> Result<u16, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 2, &hart.core);
    if !perm.r {
        // println!("5 Error! read:0x{:x}", addr);
        hart.core.trap_val = addr;
        return Err(exceptions::Exception::Load_access_fault);
    }

    load_hword(bus, addr)
}
pub fn phys_read_byte(
    addr: u32,
    hart: &mut Hart,
    bus: &mut MemoryBus,
) -> Result<u8, exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 1, &hart.core);
    if !perm.r {
        // println!("7 Error! read:0x{:x}", addr);
        hart.core.trap_val = addr;
        return Err(exceptions::Exception::Load_access_fault);
    }

    load_byte(bus, addr)
}

pub fn phys_write_word(
    addr: u32,
    data: u32,
    hart: &mut Hart,
    bus: &mut MemoryBus,
) -> Result<(), exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 4, &hart.core);
    if !perm.w {
        // println!("9 Error! write:0x{:x}", addr);
        hart.core.trap_val = addr;
        return Err(exceptions::Exception::Load_access_fault);
    }
    if hart.clint.claim(addr) {
        hart.clint.write(addr, data);
    }
    store_word(bus, addr, data)
}
pub fn phys_write_hword(
    addr: u32,
    data: u16,
    hart: &mut Hart,
    bus: &mut MemoryBus,
) -> Result<(), exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 2, &hart.core);
    if !perm.w {
        // println!("11 Error! write:0x{:x}", addr);
        hart.core.trap_val = addr;
        return Err(exceptions::Exception::Load_access_fault);
    }

    store_hword(bus, addr, data)
}
pub fn phys_write_byte(
    addr: u32,
    data: u8,
    hart: &mut Hart,
    bus: &mut MemoryBus,
) -> Result<(), exceptions::Exception> {
    let perm = pmp::pmp_check(addr, 1, &hart.core);
    if !perm.w {
        // println!("13 Error! write:0x{:x}", addr);
        hart.core.trap_val = addr;
        return Err(exceptions::Exception::Load_access_fault);
    }

    store_byte(bus, addr, data)
}
