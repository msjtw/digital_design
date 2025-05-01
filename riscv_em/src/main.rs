mod core;
mod memory;
use std::env;
use std::error::Error;
use std::process;

const RAM_SIZE: usize = 64 * 1024 * 1024;
const RAM_OFFSET: u32 = 0x80000000;
const DEBUG: bool = false;
const PRINT_START: u64 = 32082544;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Wrong arguments!");
        process::exit(1);
    }

    let file = &args[1];

    let mut memory = memory::Memory::default();
    let mut proc = core::Core::new(&mut memory);
    proc.read_data(
        &file,
        "/home/msjtw/Documents/digital_design/sixtyfourmb.dtb",
    )?;

    let mut last_cycle: u64 = 0;
    let mut watch;
    loop {
        let curr_cycle =
            ((*proc.csr(core::Csr::Cycleh) as u64) << 32) + (*proc.csr(core::Csr::Cycle) as u64);
        let diff_cycle = curr_cycle - last_cycle;
        last_cycle = curr_cycle;

        watch = proc.memory.get_word(0x83f83c9c);

        match proc.exec(diff_cycle) {
            Ok(x) => match x {
                core::State::Ok => {}
                core::State::Sleep => {
                    // println!("sleeeeeeeeep");
                }
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
        // if DEBUG && watch != proc.memory.get_word(0x83f83c9c) {
        //     // println!("{}", proc.print_reg_file());
        //     // println!("{} {}", last_cycle, proc.pc);
        //     println!("{}", proc.memory.get_word(0x83f83c9c).unwrap());
        // }
        if DEBUG && last_cycle > 44813202 {
            break;
        }
    }

    Ok(())
}
