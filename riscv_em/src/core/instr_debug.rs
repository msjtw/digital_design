// There are waaaaay to many errors in this code.

use std::fmt::format;

use crate::core::Core;
use crate::core::csr::csr_name;
use crate::core::instr_parse::{BType, IType, Instruction, JType, RType, SType, UType};
use crate::{SPIKE_DEBUG, memory};

use super::{Exception, State, csr};

fn debug_r(core: &Core, instr: &RType) -> String {
    match instr.opcode {
        0b0110011 => {
            match instr.funct3 {
                0x0 => match instr.funct7 {
                    //add
                    0x00 => format!(
                        "add\t(x{}){} = (x{}){} + (x{}){}",
                        instr.rd,
                        (i64::from(core.reg_file[instr.rs1 as usize])
                            + i64::from(core.reg_file[instr.rs2 as usize]))
                            as i32,
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.rs2,
                        core.reg_file[instr.rs2 as usize]
                    ),
                    //sub
                    0x20 => format!(
                        "sub\t(x{}){} = (x{}){} - (x{}){}",
                        instr.rd,
                        (i64::from(core.reg_file[instr.rs1 as usize])
                            - i64::from(core.reg_file[instr.rs2 as usize]))
                            as i32,
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.rs2,
                        core.reg_file[instr.rs2 as usize]
                    ),

                    //mul
                    0x01 => {
                        let a: i64 = core.reg_file[instr.rs1 as usize].into();
                        let b: i64 = core.reg_file[instr.rs2 as usize].into();
                        let tmp = a * b;
                        format!(
                            "mul\t(x{}){} = (x{}){} * (x{}){}",
                            instr.rd, tmp, instr.rs1, a, instr.rs2, b
                        )
                    }
                    _ => format!("unknown R type instruction {:?}", instr),
                },
                0x4 => match instr.funct7 {
                    //xor
                    0x00 => format!(
                        "xor\t(x{}){} = (x{}){} | (x{}){}",
                        instr.rd,
                        core.reg_file[instr.rs1 as usize] ^ core.reg_file[instr.rs2 as usize],
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.rs2,
                        core.reg_file[instr.rs2 as usize]
                    ),
                    //div
                    0x01 => format!(
                        "div\t(x{}){} = (x{}){} | (x{}){}",
                        instr.rd,
                        if core.reg_file[instr.rs2 as usize] == 0 {
                            0xffffffffu32 as i32
                        } else {
                            core.reg_file[instr.rs1 as usize] / core.reg_file[instr.rs2 as usize]
                        },
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.rs2,
                        core.reg_file[instr.rs2 as usize]
                    ),

                    _ => format!("unknown R type instruction {:?}", instr),
                },
                0x6 => match instr.funct7 {
                    //or
                    0x00 => format!(
                        "or\t(x{}){} = (x{}){} | (x{}){}",
                        instr.rd,
                        core.reg_file[instr.rs1 as usize] | core.reg_file[instr.rs2 as usize],
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.rs2,
                        core.reg_file[instr.rs2 as usize]
                    ),
                    //rem
                    0x01 => format!(
                        "rem\t(x{}) = (x{}){} % (x{}){}",
                        instr.rd,
                        // core.reg_file[instr.rs1 as usize] % core.reg_file[instr.rs2 as usize],
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.rs2,
                        core.reg_file[instr.rs2 as usize]
                    ),
                    _ => format!("unknown R type instruction {:?}", instr),
                },
                0x7 => match instr.funct7 {
                    //and
                    0x00 => format!(
                        "and\t(x{}){} = (x{}){} & (x{}){}",
                        instr.rd,
                        core.reg_file[instr.rs1 as usize] & core.reg_file[instr.rs2 as usize],
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.rs2,
                        core.reg_file[instr.rs2 as usize]
                    ),

                    //remu
                    0x01 => format!(
                        "remu\t(x{}) = (x{}){} % (x{}){}",
                        instr.rd,
                        // (core.reg_file[instr.rs1 as usize] as u32
                        //     % core.reg_file[instr.rs2 as usize] as u32)
                        //     as i32,
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.rs2,
                        core.reg_file[instr.rs2 as usize]
                    ),
                    _ => format!("unknown R type instruction {:?}", instr),
                },
                0x1 => match instr.funct7 {
                    //sll
                    0x00 => format!(
                        "sll\t(x{}){} = (x{}){} << (x{}){}",
                        instr.rd,
                        (u64::from(core.reg_file[instr.rs1 as usize] as u32)
                            << (core.reg_file[instr.rs2 as usize] as u32 & 31))
                            as i32,
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.rs2,
                        core.reg_file[instr.rs2 as usize]
                    ),
                    //mulh
                    0x01 => {
                        let a: i64 = core.reg_file[instr.rs1 as usize].into();
                        let b: i64 = core.reg_file[instr.rs2 as usize].into();
                        let tmp = (a * b) >> 32;
                        format!(
                            "mulh\t(x{}){} = (x{}){} * (x{}){}",
                            instr.rd, tmp, instr.rs1, a, instr.rs2, b
                        )
                    }
                    _ => format!("unknown R type instruction {:?}", instr),
                },
                0x5 => match instr.funct7 {
                    //srl
                    0x00 => format!(
                        "srl\t(x{}){} = (x{}){} >> (x{}){}",
                        instr.rd,
                        (u64::from(core.reg_file[instr.rs1 as usize] as u32)
                            >> (core.reg_file[instr.rs2 as usize] as u32 & 31))
                            as i32,
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.rs2,
                        core.reg_file[instr.rs2 as usize]
                    ),
                    //divu
                    0x01 => format!(
                        "divu\t(x{}){} = (x{}){} / (x{}){}",
                        instr.rd,
                        (core.reg_file[instr.rs1 as usize] as u32
                            / core.reg_file[instr.rs2 as usize] as u32)
                            as i32,
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.rs2,
                        core.reg_file[instr.rs2 as usize]
                    ),
                    //sra
                    0x20 => format!(
                        "sra\t(x{}){} = (x{}){} >> (x{}){}",
                        instr.rd,
                        core.reg_file[instr.rs1 as usize] >> core.reg_file[instr.rs2 as usize],
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.rs2,
                        core.reg_file[instr.rs2 as usize]
                    ),

                    _ => format!("unknown R type instruction {:?}", instr),
                },
                0x2 => match instr.funct7 {
                    //slt
                    0x00 => format!(
                        "slt\t(x{}){} = (x{}){} < (x{}){}",
                        instr.rd,
                        if core.reg_file[instr.rs1 as usize] < core.reg_file[instr.rs2 as usize] {
                            1
                        } else {
                            0
                        },
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.rs2,
                        core.reg_file[instr.rs2 as usize]
                    ),
                    //mulsu
                    0x01 => {
                        let a: i64 = core.reg_file[instr.rs1 as usize].into();
                        let b = u64::from(core.reg_file[instr.rs2 as usize] as u32) as i64;
                        let tmp = (a * b) >> 32;
                        format!(
                            "mulsu\t(x{}){} = (x{}){} * (x{}){}",
                            instr.rd, tmp, instr.rs1, a, instr.rs2, b
                        )
                    }
                    _ => format!("unknown R type instruction {:?}", instr),
                },
                0x3 => match instr.funct7 {
                    //sltu
                    0x00 => format!(
                        "sltu\t(x{}){} = (x{}){} < (x{}){}",
                        instr.rd,
                        if (core.reg_file[instr.rs1 as usize] as u32)
                            < (core.reg_file[instr.rs2 as usize] as u32)
                        {
                            1
                        } else {
                            0
                        },
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.rs2,
                        core.reg_file[instr.rs2 as usize]
                    ),
                    //mulu
                    0x01 => {
                        let a = u64::from(core.reg_file[instr.rs1 as usize] as u32);
                        let b = u64::from(core.reg_file[instr.rs2 as usize] as u32);
                        let tmp = (a * b) >> 32;
                        format!(
                            "mulu\t(x{}){} = (x{}){} * (x{}){}",
                            instr.rd, tmp, instr.rs1, a, instr.rs2, b
                        )
                    }
                    _ => format!("unknown R type instruction {:?}", instr),
                },

                _ => format!("unknown R type instruction {:?}", instr),
            }
        }
        0b0101111 => {
            match instr.funct5 {
                // LR.W
                0b00010 => format!("lr.w"),
                // SC.W
                0b00011 => format!("sc.w"),
                // amoswap.w
                0b00001 => format!(
                    "amoswap.w rd(x{}) rs1(x{}) rs2(x{})",
                    instr.rd, instr.rs1, instr.rs2
                ), // amoadd.w
                0b00000 => format!(
                    "amoadd.w rd(x{}) rs1(x{}) rs2(x{})",
                    instr.rd, instr.rs1, instr.rs2
                ), // amoxor.w
                0b00100 => format!(
                    "amoxor.w rd(x{}) rs1(x{}) rs2(x{})",
                    instr.rd, instr.rs1, instr.rs2
                ), // amoand.w
                0b01100 => format!(
                    "amoand.w rd(x{}) rs1(x{}) rs2(x{})",
                    instr.rd, instr.rs1, instr.rs2
                ), // amoor.w
                0b01000 => format!(
                    "amoor.w rd(x{}) rs1(x{}) rs2(x{})",
                    instr.rd, instr.rs1, instr.rs2
                ), //amomin.w
                0b10000 => format!(
                    "amomin.w rd(x{}) rs1(x{}) rs2(x{})",
                    instr.rd, instr.rs1, instr.rs2
                ), // amomax.w
                0b10100 => format!(
                    "amomax.w rd(x{}) rs1(x{}) rs2(x{})",
                    instr.rd, instr.rs1, instr.rs2
                ), // amominu.w
                0b11000 => format!(
                    "amominu.w rd(x{}) rs1(x{}) rs2(x{})",
                    instr.rd, instr.rs1, instr.rs2
                ), // amomaxiu.w
                0b11100 => format!(
                    "amomaxiu.w rd(x{}) rs1(x{}) rs2(x{})",
                    instr.rd, instr.rs1, instr.rs2
                ),
                _ => format!("unknown R type instruction {:?}", instr),
            }
        }
        _ => format!("unknown R type instruction {:?}", instr),
    }
}

