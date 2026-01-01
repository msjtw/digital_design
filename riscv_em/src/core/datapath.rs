use crate::core::{Core, Hart};
use crate::core::csr::csr_name;
use crate::core::instr_parse::{BType, IType, JType, RType, SType, UType};
use crate::core::virt_memory;
use crate::memory::MemoryBus;

use super::{Exception, State, csr};

pub fn exec_r(hart: &mut Hart, bus: &mut MemoryBus, instr: &RType) -> Result<State, Exception> {
    match instr.opcode {
        0b0110011 => {
            match instr.funct3 {
                0x0 => match instr.funct7 {
                    //add
                    0x00 => {
                        hart.core.reg_file[instr.rd as usize] =
                            (i64::from(hart.core.reg_file[instr.rs1 as usize])
                                + i64::from(hart.core.reg_file[instr.rs2 as usize]))
                                as i32;
                    }
                    //sub
                    0x20 => {
                        hart.core.reg_file[instr.rd as usize] =
                            (i64::from(hart.core.reg_file[instr.rs1 as usize])
                                - i64::from(hart.core.reg_file[instr.rs2 as usize]))
                                as i32;
                    }
                    //mul
                    0x01 => {
                        let a: i64 = hart.core.reg_file[instr.rs1 as usize].into();
                        let b: i64 = hart.core.reg_file[instr.rs2 as usize].into();
                        let tmp = a * b;
                        hart.core.reg_file[instr.rd as usize] = tmp as i32;
                    }
                    _ => return Err(Exception::Illegal_instruction),
                },
                0x4 => match instr.funct7 {
                    //xor
                    0x00 => {
                        hart.core.reg_file[instr.rd as usize] =
                            hart.core.reg_file[instr.rs1 as usize] ^ hart.core.reg_file[instr.rs2 as usize];
                    }
                    //div
                    0x01 => {
                        if instr.rs2 == 0 {
                            hart.core.reg_file[instr.rd as usize] = 0xffffffffu32 as i32;
                        } else {
                            hart.core.reg_file[instr.rd as usize] = hart.core.reg_file[instr.rs1 as usize]
                                / hart.core.reg_file[instr.rs2 as usize];
                        }
                    }
                    _ => return Err(Exception::Illegal_instruction),
                },
                0x6 => match instr.funct7 {
                    //or
                    0x00 => {
                        hart.core.reg_file[instr.rd as usize] =
                            hart.core.reg_file[instr.rs1 as usize] | hart.core.reg_file[instr.rs2 as usize];
                    }
                    //rem
                    0x01 => {
                        hart.core.reg_file[instr.rd as usize] =
                            hart.core.reg_file[instr.rs1 as usize] % hart.core.reg_file[instr.rs2 as usize];
                    }
                    _ => return Err(Exception::Illegal_instruction),
                },
                0x7 => match instr.funct7 {
                    //and
                    0x00 => {
                        hart.core.reg_file[instr.rd as usize] =
                            hart.core.reg_file[instr.rs1 as usize] & hart.core.reg_file[instr.rs2 as usize];
                    }
                    //remu
                    0x01 => {
                        hart.core.reg_file[instr.rd as usize] = (hart.core.reg_file[instr.rs1 as usize]
                            as u32
                            % hart.core.reg_file[instr.rs2 as usize] as u32)
                            as i32;
                    }
                    _ => return Err(Exception::Illegal_instruction),
                },
                0x1 => match instr.funct7 {
                    //sll
                    0x00 => {
                        hart.core.reg_file[instr.rd as usize] =
                            (u64::from(hart.core.reg_file[instr.rs1 as usize] as u32)
                                << (hart.core.reg_file[instr.rs2 as usize] as u32 & 31))
                                as i32;
                    }
                    //mulh
                    0x01 => {
                        let a: i64 = hart.core.reg_file[instr.rs1 as usize].into();
                        let b: i64 = hart.core.reg_file[instr.rs2 as usize].into();
                        let tmp = (a * b) >> 32;
                        hart.core.reg_file[instr.rd as usize] = tmp as i32;
                    }
                    _ => return Err(Exception::Illegal_instruction),
                },
                0x5 => match instr.funct7 {
                    //srl
                    0x00 => {
                        hart.core.reg_file[instr.rd as usize] =
                            (u64::from(hart.core.reg_file[instr.rs1 as usize] as u32)
                                >> (hart.core.reg_file[instr.rs2 as usize] as u32 & 31))
                                as i32;
                    }
                    //divu
                    0x01 => {
                        hart.core.reg_file[instr.rd as usize] = (hart.core.reg_file[instr.rs1 as usize]
                            as u32
                            / hart.core.reg_file[instr.rs2 as usize] as u32)
                            as i32;
                    }
                    //sra
                    0x20 => {
                        hart.core.reg_file[instr.rd as usize] =
                            hart.core.reg_file[instr.rs1 as usize] >> hart.core.reg_file[instr.rs2 as usize];
                    }
                    _ => return Err(Exception::Illegal_instruction),
                },
                0x2 => match instr.funct7 {
                    //slt
                    0x00 => {
                        hart.core.reg_file[instr.rd as usize] = if hart.core.reg_file[instr.rs1 as usize]
                            < hart.core.reg_file[instr.rs2 as usize]
                        {
                            1
                        } else {
                            0
                        };
                    }
                    //mulsu
                    0x01 => {
                        let a: i64 = hart.core.reg_file[instr.rs1 as usize].into();
                        let b = u64::from(hart.core.reg_file[instr.rs2 as usize] as u32) as i64;
                        let tmp = (a * b) >> 32;
                        hart.core.reg_file[instr.rd as usize] = tmp as i32;
                    }
                    _ => return Err(Exception::Illegal_instruction),
                },
                0x3 => match instr.funct7 {
                    //sltu
                    0x00 => {
                        hart.core.reg_file[instr.rd as usize] = if (hart.core.reg_file[instr.rs1 as usize]
                            as u32)
                            < (hart.core.reg_file[instr.rs2 as usize] as u32)
                        {
                            1
                        } else {
                            0
                        };
                    }
                    //mulu
                    0x01 => {
                        let a = u64::from(hart.core.reg_file[instr.rs1 as usize] as u32);
                        let b = u64::from(hart.core.reg_file[instr.rs2 as usize] as u32);
                        let tmp = (a * b) >> 32;
                        hart.core.reg_file[instr.rd as usize] = tmp as i32;
                    }
                    _ => return Err(Exception::Illegal_instruction),
                },

                _ => return Err(Exception::Illegal_instruction),
            };
            if hart.core.p_start {
                hart.core.instr_str = format!(
                    "{} x{} 0x{:08x}",
                    hart.core.instr_str, instr.rd, hart.core.reg_file[instr.rd as usize]
                );
            }
            hart.core.pc += 4;
        }
        0b0101111 => {
            let mut write = true;
            let write_val;
            let addr = hart.core.reg_file[instr.rs1 as usize] as u32;
            let rs2 = hart.core.reg_file[instr.rs2 as usize];
            let rd;
            match virt_memory::virt_read_word(addr, hart, bus) {
                Ok(x) => rd = x as i32,
                Err(x) => {
                    return Err(x);
                }
            };
            hart.core.reg_file[instr.rd as usize] = rd;
            match instr.funct5 {
                // LR.W
                0b00010 => {
                    hart.core.reg_file[instr.rd as usize] = rd;
                    hart.core.lr_address = addr;
                    hart.core.lr_set = rd;
                    write = false;
                    write_val = 0;
                }
                // SC.W
                0b00011 => {
                    if (hart.core.lr_set == rd) && (hart.core.lr_address == addr) {
                        write_val = rs2;
                        hart.core.reg_file[instr.rd as usize] = 0;
                    } else {
                        hart.core.reg_file[instr.rd as usize] = 1;
                        write = false;
                        write_val = 0;
                    }
                    hart.core.lr_address = 0x0;
                    // if hart.core.p_start {
                    //     hart.core.instr_str = format!(
                    //         "{} x{} 0x{:08x} mem 0x{:08x} 0x{:08x}",
                    //         hart.core.instr_str,
                    //         instr.rd,
                    //         hart.core.reg_file[instr.rd as usize],
                    //         addr,
                    //         write_val
                    //     )
                    // }
                }
                // amoswap.w
                0b00001 => {
                    write_val = rs2;
                }
                // amoadd.w
                0b00000 => {
                    write_val = rd + rs2;
                }
                // amoxor.w
                0b00100 => {
                    write_val = rd ^ rs2;
                }
                // amoand.w
                0b01100 => {
                    write_val = rd & rs2;
                }
                // amoor.w
                0b01000 => {
                    write_val = rd | rs2;
                }
                //amomin.w
                0b10000 => {
                    write_val = rd.min(rs2);
                }
                // amomax.w
                0b10100 => {
                    write_val = rd.max(rs2);
                }
                // amominu.w
                0b11000 => {
                    write_val = (rd as u32).min(rs2 as u32) as i32;
                }
                // amomaxiu.w
                0b11100 => {
                    write_val = (rd as u32).max(rs2 as u32) as i32;
                }
                _ => return Err(Exception::Illegal_instruction),
            }
            if write {
                virt_memory::virt_write_word(addr, write_val as u32, hart, bus)?;
            }
            if hart.core.p_start {
                if instr.rd != 0 {
                    if instr.funct5 != 0b11 {
                        hart.core.instr_str = format!(
                            "{} x{} 0x{:08x} mem 0x{:08x} ",
                            hart.core.instr_str, instr.rd, hart.core.reg_file[instr.rd as usize], addr,
                        );
                    } else {
                        hart.core.instr_str = format!(
                            "{} x{} 0x{:08x} ",
                            hart.core.instr_str, instr.rd, hart.core.reg_file[instr.rd as usize],
                        );
                    }
                } else if instr.funct5 != 0b00011 {
                    hart.core.instr_str = format!("{} mem 0x{:08x}", hart.core.instr_str, addr,);
                }
                if write {
                    hart.core.instr_str =
                        format!("{} mem 0x{:08x} 0x{:08x}", hart.core.instr_str, addr, write_val);
                }
            }
            hart.core.pc += 4;
        }
        _ => return Err(Exception::Illegal_instruction),
    };

    Ok(State::Ok)
}

