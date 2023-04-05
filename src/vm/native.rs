use std::time::Duration;

use super::{Result, Vm};
use crate::Value;

// https://stackoverflow.com/a/36719115
mod ffi {
    use libc::{c_int, timespec};
    extern "C" {
        pub fn clock_gettime(clk_id: c_int, tp: *mut timespec) -> c_int;
    }
}

pub(super) fn clock(arg_count: usize, vm: &mut Vm) -> Result<Value> {
    unsafe {
        let mut tp = std::mem::MaybeUninit::<libc::timespec>::uninit();
        if ffi::clock_gettime(libc::CLOCK_MONOTONIC, tp.as_mut_ptr()) == 0 {
            let tp = tp.assume_init();
            Ok(Value::Number(
                Duration::new(tp.tv_sec as u64, tp.tv_nsec as u32)
                    .as_secs_f64(),
            ))
        } else {
            panic!("{}", std::io::Error::last_os_error());
        }
    }
}
