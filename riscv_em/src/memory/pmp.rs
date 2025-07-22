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

pub fn pmp_check(addr: u32, len: u32, core: &Core) -> Option<super::MemoryPermissions> {
    let pmpaddr0_addr = csr::csr_addr(csr::Csr::pmpaddr0) as u32;
    let mut implemented = 0;
    for i in 0..15 {
        let pmpcfg = csr::read_pmpXcfg(i, core);
        let pmpcfg = PmpCfg::from(pmpcfg);
        let pmpaddr = csr::read_addr(pmpaddr0_addr + i * 4, core);
        match pmpcfg.a_mode {
            0b00 => {
                // disabled; matches no addresses
                continue;
            }
            0b01 => {
                // TOR; top of range
                implemented += 1;
                let top = pmpaddr << 2;
                let bot = match i {
                    0 => 0,
                    _ => csr::read_addr(pmpaddr0_addr + (i - 1) * 4, core),
                };
                if addr >= bot && addr + len <= top {
                    // full match
                    if pmpcfg.lock {
                        return Some(super::MemoryPermissions{r: pmpcfg.r, w: pmpcfg.w, x:pmpcfg.x});
                    }
                    else {
                        if core.mode == 3 {
                            return Some(super::MemoryPermissions{r: true, w: true, x: true});
                        }
                        else {
                            return Some(super::MemoryPermissions{r: pmpcfg.r, w: pmpcfg.w, x:pmpcfg.x});
                        }
                    }
                } else if addr < bot && addr + len > bot || addr < top && addr + len > top {
                    // partial match
                    return None;
                } else {
                    // nomatch
                    continue;
                }
            }
            0b10 => {
                // NA4: naturally aligned 4-byte region
                implemented += 1;
            }
            0b11 => {
                // NAPOT: naturally aligned power-of-two region
                implemented += 1;
            }
            _ => {}
        }
    };
    None
}
