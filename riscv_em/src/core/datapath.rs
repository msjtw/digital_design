use crate::core::Core;
use crate::core::instr_parse::{BType, IType, InstructionError, JType, RType, SType, UType};
use crate::core::syscalls;

use super::ExecError;

pub fn exec_r(core: &mut Core, instr: &RType) -> Result<(), ExecError> {
    match instr.opcode {
        0b0110011 => {
            match instr.funct3 {
                0x0 => match instr.funct7 {
                    //add
                    0x00 => {
                        core.reg_file[instr.rd as usize] =
                            core.reg_file[instr.rs1 as usize] + core.reg_file[instr.rs2 as usize];
                    }
                    //sub
                    0x20 => {
                        core.reg_file[instr.rd as usize] =
                            core.reg_file[instr.rs1 as usize] - core.reg_file[instr.rs2 as usize];
                    }
                    //mul
                    0x01 => {
                        let a: i64 = core.reg_file[instr.rs1 as usize].into();
                        let b: i64 = core.reg_file[instr.rs2 as usize].into();
                        let tmp = a * b;
                        core.reg_file[instr.rd as usize] = tmp as i32;
                    }
                    _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
                },
                0x4 => match instr.funct7 {
                    //xor
                    0x00 => {
                        core.reg_file[instr.rd as usize] =
                            core.reg_file[instr.rs1 as usize] ^ core.reg_file[instr.rs2 as usize];
                    }
                    //div
                    0x01 => {
                        core.reg_file[instr.rd as usize] =
                            core.reg_file[instr.rs1 as usize] / core.reg_file[instr.rs2 as usize];
                    }
                    _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
                },
                0x6 => match instr.funct7 {
                    //or
                    0x00 => {
                        core.reg_file[instr.rd as usize] =
                            core.reg_file[instr.rs1 as usize] | core.reg_file[instr.rs2 as usize];
                    }
                    //rem
                    0x01 => {
                        core.reg_file[instr.rd as usize] =
                            core.reg_file[instr.rs1 as usize] % core.reg_file[instr.rs2 as usize];
                    }
                    _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
                },
                0x7 => match instr.funct7 {
                    //and
                    0x00 => {
                        core.reg_file[instr.rd as usize] =
                            core.reg_file[instr.rs1 as usize] & core.reg_file[instr.rs2 as usize];
                    }
                    //remu
                    0x01 => {
                        core.reg_file[instr.rd as usize] = (core.reg_file[instr.rs1 as usize]
                            as u32
                            % core.reg_file[instr.rs2 as usize] as u32)
                            as i32;
                    }
                    _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
                },
                0x1 => match instr.funct7 {
                    //sll
                    0x00 => {
                        core.reg_file[instr.rd as usize] =
                            core.reg_file[instr.rs1 as usize] << core.reg_file[instr.rs2 as usize];
                    }
                    //mulh
                    0x01 => {
                        let a: i64 = core.reg_file[instr.rs1 as usize].into();
                        let b: i64 = core.reg_file[instr.rs2 as usize].into();
                        let tmp = (a * b) >> 32;
                        core.reg_file[instr.rd as usize] = tmp as i32;
                    }
                    _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
                },
                0x5 => match instr.funct7 {
                    //srl
                    0x00 => {
                        core.reg_file[instr.rd as usize] = (core.reg_file[instr.rs1 as usize]
                            as u32
                            >> core.reg_file[instr.rs2 as usize])
                            as i32;
                    }
                    //divu
                    0x01 => {
                        core.reg_file[instr.rd as usize] = (core.reg_file[instr.rs1 as usize]
                            as u32
                            / core.reg_file[instr.rs2 as usize] as u32)
                            as i32;
                    }
                    //sra
                    0x02 => {
                        core.reg_file[instr.rd as usize] =
                            core.reg_file[instr.rs1 as usize] >> core.reg_file[instr.rs2 as usize];
                    }
                    _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
                },
                0x2 => match instr.funct7 {
                    //slt
                    0x00 => {
                        core.reg_file[instr.rd as usize] = if core.reg_file[instr.rs1 as usize]
                            < core.reg_file[instr.rs2 as usize]
                        {
                            1
                        } else {
                            0
                        };
                    }
                    //mulsu
                    0x01 => {
                        let a: i64 = core.reg_file[instr.rs1 as usize].into();
                        let b: i64 = core.reg_file[instr.rs2 as usize] as i64;
                        let tmp = (a * b) >> 32;
                        core.reg_file[instr.rd as usize] = tmp as i32;
                    }
                    _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
                },
                0x3 => match instr.funct7 {
                    //sltu
                    0x00 => {
                        core.reg_file[instr.rd as usize] = if (core.reg_file[instr.rs1 as usize]
                            as u32)
                            < (core.reg_file[instr.rs2 as usize] as u32)
                        {
                            1
                        } else {
                            0
                        };
                    }
                    //mulu
                    0x01 => {
                        let a: i64 = core.reg_file[instr.rs1 as usize] as i64;
                        let b: i64 = core.reg_file[instr.rs2 as usize] as i64;
                        let tmp = (a * b) >> 32;
                        core.reg_file[instr.rd as usize] = tmp as i32;
                    }
                    _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
                },

                _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
            };
            core.pc += 4;
        }
        0b0101111 => match instr.funct5 {
            // LR.W
            0b00010 => {
                core.lr_address = core.reg_file[instr.rs1 as usize] as u32;
                core.lr_valid = true;
                core.reg_file[instr.rd as usize] = core
                    .memory
                    .get_word(core.reg_file[instr.rs1 as usize] as u32)
                    as i32;
                core.pc += 4;
            }
            // SC.W
            0b00011 => {
                if core.lr_valid && (core.lr_address == core.reg_file[instr.rs1 as usize] as u32) {
                    core.memory.insert_word(
                        core.reg_file[instr.rs1 as usize] as u32,
                        core.reg_file[instr.rs2 as usize] as u32,
                    );
                    core.reg_file[instr.rd as usize] = 0;
                } else {
                    core.reg_file[instr.rd as usize] = 1;
                }
                core.lr_address = 0;
                core.lr_valid = false;
                core.pc += 4;
            }
            // amoswap.w
            0b00001 => {
                core.reg_file[instr.rd as usize] = core
                    .memory
                    .get_word(core.reg_file[instr.rs1 as usize] as u32)
                    as i32;
                core.reg_file.swap(instr.rd as usize, instr.rs2 as usize);
                core.memory.insert_word(
                    core.reg_file[instr.rs1 as usize] as u32,
                    core.reg_file[instr.rd as usize] as u32,
                );
                core.pc += 4;
            }
            // amoadd.w
            0b00000 => {
                core.reg_file[instr.rd as usize] = core
                    .memory
                    .get_word(core.reg_file[instr.rs1 as usize] as u32)
                    as i32
                    + core.reg_file[instr.rs2 as usize];
                core.memory.insert_word(
                    core.reg_file[instr.rs1 as usize] as u32,
                    core.reg_file[instr.rd as usize] as u32,
                );
                core.pc += 4;
            }
            // amoxor.w
            0b00100 => {
                core.reg_file[instr.rd as usize] = core
                    .memory
                    .get_word(core.reg_file[instr.rs1 as usize] as u32)
                    as i32
                    ^ core.reg_file[instr.rs2 as usize];
                core.memory.insert_word(
                    core.reg_file[instr.rs1 as usize] as u32,
                    core.reg_file[instr.rd as usize] as u32,
                );
                core.pc += 4;
            }
            // amoand.w
            0b01100 => {
                core.reg_file[instr.rd as usize] = core
                    .memory
                    .get_word(core.reg_file[instr.rs1 as usize] as u32)
                    as i32
                    & core.reg_file[instr.rs2 as usize];
                core.memory.insert_word(
                    core.reg_file[instr.rs1 as usize] as u32,
                    core.reg_file[instr.rd as usize] as u32,
                );
                core.pc += 4;
            }
            // amoor.w
            0b01000 => {
                core.reg_file[instr.rd as usize] = core
                    .memory
                    .get_word(core.reg_file[instr.rs1 as usize] as u32)
                    as i32
                    | core.reg_file[instr.rs2 as usize];
                core.memory.insert_word(
                    core.reg_file[instr.rs1 as usize] as u32,
                    core.reg_file[instr.rd as usize] as u32,
                );
                core.pc += 4;
            }
            //amomin.w
            0b10000 => {
                let tmp = core
                    .memory
                    .get_word(core.reg_file[instr.rd as usize] as u32)
                    as i32;
                core.reg_file[instr.rd as usize] = tmp.min(core.reg_file[instr.rs2 as usize]);
                core.memory.insert_word(
                    core.reg_file[instr.rs1 as usize] as u32,
                    core.reg_file[instr.rd as usize] as u32,
                );
                core.pc += 4;
            }
            // amomax.w
            0b10100 => {
                let tmp = core
                    .memory
                    .get_word(core.reg_file[instr.rd as usize] as u32)
                    as i32;
                core.reg_file[instr.rd as usize] = tmp.max(core.reg_file[instr.rs2 as usize]);
                core.memory.insert_word(
                    core.reg_file[instr.rs1 as usize] as u32,
                    core.reg_file[instr.rd as usize] as u32,
                );
                core.pc += 4;
            }
            // amominu.w
            0b11000 => {
                let tmp = core
                    .memory
                    .get_word(core.reg_file[instr.rd as usize] as u32);
                core.reg_file[instr.rd as usize] =
                    tmp.min(core.reg_file[instr.rs2 as usize] as u32) as i32;
                core.memory.insert_word(
                    core.reg_file[instr.rs1 as usize] as u32,
                    core.reg_file[instr.rd as usize] as u32,
                );
                core.pc += 4;
            }
            // amomaxiu.w
            0b11100 => {
                let tmp = core
                    .memory
                    .get_word(core.reg_file[instr.rd as usize] as u32);
                core.reg_file[instr.rd as usize] =
                    tmp.max(core.reg_file[instr.rs2 as usize] as u32) as i32;
                core.memory.insert_word(
                    core.reg_file[instr.rs1 as usize] as u32,
                    core.reg_file[instr.rd as usize] as u32,
                );
                core.pc += 4;
            }
            _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
        },
        _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
    };

    Ok(())
}

