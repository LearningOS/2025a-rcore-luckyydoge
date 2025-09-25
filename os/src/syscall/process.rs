//! Process management syscalls

use core::{mem::size_of, ptr::copy_nonoverlapping};

use alloc::vec::Vec;

use crate::{
    mm::{translated_byte_buffer, VirtAddr, VirtPageNum, KERNEL_SPACE},
    task::{
        self, change_program_brk, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let data = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let src = &data as *const TimeVal as *const u8;
    let struct_size = size_of::<TimeVal>();
    let buffer: Vec<&mut [u8]> =
        translated_byte_buffer(current_user_token(), _ts as *const u8, struct_size);
    let mut offset = 0;
    for dst in buffer {
        let len = dst.len().min(struct_size - offset);
        let dst_ptr: *mut u8 = dst.as_mut_ptr();
        let src_ptr: *const u8 = unsafe { src.add(offset) };
        unsafe {
            copy_nonoverlapping(src_ptr, dst_ptr, len);
        }
        offset += len;
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");

    let va = VirtAddr::from(_id);
    let ptr: *mut u8 = {
        let kernel_space = KERNEL_SPACE.exclusive_access();
        if let Some(pte) = kernel_space.translate(VirtPageNum::from(va.floor())) {
            let ppn = pte.ppn();
            (ppn.0 + va.page_offset()) as *mut u8
        } else {
            return -1;
        }
    };
    match _trace_request {
        0 => unsafe {
            return *ptr as isize;
        },
        1 => {
            unsafe {
                *ptr = _data as u8;
            }
            return 0;
        }
        2 => task::get_syscall_cnt(_id) as isize,
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
