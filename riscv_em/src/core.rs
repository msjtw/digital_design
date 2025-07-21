mod csr;
mod datapath;
pub mod instr_parse;

use crate::memory::{self};
use csr::{Csr, Csr64};
use instr_parse::{Instruction, InstructionError};
use std::{error::Error, fmt, fs, u32};

#[derive(Debug)]
pub enum State {
    Ok,
    Sleep,
    Reboot,
    Shutdown,
}

#[derive(Debug)]
pub enum ExecError {
    Error,
    InstructionError(InstructionError),
    DataReadError,
    End,
}

impl From<InstructionError> for ExecError {
    fn from(err: InstructionError) -> Self {
        Self::InstructionError(err)
    }
}

impl From<object::Error> for ExecError {
    fn from(_value: object::Error) -> Self {
        Self::DataReadError
    }
}

impl Error for ExecError {}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Error => write!(f, "Error!"),
            Self::DataReadError => write!(f, "Error reading data from ELF!"),
            Self::InstructionError(x) => write!(f, "Instruction Error {x}"),
            Self::End => write!(f, "End of execution!"),
        }
    }
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
    trap_val: u32,
    is_trap: bool,
    lr_address: u32,
    lr_valid: bool,
    mode: u32,
    wfi: bool, // wait for interrupt
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

            trap: 0,
            trap_val: 0,
            is_trap: false,
            lr_address: 0,
            lr_valid: false,
            mode: 0,
            wfi: false,
        }
    }

    pub fn read_data(
        &mut self,
        kernel: &str,
        dtb: &str,
    ) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let data = fs::read(kernel)?;
        for i in 0..data.len() {
            let _ = memory::write_byte(i as u32 + super::RAM_OFFSET, data[i], self);
        }

        let data = fs::read(dtb)?;
        for i in 0..data.len() {
            let _ = memory::write_byte(
                super::RAM_OFFSET + super::RAM_SIZE as u32 - data.len() as u32 + i as u32,
                data[i],
                self
            );
        }

        self.pc = super::RAM_OFFSET;
        self.reg_file[10] = 0x00; // hart ID
        self.reg_file[11] = (super::RAM_OFFSET + super::RAM_SIZE as u32 - data.len() as u32) as i32;
        self.mode = 3;
        csr::write(Csr::misa, 0b01000000000010000001000100000001, self);
        //                             zyxvwutsrponmlkjihgfedcba
        Ok(())
    }


    pub fn exec(&mut self) -> Result<State, ExecError> {
        // if super::DEBUG {
        //     print!("|");
        // }

        let mut mip = csr::read(Csr::mip, self);
        if self.mtime > self.mtimecmp {
            mip |= 1 << 7;
            self.wfi = false;
        } else {
            mip &= !(1 << 7);
        }
        csr::write(Csr::mip, mip, self);

        if self.wfi {
            return Ok(State::Sleep);
        }

        // Global interrupt enabled
        let mstatus = csr::read(Csr::mstatus, self);
        let mie = csr::read(Csr::mie, self);
        let mip = csr::read(Csr::mip, self);
        if (mstatus & 1 << 3) != 0 && (mie & 1 << 7) != 0 && (mip & 1 << 7) != 0 {
            // machine timer interrupt
            self.trap = 0x80000007;
            self.is_trap = true;
        } else {
            if self.pc & 0b11 > 0 {
                // check instruction address aligment
                self.trap = 0;
                self.is_trap = true;
            } else {
                let cycle = csr::read_64(Csr64::mcycle, self);
                csr::write_64(Csr64::mcycle, cycle+1, self);

                let memory_result = memory::fetch_word(self.pc, self);
                if super::DEBUG && csr::read_64(Csr64::mcycle, self) > super::PRINT_START {
                    print_state(self);
                    println!(
                        "0x{:x?}: 0x{:x?}",
                        self.pc,
                        memory_result.unwrap()
                    );
                }
                if let Ok(byte_code) = memory_result {
                    let instr = Instruction::from(byte_code)?;
                    let ret = match instr {
                        Instruction::R(x) => datapath::exec_r(self, &x),
                        Instruction::I(x) => datapath::exec_i(self, &x),
                        Instruction::U(x) => datapath::exec_u(self, &x),
                        Instruction::J(x) => datapath::exec_j(self, &x),
                        Instruction::S(x) => datapath::exec_s(self, &x),
                        Instruction::B(x) => datapath::exec_b(self, &x),
                    };
                    match ret {
                        Ok(State::Ok) => {}
                        Ok(x) => return Ok(x),
                        Err(ExecError::InstructionError(InstructionError::NoInstruction))
                        | Err(ExecError::InstructionError(InstructionError::NotSupported)) => {
                            println!("Not supported: 0x{:x}", byte_code);
                            self.trap = 2;
                            self.is_trap = true;
                        }
                        Err(x) => return Err(x),
                    };
                    self.reg_file[0] = 0;
                } else {
                    // Instruction access fault
                    self.trap = 1;
                    self.is_trap = true;
                }
            }
        }

        if self.is_trap {
            // println!("it's a trap");
            // println!("{}", self.trap);
            if (self.trap as i32) < 0 {
                //interrupt
                let mideleg = csr::read(Csr::mideleg, self);
                if self.trap & mideleg > 0{
                    self.s_mode_trap_handler();
                }
                else {
                    self.m_mode_trap_handler();
                }
            }
            else{
                // exception
                let medeleg = csr::read(Csr::medeleg, self);
                if self.trap & medeleg > 0{
                    self.s_mode_trap_handler();
                }
                else {
                    self.m_mode_trap_handler();
                }
            }
        }
        else{
            let minstret = csr::read_64(Csr64::minstret, self);
            csr::write_64(Csr64::minstret, minstret+1, self);
        }

        Ok(State::Ok)
    }

    fn m_mode_trap_handler(&mut self){
        // Machine mode trap handler
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
            if self.trap > 4 && self.trap <= 7 {
                // address misaligned, access fault, ecall
                csr::write(Csr::mtval, self.trap_val, self);
            } else {
                csr::write(Csr::mtval, self.pc, self);
            }
        }

        let mstatus = csr::read(Csr::mstatus, self);
        // save mode into mpp
        let mpp = (self.mode & 0b11) << 11;
        // save mie into mpie
        let mpie = (mstatus & (1 << 3)) << 4;
        // zero mpp and mpie fields
        let mut mstatus = mstatus & !((0b11 << 11) | (0b1 << 7));
        mstatus |= mpp;
        mstatus |= mpie;
        // disable interrupts
        mstatus &= !0b1000;
        csr::write(Csr::mstatus, mstatus, self);

        // save pc
        csr::write(Csr::mepc, self.pc, self);
        // jump to handler
        self.pc = csr::read(Csr::mtvec, self);

        // enter machine mode
        self.mode = 3;
        // clear trap
        self.trap = 0;
        self.is_trap = false;
    }

    fn s_mode_trap_handler(&mut self){
        // Supervisor mode trap handler
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
            if self.trap > 4 && self.trap <= 7 {
                // address misaligned, access fault, ecall
                csr::write(Csr::stval, self.trap_val, self);
            } else {
                csr::write(Csr::stval, self.pc, self);
            }
        }

        let sstatus = csr::read(Csr::sstatus, self);
        // save mode into spp
        let spp = (self.mode & 0b1) << 8;
        // save sie into spie
        let spie = (sstatus & (0b01)) << 4;
        // zero spp and spie fields
        let mut sstatus = sstatus & !((0b1 << 8) | (0b1 << 5));
        sstatus |= spp;
        sstatus |= spie;
        // disable interrupts
        sstatus &= !0b10;
        csr::write(Csr::sstatus, sstatus, self);

        // save pc
        csr::write(Csr::sepc, self.pc, self);
        // jump to handler
        self.pc = csr::read(Csr::stvec, self);

        // enter supervisor mode
        self.mode = 1;
        // clear trap
        self.trap = 0;
        self.is_trap = false;
    }

}
pub fn print_state(core: &Core)  {
    println!(
        "Z:{:08x} ra:{:08x} sp:{:08x} gp:{:08x} tp:{:08x} t0:{:08x} t1:{:08x} t2:{:08x} s0:{:08x} s1:{:08x} a0:{:08x} a1:{:08x} a2:{:08x} a3:{:08x} a4:{:08x} a5:{:08x} a6:{:08x} a7:{:08x} s2:{:08x} s3:{:08x} s4:{:08x} s5:{:08x} s6:{:08x} s7:{:08x} s8:{:08x} s9:{:08x} s10:{:08x} s11:{:08x} t3:{:08x} t4:{:08x} t5:{:08x} t6:{:08x}",
        core.reg_file[0] as u32,
        core.reg_file[1] as u32,
        core.reg_file[2] as u32,
        core.reg_file[3] as u32,
        core.reg_file[4] as u32,
        core.reg_file[5] as u32,
        core.reg_file[6] as u32,
        core.reg_file[7] as u32,
        core.reg_file[8] as u32,
        core.reg_file[9] as u32,
        core.reg_file[10] as u32,
        core.reg_file[11] as u32,
        core.reg_file[12] as u32,
        core.reg_file[13] as u32,
        core.reg_file[14] as u32,
        core.reg_file[15] as u32,
        core.reg_file[16] as u32,
        core.reg_file[17] as u32,
        core.reg_file[18] as u32,
        core.reg_file[19] as u32,
        core.reg_file[20] as u32,
        core.reg_file[21] as u32,
        core.reg_file[22] as u32,
        core.reg_file[23] as u32,
        core.reg_file[24] as u32,
        core.reg_file[25] as u32,
        core.reg_file[26] as u32,
        core.reg_file[27] as u32,
        core.reg_file[28] as u32,
        core.reg_file[29] as u32,
        core.reg_file[30] as u32,
        core.reg_file[31] as u32,
    );
}
