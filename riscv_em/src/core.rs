pub mod csr;
mod datapath;
pub mod exceptions;
mod instr_parse;
mod virt_memory;

use crate::{
    core::csr::conuters_mirror,
    memory::{clint::Clint, *},
};
use csr::{Csr, Csr64};
use exceptions::*;
use instr_parse::Instruction;
use std::{fs, u32};

const TRAP_CLEAR: u32 = u32::MAX;

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Ok,
    Sleep,
    // Reboot,
    // Shutdown,
}

pub struct Hart {
    pub core: Core,
    pub clint: Clint,
}

#[derive(Debug)]
pub struct Core {
    pub pc: u32,
    reg_file: [i32; 32],
    pub csr_file: [u32; 4096],

    trap: u32,
    pub trap_val: u32,
    pub lr_address: u32,
    lr_set: i32,
    pub mode: u32,
    pub wfi: bool, // wait for interrupt

    instr_fetch: u32,
    pub instr_str: String,
    pub p_start: bool,
}

impl Default for Core {
    fn default() -> Self {
        Core {
            pc: 0,
            reg_file: [0; 32],
            csr_file: [0; 4096],

            trap: TRAP_CLEAR,
            trap_val: 0,
            lr_address: 0,
            lr_set: 0,
            mode: 0,
            wfi: false,

            instr_fetch: 0,
            instr_str: String::new(),
            p_start: false,
        }
    }
}

