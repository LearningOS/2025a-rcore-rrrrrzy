//! Process management syscalls
use crate::mm::{translated_byte_buffer, VirtAddr, KERNEL_SPACE};
use crate::task::{
    change_program_brk, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next, TASK_MANAGER,
};
use crate::timer::get_time_us;
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

impl TimeVal {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const Self as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let tv = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let tv_b = tv.as_bytes();
    let slices = translated_byte_buffer(
        current_user_token(),
        _tz as *const u8,
        core::mem::size_of::<TimeVal>(),
    );
    let mut offset = 0;
    for slice in slices {
        let len = slice.len();
        slice.copy_from_slice(&tv_b[offset..offset + len]);
        offset += len;
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        0 => {
            // let addr = _id as *const u8; // 这里无需再做成指针
            let kern = KERNEL_SPACE.exclusive_access();
            let va = VirtAddr::from(_id);
            match kern.translate(va.floor()) {
                Some(pte) => {
                    let ppn = pte.ppn();
                    ppn.get_bytes_array()[va.page_offset()] as isize
                }
                None => -1,
            }
        }
        1 => {
            let kern = KERNEL_SPACE.exclusive_access();
            let va = VirtAddr::from(_id);
            match kern.translate(va.floor()) {
                Some(pte) => {
                    let ppn = pte.ppn();
                    let byte = &mut ppn.get_bytes_array()[va.page_offset()];
                    *byte = _data as u8; // 修改值
                    0
                }
                None => -1,
            }
        }
        2 => TASK_MANAGER.get_syscall_count(_id),
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