fn debug_i(core: &mut Core, instr: &IType) -> String {
    match instr.opcode {
        0b0010011 => {
            match instr.funct3 {
                //addi
                0x0 => format!(
                    "addi\t(x{}) = {}\t(x{})0x{:x} + (imm){}",
                    instr.rd,
                    (i64::from(core.reg_file[instr.rs1 as usize]) + i64::from(instr.imm)) as i32,
                    instr.rs1,
                    core.reg_file[instr.rs1 as usize],
                    instr.imm
                ),
                //xori
                0x4 => format!(
                    "xori\t(x{}) = {}\t(x{})0x{:x} ^ (imm){}",
                    instr.rd,
                    core.reg_file[instr.rs1 as usize] ^ instr.imm,
                    instr.rs1,
                    core.reg_file[instr.rs1 as usize],
                    instr.imm
                ),
                //ori
                0x6 => format!(
                    "ori\t(x{}) = {}\t(x{})0x{:x} | (imm){}",
                    instr.rd,
                    core.reg_file[instr.rs1 as usize] | instr.imm,
                    instr.rs1,
                    core.reg_file[instr.rs1 as usize],
                    instr.imm
                ),
                //andi
                0x7 => format!(
                    "andi\t(x{}) = {}\t(x{})0x{:x} & (imm){}",
                    instr.rd,
                    core.reg_file[instr.rs1 as usize] & instr.imm,
                    instr.rs1,
                    core.reg_file[instr.rs1 as usize],
                    instr.imm
                ),
                //slli
                0x1 => format!(
                    "slli\t(x{}) = {}\t(x{})0x{:x} << (imm){}",
                    instr.rd,
                    core.reg_file[instr.rs1 as usize] << (instr.imm & 0b11111),
                    instr.rs1,
                    core.reg_file[instr.rs1 as usize],
                    instr.imm & 0b11111
                ),
                0x5 => match instr.funct7 {
                    //srli
                    0x00 => format!(
                        "srli\t(x{}) = {}\t(x{})0x{:x} >> (imm){}",
                        instr.rd,
                        (core.reg_file[instr.rs1 as usize] as u32 >> (instr.imm & 0b11111)) as i32,
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.imm & 0b11111
                    ),
                    //srai
                    0x20 => format!(
                        "srai\t(x{}) = {}\t(x{})0x{:x} >> (imm){}",
                        instr.rd,
                        core.reg_file[instr.rs1 as usize] >> (instr.imm & 0b11111),
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.imm & 0b11111
                    ),

                    _ => format!("unknown I type instruction {:?}", instr),
                },
                //slti
                0x2 => {
                    format!(
                        "slti\t(x{}) = {} \t(x{}){} < (imm){}",
                        instr.rd,
                        if core.reg_file[instr.rs1 as usize] < instr.imm {
                            1
                        } else {
                            0
                        },
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize],
                        instr.imm
                    )
                }
                //sltiu
                0x3 => {
                    format!(
                        "sltiu\t(x{}) = {} \t(x{}){} < (imm){}",
                        instr.rd,
                        if (core.reg_file[instr.rs1 as usize] as u32) < (instr.imm as u32) {
                            1
                        } else {
                            0
                        },
                        instr.rs1,
                        core.reg_file[instr.rs1 as usize] as u32,
                        instr.imm as u32
                    )
                }
                _ => format!("unknown I type instruction {:?}", instr),
            }
        }
        0b0000011 => {
            let addr = (core.reg_file[instr.rs1 as usize] + instr.imm) as u32;
            match instr.funct3 {
                // lb sign-extended
                0x0 => format!(
                    "lb\t(0x{}) = mem[(x{} + {:x})0x{:x}]",
                    instr.rd, instr.rs1, instr.imm, addr
                ),

                // lh
                0x1 => format!(
                    "lh\t(0x{}) = mem[(x{} + {:x})0x{:x}]",
                    instr.rd, instr.rs1, instr.imm, addr
                ),

                // lw
                0x2 => format!(
                    "lw\t(0x{}) = mem[(x{} + {:x})0x{:x}]",
                    instr.rd, instr.rs1, instr.imm, addr
                ),

                // lbu zero-extended
                0x4 => format!(
                    "lbu\t(0x{}) = mem[(x{} + {:x})0x{:x}] 0x{:x}, pa: 0x{:08x}",
                    instr.rd,
                    instr.rs1,
                    instr.imm,
                    addr,
                    memory::read_byte(addr, core).unwrap_or(0),
                    core.last_pa
                ),

                // lhu
                0x5 => format!(
                    "lhu\t(0x{}) = mem[(x{} + {:x})0x{:x}]",
                    instr.rd, instr.rs1, instr.imm, addr
                ),

                _ => format!("unknown I type instruction {:?}", instr),
            }
        }
        //jalr
        0b1100111 => {
            format!(
                "jalr\tpc = 0x{:08x}\t(x{}) = 0x{:08x}",
                (i64::from(core.reg_file[instr.rs1 as usize] as u32) + i64::from(instr.imm)) as u32,
                instr.rd,
                core.pc + 4
            )
        }
        0b1110011 => {
            let csr_addr = (instr.imm & 0xfff) as u32;
            let source = core.reg_file[instr.rs1 as usize] as u32;
            match instr.funct3 {
                // csrrw
                0b001 => {
                    let mut csr = 0;
                    if instr.rd != 0 {
                        csr = csr::read_addr(csr_addr, core).unwrap_or(0);
                    }
                    format!(
                        "csrrw\t {} = 0x{:x}\t(x{}) = 0x{:x}",
                        csr_name(csr_addr),
                        source,
                        instr.rd,
                        csr
                    )
                }
                // csrrs
                0b010 => {
                    let csr = csr::read_addr(csr_addr, core).unwrap_or(0);
                    format!(
                        "csrrs\t {} = 0x{:x}\t(x{}) = 0x{:x}",
                        csr_name(csr_addr),
                        csr | source,
                        instr.rd,
                        csr
                    )
                }
                // csrrc
                0b011 => {
                    let csr = csr::read_addr(csr_addr, core).unwrap_or(0);
                    format!(
                        "csrrc\t {} = 0x{:x}\t(x{}) = 0x{:x}",
                        csr_name(csr_addr),
                        csr & !source,
                        instr.rd,
                        csr
                    )
                }
                // csrrwi
                0b101 => {
                    let mut csr = 0;
                    if instr.rd != 0 {
                        csr = csr::read_addr(csr_addr, core).unwrap_or(0);
                    }
                    format!(
                        "csrrwi\t {} = 0x{:x}\t(x{}) = 0x{:x}",
                        csr_name(csr_addr),
                        instr.rs1,
                        instr.rd,
                        csr
                    )
                }
                // csrrsi
                0b110 => {
                    let csr = csr::read_addr(csr_addr, core).unwrap_or(0);
                    format!(
                        "csrrsi\t {} = 0x{:x}\t(x{}) = 0x{:x}",
                        csr_name(csr_addr),
                        csr | instr.rs1,
                        instr.rd,
                        csr
                    )
                }
                // csrrci
                0b111 => {
                    let csr = csr::read_addr(csr_addr, core).unwrap_or(0);
                    format!(
                        "csrrci\t {} = 0x{:x}\t(x{}) = 0x{:x}",
                        csr_name(csr_addr),
                        csr & !instr.rs1,
                        instr.rd,
                        csr
                    )
                }
                0b0 => {
                    //sfence
                    if instr.funct7 == 0b0001001 {
                        return format!("sfence");
                    }
                    match instr.imm {
                        //ecall
                        0b0 => format!("ecall from mode: {}", core.mode),
                        //ebreak
                        0b1 => format!("ebreak"),
                        // mret
                        0b001100000010 => format!("mret"),
                        // sret
                        0b000100000010 => format!("sret"),
                        // wfi
                        0b000100000101 => format!("wait for interrupt"),

                        _ => format!("unknown I type instruction {:?}", instr),
                    }
                }
                _ => format!("unknown I type instruction {:?}", instr),
            }
        }
        // fence, pause
        0b0001111 => format!("fence, pause"),

        _ => format!("unknown I type instruction {:?}", instr),
    }
}

