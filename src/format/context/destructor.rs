use ffi::*;

#[derive(Copy, Clone, Debug)]
pub enum Mode {
    Input,
    Output,
}

pub struct Destructor {
    ptr: *mut AVFormatContext,
    mode: Mode,
}

impl Destructor {
    pub unsafe fn new(ptr: *mut AVFormatContext, mode: Mode) -> Self {
        Destructor { ptr, mode }
    }
}

// SAFETY: drop runs once with exclusive ownership of the pointer, on any
// thread; no `&self` methods.
unsafe impl Send for Destructor {}
unsafe impl Sync for Destructor {}

impl Drop for Destructor {
    fn drop(&mut self) {
        unsafe {
            match self.mode {
                Mode::Input => avformat_close_input(&mut self.ptr),

                Mode::Output => {
                    avio_close((*self.ptr).pb);
                    avformat_free_context(self.ptr);
                }
            }
        }
    }
}
