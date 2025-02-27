use std::env;
use std::error::Error;
use std::process;
mod elf_parse;
mod instr_parse;
use crate::instr_parse::Instruction;

mod exec_unit {
    use std::collections::HashMap;

    use crate::instr_parse::{BType, IType, Instruction, JType, RType, SType, UType};

    struct Processor {
        pc: u32,
        reg_file: [u32; 32],
        memory: HashMap<u32, u32>,
    }

    impl Processor {
        fn exec(&mut self, instr: &Instruction) {
            match instr {
                Instruction::R(x) => self.exec_r(x),
                Instruction::I(x) => self.exec_i(x),
                Instruction::S(x) => self.exec_s(x),
                Instruction::B(x) => self.exec_b(x),
                Instruction::U(x) => self.exec_u(x),
                Instruction::J(x) => self.exec_j(x),
            }
        }

        fn exec_r(&mut self, instr: &RType) {
            ()
        }

        fn exec_i(&mut self, instr: &IType) {
            ()
        }

        fn exec_s(&mut self, instr: &SType) {
            ()
        }

        fn exec_b(&mut self, instr: &BType) {
            ()
        }

        fn exec_u(&mut self, instr: &UType) {
            ()
        }

        fn exec_j(&mut self, instr: &JType) {
            ()
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Wrong arguments!");
        process::exit(1);
    }

    let content = elf_parse::read_efl(&args[1])?;
    println!("address {:x}", content.base_address);
    println!("entry {:x}", content.entry_adress);
    for a in content.intructions {
        println!("{:8x}    {:?}", a, Instruction::from(a));
    }
    Ok(())
}
