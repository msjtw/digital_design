use std::env;
use std::error::Error;
use std::process;
mod exec_unit;
mod instr_parse;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Wrong arguments!");
        process::exit(1);
    }

    let data = fs::read(&args[1])?;
    let file = object::File::parse(&*data)?;

    let mut proc = exec_unit::Processor::read_data(&file)?;
    while let Ok(()) = proc.exec() {}
    Ok(())
}
