use std::env;
use std::error::Error;
use std::process;
mod elf_parse;
mod exec_unit;
mod instr_parse;
use crate::instr_parse::Instruction;

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
        println!("{:8x}    {:?}", a, Instruction::from(a)?);
    }
    Ok(())
}
