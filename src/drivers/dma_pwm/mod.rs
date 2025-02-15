use std::{ffi::c_void, fs::File, os::fd::AsRawFd, thread::sleep, time::Duration};

use libc::{mmap, open, MAP_SHARED, O_RDWR, O_SYNC, PROT_READ, PROT_WRITE};
use mailbox::open_mailbox;

mod mailbox;

const PHYISCAL_PERIPHERAL_BASE: usize = 0x3F00_0000;
const BUS_PERIPHERAL_BASE: usize = 0x7E00_0000;

const DMA_OFFSET: usize = 0x7000;

const DMA_CHANNEL: u32 = 6;

const PAGE_SIZE: usize = 0x1000;

const MEM_FLAG_L1_NONALLOCATING: u32 = (1 << 2) | (2 << 2);
const DMA_NO_WIDE_BURSTS: u32 = 1 << 26;
const DMA_WAIT_RESP: u32 = 1 << 3;
const DMA_CHANNEL_ABORT: u32 = 1 << 30;
const DMA_CHANNEL_RESET: u32 = 1 << 31;
const DMA_INTERRUPT_STATUS: u32 = 1 << 2;
const DMA_END_FLAG: u32 = 1 << 1;
const DMA_DISDEBUG: u32 = 1 << 28;
const DMA_WAIT_ON_WRITES: u32 = 1 << 28;
const DMA_ACTIVE: u32 = 1 << 0;

/// DMA control block linked list element
#[repr(C)]
struct DmaControlBlock {
    /// Transfer information
    ti: u32,
    /// Source address
    source_ad: u32,
    /// Destination address
    dest_ad: u32,
    /// Transfer length
    txfr_len: u32,
    /// 2D mode stride
    stride: u32,
    /// Next control block address
    nextconbk: u32,
}

/// Handle to a allocated memory fit for DMA transfers
/// This memory should remain cache-coherent and locked to a fixed bus address
/// via the mailbox property interface.
struct DmaMemoryAllocationHandle {
    virtual_memory_address: *mut u8,
    size: usize,
    bus_memory_address: *mut u8,
    mailbox_handle: u32,

    mailbox_fd: i32,
}
impl DmaMemoryAllocationHandle {
    pub unsafe fn alloc(mailbox_fd: i32, size: usize) -> Self {
        let size = size.div_ceil(PAGE_SIZE) * PAGE_SIZE;

        let mailbox_handle =
            unsafe { mailbox::alloc(mailbox_fd, size, PAGE_SIZE, MEM_FLAG_L1_NONALLOCATING) };
        let bus_memory_address = unsafe { mailbox::lock(mailbox_fd, mailbox_handle) };
        let virtual_memory_address =
            unsafe { mailbox::map_mem(bus_addr_to_phys_addr(bus_memory_address as _) as _, size) };

        Self {
            virtual_memory_address,
            size,
            bus_memory_address,
            mailbox_handle,

            mailbox_fd,
        }
    }

    unsafe fn _free(&mut self) {
        unsafe {
            mailbox::unlock(self.mailbox_fd, self.mailbox_handle);
            mailbox::free(self.mailbox_fd, self.mailbox_handle);
        }
    }

    pub unsafe fn free(mut self) {
        unsafe { self._free() };
    }
}

impl Drop for DmaMemoryAllocationHandle {
    fn drop(&mut self) {
        unsafe { self._free() };
    }
}

#[repr(C)]
struct DmaControlRegister {
    /// DMA Channel Control and Status
    cs: u32,
    /// DMA Control Block Address
    cb_addr: u32,
}

/// Converts a bus address to a physical memory address
#[inline]
const fn bus_addr_to_phys_addr(bus_addr: u32) -> u32 {
    bus_addr & !0xC0000000
}

unsafe fn map_peripheral(offset: usize, size: usize) -> *mut u8 {
    let memory = unsafe { open(c"/dev/mem".as_ptr(), O_RDWR | O_SYNC) };
    if memory == -1 {
        panic!("Failed to open /dev/mem");
    }

    let result_ptr = unsafe {
        mmap(
            std::ptr::null_mut(),
            size,
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            memory,
            (PHYISCAL_PERIPHERAL_BASE + offset) as i64,
        )
    };

    if result_ptr == -1isize as *mut c_void {
        panic!("Failed to map peripheral");
    }

    result_ptr.cast()
}

