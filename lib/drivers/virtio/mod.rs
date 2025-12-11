// TODO: Split this into a dedicated driver
// Currently just a rust rewrite of this:
// https://operating-system-in-1000-lines.vercel.app/en/15-virtio-blk#virtio-device-initialization

extern crate alloc;

use core::{
    mem::{size_of, MaybeUninit},
    sync::atomic::{fence, Ordering},
};

use alloc::alloc::{alloc, dealloc, Layout};

const SECTOR_SIZE: u64 = 512;
const PAGE_SIZE: u32 = 4096;
const VIRTQ_ENTRY_NUM: usize = 16;
const VIRTIO_DEVICE_BLK: u32 = 2;
const VIRTIO_BLK_PADDR: usize = 0x10001000;
const VIRTIO_REG_MAGIC: usize = 0x00;
const VIRTIO_REG_VERSION: usize = 0x04;
const VIRTIO_REG_DEVICE_ID: usize = 0x08;
const VIRTIO_REG_QUEUE_SEL: usize = 0x30;
const VIRTIO_REG_QUEUE_NUM_MAX: u32 = 0x34;
const VIRTIO_REG_QUEUE_NUM: usize = 0x38;
const VIRTIO_REG_QUEUE_ALIGN: usize = 0x3c;
const VIRTIO_REG_QUEUE_PFN: usize = 0x40;
const VIRTIO_REG_QUEUE_READY: u32 = 0x44;
const VIRTIO_REG_QUEUE_NOTIFY: usize = 0x50;
const VIRTIO_REG_DEVICE_STATUS: usize = 0x70;
const VIRTIO_REG_DEVICE_CONFIG: usize = 0x100;
const VIRTIO_STATUS_ACK: u32 = 1;
const VIRTIO_STATUS_DRIVER: u32 = 2;
const VIRTIO_STATUS_DRIVER_OK: u32 = 4;
const VIRTIO_STATUS_FEAT_OK: u32 = 8;
const VIRTQ_DESC_F_NEXT: u16 = 1;
const VIRTQ_DESC_F_WRITE: u16 = 2;
const VIRTQ_AVAIL_F_NO_INTERRUPT: u32 = 1;
const VIRTIO_BLK_T_IN: u32 = 0;
const VIRTIO_BLK_T_OUT: u32 = 1;

#[allow(dead_code)]
#[repr(C)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[allow(dead_code)]
#[repr(C)]
struct VirtqAvail {
    flags: u16,
    index: u16,
    ring: [u16; VIRTQ_ENTRY_NUM],
}

#[allow(dead_code)]
#[repr(C)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

#[allow(dead_code)]
#[repr(C, align(4096))]
struct VirtqUsed {
    flags: u16,
    index: u16,
    ring: [VirtqUsedElem; VIRTQ_ENTRY_NUM],
}

#[allow(dead_code)]
#[repr(C)]
struct VirtioVirtq {
    descs: [VirtqDesc; VIRTQ_ENTRY_NUM],
    avail: VirtqAvail,
    used: VirtqUsed,
    queue_index: u32,
    used_index: *mut u16,
    last_used_index: u16,
}

#[allow(dead_code)]
#[repr(C)]
struct VirtioBlkReq {
    r#type: u32,
    reserved: u32,
    sector: u64,
    data: [u8; 512],
    status: u8,
}

unsafe fn virtio_reg_read32(offset: usize) -> u32 {
    let ptr = (VIRTIO_BLK_PADDR + offset) as *const u32;
    return core::ptr::read_volatile(ptr);
}

unsafe fn virtio_reg_read64(offset: usize) -> u64 {
    let ptr = (VIRTIO_BLK_PADDR + offset) as *const u64;
    return core::ptr::read_volatile(ptr);
}

unsafe fn virtio_reg_write32(offset: usize, value: u32) {
    let ptr = (VIRTIO_BLK_PADDR + offset) as *mut u32;
    return core::ptr::write_volatile(ptr, value);
}

