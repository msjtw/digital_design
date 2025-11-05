pub mod csr;
mod datapath;
pub mod exceptions;
mod instr_parse;

use crate::memory::{self};
use csr::{Csr, Csr64};
use exceptions::*;
use instr_parse::Instruction;
use std::{fs, u32};

// there is no trap with this number
const TRAP_CLEAR: u32 = u32::MAX;

#[derive(Debug)]
pub enum State {
    Ok,
    Sleep,
    Reboot,
    Shutdown,
}

// #[derive(Debug)]
pub struct Core<'a> {
    pub pc: u32,
    reg_file: [i32; 32],
    pub csr_file: [u32; 4096],
    pub memory: &'a mut memory::Memory,

    pub mtime: u64,
    pub mtimecmp: u64,

    trap: u32,
    pub trap_val: u32,
    pub lr_address: u32,
    lr_set: i32,
    pub mode: u32,
    wfi: bool, // wait for interrupt

    pub instr_str: String,
    pub p_start: bool,
}

impl<'a> Core<'a> {
    pub fn new<'b>(memory: &'a mut memory::Memory) -> Self {
        Core {
            pc: 0,
            reg_file: [0; 32],
            csr_file: [0; 4096],
            memory,

            mtime: 0,
            mtimecmp: 0,

            trap: TRAP_CLEAR,
            trap_val: 0,
            lr_address: 0,
            lr_set: 0,
            mode: 0,
            wfi: false,

            instr_str: String::new(),
            p_start: false,
        }
    }

    pub fn read_data(
        &mut self,
        kernel: &str,
        dtb: &str,
    ) -> Result<(), Box<dyn std::error::Error + 'static>> {
        self.mode = 3;

        // kernel
        let data = fs::read(kernel)?;
        for i in 0..data.len() {
            let _ = memory::write_byte(super::RAM_OFFSET + i as u32, data[i], self);
        }

        //8 byte alligned DTB
        let mut dtb_addr = super::RAM_OFFSET + super::RAM_SIZE as u32 - data.len() as u32;
        dtb_addr >>= 3;
        dtb_addr <<= 3;
        let data = fs::read(dtb)?;
        for i in 0..data.len() {
            let _ = memory::write_byte(dtb_addr + i as u32, data[i], self);
            // self.dtb.push(data[i]);
        }
        // while self.dtb.len() % 4 != 0 {
        //     self.dtb.push(0);
        // }

        self.pc = 0x80000000;
        self.reg_file[5] = 0x00001000u32 as i32;
        self.reg_file[10] = 0x00; // hart ID
        self.reg_file[11] = dtb_addr as i32;
        self.reg_file[12] = 0;
        csr::write(Csr::misa, 0b01000000000101000001000100000001, self);
        //                            zyxvwutsrqponmlkjihgfedcba
        //                            Spent a whole week looking for a problem,
        //                            ... I missed q in alphabet.
        csr::write(Csr::menvcfgh, 0b00010000000000000000000000000000, self);
        csr::write(Csr::menvcfg, 0b00000000000000000000000000000000, self);
        csr::write(Csr::marchid, 0x5, self);
        Ok(())
    }

    pub fn exec(&mut self, max_cycles: u32) -> Result<State, Exception> {
        let mut instr_fetch = 0;
        let mut curr_cycle = 0;
        while curr_cycle < max_cycles && self.trap == TRAP_CLEAR {
            curr_cycle += 1;

            if super::DEBUG
                && (self.pc == 0xc0260c44 || csr::read_64(Csr64::mcycle, self) > super::PRINT_START)
                && !self.p_start
            {
                println!("print start!");
                self.p_start = true;
            }

            if super::DEBUG
                && (self.pc == 0x80400094 || csr::read_64(Csr64::mcycle, self) > super::PRINT_START)
            {
                self.mtime = 0xc9f4;
            }

            let mut mip = csr::read(Csr::mip, self);
            if self.mtime >= self.mtimecmp {
                mip |= 0b10000000;
                self.wfi = false;
            } else {
                mip &= !0b10000000;
            }
            csr::write(Csr::mip, mip, self);

            if self.wfi {
                return Ok(State::Sleep);
            }

            // Global interrupt enabled
            let mstatus = csr::read(Csr::mstatus, self);
            let mie = csr::read(Csr::mie, self);
            let mip = csr::read(Csr::mip, self);

            if self.mode == 3 {
                // machine interrupts only taken when MIE is set
                if mstatus & 0b1000 != 0 {
                    if mie & mip & 0b10000000 != 0 {
                        self.trap = 0x80000007;
                    }
                }
                // supervisor interrupts are never taken
            } else if self.mode == 1 {
                // machine interrupts are always taken
                if mie & mip & 0b10000000 != 0 {
                    self.trap = 0x80000007;
                }
                // supervisor interrupts only taken when SIE is set
                if mstatus & 0b10 != 0 {
                    if mie & mip & 0b100000 != 0 {
                        self.trap = 0x80000005;
                    }
                    if mie & mip & 0b10 != 0 {
                        self.trap = 0x80000001;
                    }
                }
            } else if self.mode == 0 {
                // all inerrupts are always taken
                if mie & mip & 0b10000000 != 0 {
                    self.trap = 0x80000007;
                }
                if mie & mip & 0b100000 != 0 {
                    self.trap = 0x80000005;
                }
                if mie & mip & 0b10 != 0 {
                    self.trap = 0x80000001;
                }
            }

            if self.trap == TRAP_CLEAR {
                if self.pc & 0b11 > 0 {
                    // check instruction address aligment
                    self.trap = 0;
                } else {
                    if (csr::read(Csr::mcountinhibit, self) & 0b1) == 0 {
                        let cycle = csr::read_64(Csr64::mcycle, self);
                        csr::write_64(Csr64::mcycle, cycle + 1, self);
                    }

                    match memory::fetch_word(self.pc, self) {
                        Ok(fetch_result) => {
                            instr_fetch = fetch_result;

                            if self.p_start {
                                self.instr_str = format!(
                                    "core   0: {} 0x{:08x?} (0x{:08x?})\t",
                                    self.mode, self.pc, instr_fetch
                                );
                            }

                            match Instruction::from(fetch_result) {
                                Ok(instr) => {
                                    let ret = match instr {
                                        Instruction::R(x) => datapath::exec_r(self, &x),
                                        Instruction::I(x) => datapath::exec_i(self, &x),
                                        Instruction::U(x) => datapath::exec_u(self, &x),
                                        Instruction::J(x) => datapath::exec_j(self, &x),
                                        Instruction::S(x) => datapath::exec_s(self, &x),
                                        Instruction::B(x) => datapath::exec_b(self, &x),
                                    };
                                    match ret {
                                        Ok(_) => {}
                                        Err(e) => self.trap = exception_number(e),
                                    };
                                }
                                Err(e) => self.trap = exception_number(e),
                            };
                        }
                        Err(e) => self.trap = exception_number(e),
                    };

                    if self.p_start && self.trap == TRAP_CLEAR {
                        if instr_fetch != 0x00000073 {
                            if !super::SPIKE_DEBUG {
                                print!(
                                    "core   0: {} 0x{:x?} (0x{:08x?})\t",
                                    self.mode, self.pc, instr_fetch
                                );
                                eprintln!("0x{:08x?}: 0x{:08x?}", self.pc, instr_fetch);
                            } else {
                                eprintln!("{}", self.instr_str);
                            }
                        }
                    }
                }
                self.reg_file[0] = 0;
            }
        }

        if self.trap != TRAP_CLEAR {
            // if self.trap != 2 && self.trap != 9 && self.trap != 0x80000007 && self.trap != 0x80000005 {
            // println!("it's a trap 0x{:x} trap_val 0x{:x}; mtime 0x{:x}; mode:{}; instr *0x{:08x}=0x{:08x}", self.trap,
            //    self.trap_val, self.mtime, self.mode, self.pc, instr_fetch);
            // }
            if self.trap == 2 {
                self.trap_val = instr_fetch;
            }
            if (self.trap as i32) < 0 {
                //interrupt
                let mideleg = csr::read(Csr::mideleg, self);
                if (1 << (self.trap - 0x80000000)) & mideleg > 0 && self.mode < 3 {
                    self.s_mode_trap_handler();
                } else {
                    self.m_mode_trap_handler();
                }
            } else {
                // exception
                let medeleg = csr::read(Csr::medeleg, self);
                if (1 << self.trap) & medeleg > 0 && self.mode < 3 {
                    self.s_mode_trap_handler();
                } else {
                    self.m_mode_trap_handler();
                }
            }
        } else {
            if (csr::read(Csr::mcountinhibit, self) & 0b100) == 0 {
                let minstret = csr::read_64(Csr64::minstret, self);
                csr::write_64(Csr64::minstret, minstret + 1, self);
            }
        }

        Ok(State::Ok)
    }

    fn m_mode_trap_handler(&mut self) {
        // Machine mode trap handler
        // println!("mmode trap");
        if super::DEBUG {
            // print!("o {:x} ", self.trap);
            // println!("mmode trap");
        }
        if (self.trap as i32) < 0 {
            // interrupt
            csr::write(Csr::mcause, self.trap, self);
            csr::write(Csr::mtval, 0, self);
        } else {
            // exception
            csr::write(Csr::mcause, self.trap, self);
            // address:
            // breakpoint (3); address-misaligned (0, 4, 6);
            // access-fault (1, 5, 7); page-fault(12, 13, 15);
            // faulting instruction:
            // instruction fault (2)
            match self.trap {
                0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 12 | 13 | 15 => {
                    csr::write(Csr::mtval, self.trap_val, self)
                }
                _ => csr::write(Csr::mtval, 0, self),
            };
        }

        let mstatus = csr::read(Csr::mstatus, self);
        // save mode into mpp
        let mpp = (self.mode & 0b11) << 11;
        // save mie into mpie
        let mpie = (mstatus & 0b1000) << 4;
        // zero mpp and mpie fields
        let mut mstatus = mstatus & !0b1100010000000;
        mstatus |= mpp;
        mstatus |= mpie;
        // disable interrupts
        mstatus &= !0b1000;
        csr::write(Csr::mstatus, mstatus, self);

        // save pc
        csr::write(Csr::mepc, self.pc, self);
        // jump to handler
        let mtvec = csr::read(Csr::mtvec, self);
        // if mtvec & 0b11 != 0 {
        //     println!("mtvec vectored mode")
        // }
        match mtvec & 0b11 {
            0 => self.pc = mtvec,
            1 => {
                if (self.trap as i32) < 0 {
                    // interrupt
                    self.pc = (mtvec >> 2) << 2;
                    self.pc += 4 * ((self.trap << 1) >> 1);
                } else {
                    // exception
                    self.pc = (mtvec >> 2) << 2;
                }
            }
            _ => self.pc = 0,
        }

        // enter machine mode
        self.mode = 3;
        // clear trap
        self.trap = TRAP_CLEAR;
    }

    fn s_mode_trap_handler(&mut self) {
        // Supervisor mode trap handler
        // println!("smode trap");
        if super::DEBUG {
            // print!("o {:x} ", self.trap);
            // println!("smode trap");
        }
        if (self.trap as i32) < 0 {
            // interrupt
            csr::write(Csr::scause, self.trap, self);
            csr::write(Csr::stval, 0, self);
        } else {
            // exception
            csr::write(Csr::scause, self.trap, self);
            // breakpoint (3); address-misaligned (0, 4, 6);
            // access-fault (1, 5, 7); page-fault(12, 13, 15)
            match self.trap {
                0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 12 | 13 | 15 => {
                    csr::write(Csr::stval, self.trap_val, self)
                }
                _ => csr::write(Csr::stval, 0, self),
            };
        }

        let mstatus = csr::read(Csr::mstatus, self);
        // save mode into spp
        let spp = (self.mode & 0b1) << 8;
        // save sie into spie
        let spie = (mstatus & (0b10)) << 4;
        // zero spp and spie fields
        let mut mstatus = mstatus & !0b100100000;
        mstatus |= spp;
        mstatus |= spie;
        // disable interrupts
        mstatus &= !0b10;
        csr::write(Csr::mstatus, mstatus, self);

        // save pc
        csr::write(Csr::sepc, self.pc, self);
        // jump to handler
        let stvec = csr::read(Csr::stvec, self);
        match stvec & 0b11 {
            0 => self.pc = stvec,
            1 => {
                if (self.trap as i32) < 0 {
                    // interrupt
                    self.pc = (stvec >> 2) << 2;
                    self.pc += 4 * ((self.trap << 1) >> 1);
                } else {
                    // exception
                    self.pc = (stvec >> 2) << 2;
                }
            }
            _ => self.pc = 0,
        }

        // enter supervisor mode
        self.mode = 1;
        // clear trap
        self.trap = TRAP_CLEAR;
    }
}