#[inline]
const unsafe fn nth_cb_virtual_address(
    cb: &DmaMemoryAllocationHandle,
    i: usize,
) -> *mut DmaControlBlock {
    unsafe {
        cb.virtual_memory_address
            .offset(i as isize * std::mem::size_of::<DmaControlBlock>() as isize)
    }
    .cast()
}
#[inline]
const unsafe fn nth_cb_bus_address(
    cb: &DmaMemoryAllocationHandle,
    i: usize,
) -> *mut DmaControlBlock {
    unsafe {
        cb.bus_memory_address
            .offset(i as isize * std::mem::size_of::<DmaControlBlock>() as isize)
    }
    .cast()
}

pub unsafe fn timer_read_test() {
    unsafe {
        let dma_base = map_peripheral(DMA_OFFSET, PAGE_SIZE);
        let mapped_dma_reg: *mut DmaControlRegister =
            dma_base.offset(DMA_CHANNEL as isize * 0x100).cast();
        sleep(Duration::from_micros(100));

        let mailbox = open_mailbox();

        let dma_cbs =
            DmaMemoryAllocationHandle::alloc(mailbox, 20 * std::mem::size_of::<DmaControlBlock>());
        let dma_ticks = DmaMemoryAllocationHandle::alloc(mailbox, 20 * std::mem::size_of::<u32>());
        sleep(Duration::from_micros(100));

        for i in 0..20 {
            let cb = nth_cb_virtual_address(&dma_cbs, i);
            cb.write_volatile(DmaControlBlock {
                ti: DMA_NO_WIDE_BURSTS | DMA_WAIT_RESP,
                // system timer address
                source_ad: BUS_PERIPHERAL_BASE as u32 + 0x3004,
                // copy to dma_ticks
                dest_ad: dma_ticks
                    .bus_memory_address
                    .offset(i as isize * std::mem::size_of::<u32>() as isize)
                    as u32,
                txfr_len: std::mem::size_of::<u32>() as _,
                stride: 0,
                nextconbk: nth_cb_bus_address(&dma_cbs, i + 1) as _,
            });
        }
        sleep(Duration::from_micros(100));

        // Abort and reset DMA channel
        (&raw mut (*mapped_dma_reg).cs).write_volatile(DMA_CHANNEL_ABORT);
        (&raw mut (*mapped_dma_reg).cs).write_volatile(0);
        (&raw mut (*mapped_dma_reg).cs).write_volatile(DMA_CHANNEL_RESET);
        (&raw mut (*mapped_dma_reg).cb_addr).write_volatile(0);

        (&raw mut (*mapped_dma_reg).cs).write_volatile(DMA_INTERRUPT_STATUS | DMA_END_FLAG);

        // Enable DMA channel
        (&raw mut (*mapped_dma_reg).cb_addr).write_volatile(dma_cbs.bus_memory_address as u32);
        (&raw mut (*mapped_dma_reg).cs).write_volatile((8 << 16) | (8 << 20) | DMA_DISDEBUG);
        (&raw mut (*mapped_dma_reg).cs).write_volatile(DMA_ACTIVE | DMA_WAIT_ON_WRITES);

        sleep(Duration::from_micros(100));

        let mut results = [0u32; 20];
        results.copy_from_slice(std::slice::from_raw_parts(
            dma_ticks.virtual_memory_address.cast(),
            20,
        ));

        println!("{:?}", results);

        (&raw mut (*mapped_dma_reg).cs).write_volatile(DMA_CHANNEL_ABORT);
        sleep(Duration::from_micros(100));
        (&raw mut (*mapped_dma_reg).cs).write_volatile(!DMA_ACTIVE);
        (&raw mut (*mapped_dma_reg).cs).write_volatile((*mapped_dma_reg).cs | DMA_CHANNEL_RESET);
        sleep(Duration::from_micros(100));
    }
}
