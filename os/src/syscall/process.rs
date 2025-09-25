//! Process management syscalls

use crate::{
    mm::{VirtPageNum, KERNEL_SPACE},
    task::{self, change_program_brk, exit_current_and_run_next, suspend_current_and_run_next},
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
    let kernel_space = KERNEL_SPACE.exclusive_access();
    let ptr = _ts as usize;
    if let Some(pa) = kernel_space.translate(crate::mm::VirtPageNum(ptr)) {
        let us = get_time_us();
        let pa = pa.ppn().get_mut::<TimeVal>();
        *pa = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
        return 0;
    }
    -1
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    let ptr: *mut u8 = {
        let kernel_space = KERNEL_SPACE.exclusive_access();

        if let Some(pte) = kernel_space.translate(VirtPageNum(_id)) {
            pte.ppn().get_mut::<u8>()
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