unsafe fn virtio_reg_fetch_and_or32(offset: usize, value: u32) {
    virtio_reg_write32(offset, virtio_reg_read32(offset) | value);
}

static mut blk_request_vq: *mut VirtioVirtq = core::ptr::null_mut();
static mut blk_req: *mut VirtioBlkReq = core::ptr::null_mut();
static mut blk_req_paddr: u64 = 0;
static mut blk_capacity: u64 = 0;

pub fn virtio_blk_init() {
    unsafe {
        if (virtio_reg_read32(VIRTIO_REG_MAGIC) != 0x74726976) {
            panic!("virtio: invalid magic value");
        }
        if (virtio_reg_read32(VIRTIO_REG_VERSION) != 1) {
            panic!("virtio: invalid version");
        }
        if (virtio_reg_read32(VIRTIO_REG_DEVICE_ID) != VIRTIO_DEVICE_BLK) {
            panic!("virtio: invalid device id");
        }

        // 1. Reset the device.
        virtio_reg_write32(VIRTIO_REG_DEVICE_STATUS, 0);
        // 2. Set the ACKNOWLEDGE status bit: the guest OS has noticed the device.
        virtio_reg_fetch_and_or32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_ACK);
        // 3. Set the DRIVER status bit.
        virtio_reg_fetch_and_or32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_DRIVER);
        // 5. Set the FEATURES_OK status bit.
        virtio_reg_fetch_and_or32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_FEAT_OK);
        // 7. Perform device-specific setup, including discovery of virtqueues for the device
        blk_request_vq = virtq_init(0);
        // 8. Set the DRIVER_OK status bit.
        virtio_reg_write32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_DRIVER_OK);

        // Get the disk capacity.
        blk_capacity = virtio_reg_read64(VIRTIO_REG_DEVICE_CONFIG + 0) * SECTOR_SIZE;
        println!("virtio-blk: capacity is {} bytes", blk_capacity);

        blk_req = alloc(Layout::new::<VirtioBlkReq>()) as *mut VirtioBlkReq;
        blk_req_paddr = blk_req as u64;
    }
}

fn virtq_init(index: u32) -> *mut VirtioVirtq {
    unsafe {
        // Allocate a region for the virtqueue.
        let vq = alloc(Layout::new::<VirtioVirtq>()) as *mut VirtioVirtq;
        vq.write(VirtioVirtq {
            descs: core::mem::zeroed(),
            avail: VirtqAvail {
                flags: 0,
                index: 0,
                ring: [0; VIRTQ_ENTRY_NUM],
            },
            used: VirtqUsed {
                flags: 0,
                index: 0,
                ring: core::mem::zeroed(),
            },
            queue_index: index,
            used_index: core::ptr::null_mut(),
            last_used_index: 0,
        });
        (*vq).used_index = &mut (*vq).used.index as *mut u16;

        // 1. Select the queue writing its index (first queue is 0) to QueueSel.
        virtio_reg_write32(VIRTIO_REG_QUEUE_SEL, index);
        // 5. Notify the device about the queue size by writing the size to QueueNum.
        virtio_reg_write32(VIRTIO_REG_QUEUE_NUM, VIRTQ_ENTRY_NUM as u32);
        // 6. Notify the device about the used alignment by writing its value in bytes to QueueAlign.
        virtio_reg_write32(VIRTIO_REG_QUEUE_ALIGN, 0);
        // 7. Write the physical number of the first page of the queue to the QueuePFN register.
        virtio_reg_write32(VIRTIO_REG_QUEUE_PFN, vq as usize as u32);
        return vq;
    }
}

// Notifies the device that there is a new request. `desc_index` is the index
// of the head descriptor of the new request.
fn virtq_kick(vq: *mut VirtioVirtq, desc_index: u16) {
    unsafe {
        (*vq).avail.ring[((*vq).avail.index as usize) % VIRTQ_ENTRY_NUM] = desc_index;
        (*vq).avail.index += 1;
        fence(Ordering::SeqCst);
        virtio_reg_write32(VIRTIO_REG_QUEUE_NOTIFY, (*vq).queue_index);
        (*vq).last_used_index += 1;
    }
}

