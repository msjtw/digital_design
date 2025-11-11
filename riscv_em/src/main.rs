mod core;
mod device;
mod memory;
use core::Core;
use std::env;
use std::error::Error;
use std::process;

use memory::Memory;
use std::time::SystemTime;

const RAM_SIZE: u32 = 64 * 1024 * 1024;
const RAM_OFFSET: u32 = 0x80000000;
const DEBUG: bool = false;
const SPIKE_DEBUG: bool = true;
const PRINT_START: u64 = 1e10 as u64;
const REAL_TIME: bool = false;

struct SoC<'a> {
    core: &'a mut core::Core,
    memory: &'a mut memory::Memory,
    uart: &'a mut device::ns16550::Uart,
    plic: &'a mut device::plic::Plic,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Wrong arguments!");
        process::exit(1);
    }

    // let mut _stdout = std::io::stdout().into_raw_mode().unwrap(); // Optional: raw mode

    let mut core = Core::default();
    let mut memory = Memory::default();
    let mut uart = device::ns16550::Uart::default();
    let mut plic = device::plic::Plic::default();

    let mut soc = SoC {
        core: &mut core,
        memory: &mut memory,
        uart: &mut uart,
        plic: &mut plic,
    };
    // let mut memory = memory::Memory::default();
    // let mut proc = core::Core::new(&mut memory);
    core::read_data(
        &mut soc,
        &args[1], //kernel Image
        "/home/msjtw/Documents/digital_design/riscv_em/device_tree/spike.dtb",
    )?;

    let mut last_time = SystemTime::now();

    loop {
        match core::run(&mut soc, 5000) {
            core::State::Ok => {}
            core::State::Sleep => {
                // println!("Sleep... 0x{:08x} < 0x{:08x}; {}", proc.mtime, proc.mtimecmp, i128::from(proc.mtimecmp) - i128::from(proc.mtime));
                // println!("mie: 0b{:b}", proc.csr_file[0x304]);
                // ctr = ctr.max(proc.mtimecmp);
                soc.core.mtime = soc.core.mtime.max(soc.core.mtimecmp);
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
        }

        if REAL_TIME {
            let time_diff = SystemTime::now()
                .duration_since(last_time)
                .unwrap()
                .as_millis() as u64;
            last_time = SystemTime::now();
            soc.core.mtime += time_diff;
        } else {
            soc.core.mtime += 50;
        }

        soc.core.lr_address = 0x0;
        if soc.core.p_start {
            eprintln!("mtime change 0x{:x}", soc.core.mtime);
        }
    }

    Ok(())
}
