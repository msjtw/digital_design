use crate::core::Core;
use crate::core::instr_parse::{BType, IType, InstructionError, JType, RType, SType, UType};

use super::{ExecError, State};

pub fn exec_r(core: &mut Core, instr: &RType) -> Result<State, ExecError> {
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
        0b0101111 => {
            let tmp;
            match core
                .memory
                .get_word(core.reg_file[instr.rd as usize] as u32)
            {
                Ok(x) => tmp = x as i32,
                Err(x) => {
                    core.trap = x as i32;
                    return Ok(State::Ok);
                }
            };
            match instr.funct5 {
                // LR.W
                0b00010 => {
                    core.reg_file[instr.rd as usize] = tmp;
                    core.lr_address = core.reg_file[instr.rs1 as usize] as u32;
                    core.lr_valid = true;
                    core.pc += 4;
                    return Ok(State::Ok);
                }
                // SC.W
                0b00011 => {
                    if core.lr_valid
                        && (core.lr_address == core.reg_file[instr.rs1 as usize] as u32)
                    {
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
                    return Ok(State::Ok);
                }
                // amoswap.w
                0b00001 => {
                    core.reg_file[instr.rd as usize] = tmp;
                    core.reg_file.swap(instr.rd as usize, instr.rs2 as usize);
                }
                // amoadd.w
                0b00000 => {
                    core.reg_file[instr.rd as usize] = tmp + core.reg_file[instr.rs2 as usize];
                }
                // amoxor.w
                0b00100 => {
                    core.reg_file[instr.rd as usize] = tmp ^ core.reg_file[instr.rs2 as usize];
                }
                // amoand.w
                0b01100 => {
                    core.reg_file[instr.rd as usize] = tmp & core.reg_file[instr.rs2 as usize];
                }
                // amoor.w
                0b01000 => {
                    core.reg_file[instr.rd as usize] = tmp | core.reg_file[instr.rs2 as usize];
                }
                //amomin.w
                0b10000 => {
                    core.reg_file[instr.rd as usize] = tmp.min(core.reg_file[instr.rs2 as usize]);
                }
                // amomax.w
                0b10100 => {
                    core.reg_file[instr.rd as usize] = tmp.max(core.reg_file[instr.rs2 as usize]);
                }
                // amominu.w
                0b11000 => {
                    core.reg_file[instr.rd as usize] =
                        (tmp as u32).min(core.reg_file[instr.rs2 as usize] as u32) as i32;
                }
                // amomaxiu.w
                0b11100 => {
                    core.reg_file[instr.rd as usize] =
                        (tmp as u32).max(core.reg_file[instr.rs2 as usize] as u32) as i32;
                }
                _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
            }
            core.memory.insert_word(
                core.reg_file[instr.rs1 as usize] as u32,
                core.reg_file[instr.rd as usize] as u32,
            );
            core.pc += 4;
        }
        _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
    };

    Ok(State::Ok)
}