pub fn exec_i(core: &mut Core, instr: &IType) -> Result<(), ExecError> {
    match instr.opcode {
        0b0010011 => {
            match instr.funct3 {
                //addi
                0x0 => {
                    core.reg_file[instr.rd as usize] =
                        core.reg_file[instr.rs1 as usize] + instr.imm;
                }
                //xori
                0x4 => {
                    core.reg_file[instr.rd as usize] =
                        core.reg_file[instr.rs1 as usize] ^ instr.imm;
                }
                //ori
                0x6 => {
                    core.reg_file[instr.rd as usize] =
                        core.reg_file[instr.rs1 as usize] | instr.imm;
                }
                //andi
                0x7 => {
                    core.reg_file[instr.rd as usize] =
                        core.reg_file[instr.rs1 as usize] & instr.imm;
                }
                //slli
                0x1 => {
                    core.reg_file[instr.rd as usize] =
                        core.reg_file[instr.rs1 as usize] << (instr.imm & 31);
                }
                0x5 => match instr.funct7 {
                    //srli
                    0x00 => {
                        core.reg_file[instr.rd as usize] =
                            (core.reg_file[instr.rs1 as usize] as u32 >> (instr.imm & 31)) as i32;
                    }
                    //srai
                    0x20 => {
                        core.reg_file[instr.rd as usize] =
                            core.reg_file[instr.rs1 as usize] >> (instr.imm & 31);
                    }
                    _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
                },
                //slti
                0x2 => {
                    core.reg_file[instr.rd as usize] =
                        if core.reg_file[instr.rs1 as usize] < instr.imm {
                            1
                        } else {
                            0
                        };
                }
                //sltiu
                0x3 => {
                    core.reg_file[instr.rd as usize] =
                        if (core.reg_file[instr.rs1 as usize] as u32) < (instr.imm as u32) {
                            1
                        } else {
                            0
                        };
                }
                _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
            };
            core.pc += 4;
        }
        0b0000011 => {
            let addr = (core.reg_file[instr.rs1 as usize] + instr.imm) as u32;
            match instr.funct3 {
                // lb sign-extended
                0x0 => {
                    core.reg_file[instr.rd as usize] = core.memory.get_byte(addr).into();
                }
                // lh
                0x1 => {
                    core.reg_file[instr.rd as usize] = core.memory.get_hword(addr).into();
                }
                // lw
                0x2 => {
                    core.reg_file[instr.rd as usize] = core.memory.get_word(addr) as i32;
                }
                // lbu zero-extended
                0x4 => {
                    core.reg_file[instr.rd as usize] = core.memory.get_byte(addr) as i32;
                }
                // lhu
                0x5 => {
                    core.reg_file[instr.rd as usize] = core.memory.get_hword(addr) as i32;
                }
                _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
            };
            core.pc += 4;
        }
        //jalr
        0b1100111 => {
            core.reg_file[instr.rd as usize] = (core.pc + 4) as i32;
            core.pc = (core.reg_file[instr.rs1 as usize] as i32 + instr.imm) as u32;
        }
        0b1110011 => match instr.funct7 {
            //ecall
            0x0 => {
                let syscall = syscalls::SystemCall::from(core);
                syscall.exec(core)?;
                core.pc += 4;
            }
            // csrrw
            0b001 => {
                core.reg_file[instr.rd as usize] = core.csr[instr.imm as usize] as i32;
                core.csr[instr.imm as usize] = core.reg_file[instr.rs1 as usize] as u32;
                core.pc += 4;
            }
            // csrrs
            0b010 => {
                core.reg_file[instr.rd as usize] = core.csr[instr.imm as usize] as i32;
                core.csr[instr.imm as usize] |= core.reg_file[instr.rs1 as usize] as u32;
                core.pc += 4;
            }
            // csrrc
            0b011 => {
                core.reg_file[instr.rd as usize] = core.csr[instr.imm as usize] as i32;
                core.csr[instr.imm as usize] &= !core.reg_file[instr.rs1 as usize] as u32;
                core.pc += 4;
            }
            // csrrwi
            0b101 => {
                core.reg_file[instr.rd as usize] = core.csr[instr.imm as usize] as i32;
                core.csr[instr.imm as usize] = instr.rs1;
                core.pc += 4;
            }
            // csrrsi
            0b110 => {
                core.reg_file[instr.rd as usize] = core.csr[instr.imm as usize] as i32;
                core.csr[instr.imm as usize] |= instr.rs1;
                core.pc += 4;
            }
            // csrrci
            0b111 => {
                core.reg_file[instr.rd as usize] = core.csr[instr.imm as usize] as i32;
                core.csr[instr.imm as usize] &= !instr.rs1;
                core.pc += 4;
            }
            // ebreak
            _ => return Err(ExecError::InstructionError(InstructionError::NotSupported)),
        },
        // fence, pause
        0b0001111 => core.pc += 4,

        _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
    };
    Ok(())
}

