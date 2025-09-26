//! Process management syscalls

use core::{mem::size_of, ptr::copy_nonoverlapping};

use alloc::vec::Vec;

use crate::{
    config::PAGE_SIZE,
    mm::{
        translated_byte_buffer, MapPermission, PageTable, PageTableEntry, PhysAddr, VirtAddr,
        VirtPageNum,
    },
    task::{
        self, change_program_brk, current_user_token, delete_area_from_current_task,
        exit_current_and_run_next, insert_frames_to_current_task, suspend_current_and_run_next,
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
    let mut src = &data as *const TimeVal as *const u8;
    let struct_size = size_of::<TimeVal>();
    let buffer: Vec<&mut [u8]> =
        translated_byte_buffer(current_user_token(), _ts as *const u8, struct_size);
    for dst in buffer {
        unsafe {
            copy_nonoverlapping(src, dst.as_mut_ptr(), dst.len());
            src = src.add(dst.len());
        }
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");

    match (_id >> 38) & 1 {
        0 => {
            if _id >> 39 != 0 {
                return -1;
            }
        }
        1 => {
            if (_id >> 39).count_zeros() != 0 {
                return -1;
            }
        }
        _ => {}
    }

    let va = VirtAddr::from(_id);
    let pte: PageTableEntry;
    let ptr: *mut u8 = {
        let pagetable = PageTable::from_token(current_user_token());
        if let Some(tmp) = pagetable.translate(VirtPageNum::from(va.floor())) {
            let ppn = tmp.ppn();
            pte = tmp;
            (PhysAddr::from(ppn).0 + va.page_offset()) as *mut u8
        } else {
            return -1;
        }
    };
    match _trace_request {
        0 => unsafe {
            if !pte.readable() {
                return -1;
            }
            return *ptr as isize;
        },
        1 => {
            if !pte.writable() {
                return -1;
            }
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
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if port & 0x7 == 0 || port & !0x7 != 0 || start & (PAGE_SIZE - 1) != 0 {
        return -1;
    }

    let end = start + len;
    let mut cur = start;

    let pagetable = PageTable::from_token(current_user_token());
    while cur < end {
        let va = VirtAddr::from(cur);

        if let Some(pte) = pagetable.translate(VirtPageNum::from(va.floor())) {
            if pte.is_valid() {
                return -1;
            }
        }
        cur = VirtAddr::from(va.floor()).0 + PAGE_SIZE;
    }
    let (start_va, end_va) = (VirtAddr::from(start), VirtAddr::from(end));
    let permission = {
        let mut result = MapPermission::U;
        if port & 0b1 != 0 {
            result |= MapPermission::R;
        }
        if port & 0b10 != 0 {
            result |= MapPermission::W;
        }
        if port & 0b100 != 0 {
            result |= MapPermission::X;
        }
        result
    };
    insert_frames_to_current_task(start_va, end_va, permission);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let end = start + len;
    let (start_va, end_va) = (VirtAddr::from(start), VirtAddr::from(end));

    if start_va.page_offset() != 0 {
        return -1;
    }
    match delete_area_from_current_task(start_va, end_va) {
        Some(()) => 0,
        None => -1,
    }
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
