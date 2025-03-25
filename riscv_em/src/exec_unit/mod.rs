use object::{Object, ObjectSegment};
use std::{error::Error, fmt, u32};

use crate::instr_parse::{BType, IType, Instruction, InstructionError, JType, RType, SType, UType};
mod memory;
mod syscalls;

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

pub struct Processor {
    pc: u32,
    reg_file: [i32; 32],
    csr: [u32; 4096],
    memory: memory::Memory,

    lr_address: u32,
    lr_valid: bool,
}

impl Default for Processor {
    fn default() -> Self {
        Processor {
            pc: 0,
            reg_file: [0; 32],
            csr: [0; 4096],
            memory: memory::Memory::new(),

            lr_address: 0,
            lr_valid: false,
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
            let addr = segment.address() as u32;
            for i in 0..segment.data()?.iter().len() {
                proc.memory
                    .insert_byte(addr + i as u32, segment.data()?[i as usize]);
            }
        }
        proc.reg_file[2] = 0xBFFFFFF0u32 as i32;
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
            if self.reg_file[i].to_string().len() > 5 {
                print!("    --");
            } else {
                print!("{:6}", self.reg_file[i]);
            }
        }
        println!("");
    }

    pub fn exec(&mut self) -> Result<(), InstructionError> {
        let byte_code = self.memory.get_word(self.pc);
        // print!("{:5x?};", self.pc);
        let instr = Instruction::from(byte_code)?;
        // println!(" {:?}", instr);
        match instr {
            Instruction::R(x) => self.exec_r(&x),
            Instruction::I(x) => self.exec_i(&x),
            Instruction::S(x) => self.exec_s(&x),
            Instruction::B(x) => self.exec_b(&x),
            Instruction::U(x) => self.exec_u(&x),
            Instruction::J(x) => self.exec_j(&x),
        }?;
        self.reg_file[0] = 0;
        // self.print_reg_file();

        if self.pc == 0 {
            Err(InstructionError::End)
        } else {
            Ok(())
        }
    }

    fn exec_r(&mut self, instr: &RType) -> Result<(), InstructionError> {
        match instr.opcode {
            0b0110011 => {
                match instr.funct3 {
                    0x0 => match instr.funct7 {
                        //add
                        0x00 => {
                            self.reg_file[instr.rd as usize] = self.reg_file[instr.rs1 as usize]
                                + self.reg_file[instr.rs2 as usize];
                        }
                        //sub
                        0x20 => {
                            self.reg_file[instr.rd as usize] = self.reg_file[instr.rs1 as usize]
                                - self.reg_file[instr.rs2 as usize];
                        }
                        //mul
                        0x01 => {
                            let a: i64 = self.reg_file[instr.rs1 as usize].into();
                            let b: i64 = self.reg_file[instr.rs2 as usize].into();
                            let tmp = a * b;
                            self.reg_file[instr.rd as usize] = tmp as i32;
                        }
                        _ => return Err(InstructionError::ExecutionError),
                    },
                    0x4 => match instr.funct7 {
                        //xor
                        0x00 => {
                            self.reg_file[instr.rd as usize] = self.reg_file[instr.rs1 as usize]
                                ^ self.reg_file[instr.rs2 as usize];
                        }
                        //div
                        0x01 => {
                            self.reg_file[instr.rd as usize] = self.reg_file[instr.rs1 as usize]
                                / self.reg_file[instr.rs2 as usize];
                        }
                        _ => return Err(InstructionError::ExecutionError),
                    },
                    0x6 => match instr.funct7 {
                        //or
                        0x00 => {
                            self.reg_file[instr.rd as usize] = self.reg_file[instr.rs1 as usize]
                                | self.reg_file[instr.rs2 as usize];
                        }
                        //rem
                        0x01 => {
                            self.reg_file[instr.rd as usize] = self.reg_file[instr.rs1 as usize]
                                % self.reg_file[instr.rs2 as usize];
                        }
                        _ => return Err(InstructionError::ExecutionError),
                    },
                    0x7 => match instr.funct7 {
                        //and
                        0x00 => {
                            self.reg_file[instr.rd as usize] = self.reg_file[instr.rs1 as usize]
                                & self.reg_file[instr.rs2 as usize];
                        }
                        //remu
                        0x01 => {
                            self.reg_file[instr.rd as usize] = (self.reg_file[instr.rs1 as usize]
                                as u32
                                % self.reg_file[instr.rs2 as usize] as u32)
                                as i32;
                        }
                        _ => return Err(InstructionError::ExecutionError),
                    },
                    0x1 => match instr.funct7 {
                        //sll
                        0x00 => {
                            self.reg_file[instr.rd as usize] = self.reg_file[instr.rs1 as usize]
                                << self.reg_file[instr.rs2 as usize];
                        }
                        //mulh
                        0x01 => {
                            let a: i64 = self.reg_file[instr.rs1 as usize].into();
                            let b: i64 = self.reg_file[instr.rs2 as usize].into();
                            let tmp = (a * b) >> 32;
                            self.reg_file[instr.rd as usize] = tmp as i32;
                        }
                        _ => return Err(InstructionError::ExecutionError),
                    },
                    0x5 => match instr.funct7 {
                        //srl
                        0x00 => {
                            self.reg_file[instr.rd as usize] = (self.reg_file[instr.rs1 as usize]
                                as u32
                                >> self.reg_file[instr.rs2 as usize])
                                as i32;
                        }
                        //divu
                        0x01 => {
                            self.reg_file[instr.rd as usize] = (self.reg_file[instr.rs1 as usize]
                                as u32
                                / self.reg_file[instr.rs2 as usize] as u32)
                                as i32;
                        }
                        //sra
                        0x02 => {
                            self.reg_file[instr.rd as usize] = self.reg_file[instr.rs1 as usize]
                                >> self.reg_file[instr.rs2 as usize];
                        }
                        _ => return Err(InstructionError::ExecutionError),
                    },
                    0x2 => match instr.funct7 {
                        //slt
                        0x00 => {
                            self.reg_file[instr.rd as usize] = if self.reg_file[instr.rs1 as usize]
                                < self.reg_file[instr.rs2 as usize]
                            {
                                1
                            } else {
                                0
                            };
                        }
                        //mulsu
                        0x01 => {
                            let a: i64 = self.reg_file[instr.rs1 as usize].into();
                            let b: i64 = self.reg_file[instr.rs2 as usize] as i64;
                            let tmp = (a * b) >> 32;
                            self.reg_file[instr.rd as usize] = tmp as i32;
                        }
                        _ => return Err(InstructionError::ExecutionError),
                    },
                    0x3 => match instr.funct7 {
                        //sltu
                        0x00 => {
                            self.reg_file[instr.rd as usize] = if (self.reg_file[instr.rs1 as usize]
                                as u32)
                                < (self.reg_file[instr.rs2 as usize] as u32)
                            {
                                1
                            } else {
                                0
                            };
                        }
                        //mulu
                        0x01 => {
                            let a: i64 = self.reg_file[instr.rs1 as usize] as i64;
                            let b: i64 = self.reg_file[instr.rs2 as usize] as i64;
                            let tmp = (a * b) >> 32;
                            self.reg_file[instr.rd as usize] = tmp as i32;
                        }
                        _ => return Err(InstructionError::ExecutionError),
                    },

                    _ => return Err(InstructionError::ExecutionError),
                };
                self.pc += 4;
            }
            0b0101111 => match instr.funct5 {
                // LR.W
                0b00010 => {
                    self.lr_address = self.reg_file[instr.rs1 as usize] as u32;
                    self.lr_valid = true;
                    self.reg_file[instr.rd as usize] = self
                        .memory
                        .get_word(self.reg_file[instr.rs1 as usize] as u32)
                        as i32;
                    self.pc += 4;
                }
                // SC.W
                0b00011 => {
                    if self.lr_valid
                        && (self.lr_address == self.reg_file[instr.rs1 as usize] as u32)
                    {
                        self.memory.insert_word(
                            self.reg_file[instr.rs1 as usize] as u32,
                            self.reg_file[instr.rs2 as usize] as u32,
                        );
                        self.reg_file[instr.rd as usize] = 0;
                    } else {
                        self.reg_file[instr.rd as usize] = 1;
                    }
                    self.lr_address = 0;
                    self.lr_valid = false;
                    self.pc += 4;
                }
                // amoswap.w
                0b00001 => {
                    self.reg_file[instr.rd as usize] = self
                        .memory
                        .get_word(self.reg_file[instr.rs1 as usize] as u32)
                        as i32;
                    self.reg_file.swap(instr.rd as usize, instr.rs2 as usize);
                    self.memory.insert_word(
                        self.reg_file[instr.rs1 as usize] as u32,
                        self.reg_file[instr.rd as usize] as u32,
                    );
                    self.pc += 4;
                }
                // amoadd.w
                0b00000 => {
                    self.reg_file[instr.rd as usize] = self
                        .memory
                        .get_word(self.reg_file[instr.rs1 as usize] as u32)
                        as i32
                        + self.reg_file[instr.rs2 as usize];
                    self.memory.insert_word(
                        self.reg_file[instr.rs1 as usize] as u32,
                        self.reg_file[instr.rd as usize] as u32,
                    );
                    self.pc += 4;
                }
                // amoxor.w
                0b00100 => {
                    self.reg_file[instr.rd as usize] = self
                        .memory
                        .get_word(self.reg_file[instr.rs1 as usize] as u32)
                        as i32
                        ^ self.reg_file[instr.rs2 as usize];
                    self.memory.insert_word(
                        self.reg_file[instr.rs1 as usize] as u32,
                        self.reg_file[instr.rd as usize] as u32,
                    );
                    self.pc += 4;
                }
                // amoand.w
                0b01100 => {
                    self.reg_file[instr.rd as usize] = self
                        .memory
                        .get_word(self.reg_file[instr.rs1 as usize] as u32)
                        as i32
                        & self.reg_file[instr.rs2 as usize];
                    self.memory.insert_word(
                        self.reg_file[instr.rs1 as usize] as u32,
                        self.reg_file[instr.rd as usize] as u32,
                    );
                    self.pc += 4;
                }
                // amoor.w
                0b01000 => {
                    self.reg_file[instr.rd as usize] = self
                        .memory
                        .get_word(self.reg_file[instr.rs1 as usize] as u32)
                        as i32
                        | self.reg_file[instr.rs2 as usize];
                    self.memory.insert_word(
                        self.reg_file[instr.rs1 as usize] as u32,
                        self.reg_file[instr.rd as usize] as u32,
                    );
                    self.pc += 4;
                }
                //amomin.w
                0b10000 => {
                    let tmp = self
                        .memory
                        .get_word(self.reg_file[instr.rd as usize] as u32)
                        as i32;
                    self.reg_file[instr.rd as usize] = tmp.min(self.reg_file[instr.rs2 as usize]);
                    self.memory.insert_word(
                        self.reg_file[instr.rs1 as usize] as u32,
                        self.reg_file[instr.rd as usize] as u32,
                    );
                    self.pc += 4;
                }
                // amomax.w
                0b10100 => {
                    let tmp = self
                        .memory
                        .get_word(self.reg_file[instr.rd as usize] as u32)
                        as i32;
                    self.reg_file[instr.rd as usize] = tmp.max(self.reg_file[instr.rs2 as usize]);
                    self.memory.insert_word(
                        self.reg_file[instr.rs1 as usize] as u32,
                        self.reg_file[instr.rd as usize] as u32,
                    );
                    self.pc += 4;
                }
                // amominu.w
                0b11000 => {
                    let tmp = self
                        .memory
                        .get_word(self.reg_file[instr.rd as usize] as u32);
                    self.reg_file[instr.rd as usize] =
                        tmp.min(self.reg_file[instr.rs2 as usize] as u32) as i32;
                    self.memory.insert_word(
                        self.reg_file[instr.rs1 as usize] as u32,
                        self.reg_file[instr.rd as usize] as u32,
                    );
                    self.pc += 4;
                }
                // amomaxiu.w
                0b11100 => {
                    let tmp = self
                        .memory
                        .get_word(self.reg_file[instr.rd as usize] as u32);
                    self.reg_file[instr.rd as usize] =
                        tmp.max(self.reg_file[instr.rs2 as usize] as u32) as i32;
                    self.memory.insert_word(
                        self.reg_file[instr.rs1 as usize] as u32,
                        self.reg_file[instr.rd as usize] as u32,
                    );
                    self.pc += 4;
                }
                _ => return Err(InstructionError::ExecutionError),
            },
            _ => return Err(InstructionError::ExecutionError),
        };

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
                self.pc = (self.reg_file[instr.rs1 as usize] as i32 + instr.imm) as u32;
            }
            0b1110011 => match instr.funct7 {
                //ecall
                0x0 => {
                    let syscall = syscalls::SystemCall::from(self);
                    syscall.exec(self)?;
                    self.pc += 4;
                }
                // csrrw
                0b001 => {
                    self.reg_file[instr.rd as usize] = self.csr[instr.imm as usize] as i32;
                    self.csr[instr.imm as usize] = self.reg_file[instr.rs1 as usize] as u32;
                    self.pc += 4;
                }
                // csrrs
                0b010 => {
                    self.reg_file[instr.rd as usize] = self.csr[instr.imm as usize] as i32;
                    self.csr[instr.imm as usize] |= self.reg_file[instr.rs1 as usize] as u32;
                    self.pc += 4;
                }
                // csrrc
                0b011 => {
                    self.reg_file[instr.rd as usize] = self.csr[instr.imm as usize] as i32;
                    self.csr[instr.imm as usize] &= !self.reg_file[instr.rs1 as usize] as u32;
                    self.pc += 4;
                }
                // csrrwi
                0b101 => {
                    self.reg_file[instr.rd as usize] = self.csr[instr.imm as usize] as i32;
                    self.csr[instr.imm as usize] = instr.rs1;
                    self.pc += 4;
                }
                // csrrsi
                0b110 => {
                    self.reg_file[instr.rd as usize] = self.csr[instr.imm as usize] as i32;
                    self.csr[instr.imm as usize] |= instr.rs1;
                    self.pc += 4;
                }
                // csrrci
                0b111 => {
                    self.reg_file[instr.rd as usize] = self.csr[instr.imm as usize] as i32;
                    self.csr[instr.imm as usize] &= !instr.rs1;
                    self.pc += 4;
                }
                // ebreak
                _ => return Err(InstructionError::NotSupported),
            },
            // fence, pause
            0b0001111 => self.pc += 4,

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
                self.pc = (self.pc as i32 + instr.imm) as u32;
            }
            _ => return Err(InstructionError::ExecutionError),
        };
        Ok(())
    }
}