// Returns whether there are requests being processed by the device.
fn virtq_is_busy(vq: *mut VirtioVirtq) -> bool {
    let vq = unsafe { core::ptr::read_volatile(vq) };
    let used_index = unsafe { core::ptr::read_volatile(vq.used_index) };
    return vq.last_used_index != used_index;
}

fn read_write_disk(buf: &mut [u8; SECTOR_SIZE as usize], sector: u64, is_write: bool) {
    unsafe {
        if sector >= blk_capacity / SECTOR_SIZE {
            println!(
                "virtio: tried to read/write sector={}, but capacity is {}\n",
                sector,
                blk_capacity / SECTOR_SIZE
            );
            return;
        }

        let mut blk_req_l = unsafe { core::ptr::read_volatile(blk_req) };
        // Construct the request according to the virtio-blk specification.
        blk_req_l.sector = sector;
        blk_req_l.r#type = if is_write {
            VIRTIO_BLK_T_OUT
        } else {
            VIRTIO_BLK_T_IN
        };
        unsafe {
            core::ptr::write_volatile(blk_req, blk_req_l);
        }
        if is_write {
            core::ptr::copy_nonoverlapping(
                buf.as_ptr(),
                (*blk_req).data.as_mut_ptr(),
                SECTOR_SIZE as usize,
            );
        }

        // Construct the virtqueue descriptors (using 3 descriptors).
        let vq = blk_request_vq;
        (*vq).descs[0].addr = blk_req_paddr;
        (*vq).descs[0].len = (size_of::<u32>() * 2 + size_of::<u64>()) as u32;
        (*vq).descs[0].flags = VIRTQ_DESC_F_NEXT;
        (*vq).descs[0].next = 1;

        (*vq).descs[1].addr = blk_req_paddr + (size_of::<u32>() * 2 + size_of::<u64>()) as u64;
        (*vq).descs[1].len = SECTOR_SIZE as u32;
        (*vq).descs[1].flags = VIRTQ_DESC_F_NEXT | (if is_write { 0 } else { VIRTQ_DESC_F_WRITE });
        (*vq).descs[1].next = 2;

        (*vq).descs[2].addr =
            blk_req_paddr + (size_of::<u32>() * 2 + size_of::<u64>()) as u64 + SECTOR_SIZE;
        (*vq).descs[2].len = 1;
        (*vq).descs[2].flags = VIRTQ_DESC_F_WRITE;

        // Notify the device that there is a new request.
        virtq_kick(vq, 0);

        // Wait until the device finishes processing.
        while virtq_is_busy(vq) {}

        let blk_req_l = unsafe { core::ptr::read_volatile(blk_req) };
        // virtio-blk: If a non-zero value is returned, it's an error.
        if blk_req_l.status != 0 {
            println!(
                "virtio: warn: failed to read/write sector={} status={}",
                sector, blk_req_l.status
            );
            return;
        }

        // For read operations, copy the data into the buffer.
        if !is_write {
            core::ptr::copy_nonoverlapping(
                (*blk_req).data.as_ptr(),
                buf.as_mut_ptr(),
                SECTOR_SIZE as usize,
            );
        }
    }
}

pub fn test() -> ! {
    let mut buf = [0; SECTOR_SIZE as usize];
    read_write_disk(&mut buf, 0, false);
    println!("first sector: {}", core::str::from_utf8(&buf).unwrap());

    let mut buf = [0; SECTOR_SIZE as usize];
    let test_str = "Hello, Virtio!";
    for (i, byte) in test_str.bytes().enumerate() {
        buf[i] = byte;
    }
    read_write_disk(&mut buf, 1, true);
    loop {}
}
