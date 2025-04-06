use std::{error::Error, fmt};

//register
#[derive(Debug)]
#[allow(dead_code)]
pub struct RType {
    pub opcode: u32,
    pub rd: u32,
    pub rs1: u32,
    pub rs2: u32,
    pub funct3: u32,
    pub funct5: u32,
    pub funct7: u32,
}

impl RType {
    fn from(byte_code: u32) -> Self {
        Self {
            opcode: byte_code & 127,
            rd: (byte_code & 3968) >> 7,
            rs1: (byte_code & 1015808) >> 15,
            rs2: (byte_code & 32505856) >> 20,
            funct3: (byte_code & 28672) >> 12,
            funct5: byte_code >> 27,
            funct7: byte_code >> 25,
        }
    }
}

//immediate
#[derive(Debug)]
#[allow(dead_code)]
pub struct IType {
    pub opcode: u32,
    pub rd: u32,
    pub rs1: u32,
    pub funct3: u32,
    pub funct7: u32,
    pub imm: i32,
}

impl IType {
    fn from(byte_code: u32) -> Self {
        Self {
            opcode: byte_code & 127,
            rd: (byte_code & 3968) >> 7,
            rs1: (byte_code & 1015808) >> 15,
            funct3: (byte_code & 28672) >> 12,
            funct7: (byte_code & 4261412864) >> 25,
            imm: {
                let mut arr = [0; 32];
                let mut imm = [0; 32];
                for i in 0..32 {
                    arr[i] = (byte_code & (1 << i)) >> i;
                }

                for i in 0..=11 {
                    imm[i] = arr[i + 20];
                }
                for i in 12..=31 {
                    imm[i] = imm[11]
                }

                let mut sum = 0;
                for i in 0..32 {
                    sum <<= 1;
                    sum += imm[31 - i];
                }
                sum as i32
            },
        }
    }
}

//strore
#[derive(Debug)]
#[allow(dead_code)]
pub struct SType {
    pub opcode: u32,
    pub rs1: u32,
    pub rs2: u32,
    pub funct3: u32,
    pub imm: i32,
}

impl SType {
    fn from(byte_code: u32) -> Self {
        Self {
            opcode: byte_code & 127,
            rs1: (byte_code & 1015808) >> 15,
            rs2: (byte_code & 32505856) >> 20,
            funct3: (byte_code & 28672) >> 12,
            imm: {
                let mut arr = [0; 32];
                let mut imm = [0; 32];
                for i in 0..32 {
                    arr[i] = (byte_code & (1 << i)) >> i;
                }

                for i in 0..=4 {
                    imm[i] = arr[i + 7];
                }
                for i in 5..=11 {
                    imm[i] = arr[i + 20]
                }
                for i in 12..=31 {
                    imm[i] = imm[11]
                }

                let mut sum = 0;
                for i in 0..32 {
                    sum <<= 1;
                    sum += imm[31 - i];
                }
                sum as i32
            },
        }
    }
}

//branch
#[derive(Debug)]
#[allow(dead_code)]
pub struct BType {
    pub opcode: u32,
    pub rs1: u32,
    pub rs2: u32,
    pub funct3: u32,
    pub imm: i32,
}

impl BType {
    fn from(byte_code: u32) -> Self {
        Self {
            opcode: byte_code & 127,
            rs1: (byte_code & 1015808) >> 15,
            rs2: (byte_code & 32505856) >> 20,
            funct3: (byte_code & 28672) >> 12,
            imm: {
                let mut arr = [0; 32];
                let mut imm = [0; 32];
                for i in 0..32 {
                    arr[i] = (byte_code & (1 << i)) >> i;
                }

                for i in 1..=4 {
                    imm[i] = arr[i + 7];
                }
                for i in 5..=10 {
                    imm[i] = arr[i + 20]
                }
                imm[11] = arr[7];
                imm[12] = arr[31];

                for i in 13..=31 {
                    imm[i] = imm[12];
                }

                let mut sum = 0;
                for i in 0..32 {
                    sum <<= 1;
                    sum += imm[31 - i];
                }
                sum as i32
            },
        }
    }
}

//upper immediate
#[derive(Debug)]
#[allow(dead_code)]
pub struct UType {
    pub opcode: u32,
    pub rd: u32,
    pub imm: i32,
}

impl UType {
    fn from(byte_code: u32) -> Self {
        Self {
            opcode: byte_code & 127,
            rd: (byte_code & 3968) >> 7,
            imm: ((byte_code & 4294963200) >> 12) as i32,
        }
    }
}

//jump
#[derive(Debug)]
#[allow(dead_code)]
pub struct JType {
    pub opcode: u32,
    pub rd: u32,
    pub imm: i32,
}

impl JType {
    fn from(byte_code: u32) -> Self {
        Self {
            opcode: byte_code & 127,
            rd: (byte_code & 3968) >> 7,
            imm: {
                let mut arr = [0; 32];
                let mut imm = [0; 32];
                for i in 0..32 {
                    arr[i] = (byte_code & (1 << i)) >> i;
                }

                for i in 1..=10 {
                    imm[i] = arr[i + 20];
                }
                imm[11] = arr[20];
                for i in 12..=19 {
                    imm[i] = arr[i];
                }
                imm[20] = arr[31];

                for i in 21..=31 {
                    imm[i] = imm[20];
                }

                let mut sum = 0;
                for i in 0..32 {
                    sum <<= 1;
                    sum += imm[31 - i];
                }
                sum as i32
            },
        }
    }
}

#[derive(Debug)]
pub enum InstructionError {
    NoInstruction,
    WrongOpcode,
    NotSupported,
}

impl Error for InstructionError {}

impl fmt::Display for InstructionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WrongOpcode => write!(f, "Instruction opcode doesn't match any type!"),
            Self::NoInstruction => write!(f, "No such instruction"),
            Self::NotSupported => write!(f, "Operation not supported!"),
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    R(RType),
    I(IType),
    S(SType),
    B(BType),
    U(UType),
    J(JType),
}

impl Instruction {
    pub fn from(byte_code: u32) -> Result<Self, InstructionError> {
        let opcode = byte_code & 127;
        match opcode {
            0b0110011 => Ok(Instruction::R(RType::from(byte_code))),
            0b0010011 | 0b0000011 | 0b1100111 | 0b1110011 | 0b0001111 => {
                Ok(Instruction::I(IType::from(byte_code)))
            }
            0b0100011 => Ok(Instruction::S(SType::from(byte_code))),
            0b1100011 => Ok(Instruction::B(BType::from(byte_code))),
            0b1101111 => Ok(Instruction::J(JType::from(byte_code))),
            0b0110111 | 0b0010111 => Ok(Instruction::U(UType::from(byte_code))),
            _ => {
                println!("{opcode:b}");
                println!("{byte_code:b}");
                Err(InstructionError::WrongOpcode)
            }
        }
    }
}