fn debug_s(core: &Core, instr: &SType) -> String {
    let addr = (core.reg_file[instr.rs1 as usize] + instr.imm) as u32;
    let rs2 = core.reg_file[instr.rs2 as usize];
    match instr.funct3 {
        //sb
        0x0 => format!(
            "sw\t mem[(x{}+{:x})0x{:08x}] = (x{})0x{:x}",
            instr.rs1, instr.imm, addr, instr.rs2, rs2 as u8
        ),
        //sh
        0x1 => format!(
            "sw\t mem[(x{}+{:x})0x{:08x}] = (x{})0x{:x}",
            instr.rs1, instr.imm, addr, instr.rs2, rs2 as u16
        ),
        //sw
        0x2 => format!(
            "sw\t mem[(x{}+{:x})0x{:08x}] = (x{})0x{:x}",
            instr.rs1, instr.imm, addr, instr.rs2, rs2 as u32
        ),
        _ => format!("unknown S type instruction {:?}", instr),
    }
}

fn debug_b(core: &Core, instr: &BType) -> String {
    let rs1 = core.reg_file[instr.rs1 as usize];
    let rs2 = core.reg_file[instr.rs2 as usize];
    match instr.funct3 {
        //beq
        0x0 => {
            format!(
                "beq\t if (x{}){rs1} == (x{}){rs2}; then pc = 0x{:x}",
                instr.rs1,
                instr.rs2,
                (core.pc as i32 + instr.imm) as u32
            )
        }
        //bne
        0x1 => {
            format!(
                "bne\t if (x{}){rs1} != (x{}){rs2}; then pc = 0x{:x}",
                instr.rs1,
                instr.rs2,
                (core.pc as i32 + instr.imm) as u32
            )
        }
        //blt
        0x4 => {
            format!(
                "blt\t if (x{}){rs1} < (x{}){rs2}; then pc = 0x{:x}",
                instr.rs1,
                instr.rs2,
                (core.pc as i32 + instr.imm) as u32
            )
        }
        //bge
        0x5 => {
            format!(
                "bge\t if (x{}){rs1} >= (x{}){rs2}; then pc = 0x{:x}",
                instr.rs1,
                instr.rs2,
                (core.pc as i32 + instr.imm) as u32
            )
        }
        //bltu
        0x6 => {
            format!(
                "bltu\t if (x{}){rs1} < (x{}){rs2}; then pc = 0x{:x}",
                instr.rs1 as u32,
                instr.rs2 as u32,
                (core.pc as i32 + instr.imm) as u32
            )
        }
        //bgeu
        0x7 => {
            format!(
                "bgeu\t if (x{}){rs1} >= (x{}){rs2}; then pc = 0x{:x}",
                instr.rs1 as u32,
                instr.rs2 as u32,
                (core.pc as i32 + instr.imm) as u32
            )
        }
        _ => format!("unknown B type instruction {:?}", instr),
    }
}

