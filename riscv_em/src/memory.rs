pub mod clint;
pub mod ns16550;
pub mod plic;
pub mod ram;
pub mod virtio;
mod virtio_blk;

use crate::{
    core::exceptions,
    memory::{ns16550::Uart, plic::Plic, ram::RAM, virtio::VirtioDevice},
};

#[derive(Debug, Clone, Copy)]
pub struct MemoryPermissions {
    pub r: bool,
    pub w: bool,
    pub x: bool,
}

pub struct MemoryBus {
    pub ram: RAM,
    pub uart: Uart,
    pub blk: VirtioDevice,
    pub plic: Plic,
}

pub fn load_word(bus: &mut MemoryBus, addr: u32) -> Result<u32, exceptions::Exception> {
    if bus.ram.claim(addr) {
        return Ok(bus.ram.load_word(addr));
    } else if bus.plic.claim(addr) {
        return Ok(bus.plic.read(addr));
    } else if bus.blk.claim(addr) {
        return Ok(bus.blk.read(addr));
    }
    // NOTE: maybe some error ???
    return Ok(0);
}

pub fn load_hword(bus: &mut MemoryBus, addr: u32) -> Result<u16, exceptions::Exception> {
    if bus.ram.claim(addr) {
        return Ok(bus.ram.load_hword(addr));
    }
    // NOTE: maybe some error ???
    return Ok(0);
}
pub fn load_byte(bus: &mut MemoryBus, addr: u32) -> Result<u8, exceptions::Exception> {
    if bus.ram.claim(addr) {
        return Ok(bus.ram.load_byte(addr));
    } else if bus.uart.claim(addr) {
        return Ok(bus.uart.read(addr));
    }
    // NOTE: maybe some error ???
    return Ok(0);
}

pub fn store_word(bus: &mut MemoryBus, addr: u32, data: u32) -> Result<(), exceptions::Exception> {
    if bus.ram.claim(addr) {
        bus.ram.store_word(addr, data);
    } else if bus.plic.claim(addr) {
        bus.plic.write(addr, data);
    } else if bus.blk.claim(addr) {
        bus.blk.write(addr, data);
    }
    Ok(())
}
pub fn store_hword(bus: &mut MemoryBus, addr: u32, data: u16) -> Result<(), exceptions::Exception> {
    if bus.ram.claim(addr) {
        bus.ram.store_hword(addr, data);
    }
    Ok(())
}
pub fn store_byte(bus: &mut MemoryBus, addr: u32, data: u8) -> Result<(), exceptions::Exception> {
    if bus.ram.claim(addr) {
        bus.ram.store_byte(addr, data);
    } else if bus.uart.claim(addr) {
        bus.uart.write(addr, data);
    }
    Ok(())
}
