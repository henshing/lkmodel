#![no_std]

#[macro_use]
extern crate axlog2;
extern crate alloc;
use alloc::vec;

use core::panic::PanicInfo;
use axtype::{align_up_4k, align_down_4k, phys_to_virt, virt_to_phys};
use driver_common::{BaseDriverOps, DeviceType};
use driver_block::{ramdisk, BlockDriverOps};
use driver_virtio::blk::VirtIoBlkDev;
use axhal::mem::memory_regions;

const DISK_SIZE: usize = 0x400_0000;    // 64M
const BLOCK_SIZE: usize = 0x200;        // 512

/// Entry
#[no_mangle]
pub extern "Rust" fn runtime_main(cpu_id: usize, _dtb_pa: usize) {
    axlog2::init();
    axlog2::set_max_level("debug");
    info!("[rt_driver_virtio]: ...");

    axhal::arch_init_early(cpu_id);

    info!("Initialize global memory allocator...");
    axalloc::init();

    info!("Found physcial memory regions:");
    for r in axhal::mem::memory_regions() {
        info!(
            "  [{:x?}, {:x?}) {} ({:?})",
            r.paddr,
            r.paddr + r.size,
            r.name,
            r.flags
        );
    }

    info!("Initialize kernel page table...");
    page_table::init();

    let mut alldevs = axdriver::init_drivers();
    let mut disk = alldevs.block.take_one().unwrap();

    assert_eq!(disk.device_type(), DeviceType::Block);
    assert_eq!(disk.device_name(), "virtio-blk");
    assert_eq!(disk.block_size(), BLOCK_SIZE);
    assert_eq!(disk.num_blocks() as usize, DISK_SIZE/BLOCK_SIZE);

    let block_id = 1;

    let mut buf = vec![0u8; BLOCK_SIZE];
    assert!(disk.read_block(block_id, &mut buf).is_ok());

    buf[0] = b'0';
    buf[1] = b'1';
    buf[2] = b'2';
    buf[3] = b'3';

    assert!(disk.write_block(block_id, &buf).is_ok());
    assert!(disk.flush().is_ok());

    assert!(disk.read_block(block_id, &mut buf).is_ok());
    assert!(buf[0..4] == *b"0123");

    info!("[rt_driver_virtio]: ok!");
    axhal::misc::terminate();
}

pub fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    arch_boot::panic(info)
}
