use std::process;

use crate::{
    core::{Core, csr, exceptions},
    memory,
};

use super::{MemoryPermissions, phys_read_word, phys_write_word};

const PAGESIZE: u32 = 1 << 12;
const LEVELS: u32 = 2;
const PTESIZE: u32 = 4;

const MMU_DEBUG: bool = false;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
struct PTE {
    ppn: u32,
    ppn1: u32,
    ppn0: u32,
    rsw: u8,
    d: bool,
    a: bool,
    g: bool,
    u: bool,
    x: bool,
    w: bool,
    r: bool,
    v: bool,
}

impl From<u32> for PTE {
    fn from(pte: u32) -> Self {
        PTE {
            ppn: (pte & 0b11111111111111111111110000000000) >> 10,
            ppn1: (pte & 0b11111111111100000000000000000000) >> 20,
            ppn0: (pte & 0b00000000000011111111110000000000) >> 10,
            rsw: ((pte & 0b00000000000000000000001100000000) >> 8) as u8,
            d: (pte & 0b00000000000000000000000010000000) >= 1,
            a: (pte & 0b00000000000000000000000001000000) >= 1,
            g: (pte & 0b00000000000000000000000000100000) >= 1,
            u: (pte & 0b00000000000000000000000000010000) >= 1,
            x: (pte & 0b00000000000000000000000000001000) >= 1,
            w: (pte & 0b00000000000000000000000000000100) >= 1,
            r: (pte & 0b00000000000000000000000000000010) >= 1,
            v: (pte & 0b00000000000000000000000000000001) >= 1,
        }
    }
}

impl Into<u32> for PTE {
    fn into(self) -> u32 {
        let res = (self.ppn as u32) << 10
            | (self.rsw as u32) << 8
            | (self.d as u32) << 7
            | (self.a as u32) << 6
            | (self.g as u32) << 5
            | (self.u as u32) << 4
            | (self.x as u32) << 3
            | (self.w as u32) << 2
            | (self.r as u32) << 1
            | (self.v as u32);
        res
    }
}

