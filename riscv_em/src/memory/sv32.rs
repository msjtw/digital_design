use std::process;

use crate::{core::{csr, exceptions, Core}, memory};

use super::{MemoryPermissions, phys_read_word};

const PAGESIZE: u32 = 1 << 12;
const LEVELS: u32 = 2;
const PTESIZE: u32 = 4;

const MMU_DEBUG: bool = false;

#[allow(dead_code)]
#[derive(Debug)]
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
            ppn:  (pte & 0b11111111111111111111110000000000) >> 10,
            ppn1: (pte & 0b11111111111100000000000000000000) >> 20,
            ppn0: (pte & 0b00000000000011111111110000000000) >> 10,
            rsw: ((pte & 0b00000000000000000000001100000000) >> 8) as u8,
            d:    (pte & 0b00000000000000000000000010000000) >= 1,
            a:    (pte & 0b00000000000000000000000001000000) >= 1,
            g:    (pte & 0b00000000000000000000000000100000) >= 1,
            u:    (pte & 0b00000000000000000000000000010000) >= 1,
            x:    (pte & 0b00000000000000000000000000001000) >= 1,
            w:    (pte & 0b00000000000000000000000000000100) >= 1,
            r:    (pte & 0b00000000000000000000000000000010) >= 1,
            v:    (pte & 0b00000000000000000000000000000001) >= 1,
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
            asid:  (val & 0b01111111110000000000000000000000) >> 22,
            ppn:   (val & 0b00000000001111111111111111111111),
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
            vpn1:  (val & 0b11111111110000000000000000000000) >> 22,
            vpn0:  (val & 0b00000000001111111111000000000000) >> 12,
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

pub fn translate( virt_a: u32, core: &Core,) -> Result<(u32, MemoryPermissions), Option<exceptions::Exception>> {
    let satp = csr::read(csr::Csr::satp, core);
    let satp = SATP::from(satp);

    // The satp register must be active, i.e., the effective privilege mode must be S-mode or U-mode.
    if satp.mode == 0 || core.mode > 1 {
        return Ok((
            virt_a,
            MemoryPermissions { r: true, w: true, x: true, },
        ));
    }

    let va = VA::from(virt_a);

    let mut a = satp.ppn * PAGESIZE;
    let mut i = LEVELS - 1;

    if MMU_DEBUG {
        println!("--------");
        println!("mode: {}, pc: {}", core.mode, core.pc);
        println!("satp 0b{:b}", csr::read(csr::Csr::satp, core));
        println!("satp {:?}", satp);
        println!("a:   0x{:x}", a);

        println!("va 0b{:b}", virt_a);
        println!("va 0x{:x}", virt_a);
        println!("va {:?}", va);
    }

    // level 1
    let index = va.vpn1 * PTESIZE;
    let pte_m1 = phys_read_word(a + index, core)?;

    if MMU_DEBUG {
        println!("0x{:x}", a + index);
        println!("pte_1: 0b{:b}", pte_m1);
    }

    let mut pte = PTE::from(pte_m1);

    if MMU_DEBUG {
        println!("{:?}", pte);
    }

    // process::exit(1);

    // println!("full pte");
    // for i in 0..1023 {
    //     let addr = a + (PTESIZE * i);
    //     println!("0x{:x}: 0b{:b}", addr, phys_read_word(addr, core)?);
    // }


    if !pte.v || (!pte.r && pte.w) {
        // page fault
        println!("--------");
        println!("mode: {}, pc: 0x{:08x}", core.mode, core.pc);
        println!("satp 0b{:b}", csr::read(csr::Csr::satp, core));
        println!("satp {:?}", satp);
        println!("a:   0x{:x}", a);

        println!("va 0b{:032b}", virt_a);
        println!("va 0x{:08x}", virt_a);
        println!("va {:?}", va);
        println!("page fault 1");
        println!("pte *0x{:08x} {:?}", a+index,  pte);
        println!("--------");
        return Err(None);
    }

    if !(pte.r || pte.x) {
        //level 0
        i -= 1;
        a = pte.ppn * PAGESIZE;
        let index = va.vpn0 * PTESIZE;
        let pte_m0 = phys_read_word(a + index, core)?;
        pte = PTE::from(pte_m0);
        // println!("\tpte_0: 0b{:b}", pte_m1);
        // println!("\t{:?}", pte);
        if !pte.v || (!pte.r && pte.w) {
            // page fault
            println!("page fault 2");
            return Err(None);
        }

        if !(pte.r || pte.x) {
            // level < 0
            // page fault
            println!("page fault 3");
            return Err(None);
        }
    }

    // leaf pte has been reached
    //
    if i > 0 && pte.ppn0 != 0 {
        // misaligned superpage
        println!("page fault 4");
        return Err(None);
    }

    let mstatus = csr::read(csr::Csr::mstatus, core);
    let mstatus_sum = mstatus & 1 << 18;
    let mstatus_mxr = mstatus & 1 << 19;
    if !pte.u && core.mode != 1 {
        // access supervisor page not from S-mode
        println!("page fault 4");
        return Err(None);
    }
    if pte.u && core.mode != 0 {
        // access user page not from U-mode
        // check for SUM of mstatus
        if !(core.mode == 1) || !(mstatus_sum > 0) {
            println!("page fault 5");
            return Err(None);
        }
    }

    let pa = PA {
        ppn1: pte.ppn1,
        ppn0: if i > 0 { va.vpn0 } else { pte.ppn0 },
        offset: va.offset,
    };

    let phys_a: u32 = pa.into();
    // println!("0x{:x} -> 0x{:x}", virt_a, phys_a);

    if mstatus_mxr > 0{
        // make eXecutable Readable
        return Ok((phys_a ,MemoryPermissions {r: pte.x, w: pte.w, x: pte.x}));
    }
    Ok((phys_a ,MemoryPermissions {r: pte.r, w: pte.w, x: pte.x}))
}
