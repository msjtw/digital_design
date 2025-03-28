use object::{Object, ObjectSegment};
use std::{error::Error, fmt, fs, u32};

pub mod instr_parse;
use crate::memory::{self, Memory};
use instr_parse::{Instruction, InstructionError};

mod datapath;
mod syscalls;

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

#[derive(Debug)]
pub enum Csr {
    Mscratch,
    Mtvec,
    Mie,
    Cycle,
    Cycleh,
    Mip,
    Mepc,
    Mstatus,
    Mcause,
    Mtval,
    Misa,
    Mvendorid,
}

#[derive(Debug)]
pub struct Core<'a> {
    pc: u32,
    reg_file: [i32; 32],
    csr_file: [u32; 4096],
    memory: &'a mut memory::Memory,

    trap: u32,
    lr_address: u32,
    lr_valid: bool,
    mode: u32,
    wfi: bool, // wait for interrupt
}

impl<'a> Core<'a> {
    pub fn new<'b>(memory: &'a mut memory::Memory) -> Self {
        let c = Core {
            pc: 0,
            reg_file: [0; 32],
            csr_file: [0; 4096],
            memory,

            trap: 0,
            lr_address: 0,
            lr_valid: false,
            mode: 0,
            wfi: false,
        };
        c
    }
    pub fn read_data(
        &mut self,
        kernel: &str,
        dtb: &str,
    ) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let data = fs::read(kernel)?;
        for i in 0..data.len() {
            self.memory
                .insert_byte(i as u32 + super::RAM_OFFSET, data[i]);
        }

        let data = fs::read(dtb)?;
        for i in 0..data.len() {
            self.memory.insert_byte(
                super::RAM_OFFSET + super::RAM_SIZE as u32 - data.len() as u32 + i as u32,
                data[i],
            );
        }

        self.pc = super::RAM_OFFSET;
        self.reg_file[10] = 0x00; // hart ID
        self.reg_file[11] = (super::RAM_OFFSET + super::RAM_SIZE as u32 - data.len() as u32) as i32;
        self.mode = 3;
        Ok(())
    }

    fn print_reg_file(&self) {
        for i in 0..32 {
            print!("{:6}", i);
        }
        println!("");
        println!(
            "  zero    ra    sp    gp    tp    t0    t1    t2    s0    s1    a0    a1    a2    a3    a4    a5    a6    a7    s2    s3    s4    s5    s6    s7    s8    s9   s10   s11    t3    t4    t5    t6"
        );
        for i in 0..32 {
            if self.reg_file[i].to_string().len() > 5 {
                print!("    --");
            } else {
                print!("{:6}", self.reg_file[i]);
            }
        }
        println!("");
    }

    pub fn exec(&mut self, last_op_time: u64) -> Result<State, ExecError> {
        let mtime = self.memory.csr_read(memory::Csr::Mtime) + last_op_time;
        self.memory.csr_write(memory::Csr::Mtime, mtime);
        let mtimecmp = self.memory.csr_read(memory::Csr::Mtimecmp);

        if mtime > mtimecmp {
            *self.csr(Csr::Mip) |= 1 << 7;
        } else {
            *self.csr(Csr::Mip) &= !(1 << 7);
        }

        if self.wfi {
            return Ok(State::Sleep);
        }

        let mut rd = 0;
        if (*self.csr(Csr::Mstatus) & 1 << 3) != 0 {
            // Global interrupt enabled
            if ((*self.csr(Csr::Mie) & 1 << 7) & (*self.csr(Csr::Mip) & 1 << 7)) != 0 {
                // machine timer interrupt
            }
        } else {
            let byte_code = self.memory.get_word(self.pc);
            let instr = Instruction::from(byte_code)?;
            match instr {
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
            }?;
            self.reg_file[0] = 0;
        }

        if self.trap != 0 {
            if self.trap & 1 << 31 != 0 {
                // interrupt
                *self.csr(Csr::Mcause) = self.trap;
                *self.csr(Csr::Mtval) = 0;
            } else {
                // trap
                *self.csr(Csr::Mcause) = self.trap;
                if self.trap > 5 && self.trap <= 8 {
                    // address misaligned, access fault, ecall
                    *self.csr(Csr::Mtval) = self.reg_file[rd as usize] as u32;
                } else {
                    *self.csr(Csr::Mtval) = self.pc;
                }
            }
            self.mode = 3;
            self.trap = 0;
        }

        if *self.csr(Csr::Cycle) == u32::MAX {
            *self.csr(Csr::Cycleh) += 1;
            *self.csr(Csr::Cycle) = 0;
        }
        *self.csr(Csr::Cycle) += 1;

        Ok(State::Ok)
    }

    pub fn csr(&mut self, csrname: Csr) -> &mut u32 {
        match csrname {
            Csr::Mscratch => &mut self.csr_file[0x340],
            Csr::Mtvec => &mut self.csr_file[0x305],
            Csr::Mie => &mut self.csr_file[0x304],
            Csr::Cycle => &mut self.csr_file[0xc00],
            Csr::Cycleh => &mut self.csr_file[0xc80],
            Csr::Mip => &mut self.csr_file[0x344],
            Csr::Mepc => &mut self.csr_file[0x341],
            Csr::Mstatus => &mut self.csr_file[0x300],
            Csr::Mcause => &mut self.csr_file[0x342],
            Csr::Mtval => &mut self.csr_file[0x343],
            Csr::Mvendorid => &mut self.csr_file[0xf11],
            Csr::Misa => &mut self.csr_file[0x301],
        }
    }
}
