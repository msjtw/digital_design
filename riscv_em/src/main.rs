mod core;
mod memory;
use std::env;
use std::error::Error;
use std::process;

use std::thread::sleep;
use std::time::SystemTime;
use termion::raw::IntoRawMode;


const RAM_SIZE: u32 = 64 * 1024 * 1024;
const RAM_OFFSET: u32 = 0x80000000;
const DEBUG: bool = false;
const SPIKE_DEBUG: bool = false;
const PRINT_START: u64 = 1e10 as u64;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Wrong arguments!");
        process::exit(1);
    }

    // let mut _stdout = std::io::stdout().into_raw_mode().unwrap(); // Optional: raw mode
    let mut memory = memory::Memory::default();
    let mut proc = core::Core::new(&mut memory);
    proc.read_data(
        &args[1], //kernel Image
        "/home/msjtw/Documents/digital_design/riscv_em/device_tree/spike.dtb",
    )?;

    let mut last_time = SystemTime::now();

    let mut ctr = 0;
    loop {
        // let curr_cycle =
        //     ((*proc.csr(core::Csr::Cycleh) as u64) << 32) + (*proc.csr(core::Csr::Cycle) as u64);
        // let diff_cycle = curr_cycle - last_cycle;
        // last_cycle = curr_cycle;

        ctr += 1;
        let time_diff = SystemTime::now().duration_since(last_time).unwrap().as_millis();
        last_time = SystemTime::now();
        proc.mtime += time_diff as u64;

        match proc.exec() {
            Ok(x) => match x {
                core::State::Ok => {}
                core::State::Sleep => {
                    println!("Sleep... 0x{:08x} < 0x{:08x}; {}", proc.mtime, proc.mtimecmp, i128::from(proc.mtimecmp) - i128::from(proc.mtime));
                    println!("mie: 0b{:b}", proc.csr_file[0x304]);
                    proc.mtime = proc.mtime.max(proc.mtimecmp);
                    // proc.sleep = true;
                    // let add_time = (proc.memory.csr_read(memory::Time::Mtimecmp) as i64
                    //     - proc.memory.csr_read(memory::Time::Mtime) as i64)
                    //     .max(0) as u32;
                    // proc.memory.csr_write(
                    //     memory::Time::Mtime,
                    //     proc.memory.csr_read(memory::Time::Mtimecmp),
                    // );
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
                println!("err: {:?}", x);
                break;
            }
        }

        // if DEBUG && ctr > 1e6 as u64 {
        //     break;
        // }
    }

    Ok(())
}
