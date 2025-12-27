mod registers;
use registers::*;

#[derive(Debug)]
struct VirtioQueue {
    queue_size_max: u32,
    queue_size: u32,
    queue_ready: u32,
    queue_desc_low: u32,
    // queue_des_high: u32,
    queue_driver_low: u32,
    // queue_driver_high: u32,
    queue_device_low: u32,
    // queue_device_high: u32,
    // shared memory registers ...
    queue_reset: u32,
    last_avail: u32,
}

#[derive(Debug)]
pub struct VirtioMmio<const queue_conut: usize> {
    base: u32,
    length: u32,
    interrupt_id: usize,

    device_id: u32,

    device_features: [u32; 2],
    device_features_sel: usize,
    driver_features: [u32; 2],
    driver_features_sel: usize,

    queue_sel: usize,
    queues: [VirtioQueue; queue_conut],
    queue_notify: u32,
    queue_notify_pending: bool,

    interrupt_status: u32,
    interrupt_ack: u32,
    status: u32,
    config_generation: u32,
    config: u32,
}

impl<const queue_conut: usize> VirtioMmio<queue_conut> {
    pub fn tick(&mut self) {
        if self.interrupt_status > 0 {
            // TODO: plic set interrupt
        }

        if self.status & STATUS_NEEDS_RESET > 0 {
            return;
        }

        if self.queue_notify_pending {
            self.queue_notify_pending = false;
            self.handle_notify();
        }
    }

    pub fn write(&mut self, addr: u32, data: u32) {
        let addr = addr - self.base;
        match addr {
            _DeviceFeaturesSel => {
                self.device_features_sel = data as usize;
            }
            _DriverFeatures => {
                self.driver_features[self.driver_features_sel] = data;
            }
            _DriverFeaturesSel => {
                self.driver_features_sel = data as usize;
            }
            _QueueSel => {
                self.queue_sel = data as usize;
            }
            _QueueSize => {
                self.queues[self.queue_sel].queue_size = data;
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
            _Config => {
                self.config = data;
            }
            _ => {
                // Error
                self.set_fail();
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
            _QueueSizeMax => self.queues[self.queue_sel].queue_size_max,
            _QueueReady => self.queues[self.queue_sel].queue_ready,
            _InterruptStatus => self.interrupt_status,
            _Status => self.status,
            _ConfigGeneration => self.config_generation,
            _Config => self.config,
            _ => {
                // error
                self.set_fail();
                0
            }
        };
    }

    fn set_fail(&mut self){
        self.status |= STATUS_NEEDS_RESET;
        if self.status & STATUS_DRIVER_OK > 0 {
            self.interrupt_status |= INT_ConfigurationChangeNotification;
        }
    }

    fn reset(&mut self) {
        // TODO:
    }

    fn handle_notify(&mut self) {}
}