impl PTE {
    fn set_a(self) -> Self {
        Self {
            ppn: self.ppn,
            ppn1: self.ppn1,
            ppn0: self.ppn0,
            rsw: self.rsw,
            d: self.d,
            a: true,
            g: self.g,
            u: self.u,
            x: self.x,
            w: self.w,
            r: self.r,
            v: self.v,
        }
    }
    fn set_d(self) -> Self {
        Self {
            ppn: self.ppn,
            ppn1: self.ppn1,
            ppn0: self.ppn0,
            rsw: self.rsw,
            d: true,
            a: self.a,
            g: self.g,
            u: self.u,
            x: self.x,
            w: self.w,
            r: self.r,
            v: self.v,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct SATP {
    mode: u8,
    asid: u32,
    ppn: u32,
}

impl From<u32> for SATP {
    fn from(val: u32) -> Self {
        SATP {
            mode: ((val & 0b10000000000000000000000000000000) >> 31) as u8,
            asid: (val & 0b01111111110000000000000000000000) >> 22,
            ppn: (val & 0b00000000001111111111111111111111),
        }
    }
}

#[derive(Debug)]
struct VA {
    vpn1: u32,
    vpn0: u32,
    offset: u32,
}

impl From<u32> for VA {
    fn from(val: u32) -> Self {
        VA {
            vpn1: (val & 0b11111111110000000000000000000000) >> 22,
            vpn0: (val & 0b00000000001111111111000000000000) >> 12,
            offset: val & 0b00000000000000000000111111111111,
        }
    }
}

#[derive(Debug)]
struct PA {
    ppn1: u32,
    ppn0: u32,
    offset: u32,
}

impl Into<u32> for PA {
    fn into(self) -> u32 {
        let ppn1 = self.ppn1 << 22;
        let ppn0 = self.ppn0 << 12;
        ppn1 | ppn0 | self.offset
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AccessType {
    R,
    W,
    X,
}

pub fn translate(
    virt_a: u32,
    core: &mut Core,
    a_type: AccessType,
) -> Result<(u32, MemoryPermissions), Option<exceptions::Exception>> {
    let satp = csr::read(csr::Csr::satp, core);
    let satp = SATP::from(satp);

    // The satp register must be active, i.e., the effective privilege mode must be S-mode or U-mode.
    // The MPRV (Modify PRiVilege) bit modifies the effective privilege mode.
    // When MPRV=0, loads and stores behave as of the current privilege mode.
    // When MPRV=1, loads and stores behave as though the current privilege mode were set to MPP
    let mstatus = csr::read(csr::Csr::mstatus, core);
    let mstatus_mxr = (mstatus >> 19) & 0b1;
    let mstatus_sum = (mstatus >> 18) & 0b1;
    let mprv = (mstatus >> 17) & 0b1;

    let mode;
    if a_type != AccessType::X && mprv > 0 {
        mode = (mstatus >> 11) & 0b11;
    } else {
        mode = core.mode;
    }

    if satp.mode == 0 || mode > 1 {
        return Ok((
            virt_a,
            MemoryPermissions {
                r: true,
                w: true,
                x: true,
            },
        ));
    }

    let va = VA::from(virt_a);

    let mut a = satp.ppn * PAGESIZE;
    let mut i = LEVELS - 1;

    // level 1
    let index = va.vpn1 * PTESIZE;
    let mut pte_addr = a + index;
    let pte_m1 = phys_read_word(pte_addr, core)?;

    let mut pte = PTE::from(pte_m1);

    if !pte.v || (!pte.r && pte.w) {
        // print!(" mmu6 ");
        return Err(None);
    }

    if !(pte.r || pte.x) {
        if pte.d || pte.a || pte.u {
            return Err(None);
        }
        //level 0
        i -= 1;
        a = pte.ppn * PAGESIZE;
        let index = va.vpn0 * PTESIZE;
        pte_addr = a + index;
        let pte_m0 = phys_read_word(pte_addr, core)?;
        pte = PTE::from(pte_m0);
        if !pte.v || (!pte.r && pte.w) {
            // page fault
            // print!(" mmu5 ");
            return Err(None);
        }

        if !(pte.r || pte.x) {
            // level < 0
            // page fault
            // print!(" mmu4 ");
            return Err(None);
        }

        if pte.u {
            //user page
            if (mode == 1 && mstatus_sum == 0) || mode > 1 {
                // print!(" mmu3 ");
                return Err(None);
            }
        } else {
            //supervisor page
            if mode != 1 {
                // print!(" mmu2 ");
                return Err(None);
            }
        }
    }

    // leaf pte has been reached
    if i > 0 && pte.ppn0 != 0 {
        // misaligned superpage
        // print!(" mmu1 ");
        return Err(None);
    }

    let pa = PA {
        ppn1: pte.ppn1,
        ppn0: if i > 0 { va.vpn0 } else { pte.ppn0 },
        offset: va.offset,
    };

    let phys_a: u32 = pa.into();

    let res: (u32, MemoryPermissions);
    if mstatus_mxr > 0 {
        // make eXecutable Readable
        res = (
            phys_a,
            MemoryPermissions {
                r: pte.x,
                w: pte.w,
                x: pte.x,
            },
        );
    } else {
        res = (
            phys_a,
            MemoryPermissions {
                r: pte.r,
                w: pte.w,
                x: pte.x,
            },
        );
    }

    // Svade extension
    if !pte.a || (a_type == AccessType::W && !pte.d){
        return Err(None);
    }

    // let mut pte = pte.set_a();
    // match a_type {
    //     AccessType::W => pte = pte.set_d(),
    //     _ => {}
    // }
    // let pte_u32: u32 = pte.into();
    //
    // phys_write_word(pte_addr, pte_u32, core)?;

    core.last_pa = phys_a;

    if virt_a == 0x9d2526fc {
        // core.instr_str = format!("pte0>{:?}< {}", pte, core.instr_str);
        // print!("0x{:08x} -> 0x{:08x}", virt_a, res.0);
        // println!("\t pte {:?}", pte);riscv pma
    }

    return Ok(res);
}