fn debug_u(core: &Core, instr: &UType) -> String {
    match instr.opcode {
        //lui
        0b0110111 => {
            format!("lui\t rd(x{}) = 0x{:x}", instr.rd, (instr.imm << 12) as u32)
        }
        //auipc
        0b0010111 => {
            format!(
                "auipc\t rd(x{}) = 0x{:x}",
                instr.rd,
                (i64::from(core.pc) + i64::from(instr.imm << 12)) as u32
            )
        }
        _ => format!("unknown U type instruction {:?}", instr),
    }
}

fn debug_j(core: &Core, instr: &JType) -> String {
    match instr.opcode {
        //jal
        0b1101111 => {
            format!(
                "jarl\t rd(x{}) = 0x{:x}; pc = 0x{:x}",
                instr.rd,
                core.pc + 4,
                (i64::from(core.pc) + instr.imm as i64) as u32
            )
        }
        _ => format!("unknown J type instruction {:?}", instr),
    }
}

pub fn debug_instr(core: &mut Core, byte_code: u32) -> String {
    let instr = Instruction::from(byte_code);
    if SPIKE_DEBUG {
        match instr {
            Ok(x) => match x {
                Instruction::R(x) => spike_debug_r(core, &x),
                Instruction::I(x) => spike_debug_i(core, &x),
                Instruction::U(x) => spike_debug_u(core, &x),
                Instruction::J(x) => spike_debug_j(core, &x),
                Instruction::S(x) => spike_debug_s(core, &x),
                Instruction::B(x) => spike_debug_b(core, &x),
            },
            Err(_) => String::from("parse error"),
        }
    } else {
        match instr {
            Ok(x) => match x {
                Instruction::R(x) => debug_r(core, &x),
                Instruction::I(x) => debug_i(core, &x),
                Instruction::U(x) => debug_u(core, &x),
                Instruction::J(x) => debug_j(core, &x),
                Instruction::S(x) => debug_s(core, &x),
                Instruction::B(x) => debug_b(core, &x),
            },
            Err(_) => String::from("parse error"),
        }
    }
}