impl Core {
    fn m_mode_trap_handler(&mut self) {
        // Machine mode trap handler
        // println!("mmode trap");
        if super::DEBUG {
            // print!("o {:x} ", core.trap);
            // println!("mmode trap");
        }
        if (self.trap as i32) < 0 {
            // interrupt
            csr::write(Csr::mcause, self.trap, self);
            csr::write(Csr::mtval, 0, self);
        } else {
            // exception
            csr::write(Csr::mcause, self.trap, self);
            // addressself
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
            // print!("o {:x} ", core.trap);
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

    fn check_interrupts(&mut self) {
        let mstatus = csr::read(Csr::mstatus, self);
        let mie = csr::read(Csr::mie, self);
        let mip = csr::read(Csr::mip, self);

        if self.mode == 3 {
            // machine interrupts only taken when MIE is set
            if mstatus & 0b1000 != 0 {
                if mie & mip & (1 << 11) != 0 {
                    self.trap = 0x8000000b;
                }
                if mie & mip & (1 << 7) != 0 {
                    self.trap = 0x80000007;
                }
                if mie & mip & (1 << 3) != 0 {
                    self.trap = 0x80000003;
                }
            }
            // supervisor interrupts are never taken
        } else if self.mode == 1 {
            // machine interrupts are always taken
            if mie & mip & (1 << 11) != 0 {
                self.trap = 0x8000000b;
            }
            if mie & mip & (1 << 7) != 0 {
                self.trap = 0x80000007;
            }
            if mie & mip & (1 << 3) != 0 {
                self.trap = 0x80000003;
            }
            // supervisor interrupts only taken when SIE is set
            if mstatus & 0b10 != 0 {
                if mie & mip & (1 << 9) != 0 {
                    self.trap = 0x80000009;
                }
                if mie & mip & (1 << 5) != 0 {
                    self.trap = 0x80000005;
                }
                if mie & mip & (1 << 1) != 0 {
                    self.trap = 0x80000001;
                }
            }
        } else if self.mode == 0 {
            // all inerrupts are always taken
            if mie & mip & (1 << 11) != 0 {
                self.trap = 0x8000000b;
            }
            if mie & mip & (1 << 9) != 0 {
                self.trap = 0x80000009;
            }
            if mie & mip & (1 << 7) != 0 {
                self.trap = 0x80000007;
            }
            if mie & mip & (1 << 5) != 0 {
                self.trap = 0x80000005;
            }
            if mie & mip & (1 << 3) != 0 {
                self.trap = 0x80000003;
            }
            if mie & mip & (1 << 1) != 0 {
                self.trap = 0x80000001;
            }
        }
    }
}

pub fn soc_init(
    hart: &mut Hart,
    bus: &mut MemoryBus,
    kernel: &str,
    dtb: &str,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    hart.core.mode = 3;

    // kernel
    let data = fs::read(kernel)?;
    for i in 0..data.len() {
        let _ = virt_memory::virt_write_byte(super::RAM_OFFSET + i as u32, data[i], hart, bus);
    }

    //8 byte alligned DTB
    let data = fs::read(dtb)?;
    let mut dtb_addr = super::RAM_OFFSET + super::RAM_SIZE as u32 - data.len() as u32;
    dtb_addr >>= 3;
    dtb_addr <<= 3;
    println!("dtb addr 0x{:08x}", dtb_addr);
    for i in 0..data.len() {
        let _ = virt_memory::virt_write_byte(dtb_addr + i as u32, data[i], hart, bus);
        // self.dtb.push(data[i]);
    }
    // while self.dtb.len() % 4 != 0 {
    //     self.dtb.push(0);
    // }

    hart.core.pc = 0x80000000;
    hart.core.reg_file[5] = 0x00001000u32 as i32;
    hart.core.reg_file[10] = 0x00; // hart ID
    hart.core.reg_file[11] = dtb_addr as i32;
    hart.core.reg_file[12] = 0;
    csr::write(
        Csr::misa,
        0b01000000000101000001000100000001,
        &mut hart.core,
    );
    //                            zyxvwutsrqponmlkjihgfedcba
    //                            Spent a whole week looking for a problem,
    //                            ... I missed q in alphabet.
    csr::write(
        Csr::menvcfgh,
        0b00010000000000000000000000000000,
        &mut hart.core,
    );
    csr::write(
        Csr::menvcfg,
        0b00000000000000000000000000000000,
        &mut hart.core,
    );
    csr::write(Csr::marchid, 0x5, &mut hart.core);
    Ok(())
}

pub fn hart_run(hart: &mut Hart, bus: &mut MemoryBus, max_cycles: u32) -> State {
    // TODO: devices tick
    hart.clint.tick(&mut hart.core);
    bus.uart.tick(&mut bus.plic);
    // bus.blk.tick(&);
    bus.plic.tick(&mut hart.core);

    if hart.core.wfi {
        return State::Sleep;
    }

    match tick(hart, bus, max_cycles) {
        Ok(State::Ok) => {}
        Ok(x) => {
            return x;
        }
        Err(_) => {
            if hart.core.trap != TRAP_CLEAR {
                // if core.trap == 0x80000009 {
                eprintln!(
                    "it's a trap 0x{:x} trap_val 0x{:x}; mtime 0x{:x}; mode:{}; instr *0x{:08x}=0x{:08x}",
                    hart.core.trap, hart.core.trap_val, hart.clint.mtime, hart.core.mode, hart.core.pc, hart.core.instr_fetch
                );
                // }
                if hart.core.trap == 2 {
                    hart.core.trap_val = hart.core.instr_fetch;
                }
                if (hart.core.trap as i32) < 0 {
                    //interrupt
                    let mideleg = csr::read(Csr::mideleg, &mut hart.core);
                    if (1 << (hart.core.trap - 0x80000000)) & mideleg > 0 && hart.core.mode < 3 {
                        hart.core.s_mode_trap_handler();
                    } else {
                        hart.core.m_mode_trap_handler();
                    }
                } else {
                    // exception
                    let medeleg = csr::read(Csr::medeleg, &mut hart.core);
                    if (1 << hart.core.trap) & medeleg > 0 && hart.core.mode < 3 {
                        hart.core.s_mode_trap_handler();
                    } else {
                        hart.core.m_mode_trap_handler();
                    }
                }
            }
        }
    }
    State::Ok
}

fn tick(hart: &mut Hart, bus: &mut MemoryBus, max_cycles: u32) -> Result<State, ()> {
    let mut curr_cycle = 0;

    while curr_cycle < max_cycles {
        hart.core.check_interrupts();
        csr::conuters_mirror(hart);

        if hart.core.trap != TRAP_CLEAR {
            return Err(());
        }

        hart.core.reg_file[0] = 0;
        curr_cycle += 1;

        if super::DEBUG {
            if (hart.core.pc == 0x80400094
                || csr::read_64(Csr64::mcycle, &hart.core) > super::PRINT_START)
                && !hart.core.p_start
            {
                println!("print start!");
                hart.core.p_start = true;
            }

            // if hart.core.pc == 0x80400094 {
            //     clint.mtime = 0xc9f4;
            // }
        }

        if hart.core.pc & 0b11 > 0 {
            // check instruction address alignment
            // TODO: move this check to datapath
            hart.core.trap = 0;
            return Err(());
        } else {
            if (csr::read(Csr::mcountinhibit, &hart.core) & 0b1) == 0 {
                let cycle = csr::read_64(Csr64::mcycle, &hart.core);
                csr::write_64(Csr64::mcycle, cycle + 1, &mut hart.core);
            }

            match virt_memory::virt_fetch_word(hart.core.pc, hart, bus) {
                Ok(fetch_result) => {
                    hart.core.instr_fetch = fetch_result;

                    if hart.core.p_start {
                        hart.core.instr_str = format!(
                            "core   0: {} 0x{:08x?} (0x{:08x?})\t",
                            hart.core.mode, hart.core.pc, hart.core.instr_fetch
                        );
                    }

                    match Instruction::from(fetch_result) {
                        Ok(instr) => {
                            let ret = match instr {
                                Instruction::R(x) => datapath::exec_r(hart, bus, &x),
                                Instruction::I(x) => datapath::exec_i(hart, bus, &x),
                                Instruction::U(x) => datapath::exec_u(&mut hart.core, &x),
                                Instruction::J(x) => datapath::exec_j(&mut hart.core, &x),
                                Instruction::S(x) => datapath::exec_s(hart, bus, &x),
                                Instruction::B(x) => datapath::exec_b(&mut hart.core, &x),
                            };
                            match ret {
                                Ok(State::Ok) => {}
                                Ok(x) => {
                                    return Ok(x);
                                }
                                Err(e) => {
                                    hart.core.trap = exception_number(&e);
                                    return Err(());
                                }
                            };
                        }
                        Err(e) => {
                            hart.core.trap = exception_number(&e);
                            return Err(());
                        }
                    };
                }
                Err(e) => {
                    hart.core.trap = exception_number(&e);
                    return Err(());
                }
            };
            if (csr::read(Csr::mcountinhibit, &hart.core) & 0b100) == 0 {
                let minstret = csr::read_64(Csr64::minstret, &hart.core);
                csr::write_64(Csr64::minstret, minstret + 1, &mut hart.core);
            }

            if hart.core.p_start {
                if hart.core.instr_fetch != 0x00000073 {
                    if !super::SPIKE_DEBUG {
                        print!(
                            "hart.core   0: {} 0x{:x?} (0x{:08x?})\t",
                            hart.core.mode, hart.core.pc, hart.core.instr_fetch
                        );
                        eprintln!("0x{:08x?}: 0x{:08x?}", hart.core.pc, hart.core.instr_fetch);
                    } else {
                        eprintln!("{}", hart.core.instr_str);
                    }
                }
            }
        }
    }
    Ok(State::Ok)
}
