use object::{Object, ObjectSegment};
use std::{error::Error, fmt, u32};

pub mod instr_parse;
use crate::memory;
use instr_parse::{Instruction, InstructionError};

mod datapath;
mod syscalls;

#[derive(Debug)]
pub enum ExecError {
    Error,
    NotLittleEndian,
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
            Self::NotLittleEndian => write!(f, "ELF must be little endian!"),
            Self::DataReadError => write!(f, "Error reading data from ELF!"),
            Self::InstructionError(x) => write!(f, "Instruction Error {x}"),
            Self::End => write!(f, "End of execution!"),
        }
    }
}

#[derive(Debug)]
pub struct Core<'a> {
    pc: u32,
    reg_file: [i32; 32],
    csr: [u32; 4096],
    memory: &'a mut memory::Memory,

    lr_address: u32,
    lr_valid: bool,
}

impl<'a> Core<'a> {
    pub fn new<'b>(memory: &'a mut memory::Memory) -> Self {
        let c = Core {
            pc: 0,
            reg_file: [0; 32],
            csr: [0; 4096],
            memory,
            lr_address: 0,
            lr_valid: false,
        };
        c
    }
    pub fn read_data(&mut self, elf: &object::File) -> Result<(), ExecError> {
        if !elf.is_little_endian() {
            return Err(ExecError::NotLittleEndian);
        }
        self.pc = elf.entry() as u32;
        for segment in elf.segments() {
            let addr = segment.address() as u32;
            for i in 0..segment.data()?.iter().len() {
                self.memory
                    .insert_byte(addr + i as u32, segment.data()?[i as usize]);
            }
        }
        self.reg_file[2] = 0xBFFFFFF0u32 as i32;
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

    pub fn exec(&mut self) -> Result<(), ExecError> {
        let byte_code = self.memory.get_word(self.pc);
        // print!("{:5x?};", self.pc);
        let instr = Instruction::from(byte_code)?;
        // println!(" {:?}", instr);
        match instr {
            Instruction::R(x) => datapath::exec_r(self, &x),
            Instruction::I(x) => datapath::exec_i(self, &x),
            Instruction::S(x) => datapath::exec_s(self, &x),
            Instruction::B(x) => datapath::exec_b(self, &x),
            Instruction::U(x) => datapath::exec_u(self, &x),
            Instruction::J(x) => datapath::exec_j(self, &x),
        }?;
        self.reg_file[0] = 0;
        // self.print_reg_file();

        if self.pc == 0 {
            Err(ExecError::End)
        } else {
            Ok(())
        }
    }
}
