use instr::Instruction;
use object::read::elf::FileHeader;
use object::{Endian, Object, ObjectSection, elf};
use std::env;
use std::error::Error;
use std::fs;
use std::process;
mod instr;

struct ElfData {
    intructions: Vec<u32>,
    base_address: u64,
    entry_adress: u64,
}

fn read_efl(path: &str) -> Result<ElfData, Box<dyn Error>> {
    let file = fs::read(path)?;
    let elf_data = elf::FileHeader32::<object::Endianness>::parse(&*file)?;
    let file_data = object::File::parse(&*file)?;
    let text_data = file_data.section_by_name(".text").unwrap();

    let endianness = elf_data.endian()?;
    let mut assembly = Vec::new();
    let mut read = 0;
    let mut total = 0;
    for i in 0..text_data.size() {
        let byte = text_data.data()?[i as usize] as u32;
        if endianness.is_little_endian() {
            total += byte << (8 * read);
        } else {
            total <<= 8;
            total += byte;
        }
        read += 1;
        if read == 4 {
            assembly.push(total);
            read = 0;
            total = 0;
        }
    }

    Ok(ElfData {
        intructions: assembly,
        base_address: text_data.address(),
        entry_adress: file_data.entry(),
    })
}

/// Reads a file and displays the name of each symbol.
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Wrong arguments!");
        process::exit(1);
    }

    let content = read_efl(&args[1])?;
    println!("address {:x}", content.base_address);
    println!("entry {:x}", content.entry_adress);
    for a in content.intructions {
        print!("{:8x}    ", a);
        match instr::Instruction::from(a) {
            instr::Instruction::R(x) => println!("register"),
            instr::Instruction::I(x) => println!("immediate"),
            instr::Instruction::U(x) => println!("upper immediate"),
            instr::Instruction::S(x) => println!("store"),
            instr::Instruction::B(x) => println!("branch"),
            instr::Instruction::J(x) => println!("jump"),
            _ => println!("nie ma takiej"),
        };
    }
    Ok(())
}