pub fn exec_i(hart: &mut Hart, bus: &mut MemoryBus, instr: &IType) -> Result<State, Exception> {
    match instr.opcode {
        0b0010011 => {
            match instr.funct3 {
                //addi
                0x0 => {
                    hart.core.reg_file[instr.rd as usize] =
                        (i64::from(hart.core.reg_file[instr.rs1 as usize]) + i64::from(instr.imm))
                            as i32;
                }
                //xori
                0x4 => {
                    hart.core.reg_file[instr.rd as usize] =
                        hart.core.reg_file[instr.rs1 as usize] ^ instr.imm;
                }
                //ori
                0x6 => {
                    hart.core.reg_file[instr.rd as usize] =
                        hart.core.reg_file[instr.rs1 as usize] | instr.imm;
                }
                //andi
                0x7 => {
                    hart.core.reg_file[instr.rd as usize] =
                        hart.core.reg_file[instr.rs1 as usize] & instr.imm;
                }
                //slli
                0x1 => {
                    hart.core.reg_file[instr.rd as usize] =
                        hart.core.reg_file[instr.rs1 as usize] << (instr.imm & 0b11111);
                }
                0x5 => match instr.funct7 {
                    //srli
                    0x00 => {
                        hart.core.reg_file[instr.rd as usize] =
                            (hart.core.reg_file[instr.rs1 as usize] as u32 >> (instr.imm & 0b11111))
                                as i32;
                    }
                    //srai
                    0x20 => {
                        hart.core.reg_file[instr.rd as usize] =
                            hart.core.reg_file[instr.rs1 as usize] >> (instr.imm & 0b11111);
                    }
                    _ => return Err(Exception::Illegal_instruction),
                },
                //slti
                0x2 => {
                    hart.core.reg_file[instr.rd as usize] =
                        if hart.core.reg_file[instr.rs1 as usize] < instr.imm {
                            1
                        } else {
                            0
                        };
                }
                //sltiu
                0x3 => {
                    hart.core.reg_file[instr.rd as usize] =
                        if (hart.core.reg_file[instr.rs1 as usize] as u32) < (instr.imm as u32) {
                            1
                        } else {
                            0
                        };
                }
                _ => return Err(Exception::Illegal_instruction),
            };
            if hart.core.p_start && instr.rd != 0 {
                hart.core.instr_str = format!(
                    "{} x{} 0x{:08x}",
                    hart.core.instr_str, instr.rd, hart.core.reg_file[instr.rd as usize]
                );
            }
            hart.core.pc += 4;
        }
        0b0000011 => {
            let addr = (hart.core.reg_file[instr.rs1 as usize] + instr.imm) as u32;
            match instr.funct3 {
                // lb sign-extended
                0x0 => {
                    match virt_memory::virt_read_byte(addr, hart, bus) {
                        Ok(x) => hart.core.reg_file[instr.rd as usize] = i32::from(x as i8),
                        Err(e) => return Err(e),
                    };
                }
                // lh
                0x1 => {
                    match virt_memory::virt_read_hword(addr, hart, bus) {
                        Ok(x) => hart.core.reg_file[instr.rd as usize] = i32::from(x as i16),
                        Err(e) => return Err(e),
                    };
                }
                // lw
                0x2 => {
                    match virt_memory::virt_read_word(addr, hart, bus) {
                        Ok(x) => hart.core.reg_file[instr.rd as usize] = x as i32,
                        Err(e) => return Err(e),
                    };
                }
                // lbu zero-extended
                0x4 => {
                    match virt_memory::virt_read_byte(addr, hart, bus) {
                        Ok(x) => hart.core.reg_file[instr.rd as usize] = x as i32,
                        Err(e) => return Err(e),
                    };
                }
                // lhu
                0x5 => {
                    match virt_memory::virt_read_hword(addr, hart, bus) {
                        Ok(x) => hart.core.reg_file[instr.rd as usize] = x as i32,
                        Err(e) => return Err(e),
                    };
                }
                _ => return Err(Exception::Illegal_instruction),
            };
            if hart.core.p_start {
                hart.core.instr_str = format!(
                    "{} x{} 0x{:08x} mem 0x{:08x}",
                    hart.core.instr_str, instr.rd, hart.core.reg_file[instr.rd as usize], addr
                );
            }
            hart.core.pc += 4;
        }
        //jalr
        0b1100111 => {
            let tmp_pc = hart.core.pc;
            hart.core.pc =
                (i64::from(hart.core.reg_file[instr.rs1 as usize] as u32) + i64::from(instr.imm)) as u32;
            hart.core.reg_file[instr.rd as usize] = (tmp_pc + 4) as i32;
            if hart.core.p_start && instr.rd != 0 {
                hart.core.instr_str = format!(
                    "{} x{} 0x{:08x}",
                    hart.core.instr_str, instr.rd, hart.core.reg_file[instr.rd as usize]
                );
            }
        }
        0b1110011 => {
            let csr_addr = (instr.imm & 0xfff) as u32;
            let source = hart.core.reg_file[instr.rs1 as usize] as u32;
            match instr.funct3 {
                // csrrw
                0b001 => {
                    let csr;
                    if instr.rd != 0 {
                        csr = csr::read_addr(csr_addr, &hart.core)?;
                        hart.core.reg_file[instr.rd as usize] = csr as i32;
                    }
                    csr::write_addr(csr_addr, source, &mut hart.core)?;
                    if hart.core.p_start && instr.rd != 0 {
                        hart.core.instr_str = format!(
                            "{} x{} 0x{:08x}",
                            hart.core.instr_str, instr.rd, hart.core.reg_file[instr.rd as usize]
                        )
                    }
                    if hart.core.p_start {
                        if csr_addr == 0x100 {
                            hart.core.instr_str = format!(
                                "{} c768_mstatus 0x{:08x}",
                                hart.core.instr_str,
                                csr::read(csr::Csr::mstatus, &hart.core)
                            )
                        } else {
                            hart.core.instr_str = format!(
                                "{} c{}_{} 0x{:08x}",
                                hart.core.instr_str,
                                csr_addr,
                                csr_name(csr_addr),
                                source
                            )
                        }
                    }
                    hart.core.pc += 4;
                }
                // csrrs
                0b010 => {
                    let csr;
                    match csr::read_addr(csr_addr, &hart.core) {
                        Ok(x) => csr = x,
                        Err(e) => {
                            return Err(e);
                        }
                    };
                    hart.core.reg_file[instr.rd as usize] = csr as i32;
                    if hart.core.p_start && instr.rd != 0 {
                        hart.core.instr_str = format!(
                            "{} x{} 0x{:08x}",
                            hart.core.instr_str, instr.rd, hart.core.reg_file[instr.rd as usize]
                        )
                    }
                    if instr.rs1 != 0 {
                        csr::write_addr(csr_addr, csr | source, &mut hart.core)?;
                        if hart.core.p_start {
                            if csr_addr == 0x100 {
                                hart.core.instr_str = format!(
                                    "{} c768_mstatus 0x{:08x}",
                                    hart.core.instr_str,
                                    csr::read(csr::Csr::mstatus, &hart.core)
                                )
                            } else {
                                hart.core.instr_str = format!(
                                    "{} c{}_{} 0x{:08x}",
                                    hart.core.instr_str,
                                    csr_addr,
                                    csr_name(csr_addr),
                                    csr | source
                                )
                            }
                        }
                    }
                    hart.core.pc += 4;
                }
                // csrrc
                0b011 => {
                    let csr;
                    match csr::read_addr(csr_addr, &hart.core) {
                        Ok(x) => csr = x,
                        Err(e) => {
                            return Err(e);
                        }
                    };
                    hart.core.reg_file[instr.rd as usize] = csr as i32;
                    if hart.core.p_start && instr.rd != 0 {
                        hart.core.instr_str = format!(
                            "{} x{} 0x{:08x}",
                            hart.core.instr_str, instr.rd, hart.core.reg_file[instr.rd as usize]
                        )
                    }
                    if instr.rs1 != 0 {
                        csr::write_addr(csr_addr, csr & !source, &mut hart.core)?;
                        if hart.core.p_start {
                            if csr_addr == 0x100 {
                                hart.core.instr_str = format!(
                                    "{} c768_mstatus 0x{:08x}",
                                    hart.core.instr_str,
                                    csr::read(csr::Csr::mstatus, &hart.core)
                                )
                            } else {
                                hart.core.instr_str = format!(
                                    "{} c{}_{} 0x{:08x}",
                                    hart.core.instr_str,
                                    csr_addr,
                                    csr_name(csr_addr),
                                    csr & !source
                                )
                            }
                        }
                    }
                    hart.core.pc += 4;
                }
                // csrrwi
                0b101 => {
                    let mut csr = 0;
                    if instr.rd != 0 {
                        csr = csr::read_addr(csr_addr, &hart.core)?;
                    }
                    hart.core.reg_file[instr.rd as usize] = csr as i32;
                    if hart.core.p_start && instr.rd != 0 {
                        hart.core.instr_str = format!(
                            "{} x{} 0x{:08x}",
                            hart.core.instr_str, instr.rd, hart.core.reg_file[instr.rd as usize]
                        )
                    }
                    csr::write_addr(csr_addr, instr.rs1, &mut hart.core)?;
                    if hart.core.p_start {
                        if csr_addr == 0x100 {
                            hart.core.instr_str = format!(
                                "{} c768_mstatus 0x{:08x}",
                                hart.core.instr_str,
                                csr::read(csr::Csr::mstatus, &hart.core)
                            )
                        } else {
                            hart.core.instr_str = format!(
                                "{} c{}_{} 0x{:08x}",
                                hart.core.instr_str,
                                csr_addr,
                                csr_name(csr_addr),
                                instr.rs1
                            )
                        }
                    }
                    hart.core.pc += 4;
                }
                // csrrsi
                0b110 => {
                    let csr;
                    match csr::read_addr(csr_addr, &hart.core) {
                        Ok(x) => csr = x,
                        Err(e) => {
                            return Err(e);
                        }
                    };
                    hart.core.reg_file[instr.rd as usize] = csr as i32;
                    if hart.core.p_start && instr.rd != 0 {
                        hart.core.instr_str = format!(
                            "{} x{} 0x{:08x}",
                            hart.core.instr_str, instr.rd, hart.core.reg_file[instr.rd as usize]
                        )
                    }
                    if instr.rs1 != 0 {
                        csr::write_addr(csr_addr, csr | instr.rs1, &mut hart.core)?;
                        if hart.core.p_start {
                            if csr_addr == 0x100 {
                                hart.core.instr_str = format!(
                                    "{} c768_mstatus 0x{:08x}",
                                    hart.core.instr_str,
                                    csr::read(csr::Csr::mstatus, &hart.core)
                                )
                            } else {
                                hart.core.instr_str = format!(
                                    "{} c{}_{} 0x{:08x}",
                                    hart.core.instr_str,
                                    csr_addr,
                                    csr_name(csr_addr),
                                    csr | instr.rs1
                                )
                            }
                        }
                    }
                    hart.core.pc += 4;
                }
                // csrrci
                0b111 => {
                    let csr;
                    match csr::read_addr(csr_addr, &hart.core) {
                        Ok(x) => csr = x,
                        Err(e) => {
                            return Err(e);
                        }
                    };
                    hart.core.reg_file[instr.rd as usize] = csr as i32;
                    if hart.core.p_start && instr.rd != 0 {
                        hart.core.instr_str = format!(
                            "{} x{} 0x{:08x}",
                            hart.core.instr_str, instr.rd, hart.core.reg_file[instr.rd as usize]
                        )
                    }
                    if instr.rs1 != 0 {
                        csr::write_addr(csr_addr, csr & !instr.rs1, &mut hart.core)?;
                        if hart.core.p_start {
                            if csr_addr == 0x100 {
                                hart.core.instr_str = format!(
                                    "{} c768_mstatus 0x{:08x}",
                                    hart.core.instr_str,
                                    csr::read(csr::Csr::mstatus, &hart.core)
                                )
                            } else {
                                hart.core.instr_str = format!(
                                    "{} c{}_{} 0x{:08x}",
                                    hart.core.instr_str,
                                    csr_addr,
                                    csr_name(csr_addr),
                                    csr & !instr.rs1
                                )
                            }
                        }
                    }
                    hart.core.pc += 4;
                }
                0b0 => {
                    //sfence
                    if instr.funct7 == 0b0001001 {
                        hart.core.pc += 4;
                        return Ok(State::Ok);
                    }
                    match instr.imm {
                        //ecall
                        0b0 => {
                            if hart.core.mode == 3 {
                                return Err(Exception::Environment_call_from_Mmode);
                            } else if hart.core.mode == 1 {
                                return Err(Exception::Environment_call_from_Smode);
                            } else if hart.core.mode == 0 {
                                return Err(Exception::Environment_call_from_Umode);
                            }
                            // hart.core.pc += 4;
                        }
                        //ebreak
                        0b1 => {
                            return Err(Exception::Breakpoint);
                            // hart.core.pc += 4;
                        }
                        // mret
                        0b001100000010 => {
                            // println!("mret");
                            let mut mstatus = csr::read(super::Csr::mstatus, &hart.core);
                            // restore last mode and set mpp = 0
                            hart.core.mode = (mstatus >> 11) & 0b11;
                            if hart.core.mode < 3 {
                                mstatus &= !(1 << 17);
                            }
                            mstatus &= !(0b11 << 11);
                            // restore mie and set mpie to 1
                            mstatus &= !0b1000;
                            mstatus |= (mstatus & 0b10000000) >> 4;
                            mstatus |= 0b10000000;
                            csr::write(super::Csr::mstatus, mstatus, &mut hart.core);
                            // restore pc
                            hart.core.pc = csr::read(super::Csr::mepc, &hart.core);
                            if hart.core.p_start {
                                hart.core.instr_str = format!(
                                    "{} c768_mstatus 0x{:08x} c784_mstatush 0x{:08x}",
                                    hart.core.instr_str,
                                    csr::read(csr::Csr::mstatus, &hart.core),
                                    csr::read(csr::Csr::mstatush, &hart.core)
                                )
                            }
                        }
                        // sret
                        0b000100000010 => {
                            // format!("sret");
                            let mut mstatus = csr::read(super::Csr::mstatus, &mut hart.core);
                            // restore last mode and set spp = 0
                            hart.core.mode = (mstatus >> 8) & 0b1;
                            if hart.core.mode < 3 {
                                mstatus &= !(1 << 17);
                            }
                            mstatus &= !(0b1 << 8);
                            // restore sie and set spie to 1
                            mstatus &= !0b10;
                            mstatus |= (mstatus & 0b100000) >> 4;
                            mstatus |= 0b100000;
                            csr::write(super::Csr::mstatus, mstatus, &mut hart.core);
                            // restore pc
                            hart.core.pc = csr::read(super::Csr::sepc, &hart.core);
                            if hart.core.p_start {
                                hart.core.instr_str = format!(
                                    "{} c768_mstatus 0x{:08x}",
                                    hart.core.instr_str,
                                    csr::read(csr::Csr::mstatus, &hart.core),
                                )
                            }
                        }
                        // wfi
                        0b000100000101 => {
                            // *hart.core.csr(super::Csr::Mstatus) |= 1 << 3;
                            hart.core.wfi = true;
                            hart.core.pc += 4;
                            return Ok(State::Sleep);
                        }
                        _ => return Err(Exception::Illegal_instruction),
                    }
                }
                _ => return Err(Exception::Illegal_instruction),
            };
        }
        // fence, pause
        0b0001111 => {
            hart.core.pc += 4;
        }

        _ => return Err(Exception::Illegal_instruction),
    };
    Ok(State::Ok)
}

