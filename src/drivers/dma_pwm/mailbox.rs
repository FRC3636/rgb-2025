//! Dead-simple mailbox implementation adapted from https://github.com/raspberrypi/userland/blob/master/host_applications/linux/apps/hello_pi/hello_fft/mailbox.c

use libc::{open, O_RDWR, O_SYNC};

use crate::drivers::dma_pwm::PAGE_SIZE;

const IOCTL_MBOX_PROPERTY: i32 = 0xC008_6400;

pub type MailboxMemoryHandle = u32;

unsafe fn mailbox_property(fd: i32, buf: &mut [u32]) -> i32 {
    let result = unsafe { libc::ioctl(fd, IOCTL_MBOX_PROPERTY, buf.as_mut_ptr()) };

    if result < 0 {
        panic!("ioctl failed: {}", result);
    }

    result
}

/// Allocates memory and returns a handle to it
pub unsafe fn alloc(fd: i32, size: usize, align: usize, flags: u32) -> MailboxMemoryHandle {
    let mut prop_buffer = [0u32; 32];
    // buffer size in bytes
    prop_buffer[0] = (9 * 4) as u32;
    // tag id
    prop_buffer[2] = 0x3000c;
    // buffer size
    prop_buffer[3] = 12;
    // data size
    prop_buffer[4] = 12;
    // raspberry pi themselves seem unsure? "(num bytes? or pages?)"
    prop_buffer[5] = size as u32;
    // alignment
    prop_buffer[6] = align as u32;
    // MEM_FLAG_L1_NONALLOCATING
    prop_buffer[7] = flags;

    unsafe { mailbox_property(fd, &mut prop_buffer) };

    prop_buffer[5]
}

/// Frees a handle to memory
pub unsafe fn free(fd: i32, handle: MailboxMemoryHandle) -> u32 {
    let mut prop_buffer = [0u32; 32];
    // buffer size in bytes
    prop_buffer[0] = (7 * 4) as u32;
    // tag id
    prop_buffer[2] = 0x3000f;
    // buffer size
    prop_buffer[3] = 4;
    // data size
    prop_buffer[4] = 4;
    prop_buffer[5] = handle;

    unsafe { mailbox_property(fd, &mut prop_buffer) };

    prop_buffer[5]
}

/// Locks a handle to a fixed bus address
pub unsafe fn lock(fd: i32, handle: MailboxMemoryHandle) -> *mut u8 {
    let mut prop_buffer = [0u32; 32];
    // buffer size in bytes
    prop_buffer[0] = (7 * 4) as u32;
    // tag id
    prop_buffer[2] = 0x3000d;
    // buffer size
    prop_buffer[3] = 4;
    // data size
    prop_buffer[4] = 4;
    prop_buffer[5] = handle;
    prop_buffer[6] = 0;

    unsafe { mailbox_property(fd, &mut prop_buffer) };

    prop_buffer[5] as *mut u8
}

/// Locks a handle to a fixed bus address
pub unsafe fn unlock(fd: i32, handle: MailboxMemoryHandle) {
    let mut prop_buffer = [0u32; 32];
    // buffer size in bytes
    prop_buffer[0] = (7 * 4) as u32;
    // tag id
    prop_buffer[2] = 0x3000e;
    // buffer size
    prop_buffer[3] = 4;
    // data size
    prop_buffer[4] = 4;
    prop_buffer[5] = handle;
    prop_buffer[6] = 0;

    unsafe { mailbox_property(fd, &mut prop_buffer) };
}

pub unsafe fn map_mem(base: usize, size: usize) -> *mut u8 {
    let memory = unsafe { open(c"/dev/mem".as_ptr(), O_RDWR | O_SYNC) };

    if memory < 0 {
        panic!("Failed to open /dev/mem");
    }

    let offset = base % PAGE_SIZE;
    let base = base - offset;
    let size = size + offset;

    let result_ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            memory,
            base as i64,
        )
    };

    if result_ptr == -1isize as *mut libc::c_void {
        panic!("Failed to map");
    }

    result_ptr as *mut u8
}

pub unsafe fn unmap_mem(ptr: *mut u8, size: usize) {
    let offset = ptr as usize % PAGE_SIZE;

    let ptr = (ptr as usize - offset) as *mut u8;
    let size = size + offset;

    let result = unsafe { libc::munmap(ptr as *mut libc::c_void, size) };

    if result != 0 {
        panic!("Failed to unmap");
    }
}

pub unsafe fn open_mailbox() -> i32 {
    let fd = unsafe { libc::open(c"/dev/vcio".as_ptr(), 0) };

    if fd < 0 {
        panic!("Failed to open mailbox");
    }

    fd
}