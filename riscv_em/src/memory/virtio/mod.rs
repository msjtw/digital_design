#![allow(non_upper_case_globals, non_snake_case)]
pub mod registers;
use registers::*;

use crate::memory::{virtio_blk::VirtioBlk, *};

#[derive(Debug, Default)]
pub struct Descriptor {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

impl Descriptor {
    pub fn read(addr: u32, ram: &mut RAM) -> Self {
        Self {
            addr: ram.load_word(addr) as u64,
            len: ram.load_word(addr + 8),
            flags: ram.load_hword(addr + 12),
            next: ram.load_hword(addr + 14),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct VirtioQueue {
    queue_size_max: u16,
    pub queue_size: u16,
    queue_ready: u32,
    pub queue_desc_low: u32,
    // queue_des_high: u32,
    pub queue_driver_low: u32,
    // queue_driver_high: u32,
    pub queue_device_low: u32,
    // shared memory registers ...
    queue_reset: u32,
    pub last_avail: u16,
}

pub struct VirtioMmio<const QCOUNT: usize> {
    pub base: u32,
    pub length: u32,
    pub interrupt_id: usize,

    pub device_id: u32,

    pub device_features: [u32; 2],
    pub device_features_sel: usize,
    pub driver_features: [u32; 2],
    pub driver_features_sel: usize,

    pub queue_sel: usize,
    pub queues: [VirtioQueue; QCOUNT],
    pub queue_notify: u32,
    pub queue_notify_pending: bool,

    pub interrupt_status: u32,
    pub interrupt_ack: u32,
    pub status: u32,
    pub config_generation: u32,
}

pub struct VirtioDevice {
    pub mmio: VirtioMmio<1>,
    pub device: VirtioBlk,
}

impl Default for VirtioDevice {
    fn default() -> Self {
        VirtioDevice {
            mmio: VirtioMmio::<1> {
                base: 0x4200000,
                length: 0x200,
                interrupt_id: 3,
                device_id: 2,
                device_features: [0, 1],
                device_features_sel: 0,
                driver_features: [0; 2],
                driver_features_sel: 0,
                queue_sel: 0,
                queues: [VirtioQueue::default(); 1],
                queue_notify: 0,
                queue_notify_pending: false,
                interrupt_status: 0,
                interrupt_ack: 0,
                status: 0,
                config_generation: 0,
            },
            device: VirtioBlk::default(),
        }
    }
}

impl VirtioDevice {
    pub fn claim(&self, addr: u32) -> bool {
        if addr >= self.mmio.base && addr < self.mmio.base + self.mmio.length {
            return true;
        }
        return false;
    }

    pub fn tick(&mut self, plic: &mut Plic, ram: &mut RAM) {
        if self.mmio.interrupt_status > 0 {
            plic.intt_active |= 1 << self.mmio.interrupt_id;
        } else {
            plic.intt_active &= !(1 << self.mmio.interrupt_id);
        }

        if self.mmio.status & STATUS_NEEDS_RESET > 0 {
            return;
        }

        if self.mmio.queue_notify_pending {
            self.mmio.queue_notify_pending = false;
            match self.handle_notify(ram) {
                Ok(_) => {}
                Err(_) => {
                    self.set_fail();
                }
            }
        }
    }

    pub fn write(&mut self, addr: u32, data: u32) {
        let addr = addr - self.mmio.base;
        match addr {
            _DeviceFeaturesSel => {
                if data > 1 {
                    self.set_fail();
                } else {
                    self.mmio.device_features_sel = data as usize;
                }
            }
            _DriverFeatures => {
                self.mmio.driver_features[self.mmio.driver_features_sel] = data;
            }
            _DriverFeaturesSel => {
                if data > 1 {
                    self.set_fail();
                } else {
                    self.mmio.driver_features_sel = data as usize;
                }
            }
            _QueueSel => {
                self.mmio.queue_sel = data as usize;
            }
            _QueueSize => {
                self.mmio.queues[self.mmio.queue_sel].queue_size = data as u16;
            }
            _QueueReady => {
                self.mmio.queues[self.mmio.queue_sel].queue_ready = data;
            }
            _QueueNotify => {
                self.mmio.queue_notify = data;
                self.mmio.queue_notify_pending = true;
            }
            _InterruptACK => {
                // clear interrupt bits
                self.mmio.interrupt_status &= !data;
            }
            _Status => {
                if data == 0 {
                    self.reset();
                } else {
                    self.mmio.status |= data;
                }
            }
            _QueueDescLow => {
                self.mmio.queues[self.mmio.queue_sel].queue_desc_low = data;
            }
            _QueueDriverLow => {
                self.mmio.queues[self.mmio.queue_sel].queue_driver_low = data;
            }
            _QueueDeviceLow => {
                self.mmio.queues[self.mmio.queue_sel].queue_device_low = data;
            }
            _QueueReset => {
                self.mmio.queues[self.mmio.queue_sel].queue_reset = data;
            }
            _ => {
                if addr >= _Config && addr < _Config + self.device.config_size {
                    self.device
                        .config
                        .write_word((addr - _Config) as usize, data);
                } else {
                    // Error
                    self.set_fail();
                }
            }
        }
    }

    pub fn read(&mut self, addr: u32) -> u32 {
        let addr = addr - self.mmio.base;
        return match addr {
            _MagicValue => 0x74726976,
            _Version => 0x2,
            _DeviceID => self.mmio.device_id,
            _VendorID => 0x0,
            _DeviceFeatures => self.mmio.device_features[self.mmio.device_features_sel],
            _QueueSizeMax => self.mmio.queues[self.mmio.queue_sel].queue_size_max as u32,
            _QueueReady => self.mmio.queues[self.mmio.queue_sel].queue_ready,
            _InterruptStatus => self.mmio.interrupt_status,
            _Status => self.mmio.status,
            _ConfigGeneration => self.mmio.config_generation,
            _ => {
                if addr >= _Config && addr < _Config + self.device.config_size {
                    self.device.config.read_word((addr - _Config) as usize)
                } else {
                    // Error
                    self.set_fail();
                    0
                }
            }
        };
    }

    fn set_fail(&mut self) {
        self.mmio.status |= STATUS_NEEDS_RESET;
        if self.mmio.status & STATUS_DRIVER_OK > 0 {
            self.mmio.interrupt_status |= INT_ConfigurationChangeNotification;
        }
    }

    fn reset(&mut self) {
        // TODO:
    }

    fn handle_notify(&mut self, ram: &mut RAM) -> Result<(), ()> {
        // there is index to read in avaliable ring
        let queue = &mut self.mmio.queues[self.mmio.queue_notify as usize];
        let avail_idx = ram.load_hword(queue.queue_driver_low + 2); // one behind index of last written entry
        let mut used_idx = ram.load_hword(queue.queue_device_low + 2);
        while queue.last_avail != avail_idx {
            // while not all descriptor chain heads had been read
            let avail_queue_idx = queue.last_avail % queue.queue_size; // ring index of last unread head
            // index of chain head in avail ring
            let head_idx =
                ram.load_hword(queue.queue_driver_low + 4 + (2 * avail_queue_idx as u32)); // 4 bytes in the available ring are for flags and idx

            // TODO: process chain
            let nbytes;
            match self.device.process_chain(queue, head_idx, ram) {
                Ok(len) => nbytes = len,
                Err(_) => {
                    self.set_fail();
                    return Err(());
                }
            }

            let used_queue_idx = used_idx % queue.queue_size; // ring index of last unread head
            let used_ring_addr = queue.queue_driver_low + 4 + (8 * used_queue_idx as u32);
            ram.store_word(used_ring_addr, head_idx as u32);
            ram.store_word(used_ring_addr + 4, nbytes);

            queue.last_avail += 1;
            used_idx += 1;
        }

        // flags field of used ring needs to be 0
        ram.store_hword(queue.queue_device_low, 0);
        // write new idx to used ring
        ram.store_hword(queue.queue_device_low + 2, used_idx);

        // INTERRUPT
        let used_ring_flags = ram.load_hword(queue.queue_device_low);
        if used_ring_flags != 1 {
            // If flags is 1, the device SHOULD NOT send a notification
            self.mmio.interrupt_status |= INT_UsedBufferNotification;
        }

        Ok(())
    }
}

pub trait VirtioDev {
    fn process_chain(
        &mut self,
        queue: &mut VirtioQueue,
        head_idx: u16,
        ram: &mut RAM,
    ) -> Result<u32, ()>;
}

pub trait VirtioConfig {
    fn read_word(&mut self, addr: usize) -> u32 {
        let base = self as *const _ as *const u8;
        unsafe {
            let d = *base.add(addr) as u32;
            let c = *base.add(1) as u32;
            let b = *base.add(2) as u32;
            let a = *base.add(3) as u32;
            (a << 24) + (b << 16) + (c << 8) + d
        }
    }
    fn write_word(&mut self, addr: usize, data: u32) {
        let base = self as *mut _ as *mut u8;
        let mask: u32 = (1 << 8) - 1;
        let d: u8 = (data & mask) as u8;
        let c: u8 = ((data & mask << 8) >> 8) as u8;
        let b: u8 = ((data & mask << 16) >> 16) as u8;
        let a: u8 = ((data & mask << 24) >> 24) as u8;
        unsafe {
            *base.add(addr) = d;
            *base.add(1) = c;
            *base.add(2) = b;
            *base.add(3) = a;
        }
    }
}