pub fn exec_s(hart: &mut Hart, bus: &mut MemoryBus, instr: &SType) -> Result<State, Exception> {
    let addr = (hart.core.reg_file[instr.rs1 as usize] + instr.imm) as u32;
    let rs2 = hart.core.reg_file[instr.rs2 as usize];

    if hart.core.p_start {
        hart.core.instr_str = match instr.funct3 {
            //sb
            0x0 => format!("{} mem 0x{:08x} 0x{:02x}", hart.core.instr_str, addr, rs2 as u8),
            //sh
            0x1 => format!("{} mem 0x{:08x} 0x{:04x}", hart.core.instr_str, addr, rs2 as u16),
            //sw
            0x2 => format!("{} mem 0x{:08x} 0x{:08x}", hart.core.instr_str, addr, rs2),
            _ => format!("{}", hart.core.instr_str),
        };
    }

    match instr.funct3 {
        //sb
        0x0 => virt_memory::virt_write_byte(addr, rs2 as u8, hart, bus)?,
        //sh
        0x1 => virt_memory::virt_write_hword(addr, rs2 as u16, hart, bus)?,
        //sw
        0x2 => virt_memory::virt_write_word(addr, rs2 as u32, hart, bus)?,
        _ => return Err(Exception::Illegal_instruction),
    };

    hart.core.pc += 4;
    Ok(State::Ok)
}

