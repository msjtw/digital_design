use std::collections::HashMap;

use crate::{
    elf_parse,
    instr_parse::{BType, IType, Instruction, InstructionError, JType, RType, SType, UType},
};

pub struct Processor {
    pc: u32,
    reg_file: [i32; 32],
    memory: HashMap<u32, u32>,
}

impl Processor {
    pub fn read_data(elf: &elf_parse::ElfData) -> Processor {
        let mut proc: Processor = Processor {
            pc: elf.entry_adress,
            reg_file: [0; 32],
            memory: HashMap::new(),
        };
        let mut addr = elf.base_address;
        for instr in &elf.intructions {
            proc.memory.insert(addr, *instr);
            addr += 4;
        }
        proc
    }

    pub fn exec(&mut self) -> Result<(), InstructionError> {
        let instr = Instruction::from(self.memory.get(&self.pc).copied().unwrap_or(0))?;
        match instr {
            Instruction::R(x) => self.exec_r(&x),
            Instruction::I(x) => self.exec_i(&x),
            Instruction::S(x) => self.exec_s(&x),
            Instruction::B(x) => self.exec_b(&x),
            Instruction::U(x) => self.exec_u(&x),
            Instruction::J(x) => self.exec_j(&x),
        }?;
        self.pc += 4;
        Ok(())
    }

