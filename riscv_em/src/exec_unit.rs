use std::collections::HashMap;

use crate::{
    elf_parse,
    instr_parse::{BType, IType, Instruction, InstructionError, JType, RType, SType, UType},
};

struct Processor {
    pc: u32,
    reg_file: [i32; 32],
    memory: HashMap<u32, u32>,
}

impl Processor {
    fn read_data(&mut self, elf: &elf_parse::ElfData) {
        self.pc = elf.entry_adress;
        self.reg_file = [0; 32];
        let mut addr = elf.base_address;
        for instr in &elf.intructions {
            self.memory.insert(addr, *instr);
            addr += 4;
        }
    }

    fn exec(&mut self, instr: &Instruction) -> Result<(), InstructionError> {
        match instr {
            Instruction::R(x) => self.exec_r(x),
            Instruction::I(x) => self.exec_i(x),
            Instruction::S(x) => self.exec_s(x),
            Instruction::B(x) => self.exec_b(x),
            Instruction::U(x) => self.exec_u(x),
            Instruction::J(x) => self.exec_j(x),
        };
        self.pc += 4;
        Ok(())
    }

    fn exec_r(&mut self, instr: &RType) -> Result<(), InstructionError> {
        match instr.funct3 {
            0x0 => match instr.funct7 {
                0x00 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] + self.reg_file[instr.rs2 as usize];
                    Ok(())
                }
                0x20 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] - self.reg_file[instr.rs2 as usize];
                    Ok(())
                }
                _ => Err(InstructionError::ExecutionError),
            },
            0x4 => {
                self.reg_file[instr.rd as usize] =
                    self.reg_file[instr.rs1 as usize] ^ self.reg_file[instr.rs2 as usize];
                Ok(())
            }
            0x6 => {
                self.reg_file[instr.rd as usize] =
                    self.reg_file[instr.rs1 as usize] | self.reg_file[instr.rs2 as usize];
                Ok(())
            }
            0x7 => {
                self.reg_file[instr.rd as usize] =
                    self.reg_file[instr.rs1 as usize] & self.reg_file[instr.rs2 as usize];
                Ok(())
            }
            0x1 => {
                self.reg_file[instr.rd as usize] =
                    self.reg_file[instr.rs1 as usize] << self.reg_file[instr.rs2 as usize];
                Ok(())
            }
            0x5 => match instr.funct7 {
                0x00 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] >> self.reg_file[instr.rs2 as usize];
                    Ok(())
                }
                0x02 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] >> self.reg_file[instr.rs2 as usize];
                    Ok(())
                }
                _ => Err(InstructionError::ExecutionError),
            },
            0x2 => {
                self.reg_file[instr.rd as usize] =
                    if self.reg_file[instr.rs1 as usize] < self.reg_file[instr.rs2 as usize] {
                        1
                    } else {
                        0
                    };
                Ok(())
            }
            0x3 => {
                self.reg_file[instr.rd as usize] =
                    if self.reg_file[instr.rs1 as usize] < self.reg_file[instr.rs2 as usize] {
                        1
                    } else {
                        0
                    };
                Ok(())
            }

            _ => Err(InstructionError::ExecutionError),
        }
    }

    fn exec_i(&mut self, instr: &IType) -> Result<(), InstructionError> {
        ()
    }

    fn exec_s(&mut self, instr: &SType) -> Result<(), InstructionError> {
        ()
    }

    fn exec_b(&mut self, instr: &BType) -> Result<(), InstructionError> {
        ()
    }

    fn exec_u(&mut self, instr: &UType) -> Result<(), InstructionError> {
        ()
    }

    fn exec_j(&mut self, instr: &JType) -> Result<(), InstructionError> {
        ()
    }
}
