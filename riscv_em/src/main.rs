mod core;
mod device;
mod memory;
use clap::Parser;
use core::Core;
use std::env;
use std::error::Error;
use std::process;

use std::time::SystemTime;

use crate::memory::*;

const RAM_SIZE: u32 = 64 * 1024 * 1024;
const RAM_OFFSET: u32 = 0x80000000;
const DEBUG: bool = false;
const SPIKE_DEBUG: bool = true;
const PRINT_START: u64 = 0 as u64;
const REAL_TIME: bool = false;

/// RISCV (rv32ima) emulator
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// bios loaded at 0x8000000
    #[arg(short, long)]
    bios: Option<String>,

    /// kernel loaded at 0x80200000
    #[arg(short, long)]
    kernel: Option<String>,

    #[arg(short, long)]
    drive: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut hart = core::Hart {
        core: Core::default(),
        clint: clint::Clint::default(),
    };

    let mut vblk = virtio_blk::VirtioBlk::default();

    if args.bios.is_none() {
        println!("Wrong arguments!");
        process::exit(1);
    }
    if args.drive.is_some() {
        vblk.init(&args.drive.unwrap());
    }

    let mut bus = memory::MemoryBus {
        ram: ram::RAM::default(),
        uart: ns16550::Uart::default(),
        blk: virtio::VirtioDevice::new(Box::new(vblk)),
        plic: plic::Plic::default(),
    };

    core::soc_init(
        &mut hart,
        &mut bus,
        &args.bios.unwrap(), //kernel Image
        "/home/msjtw/Documents/digital_design/riscv_em/device_tree/spike.dtb",
    )?;

    let mut last_time = SystemTime::now();

    loop {
        match core::hart_run(&mut hart, &mut bus, 5000) {
            core::State::Ok => {}
            core::State::Sleep => {
                // println!("Sleep... 0x{:08x} < 0x{:08x}; {}", proc.mtime, proc.mtimecmp, i128::from(proc.mtimecmp) - i128::from(proc.mtime));
                // println!("mie: 0b{:b}", proc.csr_file[0x304]);
                // ctr = ctr.max(proc.mtimecmp);
                // soc.core.mtime = soc.core.mtime.max(soc.core.mtimecmp);
                // proc.sleep = true;
                // let add_time = (proc.memory.csr_read(memory::Time::Mtimecmp) as i64
                //     - proc.memory.csr_read(memory::Time::Mtime) as i64)
                //     .max(0) as u32;
                // proc.memory.csr_write(
                //     memory::Time::Mtime,
                //     proc.memory.csr_read(memory::Time::Mtimecmp),
                // );
            } // core::State::Reboot => {
              //     println!("Shutting down...");
              //     break;
              // }
              // core::State::Shutdown => {
              //     println!("Shutting down...");
              //     break;
              // }
        }

        if REAL_TIME {
            let time_diff = SystemTime::now()
                .duration_since(last_time)
                .unwrap()
                .as_millis() as u32;
            last_time = SystemTime::now();
            hart.clint.mtime += time_diff;
        } else {
            hart.clint.mtime += 50;
        }

        hart.core.lr_address = 0x0;
        if hart.core.p_start {
            eprintln!("mtime change 0x{:x}", hart.clint.mtime);
        }
    }
}