pub fn spike_debug_r(core: &mut Core, instr: &RType) -> String {
    return match instr.opcode {
        0b0110011 => {
            format!("x{} 0x{:08x}", instr.rd, core.reg_file[instr.rd as usize])
        }
        0b0101111 => {
            let mut write = true;
            let addr = core.reg_file[instr.rs1 as usize] as u32;
            let val;
            match memory::read_word(addr, core) {
                Ok(x) => val = x as i32,
                Err(x) => {
                    return format!("");
                }
            };
            let mut res = format!("");
            match instr.funct5 {
                // SC.W
                0b00011 => {
                    return format!(
                        "{res} x{} 0x{:08x} mem 0x{:08x} 0x{:08x}",
                        instr.rd, core.reg_file[instr.rd as usize], addr, val
                    );
                }
                _ => {}
            }
            if instr.funct5 != 0b00011 {
                if instr.rd == 0 {
                    res = format!("{res} mem 0x{:08x}", addr,);
                } else {
                    res = format!(
                        "{res} x{} 0x{:08x} mem 0x{:08x} ",
                        instr.rd, core.reg_file[instr.rd as usize], addr,
                    );
                }
                if write {
                    res = format!("{res} mem 0x{:08x} 0x{:08x}", addr, val);
                }
            }
            res
        }
        _ => return format!(""),
    };
}

