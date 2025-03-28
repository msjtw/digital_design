mod core;
mod memory;
use std::env;
use std::error::Error;
use std::io;
use std::process;

const RAM_SIZE: usize = 64 * (1 << 10);
const RAM_OFFSET: u32 = 0x80000000;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Wrong arguments!");
        process::exit(1);
    }

    let file = &args[1];

    let mut memory = memory::Memory::default();
    let mut proc = core::Core::new(&mut memory);
    proc.read_data(&file, "devicetree.dtb")?;

    let mut last_cycle: u64 = 0;

    loop {
        let curr_cycle =
            (*proc.csr(core::Csr::Cycleh) as u64) << 32 + (*proc.csr(core::Csr::Cycle) as u64);
        let diff_cycle = curr_cycle - last_cycle;
        last_cycle = curr_cycle;

        // let mut guess = String::new();
        match proc.exec(diff_cycle) {
            Ok(x) => match x {
                core::State::Ok => {}
                core::State::Sleep => {}
                core::State::Reboot => {
                    println!("Shutting down...");
                    break;
                }
                core::State::Shutdown => {
                    println!("Shutting down...");
                    break;
                }
            },
            Err(x) => {
                println!("{:?}", x);
                break;
            }
        }

        // io::stdin()
        //     .read_line(&mut guess)
        //     .expect("Failed to read line");
    }

    Ok(())
}
