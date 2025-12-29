#![allow(non_camel_case_types)]
use crate::{SoC, core::exceptions};

use super::virtio::*;

struct VirtioBlk {
    vmmio: VirtioMmio<2>,
    config: Box<virtio_blk_config>,
}

impl Default for VirtioBlk {
    fn default() -> Self {
        let mut config = Box::new(virtio_blk_config::default());
        VirtioBlk {
            vmmio: VirtioMmio::<2> {
                base: 0x4200000,
                length: 0x200,
                interrupt_id: 3,

                device_id: 2,

                device_features: [0; 2],
                device_features_sel: 0,
                driver_features: [0; 2],
                driver_features_sel: 0,

                queue_sel: 0,
                queues: [VirtioQueue::default(); 2],
                queue_notify: 0,
                queue_notify_pending: false,

                interrupt_status: 0,
                interrupt_ack: 0,
                status: 0,
                config_generation: 0,
                config: std::ptr::NonNull::from(&mut *config),
                config_size: 8,

                chain_process_function: blk_process_chain,
            },
            config,
        }
    }
}

fn blk_process_chain(head_idx: u16, soc: &mut SoC) -> Result<u32, exceptions::Exception> {
    Ok(0)
}

#[derive(Default)]
#[repr(C, packed)]
pub struct virtio_blk_config {
    capacity: u64,
    size_max: u32,
    seg_max: u32,
    geometry: virtio_blk_geometry,
    blk_size: u32,
    topology: virtio_blk_topology,
    writeback: u8,
    unused0: u8,
    num_queues: u16,
    max_discard_sectors: u32,
    max_discard_seg: u32,
    discard_sector_alignment: u32,
    max_write_zeroes_sectors: u32,
    max_write_zeroes_seg: u32,
    write_zeroes_may_unmap: u8,
    unused1: [u8; 3],
    max_secure_erase_sectors: u32,
    max_secure_erase_seg: u32,
    secure_erase_sector_alignment: u32,
    zoned: virtio_blk_zoned_characteristics,
}

#[derive(Default)]
#[repr(C, packed)]
struct virtio_blk_geometry {
    cylinders: u16,
    heads: u8,
    sectors: u8,
}

#[derive(Default)]
#[repr(C, packed)]
struct virtio_blk_topology {
    // # of logical blocks per physical block (log2)
    physical_block_exp: u8,
    // offset of first aligned logical block
    alignment_offset: u8,
    // suggested minimum I/O size in blocks
    min_io_size: u16,
    // optimal (suggested maximum) I/O size in blocks
    opt_io_size: u32,
}

#[derive(Default)]
#[repr(C, packed)]
struct virtio_blk_zoned_characteristics {
    zone_sectors: u32,
    max_open_zones: u32,
    max_active_zones: u32,
    max_append_sectors: u32,
    write_granularity: u32,
    model: u8,
    unused2: [u8; 3],
}

impl VirtioConfig for virtio_blk_config {}