pub fn exec_s(core: &mut Core, instr: &SType) -> Result<(), ExecError> {
    let addr = (core.reg_file[instr.rs1 as usize] + instr.imm) as u32;
    match instr.funct3 {
        //sb
        0x0 => core
            .memory
            .insert_byte(addr, core.reg_file[instr.rs2 as usize] as u8),
        //sh
        0x1 => core
            .memory
            .insert_hword(addr, core.reg_file[instr.rs2 as usize] as u16),
        //sw
        0x2 => core
            .memory
            .insert_word(addr, core.reg_file[instr.rs2 as usize] as u32),
        _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
    };
    core.pc += 4;
    Ok(())
}

pub fn exec_b(core: &mut Core, instr: &BType) -> Result<(), ExecError> {
    let rs1 = core.reg_file[instr.rs1 as usize];
    let rs2 = core.reg_file[instr.rs2 as usize];
    match instr.funct3 {
        //beq
        0x0 => {
            if rs1 == rs2 {
                core.pc = (core.pc as i32 + instr.imm) as u32;
                return Ok(());
            }
        }
        //bne
        0x1 => {
            if rs1 != rs2 {
                core.pc = (core.pc as i32 + instr.imm) as u32;
                return Ok(());
            }
        }
        //blt
        0x4 => {
            if rs1 < rs2 {
                core.pc = (core.pc as i32 + instr.imm) as u32;
                return Ok(());
            }
        }
        //bge
        0x5 => {
            if rs1 >= rs2 {
                core.pc = (core.pc as i32 + instr.imm) as u32;
                return Ok(());
            }
        }
        //bltu
        0x6 => {
            if (rs1 as u32) < (rs2 as u32) {
                core.pc = (core.pc as i32 + instr.imm) as u32;
                return Ok(());
            }
        }
        //bgeu
        0x7 => {
            if rs1 as u32 >= rs2 as u32 {
                core.pc = (core.pc as i32 + instr.imm) as u32;
                return Ok(());
            };
        }
        _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
    };
    core.pc += 4;
    Ok(())
}

pub fn exec_u(core: &mut Core, instr: &UType) -> Result<(), ExecError> {
    match instr.opcode {
        //lui
        0b0110111 => {
            core.reg_file[instr.rd as usize] = instr.imm << 12;
        }
        //auipc
        0b0010111 => {
            core.reg_file[instr.rd as usize] = core.pc as i32 + (instr.imm << 12);
        }
        _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
    };
    core.pc += 4;
    Ok(())
}

pub fn exec_j(core: &mut Core, instr: &JType) -> Result<(), ExecError> {
    match instr.opcode {
        //jal
        0b1101111 => {
            core.reg_file[instr.rd as usize] = (core.pc + 4) as i32;
            core.pc = (core.pc as i32 + instr.imm) as u32;
        }
        _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
    };
    Ok(())
}
