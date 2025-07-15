mod core;
mod memory;
use std::env;
use std::error::Error;
use std::process;

use termion::raw::IntoRawMode;

const RAM_SIZE: usize = 64 * 1024 * 1024;
const RAM_OFFSET: u32 = 0x80000000;
const DEBUG: bool =  false;
const PRINT_START: u64 = 0;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Wrong arguments!");
        process::exit(1);
    }

    let mut stdout = std::io::stdout().into_raw_mode().unwrap(); // Optional: raw mode
    let mut memory = memory::Memory::default();
    let mut proc = core::Core::new(&mut memory);
    proc.read_data(
        &args[1], //kernel Image
        "/home/msjtw/Documents/digital_design/sixtyfourmb.dtb",
    )?;

    let mut last_cycle: u64 = 0;
    loop {
        // let curr_cycle =
        //     ((*proc.csr(core::Csr::Cycleh) as u64) << 32) + (*proc.csr(core::Csr::Cycle) as u64);
        // let diff_cycle = curr_cycle - last_cycle;
        // last_cycle = curr_cycle;

        match proc.exec(1) {
            Ok(x) => match x {
                core::State::Ok => {}
                core::State::Sleep => {
                    // println!("sleeeeeeeeep");
                    let add_time = (proc.memory.csr_read(memory::Time::Mtimecmp) as i64
                        - proc.memory.csr_read(memory::Time::Mtime) as i64)
                        .max(0) as u32;
                    proc.memory.csr_write(
                        memory::Time::Mtime,
                        proc.memory.csr_read(memory::Time::Mtimecmp),
                    );
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

        if DEBUG && last_cycle > 1000000 {
            break;
        }
    }

    Ok(())
}
