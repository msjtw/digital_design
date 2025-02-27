use std::env;
use std::error::Error;
use std::process;
mod elf_parse;
mod instr;

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
        println!("{:8x}    {:?}", a, instr::Instruction::from(a));
        // match instr::Instruction::from(a) {
        //     instr::Instruction::R(x) => println!("register"),
        //     instr::Instruction::I(x) => println!("immediate"),
        //     instr::Instruction::U(x) => println!("upper immediate"),
        //     instr::Instruction::S(x) => println!("store"),
        //     instr::Instruction::B(x) => println!("branch"),
        //     instr::Instruction::J(x) => println!("jump"),
        //     _ => println!("nie ma takiej"),
        // };
    }
    Ok(())
}
