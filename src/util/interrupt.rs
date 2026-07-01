use std::panic;
use std::process;

use ffi::*;
use libc::{c_int, c_void};

pub struct Interrupt {
    pub interrupt: AVIOInterruptCB,
    pub guard: InterruptGuard,
}

pub struct InterruptGuard {
    opaque: *mut c_void,
    drop_fn: unsafe fn(*mut c_void),
}

unsafe impl Send for InterruptGuard {}
unsafe impl Sync for InterruptGuard {}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        if !self.opaque.is_null() {
            unsafe { (self.drop_fn)(self.opaque) };
            self.opaque = std::ptr::null_mut();
        }
    }
}

extern "C" fn callback<F>(opaque: *mut c_void) -> c_int
where
    F: FnMut() -> bool,
{
    // Clippy suggests to remove &mut, but it doesn't compile then (move occurs because value has type `F`, which does not implement the `Copy` trait)
    #[allow(clippy::needless_borrow)]
    match panic::catch_unwind(|| (unsafe { &mut *(opaque as *mut F) })()) {
        Ok(ret) => ret as c_int,
        Err(_) => process::abort(),
    }
}

unsafe fn drop_box<F>(opaque: *mut c_void) {
    drop(Box::from_raw(opaque as *mut F));
}

pub fn new<F>(opaque: Box<F>) -> Interrupt
where
    F: FnMut() -> bool + Send + 'static,
{
    let opaque = Box::into_raw(opaque) as *mut c_void;
    let interrupt_cb = AVIOInterruptCB {
        callback: Some(callback::<F>),
        opaque,
    };
    Interrupt {
        interrupt: interrupt_cb,
        guard: InterruptGuard {
            opaque,
            drop_fn: drop_box::<F>,
        },
    }
}
