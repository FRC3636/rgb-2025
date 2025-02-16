use std::{ffi::c_void, thread::sleep, time::Duration, u32};

use libc::{MAP_SHARED, O_RDWR, O_SYNC, PROT_READ, PROT_WRITE, mmap, open};
use mailbox::open_mailbox;

mod mailbox;

const PHYISCAL_PERIPHERAL_BASE: usize = 0xFE000000;
const BUS_PERIPHERAL_BASE: usize = 0x7E00_0000;

const DMA_OFFSET: usize = 0x7000;
const PWM_OFFSET: usize = 0x20_C000;
const GPIO_OFFSET: usize = 0x20_0000;
const CM_OFFSET: usize = 0x10_1000;
const SYSTEM_TIMER_OFFSET: usize = 0x3000;

const DMA_CHANNEL: u32 = 3;

const PAGE_SIZE: usize = 0x1000;

const MEM_FLAG_L1_NONALLOCATING: u32 = (1 << 2) | (2 << 2);

const DMA_NO_WIDE_BURSTS: u32 = 1 << 26;
const DMA_WAIT_RESP: u32 = 1 << 3;
const DMA_DEST_DREQ: u32 = 1 << 6;
const DMA_CHANNEL_ABORT: u32 = 1 << 30;
const DMA_CHANNEL_RESET: u32 = 1 << 31;
const DMA_INTERRUPT_STATUS: u32 = 1 << 2;
const DMA_END_FLAG: u32 = 1 << 1;
const DMA_DISDEBUG: u32 = 1 << 29;
const DMA_WAIT_ON_WRITES: u32 = 1 << 6;
const DMA_ACTIVE: u32 = 1 << 0;

const PWM0_DREQ: u32 = 5;

const PLLD_FREQ: u32 = 500_000_000;
const PLLD_DIV: u32 = 5;

const PLLD_ACTUAL_FREQ: u32 = PLLD_FREQ / PLLD_DIV;

// what in the silly
const CM_PASSWORD: u32 = 0x5A << 24;
/// Phase-locked-loop (500Mhz clock)
const CM_SOURCE_PLLD: u32 = 6;
const CM_KILL: u32 = 1 << 5;
const CM_ENABLE: u32 = 1 << 4;
const CM_BUSY: u32 = 1 << 7;

const PWM_DMA_ENABLE: u32 = 1 << 31;
const PWM_CLEAR_FIFO: u32 = 1 << 6;
const PWM_USE_FIFO1: u32 = 1 << 5;
const PWM_ENABLE_PWEN1: u32 = 1 << 0;
const PWM_MODE1_ENABLE_SERIALIZER: u32 = 1 << 1;

/// DMA control block linked list element
#[repr(C)]
#[derive(Debug, Clone, Copy)]
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
    padding: [u32; 2],
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
            mailbox::unmap_mem(self.virtual_memory_address, self.size);
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
#[repr(C)]
struct PwmControlRegister {
    /// PWM control
    ctl: u32,
    /// PWM status
    sta: u32,
    /// PWM DMA configuration
    dmac: u32,
    /// PWM Channel 1 range
    rng1: u32,
    /// PWM Channel 1 data (used if CTL USEFi = 0)
    dat1: u32,
    /// PWM FIFO input (used if CTL USEFi = 1)
    fif1: u32,
    /// PWM Channel 2 range
    rng2: u32,
    /// PWM Channel 2 data (used if CTL USEFi = 0)
    dat2: u32,
}

#[repr(C)]
struct SystemTimerControlRegister {
    /// System timer control and status
    cs: u32,
    /// System timer counter lower 32 bits
    clo: u32,
    /// System timer counter higher 32 bits
    chi: u32,
    /// System timer compare 0
    c0: u32,
    /// System timer compare 1
    c1: u32,
    /// System timer compare 2
    c2: u32,
    /// System timer compare 3
    c3: u32,
}

#[repr(C)]
struct ClockManagerControlRegister {
    /// General purpose clock control
    gp0_ctl: u32,
    /// General purpose clock divisors
    gp0_div: u32,
    /// General purpose clock control
    gp1_ctl: u32,
    /// General purpose clock divisors
    gp1_div: u32,
    /// General purpose clock control
    gp2_ctl: u32,
    /// General purpose clock divisors
    gp2_div: u32,
}

