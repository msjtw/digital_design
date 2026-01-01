use crate::core::{Core, csr};

// total of 31 interrupt sources, one context (no. 1), no priority
// "hart context is a given privilege mode on a given hart"
// NOTE: Only context 1 is implemented, so only supervisor interrupts work.
// TODO: Implement contexts for all execution levels.
const INT_PENDING_OFF: usize = 0x1000; // Interrupt Pending bit 0-31
const INT_ENABLED_OFF: usize = 0x2080; // Enable bits for sources 0-31 on context 1
const INT_THRESHOLD_OFF: usize = 0x201000; // Priority threshold for context 1
const INT_CLAIM_OFF: usize = 0x201004; // Claim/complete for context 1

pub struct Plic {
    base: usize,
    length: usize,

    pub intt_active: u32,
    intt_pending: u32,
    intt_enabled: u32,
    intt_masked: u32,
    //  Once plic records first interrupt from source it masks (ignores) all later interrupt signals
    //  until interrupt has been completed.
    //  Interrupt pending bit is cleared when interrupt is claimed by hart (but is still masked).
    //  Interrupt mask is cleared when interrupt is completed.
}

impl Default for Plic {
    fn default() -> Self {
        Plic {
            base: 0xc000000,
            length: 0x1000000,
            intt_active: 0,
            intt_pending: 0,
            intt_enabled: 0,
            intt_masked: 0,
        }
    }
}

impl Plic {
    pub fn claim(&self, addr: u32) -> bool {
        if addr as usize >= self.base && (addr as usize) < self.base + self.length {
            return true;
        }
        return false;
    }

    pub fn tick(&mut self, core: &mut Core) {
        // Until interrupt is completed further signals are ignored.
        self.intt_pending |= self.intt_active & !self.intt_masked;
        self.intt_masked |= self.intt_active;
        let mut mip = csr::read(csr::Csr::mip, core);
        if self.intt_pending & self.intt_enabled != 0 {
            mip |= 1 << 9;
        } else {
            // FIX: It works but is implemented wrong. Check specification of meip and siep.
            mip &= !(1 << 9);
        }
        csr::write(csr::Csr::mip, mip, core);
    }

    pub fn read(&mut self, addr: u32) -> u32 {
        let addr = addr as usize - self.base;
        let ret;
        if addr > 0 && addr < 0x20 {
            // no interrupt 0
            ret = 1;
        } else {
            ret = match addr {
                INT_PENDING_OFF => self.intt_pending,
                INT_ENABLED_OFF => self.intt_enabled,
                INT_THRESHOLD_OFF => 0, // no priority; threshold 0
                INT_CLAIM_OFF => {
                    // claim
                    let candidates = self.intt_pending & self.intt_enabled;
                    if candidates > 0 {
                        let intt_num = (candidates as f64).log2() as u32; // return id of highest priority interrupt
                        self.intt_pending &= !(1 << intt_num); // and set it's intt_pending bit to 0
                        intt_num
                    } else {
                        0
                    }
                }
                _ => 0,
            };
        }
        return ret;
    }

    pub fn write(&mut self, addr: u32, data: u32) {
        let addr = addr as usize - self.base;
        if addr < 0x20 {
        } else {
            match addr {
                INT_PENDING_OFF => {}
                INT_ENABLED_OFF => self.intt_enabled = data & !(1), // interrupt 0 doesnt exist
                INT_THRESHOLD_OFF => {}                             // hardwired to 0
                INT_CLAIM_OFF => {
                    // completion
                    // reset corresponding bit from intt_mask
                    self.intt_masked &= !(1 << data);
                }
                _ => {}
            };
        }
    }
}
