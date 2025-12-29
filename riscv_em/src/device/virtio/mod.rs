mod registers;

use crate::{
    SoC,
    core::exceptions::{self, Exception},
    device::virtio_blk::virtio_blk_config,
    memory::{phys_read_hword, phys_write_hword, phys_write_word},
};
use registers::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct VirtioQueue {
    queue_size_max: u16,
    queue_size: u16,
    queue_ready: u32,
    queue_desc_low: u32,
    // queue_des_high: u32,
    queue_driver_low: u32,
    // queue_driver_high: u32,
    queue_device_low: u32,
    // queue_device_high: u32,
    // shared memory registers ...
    queue_reset: u32,
    last_avail: u16,
}

pub struct VirtioMmio<const queue_conut: usize> {
    pub base: u32,
    pub length: u32,
    pub interrupt_id: usize,

    pub device_id: u32,

    pub device_features: [u32; 2],
    pub device_features_sel: usize,
    pub driver_features: [u32; 2],
    pub driver_features_sel: usize,

    pub queue_sel: usize,
    pub queues: [VirtioQueue; queue_conut],
    pub queue_notify: u32,
    pub queue_notify_pending: bool,

    pub interrupt_status: u32,
    pub interrupt_ack: u32,
    pub status: u32,
    pub config_generation: u32,
    pub config: std::ptr::NonNull<dyn VirtioConfig>,
    pub config_size: u32,

    pub chain_process_function: fn(u16, &mut SoC) -> Result<u32, exceptions::Exception>,
}

impl<const queue_conut: usize> VirtioMmio<queue_conut> {
    fn tick(&mut self, soc: &mut SoC) {
        if self.interrupt_status > 0 {
            // TODO: plic set interrupt
        }

        if self.status & STATUS_NEEDS_RESET > 0 {
            return;
        }

        if self.queue_notify_pending {
            self.queue_notify_pending = false;
            self.handle_notify(soc);
        }
    }

    pub fn write(&mut self, addr: u32, data: u32) {
        let addr = addr - self.base;
        match addr {
            _DeviceFeaturesSel => {
                if data > 1 {
                    self.set_fail();
                } else {
                    self.device_features_sel = data as usize;
                }
            }
            _DriverFeatures => {
                self.driver_features[self.driver_features_sel] = data;
            }
            _DriverFeaturesSel => {
                if data > 1 {
                    self.set_fail();
                } else {
                    self.driver_features_sel = data as usize;
                }
            }
            _QueueSel => {
                self.queue_sel = data as usize;
            }
            _QueueSize => {
                self.queues[self.queue_sel].queue_size = data as u16;
            }
            _QueueReady => {
                self.queues[self.queue_sel].queue_ready = data;
            }
            _QueueNotify => {
                self.queue_notify = data;
                self.queue_notify_pending = true;
            }
            _InterruptACK => {
                // clear interrupt bits
                self.interrupt_status &= !data;
            }
            _Status => {
                if data == 0 {
                    self.reset();
                } else {
                    self.status |= data;
                }
            }
            _QueueDescLow => {
                self.queues[self.queue_sel].queue_desc_low = data;
            }
            _QueueDriverLow => {
                self.queues[self.queue_sel].queue_driver_low = data;
            }
            _QueueDeviceLow => {
                self.queues[self.queue_sel].queue_device_low = data;
            }
            _QueueReset => {
                self.queues[self.queue_sel].queue_reset = data;
            }
            _ => {
                if addr >= _Config && addr < _Config + self.config_size {
                    unsafe {
                        self.config.as_mut().write_word((addr - _Config) as usize, data);
                    }
                } else {
                    // Error
                    self.set_fail();
                }
            }
        }
    }

    pub fn read(&mut self, addr: u32) -> u32 {
        let addr = addr - self.base;
        return match addr {
            _MagicValue => 0x74726976,
            _Version => 0x2,
            _DeviceID => self.device_id,
            _VendorID => 0x0,
            _DeviceFeatures => self.device_features[self.device_features_sel],
            _QueueSizeMax => self.queues[self.queue_sel].queue_size_max as u32,
            _QueueReady => self.queues[self.queue_sel].queue_ready,
            _InterruptStatus => self.interrupt_status,
            _Status => self.status,
            _ConfigGeneration => self.config_generation,
            _ => {
                if addr >= _Config && addr < _Config + self.config_size {
                    unsafe {
                        self.config.as_mut().read_word((addr - _Config) as usize)
                    }
                } else {
                    // Error
                    self.set_fail();
                    0
                }
            }
        };
    }

    fn set_fail(&mut self) {
        self.status |= STATUS_NEEDS_RESET;
        if self.status & STATUS_DRIVER_OK > 0 {
            self.interrupt_status |= INT_ConfigurationChangeNotification;
        }
    }

    fn reset(&mut self) {
        // TODO:
    }

    fn handle_notify(&mut self, soc: &mut SoC) -> Result<(), exceptions::Exception> {
        // there is index to read in avaliable ring
        let queue = &mut self.queues[self.queue_notify as usize];
        let avail_idx = phys_read_hword(queue.queue_driver_low + 2, soc)?; // one behind index of last written entry
        let mut used_idx = phys_read_hword(queue.queue_device_low + 2, soc)?;
        while queue.last_avail != avail_idx {
            // while not all descriptor chain heads had been read
            let avail_queue_idx = queue.last_avail % queue.queue_size; // ring index of last unread head
            let head_idx = phys_read_hword(
                // index of chain head in avail ring
                queue.queue_driver_low + 4 + (2 * avail_queue_idx as u32), // 4 bytes in the available ring are for flags and idx
                soc,
            )?;

            // TODO: process chain
            let len = (self.chain_process_function)(head_idx, soc)?;

            let used_queue_idx = used_idx % queue.queue_size; // ring index of last unread head
            let used_ring_addr = queue.queue_driver_low + 4 + (8 * used_queue_idx as u32);
            phys_write_word(used_ring_addr, head_idx as u32, soc)?;
            phys_write_word(used_ring_addr + 4, len, soc)?;

            queue.last_avail += 1;
            used_idx += 1;
        }

        // flags field of used ring needs to be 0
        phys_write_hword(queue.queue_device_low, 0, soc)?;
        // write new idx to used ring
        phys_write_hword(queue.queue_device_low + 2, used_idx, soc)?;

        // INTERRUPT
        let used_ring_flags = phys_read_hword(queue.queue_device_low, soc)?;
        if used_ring_flags != 1 {
            // If flags is 1, the device SHOULD NOT send a notification
            self.interrupt_status |= INT_UsedBufferNotification;
        }

        Ok(())
    }
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
