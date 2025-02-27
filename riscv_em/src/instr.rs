//register
#[derive(Debug)]
pub struct RType {
    opcode: u32,
    rd: u32,
    rs1: u32,
    rs2: u32,
    funct3: u32,
    funct7: u32,
}

impl RType {
    fn from(byte_code: u32) -> Self {
        Self {
            opcode: byte_code & 127,
            rd: (byte_code & 3968) >> 7,
            rs1: (byte_code & 1015808) >> 15,
            rs2: (byte_code & 32505856) >> 20,
            funct3: (byte_code & 28672) >> 12,
            funct7: (byte_code & 4261412864) >> 25,
        }
    }
}

//immediate
#[derive(Debug)]
pub struct IType {
    opcode: u32,
    rd: u32,
    rs1: u32,
    funct3: u32,
    imm: i32,
}

impl IType {
    fn from(byte_code: u32) -> Self {
        Self {
            opcode: byte_code & 127,
            rd: (byte_code & 3968) >> 7,
            rs1: (byte_code & 1015808) >> 15,
            funct3: (byte_code & 28672) >> 12,
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
pub struct SType {
    opcode: u32,
    rs1: u32,
    rs2: u32,
    funct3: u32,
    imm: i32,
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
pub struct BType {
    opcode: u32,
    rs1: u32,
    rs2: u32,
    funct3: u32,
    imm: i32,
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
pub struct UType {
    opcode: u32,
    rd: u32,
    imm: u32,
}

impl UType {
    fn from(byte_code: u32) -> Self {
        Self {
            opcode: byte_code & 127,
            rd: (byte_code & 3968) >> 7,
            imm: (byte_code & 4294963200),
        }
    }
}

//jump
#[derive(Debug)]
pub struct JType {
    opcode: u32,
    rd: u32,
    imm: i32,
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

                imm[20] = arr[31];
                for i in 21..=30 {
                    imm[i - 20] = arr[i];
                }
                imm[11] = arr[20];
                for i in 12..=19 {
                    imm[i] = arr[i];
                }
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
pub enum Instruction {
    R(RType),
    I(IType),
    S(SType),
    B(BType),
    U(UType),
    J(JType),
    NoInstr,
}

impl Instruction {
    pub fn from(byte_code: u32) -> Self {
        let opcode = byte_code & 127;
        match opcode {
            0b0110011 => Instruction::R(RType::from(byte_code)),
            0b0010011 => Instruction::I(IType::from(byte_code)),
            0b0000011 => Instruction::I(IType::from(byte_code)),
            0b0100011 => Instruction::S(SType::from(byte_code)),
            0b1100011 => Instruction::B(BType::from(byte_code)),
            0b1101111 => Instruction::J(JType::from(byte_code)),
            0b1100111 => Instruction::I(IType::from(byte_code)),
            0b0110111 => Instruction::U(UType::from(byte_code)),
            0b0010111 => Instruction::U(UType::from(byte_code)),
            0b1110011 => Instruction::I(IType::from(byte_code)),
            _ => Instruction::NoInstr,
        }
    }
}
