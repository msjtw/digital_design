#![allow(non_upper_case_globals)]

#[derive(Debug)]
enum RW {
    R,
    W,
}

pub const _MagicValue: u32 = 0x000;
pub const _Version: u32 = 0x004;
pub const _DeviceID: u32 = 0x008;
pub const _VendorID: u32 = 0x00c;
pub const _DeviceFeatures: u32 = 0x010;
pub const _DeviceFeaturesSel: u32 = 0x014;
pub const _DriverFeatures: u32 = 0x020;
pub const _DriverFeaturesSel: u32 = 0x024;
pub const _QueueSel: u32 = 0x030;
pub const _QueueSizeMax: u32 = 0x034;
pub const _QueueSize: u32 = 0x038;
pub const _QueueReady: u32 = 0x044;
pub const _QueueNotify: u32 = 0x050;
pub const _InterruptStatus: u32 = 0x060;
pub const _InterruptACK: u32 = 0x064;
pub const _Status: u32 = 0x070;
pub const _QueueDescLow: u32 = 0x080;
pub const _QueueDriverLow: u32 = 0x090;
pub const _QueueDeviceLow: u32 = 0x0a0;
pub const _QueueReset: u32 = 0x0c0;
pub const _ConfigGeneration: u32 = 0x0fc;
pub const _Config: u32 = 0x100;

pub const STATUS_ACKNOWLEDGE: u32 = 1;
pub const STATUS_DRIVER: u32 = 2;
pub const STATUS_DRIVER_OK: u32 = 4;
pub const STATUS_FEATURES_OK: u32 = 8;
pub const STATUS_NEEDS_RESET: u32 = 64;
pub const STATUS_FAILED: u32 = 128;

pub const INT_UsedBufferNotification: u32 = 0b01;
pub const INT_ConfigurationChangeNotification: u32 = 0b10;