pub fn exec_b(core: &mut Core, instr: &BType) -> Result<State, Exception> {
    let rs1 = core.reg_file[instr.rs1 as usize];
    let rs2 = core.reg_file[instr.rs2 as usize];
    let last_pc = core.pc;
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
        _ => return Err(Exception::Illegal_instruction),
    };
    if core.pc == last_pc {
        core.pc += 4;
    }
    Ok(State::Ok)
}

pub fn exec_u(core: &mut Core, instr: &UType) -> Result<State, Exception> {
    match instr.opcode {
        //lui
        0b0110111 => {
            core.reg_file[instr.rd as usize] = (instr.imm << 12) as i32;
        }
        //auipc
        0b0010111 => {
            core.reg_file[instr.rd as usize] =
                (i64::from(core.pc) + i64::from(instr.imm << 12)) as i32;
        }
        _ => return Err(Exception::Illegal_instruction),
    };
    if core.p_start {
        core.instr_str = format!(
            "{} x{} 0x{:08x}",
            core.instr_str, instr.rd, core.reg_file[instr.rd as usize]
        );
    }
    core.pc += 4;
    Ok(State::Ok)
}

pub fn exec_j(core: &mut Core, instr: &JType) -> Result<State, Exception> {
    match instr.opcode {
        //jal
        0b1101111 => {
            core.reg_file[instr.rd as usize] = (core.pc + 4) as i32;
            core.pc = (core.pc as i32 + instr.imm) as u32;
        }
        _ => return Err(Exception::Illegal_instruction),
    };
    if core.p_start {
        if instr.rd != 0 {
            core.instr_str = format!(
                "{} x{} 0x{:08x}",
                core.instr_str, instr.rd, core.reg_file[instr.rd as usize]
            );
        }
    }
    Ok(State::Ok)
}
