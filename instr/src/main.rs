mod instr_parse;
use std::io;

fn main() -> io::Result<()> {
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff)?;
        let bytes: u32 = buff.trim().parse().unwrap();
        let a = instr_parse::Instruction::from(bytes);
        println!("{:?}", a);
    }
}