/// Converts a bus address to a physical memory address
#[inline]
const fn bus_addr_to_phys_addr(bus_addr: u32) -> u32 {
    bus_addr & !0xC0000000
}

unsafe fn map_peripheral(offset: usize, size: usize) -> *mut u8 {
    let memory = unsafe { open(c"/dev/mem".as_ptr(), O_RDWR | O_SYNC) };
    if memory < 0 {
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

unsafe fn enable_hardware_timer(cm_ctrl: *mut ClockManagerControlRegister) {
    unsafe {
        // Wait for clock to become not busy
        while ((&raw mut (*cm_ctrl).gp0_ctl).read_volatile() & CM_BUSY) != 0 {
            // Kill clock
            (&raw mut (*cm_ctrl).gp0_ctl).write_volatile(CM_PASSWORD | CM_KILL);
        }

        // Set clock source to PLLD
        (&raw mut (*cm_ctrl).gp0_ctl).write_volatile(CM_PASSWORD | CM_SOURCE_PLLD);
        sleep(Duration::from_micros(10));
        // Set divisor to 5 to end with an effective frequency of 100MHz
        (&raw mut (*cm_ctrl).gp0_div).write_volatile(CM_PASSWORD | (PLLD_DIV << 12));
        sleep(Duration::from_micros(10));
        // Enable clock
        (&raw mut (*cm_ctrl).gp0_ctl).write_volatile(
            (&raw const (*cm_ctrl).gp0_div).read_volatile() | CM_PASSWORD | CM_ENABLE,
        );
    }
}

unsafe fn start_dma(dma_ctrl: *mut DmaControlRegister, dma_cbs: &DmaMemoryAllocationHandle) {
    unsafe {
        // Abort and reset DMA channel
        (&raw mut (*dma_ctrl).cs).write_volatile(DMA_CHANNEL_ABORT);
        (&raw mut (*dma_ctrl).cs).write_volatile(0);
        (&raw mut (*dma_ctrl).cs).write_volatile(DMA_CHANNEL_RESET);
        (&raw mut (*dma_ctrl).cb_addr).write_volatile(0);

        (&raw mut (*dma_ctrl).cs).write_volatile(DMA_INTERRUPT_STATUS | DMA_END_FLAG);

        // Enable DMA channel
        (&raw mut (*dma_ctrl).cb_addr).write_volatile(dma_cbs.bus_memory_address as u32);
        (&raw mut (*dma_ctrl).cs).write_volatile((8 << 16) | (8 << 20) | DMA_DISDEBUG);
        (&raw mut (*dma_ctrl).cs).write_volatile(
            (&raw mut (*dma_ctrl).cs).read_volatile() | DMA_ACTIVE | DMA_WAIT_ON_WRITES,
        );
    }
}

unsafe fn stop_dma(dma_ctrl: *mut DmaControlRegister) {
    unsafe {
        // Abort current transfer
        (&raw mut (*dma_ctrl).cs).write_volatile(DMA_CHANNEL_ABORT);
        sleep(Duration::from_micros(100));
        // Clear the active bit
        (&raw mut (*dma_ctrl).cs)
            .write_volatile((&raw mut (*dma_ctrl).cs).read_volatile() & !DMA_ACTIVE);
        // Reset the DMA device
        (&raw mut (*dma_ctrl).cs)
            .write_volatile((&raw mut (*dma_ctrl).cs).read_volatile() | DMA_CHANNEL_RESET);
        sleep(Duration::from_micros(100));
    }
}

unsafe fn start_pwm(pwm_ctrl: *mut PwmControlRegister) {
    unsafe {
        // Reset PWM
        (&raw mut (*pwm_ctrl).ctl).write_volatile(0);
        sleep(Duration::from_micros(10));
        (&raw mut (*pwm_ctrl).sta).write_volatile(u32::MAX);
        sleep(Duration::from_micros(10));

        let target_micros = 100;
        let cycles = PLLD_ACTUAL_FREQ / 1_000_000 * target_micros;

        // Set range
        (&raw mut (*pwm_ctrl).rng1).write_volatile(cycles);

        // Enable PWM DMA and set thresholds for PANIC and DREQ to 15
        (&raw mut (*pwm_ctrl).dmac).write_volatile(PWM_DMA_ENABLE | (15 << 8) | 15);
        sleep(Duration::from_micros(10));

        // Clear FIF1
        (&raw mut (*pwm_ctrl).ctl).write_volatile(PWM_CLEAR_FIFO);
        sleep(Duration::from_micros(10));

        // Enable PWM and use FIFO
        (&raw mut (*pwm_ctrl).ctl)
            .write_volatile(PWM_MODE1_ENABLE_SERIALIZER | PWM_USE_FIFO1 | PWM_ENABLE_PWEN1);
    }
}

pub unsafe fn timer_read_test(num_reads: usize) {
    let num_cbs = num_reads * 2;

    unsafe {
        let dma_base = map_peripheral(DMA_OFFSET, PAGE_SIZE);

        let mapped_dma_reg: *mut DmaControlRegister =
            dma_base.offset(DMA_CHANNEL as isize * 0x100).cast();

        let mapped_pwm_reg: *mut PwmControlRegister =
            map_peripheral(PWM_OFFSET, size_of::<PwmControlRegister>()).cast();
        let mapped_sys_timer_reg: *mut SystemTimerControlRegister =
            map_peripheral(SYSTEM_TIMER_OFFSET, size_of::<SystemTimerControlRegister>()).cast();
        let mapped_cm_reg: *mut ClockManagerControlRegister =
            map_peripheral(CM_OFFSET, size_of::<ClockManagerControlRegister>()).cast();

        sleep(Duration::from_micros(100));

        let mailbox = open_mailbox();

        let dma_cbs = DmaMemoryAllocationHandle::alloc(
            mailbox,
            num_cbs * std::mem::size_of::<DmaControlBlock>(),
        );
        let dma_ticks =
            DmaMemoryAllocationHandle::alloc(mailbox, num_reads * std::mem::size_of::<u32>());
        sleep(Duration::from_micros(100));

        let dummy_data = 0u32;
        for i in 0..num_reads {
            let cb = nth_cb_virtual_address(&dma_cbs, i * 2);
            cb.write_volatile(DmaControlBlock {
                ti: DMA_NO_WIDE_BURSTS | DMA_WAIT_RESP,
                // system timer address
                source_ad: BUS_PERIPHERAL_BASE as u32 + SYSTEM_TIMER_OFFSET as u32 + 0x4,
                // copy to dma_ticks
                dest_ad: dma_ticks
                    .bus_memory_address
                    .offset(i as isize * std::mem::size_of::<u32>() as isize)
                    as u32,
                txfr_len: std::mem::size_of::<u32>() as _,
                stride: 0,
                nextconbk: nth_cb_bus_address(&dma_cbs, i * 2 + 1) as _,
                padding: [0, 0],
            });

            let cb = nth_cb_virtual_address(&dma_cbs, i * 2 + 1);
            cb.write_volatile(DmaControlBlock {
                ti: DMA_NO_WIDE_BURSTS | DMA_WAIT_RESP | DMA_DEST_DREQ | (PWM0_DREQ << 16),
                // system timer address
                source_ad: &raw const dummy_data as _,
                // copy to dma_ticks
                dest_ad: (BUS_PERIPHERAL_BASE + PWM_OFFSET + 0x20) as _,
                txfr_len: std::mem::size_of::<u32>() as _,
                stride: 0,
                nextconbk: if i < num_reads - 1 {
                    nth_cb_bus_address(&dma_cbs, i * 2 + 2) as _
                } else {
                    0
                },
                padding: [0, 0],
            });
        }

        enable_hardware_timer(mapped_cm_reg);
        sleep(Duration::from_micros(100));

        start_pwm(mapped_pwm_reg);
        sleep(Duration::from_micros(100));

        start_dma(mapped_dma_reg, &dma_cbs);
        sleep(Duration::from_micros(100));

        let mut results = [0u32; 20];
        results.copy_from_slice(std::slice::from_raw_parts(
            dma_ticks.virtual_memory_address.cast(),
            20,
        ));

        println!("{:#?}", results);

        stop_dma(mapped_dma_reg);
    }
}
