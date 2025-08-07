use crate::core::csr;
use crate::memory::Core;

#[derive(Debug)]
struct PmpCfg {
    lock: bool,
    a_mode: u8,
    r: bool,
    w: bool,
    x: bool,
}

impl PmpCfg {
    fn from(cfg: u8) -> Self {
        Self {
            lock: (cfg & 0b10000000) > 0,
            a_mode: (cfg & 0b00011000) >> 3,
            x: (cfg & 0b00000100) > 0,
            w: (cfg & 0b00000010) > 0,
            r: (cfg & 0b00000001) > 0,
        }
    }
}

pub fn pmp_check(addr: u32, len: u32, core: &Core) -> super::MemoryPermissions {
    // return super::MemoryPermissions { r: true, w: true, x: true, };
    let pmpaddr0_addr = csr::csr_addr(csr::Csr::pmpaddr0);
    for i in 0..15usize {
        let pmpcfg = csr::read_pmpXcfg(i as u32, core);
        let pmpcfg = PmpCfg::from(pmpcfg);
        let pmpaddr = core.csr_file[pmpaddr0_addr + i] as u64;
        let top: u64;
        let bot: u64;
        match pmpcfg.a_mode {
            0b00 => {
                // disabled; matches no addresses
                continue;
            }
            0b01 => {
                // TOR; top of range
                top = pmpaddr << 2;
                bot = match i {
                    0 => 0,
                    _ => core.csr_file[(pmpaddr0_addr + (i - 1) * 4) as usize] as u64,
                };
            }
            0b10 => {
                // NA4: naturally aligned 4-byte region
                bot = pmpaddr << 2;
                top = bot + 4;
            }
            0b11 => {
                // NAPOT: naturally aligned power-of-two region
                // print!("napot\t");
                let mut a = 1;
                let mut pow: i32 = 0;
                while pmpaddr & a > 0 {
                    a <<= 1;
                    pow += 1;
                }
                bot = (pmpaddr >> pow) << (pow + 2);
                // println!("pmpaddr: 0x{:x}", pmpaddr);
                // println!("pow: {}, bot: 0x{:x}, size: 0x{:x}", pow, bot, (1 << (pow + 3)));
                top = bot + (1 << (pow + 3)) -1;
            }
            _ => {
                top = 0;
                bot = 0;
            }
        };
        let bot = bot as u32;
        let top = top as u32;
        // println!("{i} :0x{:x}", pmpaddr);
        // println!("\t0x{:x} - 0x{:x}", bot, top);
        if addr >= bot && addr + len <= top {
            // full match
            if pmpcfg.lock {
                return super::MemoryPermissions { r: pmpcfg.r, w: pmpcfg.w, x: pmpcfg.x, };
            } else {
                if core.mode == 3 {
                    return super::MemoryPermissions { r: true, w: true, x: true, };
                } else {
                    return super::MemoryPermissions { r: pmpcfg.r, w: pmpcfg.w, x: pmpcfg.x, };
                }
            }
        } else if addr < bot && addr + len > bot || addr < top && addr + len > top {
            // partial match
            return super::MemoryPermissions { r: false, w: false, x: false, };
        } else {
            // nomatch
            continue;
        }
    }

    if core.mode == 3 {
        return super::MemoryPermissions { r: true, w: true, x: true, };
    }
    return super::MemoryPermissions { r: false, w: false, x: false, };
}