pub fn spike_debug_i(core: &Core, instr: &IType) -> String {
    match instr.opcode {
        0b0010011 => {
            return if instr.rd != 0 {
                format!("x{} 0x{:08x}", instr.rd, core.reg_file[instr.rd as usize])
            } else {
                format!("")
            };
        }
        0b0000011 => {
            let addr = (core.reg_file[instr.rs1 as usize] + instr.imm) as u32;
            return format!(
                "x{} 0x{:08x} mem 0x{:08x}",
                instr.rd, core.reg_file[instr.rd as usize], addr
            );
        }
        //jalr
        0b1100111 => {
            if instr.rd != 0 {
                return format!("x{} 0x{:08x}", instr.rd, core.reg_file[instr.rd as usize]);
            } else {
                return format!("");
            }
        }
        0b1110011 => {
            let csr_addr = (instr.imm & 0xfff) as u32;
            let source = core.reg_file[instr.rs1 as usize] as u32;
            return match instr.funct3 {
                // csrrw
                0b001 => {
                    let mut res = format!("");
                    if instr.rd != 0 {
                        res = format!("x{} 0x{:08x}", instr.rd, core.reg_file[instr.rd as usize])
                    }
                    if csr_addr == 0x100 {
                        res = format!(
                            "{res} c768_mstatus 0x{:08x}",
                            csr::read(csr::Csr::mstatus, core)
                        )
                    } else {
                        res = format!(
                            "{res} c{}_{} 0x{:08x}",
                            csr_addr,
                            csr_name(csr_addr),
                            source
                        )
                    }
                    res
                }
                // csrrs
                0b010 => {
                    let csr;
                    match csr::read_addr(csr_addr, core) {
                        Ok(x) => csr = x,
                        Err(e) => {
                            return format!("");
                        }
                    };
                    let mut res = format!("");
                    if instr.rd != 0 {
                        res = format!("x{} 0x{:08x}", instr.rd, core.reg_file[instr.rd as usize])
                    }
                    if instr.rs1 != 0 {
                        if csr_addr == 0x100 {
                            res = format!(
                                "{res} c768_mstatus 0x{:08x}",
                                csr::read(csr::Csr::mstatus, core)
                            )
                        } else {
                            res =
                                format!("{res} c{}_{} 0x{:08x}", csr_addr, csr_name(csr_addr), csr)
                        }
                    }
                    res
                }
                // csrrc
                0b011 => {
                    let csr;
                    match csr::read_addr(csr_addr, core) {
                        Ok(x) => csr = x,
                        Err(e) => {
                            return format!("");
                        }
                    };
                    let mut res = format!("");
                    if instr.rd != 0 {
                        res = format!("x{} 0x{:08x}", instr.rd, core.reg_file[instr.rd as usize])
                    }
                    if instr.rs1 != 0 {
                        if csr_addr == 0x100 {
                            res = format!(
                                "{res} c768_mstatus 0x{:08x}",
                                csr::read(csr::Csr::mstatus, core)
                            )
                        } else {
                            res =
                                format!("{res} c{}_{} 0x{:08x}", csr_addr, csr_name(csr_addr), csr)
                        }
                    }
                    res
                }
                // csrrwi
                0b101 => {
                    let csr;
                    match csr::read_addr(csr_addr, core) {
                        Ok(x) => csr = x,
                        Err(e) => {
                            return format!("");
                        }
                    };

                    let mut res = format!("");
                    if instr.rd != 0 {
                        res = format!("x{} 0x{:08x}", instr.rd, core.reg_file[instr.rd as usize])
                    }
                    if csr_addr == 0x100 {
                        res = format!(
                            "{res} c768_mstatus 0x{:08x}",
                            csr::read(csr::Csr::mstatus, core)
                        )
                    } else {
                        res = format!(
                            "{res} c{}_{} 0x{:08x}",
                            csr_addr,
                            csr_name(csr_addr),
                            instr.rs1
                        )
                    }
                    res
                }
                // csrrsi
                0b110 => {
                    let csr;
                    match csr::read_addr(csr_addr, core) {
                        Ok(x) => csr = x,
                        Err(e) => {
                            return format!("");
                        }
                    };
                    let mut res = format!("");
                    if instr.rd != 0 {
                        res = format!("x{} 0x{:08x}", instr.rd, core.reg_file[instr.rd as usize])
                    }
                    if instr.rs1 != 0 {
                        if csr_addr == 0x100 {
                            res = format!(
                                "{res} c768_mstatus 0x{:08x}",
                                csr::read(csr::Csr::mstatus, core)
                            )
                        } else {
                            res = format!(
                                "{res} c{}_{} 0x{:08x}",
                                csr_addr,
                                csr_name(csr_addr),
                                csr | instr.rs1
                            )
                        }
                    }
                    res
                }
                // csrrci
                0b111 => {
                    let csr;
                    match csr::read_addr(csr_addr, core) {
                        Ok(x) => csr = x,
                        Err(e) => {
                            return format!("");
                        }
                    };
                    let mut res = format!("");
                    if instr.rd != 0 {
                        res = format!("x{} 0x{:08x}", instr.rd, core.reg_file[instr.rd as usize])
                    }
                    if instr.rs1 != 0 {
                        if csr_addr == 0x100 {
                            res = format!(
                                "{res} c768_mstatus 0x{:08x}",
                                csr::read(csr::Csr::mstatus, core)
                            )
                        } else {
                            res = format!(
                                "{res} c{}_{} 0x{:08x}",
                                csr_addr,
                                csr_name(csr_addr),
                                csr & !instr.rs1
                            )
                        }
                    }
                    res
                }
                0b0 => {
                    let res;
                    res = match instr.imm {
                        // mret
                        0b001100000010 => {
                            format!(
                                " c768_mstatus 0x{:08x} c784_mstatush 0x{:08x}",
                                csr::read(csr::Csr::mstatus, core),
                                csr::read(csr::Csr::mstatush, core)
                            )
                        }
                        _ => format!(""),
                    };
                    res
                }
                _ => return format!(""),
            };
        }
        _ => return format!(""),
    }
}

pub fn spike_debug_s(core: &Core, instr: &SType) -> String {
    let addr = (core.reg_file[instr.rs1 as usize] + instr.imm) as u32;
    let rs2 = core.reg_file[instr.rs2 as usize];

    return match instr.funct3 {
        //sb
        0x0 => format!("mem 0x{:08x} 0x{:02x}", addr, rs2 as u8),
        //sh
        0x1 => format!("mem 0x{:08x} 0x{:04x}", addr, rs2 as u16),
        //sw
        0x2 => format!("mem 0x{:08x} 0x{:08x}", addr, rs2),
        _ => format!(""),
    };
}

pub fn spike_debug_b(core: &Core, instr: &BType) -> String {
    format!("")
}

pub fn spike_debug_u(core: &Core, instr: &UType) -> String {
    format!("x{} 0x{:08x}", instr.rd, core.reg_file[instr.rd as usize])
}

pub fn spike_debug_j(core: &Core, instr: &JType) -> String {
    if instr.rd != 0 {
        return format!("x{} 0x{:08x}", instr.rd, core.reg_file[instr.rd as usize]);
    }
    format!("")
}