    fn exec_r(&mut self, instr: &RType) -> Result<(), InstructionError> {
        match instr.funct3 {
            0x0 => match instr.funct7 {
                //add
                0x00 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] + self.reg_file[instr.rs2 as usize];
                    Ok(())
                }
                //sub
                0x20 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] - self.reg_file[instr.rs2 as usize];
                    Ok(())
                }
                _ => Err(InstructionError::ExecutionError),
            },
            //xor
            0x4 => {
                self.reg_file[instr.rd as usize] =
                    self.reg_file[instr.rs1 as usize] ^ self.reg_file[instr.rs2 as usize];
                Ok(())
            }
            //or
            0x6 => {
                self.reg_file[instr.rd as usize] =
                    self.reg_file[instr.rs1 as usize] | self.reg_file[instr.rs2 as usize];
                Ok(())
            }
            //and
            0x7 => {
                self.reg_file[instr.rd as usize] =
                    self.reg_file[instr.rs1 as usize] & self.reg_file[instr.rs2 as usize];
                Ok(())
            }
            //sll
            0x1 => {
                self.reg_file[instr.rd as usize] =
                    self.reg_file[instr.rs1 as usize] << self.reg_file[instr.rs2 as usize];
                Ok(())
            }
            0x5 => match instr.funct7 {
                //srl
                0x00 => {
                    self.reg_file[instr.rd as usize] = (self.reg_file[instr.rs1 as usize] as u32
                        >> self.reg_file[instr.rs2 as usize])
                        as i32;
                    Ok(())
                }
                //sra
                0x02 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] >> self.reg_file[instr.rs2 as usize];
                    Ok(())
                }
                _ => Err(InstructionError::ExecutionError),
            },
            //slt
            0x2 => {
                self.reg_file[instr.rd as usize] =
                    if self.reg_file[instr.rs1 as usize] < self.reg_file[instr.rs2 as usize] {
                        1
                    } else {
                        0
                    };
                Ok(())
            }
            //sltu
            0x3 => {
                self.reg_file[instr.rd as usize] = if (self.reg_file[instr.rs1 as usize] as u32)
                    < (self.reg_file[instr.rs2 as usize] as u32)
                {
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
        match instr.opcode {
            0b0010011 => match instr.funct3 {
                //addi
                0x0 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] + instr.imm;
                    Ok(())
                }
                //xori
                0x4 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] ^ instr.imm;
                    Ok(())
                }
                //ori
                0x6 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] | instr.imm;
                    Ok(())
                }
                //andi
                0x7 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] & instr.imm;
                    Ok(())
                }
                //slli
                0x1 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] << (instr.imm & 31);
                    Ok(())
                }
                0x5 => match instr.funct7 {
                    //srli
                    0x00 => {
                        self.reg_file[instr.rd as usize] =
                            (self.reg_file[instr.rs1 as usize] as u32 >> (instr.imm & 31)) as i32;
                        Ok(())
                    }
                    //srai
                    0x20 => {
                        self.reg_file[instr.rd as usize] =
                            self.reg_file[instr.rs1 as usize] >> (instr.imm & 31);
                        Ok(())
                    }
                    _ => Err(InstructionError::ExecutionError),
                },
                //slti
                0x2 => {
                    self.reg_file[instr.rd as usize] =
                        if self.reg_file[instr.rs1 as usize] < instr.imm {
                            1
                        } else {
                            0
                        };
                    Ok(())
                }
                //sltiu
                0x3 => {
                    self.reg_file[instr.rd as usize] =
                        if (self.reg_file[instr.rs1 as usize] as u32) < (instr.imm as u32) {
                            1
                        } else {
                            0
                        };
                    Ok(())
                }
                _ => Err(InstructionError::ExecutionError),
            },
            0b0000011 => {
                self.reg_file[instr.rd as usize] = self
                    .memory
                    .get(&((self.reg_file[instr.rs1 as usize] + instr.imm) as u32))
                    .copied()
                    .unwrap_or(0) as i32;
                match instr.funct3 {
                    // lb sign-extended
                    0x0 => {
                        self.reg_file[instr.rd as usize] &= 255;
                        for i in 8..32 {
                            self.reg_file[instr.rd as usize] |=
                                (self.reg_file[instr.rd as usize] & 1 << 7) << i - 7;
                        }
                        Ok(())
                    }
                    // lh
                    0x1 => {
                        self.reg_file[instr.rd as usize] &= 65535;
                        for i in 16..32 {
                            self.reg_file[instr.rd as usize] |=
                                (self.reg_file[instr.rd as usize] & 1 << 15) << i - 15;
                        }
                        Ok(())
                    }
                    // lw
                    0x2 => {
                        self.reg_file[instr.rd as usize];
                        Ok(())
                    }
                    // lbu zero-extended
                    0x4 => {
                        self.reg_file[instr.rd as usize] &= 255;
                        Ok(())
                    }
                    // lhu
                    0x5 => {
                        self.reg_file[instr.rd as usize] &= 65535;
                        Ok(())
                    }
                    _ => Err(InstructionError::ExecutionError),
                }
            }
            //jalr
            0b1100111 => {
                self.reg_file[instr.rd as usize] = (self.pc + 4) as i32;
                self.pc = (instr.rs1 as i32 + instr.imm) as u32;
                Ok(())
            }
            //ecall and ebreak
            0b1110011 => Err(InstructionError::NotSupported),
            _ => Err(InstructionError::ExecutionError),
        }
    }

    fn exec_s(&mut self, instr: &SType) -> Result<(), InstructionError> {
        let addr = (instr.rs1 as i32 + instr.imm) as u32;
        let mut val = self.memory.get(&addr).copied().unwrap_or(0);
        match instr.funct3 {
            //sb
            0x0 => {
                let mut bitmask: u32 = !255;
                val &= bitmask;
                bitmask = !bitmask;
                val |= self.reg_file[instr.rs2 as usize] as u32 & bitmask;
            }
            //sh
            0x1 => {
                let mut bitmask: u32 = !65535;
                val &= bitmask;
                bitmask = !bitmask;
                val |= self.reg_file[instr.rs2 as usize] as u32 & bitmask;
            }
            //sw
            0x2 => val = self.reg_file[instr.rs2 as usize] as u32,
            _ => return Err(InstructionError::ExecutionError),
        };
        self.memory.insert(addr, val);
        Ok(())
    }

    fn exec_b(&mut self, instr: &BType) -> Result<(), InstructionError> {
        let rs1 = self.reg_file[instr.rs1 as usize];
        let rs2 = self.reg_file[instr.rs2 as usize];
        match instr.funct3 {
            //beq
            0x0 => {
                if rs1 == rs2 {
                    self.pc = (self.pc as i32 + instr.imm) as u32;
                };
                Ok(())
            }
            //bne
            0x1 => {
                if rs1 != rs2 {
                    self.pc = (self.pc as i32 + instr.imm) as u32;
                };
                Ok(())
            }
            //blt
            0x4 => {
                if rs1 < rs2 {
                    self.pc = (self.pc as i32 + instr.imm) as u32;
                };
                Ok(())
            }
            //bge
            0x5 => {
                if rs1 >= rs2 {
                    self.pc = (self.pc as i32 + instr.imm) as u32;
                };
                Ok(())
            }
            //bltu
            0x6 => {
                if (rs1 as u32) < (rs2 as u32) {
                    self.pc = (self.pc as i32 + instr.imm) as u32;
                };
                Ok(())
            }
            //bgeu
            0x7 => {
                if rs1 as u32 >= rs2 as u32 {
                    self.pc = (self.pc as i32 + instr.imm) as u32;
                };
                Ok(())
            }
            _ => Err(InstructionError::ExecutionError),
        }
    }

    fn exec_u(&mut self, instr: &UType) -> Result<(), InstructionError> {
        match instr.opcode {
            //lui
            0b0110111 => {
                self.reg_file[instr.rd as usize] = instr.imm << 12;
                Ok(())
            }
            //auipc
            0b0010111 => {
                self.reg_file[instr.rd as usize] = self.pc as i32 + (instr.imm << 12);
                Ok(())
            }
            _ => Err(InstructionError::ExecutionError),
        }
    }

    fn exec_j(&mut self, instr: &JType) -> Result<(), InstructionError> {
        match instr.opcode {
            //jalr
            0b1101111 => {
                self.reg_file[instr.rd as usize] = (self.pc + 4) as i32;
                self.pc = (self.pc as i32 + instr.imm) as u32;
                Ok(())
            }
            _ => Err(InstructionError::ExecutionError),
        }
    }
}
