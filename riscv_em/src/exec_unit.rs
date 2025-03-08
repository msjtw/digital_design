use object::{Object, ObjectSegment};
use std::collections::HashMap;
use std::default;
use std::{error::Error, fmt};

use crate::instr_parse::{BType, IType, Instruction, InstructionError, JType, RType, SType, UType};

#[derive(Debug)]
pub enum ElfError {
    NotLittleEndian,
    DataReadError,
}

impl From<object::Error> for ElfError {
    fn from(_value: object::Error) -> Self {
        Self::DataReadError
    }
}

impl Error for ElfError {}

impl fmt::Display for ElfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotLittleEndian => write!(f, "ELF must be little endian!"),
            Self::DataReadError => write!(f, "Error reading data from ELF!"),
        }
    }
}

struct Memory {
    data: HashMap<u32, u8>,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            data: HashMap::new(),
        }
    }
}

impl Memory {
    fn get_word(&self, mut address: u32) -> u32 {
        let d: u32 = self.data.get(&address).copied().unwrap_or(0).into();
        address += 1;
        let c: u32 = self.data.get(&address).copied().unwrap_or(0).into();
        address += 1;
        let b: u32 = self.data.get(&address).copied().unwrap_or(0).into();
        address += 1;
        let a: u32 = self.data.get(&address).copied().unwrap_or(0).into();
        (a << 24) + (b << 16) + (c << 8) + d
    }
    fn get_hword(&self, mut address: u32) -> u16 {
        let b: u16 = self.data.get(&address).copied().unwrap_or(0).into();
        address += 1;
        let a: u16 = self.data.get(&address).copied().unwrap_or(0).into();
        (a << 8) + b
    }
    fn get_byte(&self, address: u32) -> u8 {
        self.data.get(&address).copied().unwrap_or(0)
    }
    fn insert_word(&mut self, address: u32, data: u32) {
        let mut mask: u32 = (2 << 8) - 1;
        let d: u8 = (data & mask).try_into().unwrap();
        mask <<= 8;
        let c: u8 = ((data & mask) >> 8).try_into().unwrap();
        mask <<= 8;
        let b: u8 = ((data & mask) >> 16).try_into().unwrap();
        mask <<= 8;
        let a: u8 = ((data & mask) >> 24).try_into().unwrap();
        self.data.insert(address, d);
        self.data.insert(address + 1, c);
        self.data.insert(address + 2, b);
        self.data.insert(address + 3, a);
    }
    fn insert_hword(&mut self, address: u32, data: u16) {
        let mut mask: u16 = (2 << 8) - 1;
        let d: u8 = (data & mask).try_into().unwrap();
        mask <<= 8;
        let c: u8 = ((data & mask) >> 8).try_into().unwrap();
        self.data.insert(address, d);
        self.data.insert(address + 1, c);
    }
    fn insert_byte(&mut self, address: u32, data: u8) {
        self.data.insert(address, data);
    }
}

pub struct Processor {
    pc: u32,
    reg_file: [i32; 32],
    memory: Memory,
}

impl Default for Processor {
    fn default() -> Self {
        Processor {
            pc: 0,
            reg_file: [0; 32],
            memory: Memory {
                ..Default::default()
            },
        }
    }
}