pub fn exec_i(core: &mut Core, instr: &IType) -> Result<State, ExecError> {
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
                    match core.memory.get_byte(addr) {
                        Ok(x) => core.reg_file[instr.rd as usize] = x.into(),
                        Err(x) => {
                            core.trap = x as i32;
                            return Ok(State::Ok);
                        }
                    };
                }
                // lh
                0x1 => {
                    match core.memory.get_hword(addr) {
                        Ok(x) => core.reg_file[instr.rd as usize] = x.into(),
                        Err(x) => {
                            core.trap = x as i32;
                            return Ok(State::Ok);
                        }
                    };
                }
                // lw
                0x2 => {
                    match core.memory.get_word(addr) {
                        Ok(x) => core.reg_file[instr.rd as usize] = x as i32,
                        Err(x) => {
                            core.trap = x as i32;
                            return Ok(State::Ok);
                        }
                    };
                }
                // lbu zero-extended
                0x4 => {
                    match core.memory.get_byte(addr) {
                        Ok(x) => core.reg_file[instr.rd as usize] = x as i32,
                        Err(x) => {
                            core.trap = x as i32;
                            return Ok(State::Ok);
                        }
                    };
                }
                // lhu
                0x5 => {
                    match core.memory.get_hword(addr) {
                        Ok(x) => core.reg_file[instr.rd as usize] = x as i32,
                        Err(x) => {
                            core.trap = x as i32;
                            return Ok(State::Ok);
                        }
                    };
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
            // csr_filerw
            0b001 => {
                core.reg_file[instr.rd as usize] = core.csr_file[instr.imm as usize] as i32;
                core.csr_file[instr.imm as usize] = core.reg_file[instr.rs1 as usize] as u32;
                core.pc += 4;
            }
            // csr_filers
            0b010 => {
                core.reg_file[instr.rd as usize] = core.csr_file[instr.imm as usize] as i32;
                core.csr_file[instr.imm as usize] |= core.reg_file[instr.rs1 as usize] as u32;
                core.pc += 4;
            }
            // csr_filerc
            0b011 => {
                core.reg_file[instr.rd as usize] = core.csr_file[instr.imm as usize] as i32;
                core.csr_file[instr.imm as usize] &= !core.reg_file[instr.rs1 as usize] as u32;
                core.pc += 4;
            }
            // csr_filerwi
            0b101 => {
                core.reg_file[instr.rd as usize] = core.csr_file[instr.imm as usize] as i32;
                core.csr_file[instr.imm as usize] = instr.rs1;
                core.pc += 4;
            }
            // csr_filersi
            0b110 => {
                core.reg_file[instr.rd as usize] = core.csr_file[instr.imm as usize] as i32;
                core.csr_file[instr.imm as usize] |= instr.rs1;
                core.pc += 4;
            }
            // csr_filerci
            0b111 => {
                core.reg_file[instr.rd as usize] = core.csr_file[instr.imm as usize] as i32;
                core.csr_file[instr.imm as usize] &= !instr.rs1;
                core.pc += 4;
            }
            0b0 => {
                match instr.imm {
                    //ecall
                    0b0 => {
                        if core.mode == 3 {
                            // machine ecall
                            core.trap = 11;
                        } else {
                            // user ecall
                            core.trap = 8;
                        }
                        core.pc += 4;
                    }
                    //ebreak
                    0b1 => {
                        core.trap = 3;
                        core.pc += 4;
                    }
                    _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
                };
                core.pc += 4;
            }
            // mret
            0b0011000 => {
                // restore last mode and save current
                core.mode = (*core.csr(super::Csr::Mstatus) >> 11) & 0b11;
                *core.csr(super::Csr::Mstatus) &= !0b11 << 11;
                *core.csr(super::Csr::Mstatus) |= core.mode << 11;
                // restore mie and set mpie to 1
                *core.csr(super::Csr::Mstatus) &= !0b1000;
                *core.csr(super::Csr::Mstatus) |= (*core.csr(super::Csr::Mstatus) & 1 << 7) >> 4;
                *core.csr(super::Csr::Mstatus) |= 1 << 7;
                // restore pc
                core.pc = *core.csr(super::Csr::Mepc);
            }
            0b0001000 => match instr.imm {
                // wfi
                0b000100000101 => {
                    *core.csr(super::Csr::Mstatus) &= 1 << 3;
                    core.wfi = true;
                    core.pc += 4;
                    return Ok(State::Sleep);
                }
                _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
            },
            _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
        },
        // fence, pause
        0b0001111 => core.pc += 4,

        _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
    };
    Ok(State::Ok)
}

pub fn exec_s(core: &mut Core, instr: &SType) -> Result<State, ExecError> {
    let addr = (core.reg_file[instr.rs1 as usize] + instr.imm) as u32;
    let x = match instr.funct3 {
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
    match x {
        Ok(_) => core.pc += 4,
        Err(x) => core.trap = x as i32,
    }
    Ok(State::Ok)
}

pub fn exec_b(core: &mut Core, instr: &BType) -> Result<State, ExecError> {
    let rs1 = core.reg_file[instr.rs1 as usize];
    let rs2 = core.reg_file[instr.rs2 as usize];
    match instr.funct3 {
        //beq
        0x0 => {
            if rs1 == rs2 {
                core.pc = (core.pc as i32 + instr.imm) as u32;
            };
        }
        //bne
        0x1 => {
            if rs1 != rs2 {
                core.pc = (core.pc as i32 + instr.imm) as u32;
            };
        }
        //blt
        0x4 => {
            if rs1 < rs2 {
                core.pc = (core.pc as i32 + instr.imm) as u32;
            };
        }
        //bge
        0x5 => {
            if rs1 >= rs2 {
                core.pc = (core.pc as i32 + instr.imm) as u32;
            };
        }
        //bltu
        0x6 => {
            if (rs1 as u32) < (rs2 as u32) {
                core.pc = (core.pc as i32 + instr.imm) as u32;
            };
        }
        //bgeu
        0x7 => {
            if rs1 as u32 >= rs2 as u32 {
                core.pc = (core.pc as i32 + instr.imm) as u32;
            };
        }
        _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
    };
    core.pc += 4;
    Ok(State::Ok)
}

pub fn exec_u(core: &mut Core, instr: &UType) -> Result<State, ExecError> {
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
    Ok(State::Ok)
}

pub fn exec_j(core: &mut Core, instr: &JType) -> Result<State, ExecError> {
    match instr.opcode {
        //jal
        0b1101111 => {
            core.reg_file[instr.rd as usize] = (core.pc + 4) as i32;
            core.pc = (core.pc as i32 + instr.imm) as u32;
        }
        _ => return Err(ExecError::InstructionError(InstructionError::NoInstruction)),
    };
    Ok(State::Ok)
}
