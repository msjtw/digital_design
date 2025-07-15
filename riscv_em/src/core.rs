mod csr_file;
mod datapath;
pub mod instr_parse;

use crate::memory::{self};
use csr_file::{CSR_file, Csr};
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
    csr_file: CSR_file,
    pub memory: &'a mut memory::Memory,

    pub inst_count: u64,
    trap: u32,
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
            csr_file: CSR_file::default(),
            memory,

            inst_count: 0,

            trap: 0,
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
            let _ = self
                .memory
                .insert_byte(i as u32 + super::RAM_OFFSET, data[i]);
        }

        let data = fs::read(dtb)?;
        for i in 0..data.len() {
            let _ = self.memory.insert_byte(
                super::RAM_OFFSET + super::RAM_SIZE as u32 - data.len() as u32 + i as u32,
                data[i],
            );
        }

        self.pc = super::RAM_OFFSET;
        self.reg_file[10] = 0x00; // hart ID
        self.reg_file[11] = (super::RAM_OFFSET + super::RAM_SIZE as u32 - data.len() as u32) as i32;
        self.mode = 3;
        self.csr_file
            .write(Csr::misa, 0b01000000000000000001000100000001, self.mode);
        Ok(())
    }

    pub fn print_reg_file(&self) -> String {
        format!(
            "Z:{:08x} ra:{:08x} sp:{:08x} gp:{:08x} tp:{:08x} t0:{:08x} t1:{:08x} t2:{:08x} s0:{:08x} s1:{:08x} a0:{:08x} a1:{:08x} a2:{:08x} a3:{:08x} a4:{:08x} a5:{:08x} a6:{:08x} a7:{:08x} s2:{:08x} s3:{:08x} s4:{:08x} s5:{:08x} s6:{:08x} s7:{:08x} s8:{:08x} s9:{:08x} s10:{:08x} s11:{:08x} t3:{:08x} t4:{:08x} t5:{:08x} t6:{:08x}",
            self.reg_file[0] as u32,
            self.reg_file[1] as u32,
            self.reg_file[2] as u32,
            self.reg_file[3] as u32,
            self.reg_file[4] as u32,
            self.reg_file[5] as u32,
            self.reg_file[6] as u32,
            self.reg_file[7] as u32,
            self.reg_file[8] as u32,
            self.reg_file[9] as u32,
            self.reg_file[10] as u32,
            self.reg_file[11] as u32,
            self.reg_file[12] as u32,
            self.reg_file[13] as u32,
            self.reg_file[14] as u32,
            self.reg_file[15] as u32,
            self.reg_file[16] as u32,
            self.reg_file[17] as u32,
            self.reg_file[18] as u32,
            self.reg_file[19] as u32,
            self.reg_file[20] as u32,
            self.reg_file[21] as u32,
            self.reg_file[22] as u32,
            self.reg_file[23] as u32,
            self.reg_file[24] as u32,
            self.reg_file[25] as u32,
            self.reg_file[26] as u32,
            self.reg_file[27] as u32,
            self.reg_file[28] as u32,
            self.reg_file[29] as u32,
            self.reg_file[30] as u32,
            self.reg_file[31] as u32,
        )
    }

    pub fn exec(&mut self, last_op_time: u64) -> Result<State, ExecError> {
        self.inst_count += 1;
        // if super::DEBUG {
        //     print!("|");
        // }

        let mtime = self.memory.csr_read(memory::Time::Mtime) + last_op_time;
        self.memory.csr_write(memory::Time::Mtime, mtime).unwrap();

        let mtimecmp = self.memory.csr_read(memory::Time::Mtimecmp);

        let mut mip = self.csr_file.read(Csr::mip, self.mode);
        if mtime > mtimecmp {
            mip |= 1 << 7;
            self.wfi = false;
        } else {
            mip &= !(1 << 7);
        }
        self.csr_file.write(Csr::mip, mip, self.mode);

        if self.wfi {
            return Ok(State::Sleep);
        }

        let mut rd = 0;
        // Global interrupt enabled
        let mstatus = self.csr_file.read(Csr::mstatus, self.mode);
        let mie = self.csr_file.read(Csr::mie, self.mode);
        let mip = self.csr_file.read(Csr::mip, self.mode);
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
                let cycle = self.csr_file.read_mcycle();
                self.csr_file.write_mcycle(cycle+1);

                let memory_result = self.memory.get_word(self.pc);
                if super::DEBUG && self.memory.csr_read(memory::Time::Mtime) > super::PRINT_START {
                    println!("{}", self.print_reg_file());
                    println!(
                        "mstatus:{} {:x} {:?}: {:?}",
                        self.csr_file.read(Csr::mstatus, 3),
                        mtimecmp.max(mtime) - mtime,
                        self.pc,
                        memory_result.unwrap()
                    );
                }
                if let Ok(byte_code) = memory_result {
                    let instr = Instruction::from(byte_code)?;
                    let ret = match instr {
                        Instruction::R(x) => {
                            rd = x.rd;
                            datapath::exec_r(self, &x)
                        }
                        Instruction::I(x) => {
                            rd = x.rd;
                            datapath::exec_i(self, &x)
                        }
                        Instruction::U(x) => {
                            rd = x.rd;
                            datapath::exec_u(self, &x)
                        }
                        Instruction::J(x) => {
                            rd = x.rd;
                            datapath::exec_j(self, &x)
                        }
                        Instruction::S(x) => datapath::exec_s(self, &x),
                        Instruction::B(x) => datapath::exec_b(self, &x),
                    };
                    match ret {
                        Ok(State::Ok) => {}
                        Ok(x) => return Ok(x),
                        Err(ExecError::InstructionError(InstructionError::NoInstruction))
                        | Err(ExecError::InstructionError(InstructionError::NotSupported)) => {
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
            // Machine mode trap handler
            if super::DEBUG {
                // print!("o {:x} ", self.trap);
            }
            if (self.trap as i32) < 0 {
                // interrupt
                self.csr_file.write(Csr::mcause, self.trap, self.mode);
                self.csr_file.write(Csr::mtval, 0, self.mode);
            } else {
                // trap
                self.csr_file.write(Csr::mcause, self.trap, self.mode);
                if self.trap > 4 && self.trap <= 7 {
                    // address misaligned, access fault, ecall
                    self.csr_file.write(Csr::mtval, self.reg_file[rd as usize] as u32, self.mode);
                } else {
                    self.csr_file.write(Csr::mtval, self.pc, self.mode);
                }
            }

            let mstatus = self.csr_file.read(Csr::mstatus, self.mode);
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
            self.csr_file.write(Csr::mstatus, mstatus, self.mode);

            // save pc
            self.csr_file.write(Csr::mepc, self.pc, self.mode);
            // jump to handler
            self.pc = self.csr_file.read(Csr::mtvec, self.mode);

            // enter machine mode
            self.mode = 3;
            // clear trap
            self.trap = 0;
            self.is_trap = false;

        }

        Ok(State::Ok)
    }


}
