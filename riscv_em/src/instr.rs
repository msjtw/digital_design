//register
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
pub struct IType {
    opcode: u32,
    rd: u32,
    rs1: u32,
    funct3: u32,
    imm: u32,
}

impl IType {
    fn from(byte_code: u32) -> Self {
        Self {
            opcode: byte_code & 127,
            rd: (byte_code & 3968) >> 7,
            rs1: (byte_code & 1015808) >> 15,
            funct3: (byte_code & 28672) >> 12,
            imm: (byte_code & 4293918720) >> 20,
        }
    }
}

//strore
pub struct SType {
    opcode: u32,
    rs1: u32,
    rs2: u32,
    funct3: u32,
    imm: u32,
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
                for i in 0..31 {
                    arr[i] = byte_code & (1 << i);
                }

                for i in 11..5 {
                    imm[i] = arr[i + 20];
                }
                for i in 4..0 {
                    imm[i] = arr[i + 7]
                }

                let mut sum = 0;
                for i in 0..31 {
                    sum <<= 1;
                    sum += imm[i];
                }
                sum
            },
        }
    }
}

//branch
pub struct BType {
    opcode: u32,
    rs1: u32,
    rs2: u32,
    funct3: u32,
    imm: u32,
}

impl BType {
    fn from(byte_code: u32) -> Self {
        Self {
            opcode: byte_code & 127,
            rs1: (byte_code & 1015808) >> 15,
            rs2: (byte_code & 32505856) >> 20,
            funct3: (byte_code & 28672) >> 12,
            imm: ((byte_code & 2147483648) >> 19)
                + ((byte_code & 127) << 4)
                + ((byte_code & 2113929216) >> 20)
                + ((byte_code & 3840) >> 7),
        }
    }
}

//upper immediate
pub struct UType {
    opcode: u32,
    rd: u32,
    imm: u32,
}

impl UType {
    fn from(byte_code: u32) -> Self {
        Self {
            opcode: byte_code & 127,
            rd: (byte_code & 3968) >> 6,
            imm: (byte_code & 4294963200) >> 12,
        }
    }
}

//jump
pub struct JType {
    opcode: u32,
    rd: u32,
    imm: u32,
}

impl JType {
    fn from(byte_code: u32) -> Self {
        Self {
            opcode: byte_code & 127,
            rd: (byte_code & 3968) >> 6,
            imm: {
                let mut arr = [0; 32];
                let mut imm = [0; 32];
                for i in 0..31 {
                    arr[i] = byte_code & (1 << i);
                }

                imm[20] = arr[31];
                for i in 19..12 {
                    imm[i] = arr[i];
                }
                imm[11] = arr[20];
                for i in 10..1 {
                    imm[i] = arr[i + 20]
                }

                let mut sum = 0;
                for i in 0..31 {
                    sum <<= 1;
                    sum += imm[i];
                }
                sum
            },
        }
    }
}

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
