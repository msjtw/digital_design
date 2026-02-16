#![allow(non_camel_case_types)]

use crate::memory::ram::RAM;

use super::virtio::{registers::*, *};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::linux::fs::MetadataExt;

const VIRTIO_BLK_T_IN: u32 = 0;
const VIRTIO_BLK_T_OUT: u32 = 1;

const VIRTIO_BLK_S_OK: u8 = 0;
const VIRTIO_BLK_S_IOERR: u8 = 1;
const VIRTIO_BLK_S_UNSUPP: u8 = 2;

const DISK_BLK_SIZE: u64 = 512;

pub struct VirtioBlk {
    pub config: virtio_blk_config,
    pub config_size: u32,
    pub drive: Option<File>,
}

impl VirtioBlk {
    pub fn init(&mut self) {
        let drive = match OpenOptions::new().read(true).write(true).open("disk_file") {
            Ok(file) => file,
            Err(err) => panic!("disk file error: {:?}", err),
        };
        let meta = match fs::metadata("disk_file") {
            Ok(meta) => meta,
            Err(err) => panic!("disk file error: {:?}", err),
        };
        self.config = virtio_blk_config::default();
        self.config.capacity = ((meta.st_size() - 1) / DISK_BLK_SIZE) + 1;
        self.config_size = size_of::<virtio_blk_config>() as u32;
        self.drive = Some(drive);
    }
}

impl Default for VirtioBlk {
    fn default() -> Self {
        let mut config = virtio_blk_config::default();
        config.capacity = 0;
        VirtioBlk {
            config,
            config_size: 0,
            drive: None,
        }
    }
}
impl VirtioDev for VirtioBlk {
    fn get_config(&mut self) -> &mut dyn VirtioConfig {
        let base = &self.config as *const _ as *const u8;
        &mut self.config
    }

    fn get_conf_size(&self) -> u32 {
        self.config_size
    }

    fn process_chain(
        &mut self,
        queue: &mut VirtioQueue,
        head_idx: u16,
        ram: &mut RAM,
    ) -> Result<u32, ()> {
        // Blk request is divided into data fields of 3 descriptors:
        // first (length 4):
        //     le32 type
        //     le32 reserved
        //     le64 sector
        // second (desc.len field is length of data):
        //     u8 data[]
        // third (length 1):
        //     u8 status
        // This is not directly specified in virtio specification.
        //

        if self.drive.is_none() {
            return Err(());
        }

        let head_addr = queue.queue_desc_low + 16 * head_idx as u32;
        let head_desc = Descriptor::read(head_addr, ram);
        let op_type = ram.load_word(head_desc.addr as u32);

        if head_desc.flags & VIRTQ_DESC_F_NEXT == 0 {
            return Err(());
        }
        let data_addr = queue.queue_desc_low + (16 * head_desc.next as u32);
        let data_desc = Descriptor::read(data_addr, ram);
        if data_desc.flags & VIRTQ_DESC_F_NEXT == 0 {
            return Err(());
        }
        let status_addr = queue.queue_desc_low + (16 * data_desc.next as u32);
        let status_desc = Descriptor::read(status_addr, ram);
        if status_desc.flags & VIRTQ_DESC_F_NEXT != 0 {
            return Err(());
        }

        // skip reserved
        let sector = ram.load_word(head_desc.addr as u32 + 8) as u64;

        // FIX: Add check if sector in range. Write VIRTIO_BLK_S_IOERR to status.

        match op_type {
            VIRTIO_BLK_T_IN => {
                // read
                let mut buf = vec![0u8; data_desc.len as usize];
                self.drive
                    .as_ref()
                    .unwrap()
                    .seek(SeekFrom::Start(sector * 512))
                    .map_err(|_| ())?;
                self.drive
                    .as_ref()
                    .unwrap()
                    .read(&mut buf)
                    .map_err(|_| ())?;
                for i in 0..(data_desc.len as usize) {
                    ram.store_byte(data_desc.addr as u32 + i as u32, buf[i]);
                }
            }
            VIRTIO_BLK_T_OUT => {
                // write
                let mut buf = vec![0u8; data_desc.len as usize];
                for i in 0..(data_desc.len as usize) {
                    buf[i] = ram.load_byte(data_desc.addr as u32 + i as u32);
                }
                self.drive
                    .as_ref()
                    .unwrap()
                    .seek(SeekFrom::Start(sector * 512))
                    .map_err(|_| ())?;
                self.drive
                    .as_ref()
                    .unwrap()
                    .write(&mut buf)
                    .map_err(|_| ())?;
            }
            _ => {
                // unsuported or not valid
                ram.store_byte(status_desc.addr as u32, VIRTIO_BLK_S_UNSUPP);
                return Err(());
            }
        }

        return Ok(data_desc.len);
    }
}

#[derive(Default)]
#[repr(C, packed)]
pub struct virtio_blk_config {
    pub capacity: u64,
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

#[derive(Default, Debug)]
#[repr(C, packed)]
struct virtio_blk_geometry {
    cylinders: u16,
    heads: u8,
    sectors: u8,
}

#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
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