impl Processor {
    pub fn read_data(elf: &object::File) -> Result<Processor, ElfError> {
        if !elf.is_little_endian() {
            return Err(ElfError::NotLittleEndian);
        }
        let mut proc: Processor = Processor {
            pc: elf.entry() as u32,
            ..Default::default()
        };
        for segment in elf.segments() {
            let addr = segment.address();
            for i in 0..segment.size() {
                proc.memory
                    .insert_byte((addr + i).try_into().unwrap(), segment.data()?[i as usize]);
            }
        }
        proc.reg_file[2] = 100;
        Ok(proc)
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
            print!("{:6}", self.reg_file[i]);
        }
        println!("");
    }

    pub fn exec(&mut self) -> Result<(), InstructionError> {
        let byte_code = self.memory.get_word(self.pc);
        let instr = Instruction::from(byte_code)?;
        println!("{:?} {:?}", self.pc, instr);
        match instr {
            Instruction::R(x) => self.exec_r(&x),
            Instruction::I(x) => self.exec_i(&x),
            Instruction::S(x) => self.exec_s(&x),
            Instruction::B(x) => self.exec_b(&x),
            Instruction::U(x) => self.exec_u(&x),
            Instruction::J(x) => self.exec_j(&x),
        }?;
        self.reg_file[0] = 0;
        self.print_reg_file();

        if self.pc == 0 {
            Err(InstructionError::End)
        } else {
            Ok(())
        }
    }

    fn exec_r(&mut self, instr: &RType) -> Result<(), InstructionError> {
        match instr.funct3 {
            0x0 => match instr.funct7 {
                //add
                0x00 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] + self.reg_file[instr.rs2 as usize];
                }
                //sub
                0x20 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] - self.reg_file[instr.rs2 as usize];
                }
                _ => return Err(InstructionError::ExecutionError),
            },
            //xor
            0x4 => {
                self.reg_file[instr.rd as usize] =
                    self.reg_file[instr.rs1 as usize] ^ self.reg_file[instr.rs2 as usize];
            }
            //or
            0x6 => {
                self.reg_file[instr.rd as usize] =
                    self.reg_file[instr.rs1 as usize] | self.reg_file[instr.rs2 as usize];
            }
            //and
            0x7 => {
                self.reg_file[instr.rd as usize] =
                    self.reg_file[instr.rs1 as usize] & self.reg_file[instr.rs2 as usize];
            }
            //sll
            0x1 => {
                self.reg_file[instr.rd as usize] =
                    self.reg_file[instr.rs1 as usize] << self.reg_file[instr.rs2 as usize];
            }
            0x5 => match instr.funct7 {
                //srl
                0x00 => {
                    self.reg_file[instr.rd as usize] = (self.reg_file[instr.rs1 as usize] as u32
                        >> self.reg_file[instr.rs2 as usize])
                        as i32;
                }
                //sra
                0x02 => {
                    self.reg_file[instr.rd as usize] =
                        self.reg_file[instr.rs1 as usize] >> self.reg_file[instr.rs2 as usize];
                }
                _ => return Err(InstructionError::ExecutionError),
            },
            //slt
            0x2 => {
                self.reg_file[instr.rd as usize] =
                    if self.reg_file[instr.rs1 as usize] < self.reg_file[instr.rs2 as usize] {
                        1
                    } else {
                        0
                    };
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
            }

            _ => return Err(InstructionError::ExecutionError),
        };
        self.pc += 4;
        Ok(())
    }

    fn exec_i(&mut self, instr: &IType) -> Result<(), InstructionError> {
        match instr.opcode {
            0b0010011 => {
                match instr.funct3 {
                    //addi
                    0x0 => {
                        self.reg_file[instr.rd as usize] =
                            self.reg_file[instr.rs1 as usize] + instr.imm;
                    }
                    //xori
                    0x4 => {
                        self.reg_file[instr.rd as usize] =
                            self.reg_file[instr.rs1 as usize] ^ instr.imm;
                    }
                    //ori
                    0x6 => {
                        self.reg_file[instr.rd as usize] =
                            self.reg_file[instr.rs1 as usize] | instr.imm;
                    }
                    //andi
                    0x7 => {
                        self.reg_file[instr.rd as usize] =
                            self.reg_file[instr.rs1 as usize] & instr.imm;
                    }
                    //slli
                    0x1 => {
                        self.reg_file[instr.rd as usize] =
                            self.reg_file[instr.rs1 as usize] << (instr.imm & 31);
                    }
                    0x5 => match instr.funct7 {
                        //srli
                        0x00 => {
                            self.reg_file[instr.rd as usize] =
                                (self.reg_file[instr.rs1 as usize] as u32 >> (instr.imm & 31))
                                    as i32;
                        }
                        //srai
                        0x20 => {
                            self.reg_file[instr.rd as usize] =
                                self.reg_file[instr.rs1 as usize] >> (instr.imm & 31);
                        }
                        _ => return Err(InstructionError::ExecutionError),
                    },
                    //slti
                    0x2 => {
                        self.reg_file[instr.rd as usize] =
                            if self.reg_file[instr.rs1 as usize] < instr.imm {
                                1
                            } else {
                                0
                            };
                    }
                    //sltiu
                    0x3 => {
                        self.reg_file[instr.rd as usize] =
                            if (self.reg_file[instr.rs1 as usize] as u32) < (instr.imm as u32) {
                                1
                            } else {
                                0
                            };
                    }
                    _ => return Err(InstructionError::ExecutionError),
                };
                self.pc += 4;
            }
            0b0000011 => {
                let addr: u32 = (self.reg_file[instr.rs1 as usize] + instr.imm)
                    .try_into()
                    .unwrap();
                match instr.funct3 {
                    // lb sign-extended
                    0x0 => {
                        self.reg_file[instr.rd as usize] = self.memory.get_byte(addr).into();
                    }
                    // lh
                    0x1 => {
                        self.reg_file[instr.rd as usize] = self.memory.get_hword(addr).into();
                    }
                    // lw
                    0x2 => {
                        self.reg_file[instr.rd as usize] = self.memory.get_word(addr) as i32;
                    }
                    // lbu zero-extended
                    0x4 => {
                        self.reg_file[instr.rd as usize] = self.memory.get_byte(addr) as i32;
                    }
                    // lhu
                    0x5 => {
                        self.reg_file[instr.rd as usize] = self.memory.get_hword(addr) as i32;
                    }
                    _ => return Err(InstructionError::ExecutionError),
                };
                self.pc += 4;
            }
            //jalr
            0b1100111 => {
                self.reg_file[instr.rd as usize] = (self.pc + 4) as i32;
                self.pc = (instr.rs1 as i32 + instr.imm) as u32;
            }
            //ecall and ebreak
            0b1110011 => return Err(InstructionError::NotSupported),
            _ => return Err(InstructionError::ExecutionError),
        };
        Ok(())
    }

    fn exec_s(&mut self, instr: &SType) -> Result<(), InstructionError> {
        let addr = (self.reg_file[instr.rs1 as usize] + instr.imm) as u32;
        match instr.funct3 {
            //sb
            0x0 => self
                .memory
                .insert_byte(addr, self.reg_file[instr.rs2 as usize] as u8),
            //sh
            0x1 => self
                .memory
                .insert_hword(addr, self.reg_file[instr.rs2 as usize] as u16),
            //sw
            0x2 => self
                .memory
                .insert_word(addr, self.reg_file[instr.rs2 as usize] as u32),
            _ => return Err(InstructionError::ExecutionError),
        };
        self.pc += 4;
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
                    return Ok(());
                }
            }
            //bne
            0x1 => {
                if rs1 != rs2 {
                    self.pc = (self.pc as i32 + instr.imm) as u32;
                    return Ok(());
                }
            }
            //blt
            0x4 => {
                if rs1 < rs2 {
                    self.pc = (self.pc as i32 + instr.imm) as u32;
                    return Ok(());
                }
            }
            //bge
            0x5 => {
                if rs1 >= rs2 {
                    self.pc = (self.pc as i32 + instr.imm) as u32;
                    return Ok(());
                }
            }
            //bltu
            0x6 => {
                if (rs1 as u32) < (rs2 as u32) {
                    self.pc = (self.pc as i32 + instr.imm) as u32;
                    return Ok(());
                }
            }
            //bgeu
            0x7 => {
                if rs1 as u32 >= rs2 as u32 {
                    self.pc = (self.pc as i32 + instr.imm) as u32;
                    return Ok(());
                };
            }
            _ => return Err(InstructionError::ExecutionError),
        };
        self.pc += 4;
        Ok(())
    }

    fn exec_u(&mut self, instr: &UType) -> Result<(), InstructionError> {
        match instr.opcode {
            //lui
            0b0110111 => {
                self.reg_file[instr.rd as usize] = instr.imm << 12;
            }
            //auipc
            0b0010111 => {
                self.reg_file[instr.rd as usize] = self.pc as i32 + (instr.imm << 12);
            }
            _ => return Err(InstructionError::ExecutionError),
        };
        self.pc += 4;
        Ok(())
    }

    fn exec_j(&mut self, instr: &JType) -> Result<(), InstructionError> {
        match instr.opcode {
            //jal
            0b1101111 => {
                self.reg_file[instr.rd as usize] = (self.pc + 4) as i32;
                self.pc = (self.pc as i32 + instr.imm - 4) as u32;
            }
            _ => return Err(InstructionError::ExecutionError),
        };
        Ok(())
    }
}
