use super::StreamIo;
use ffi::*;

#[derive(Debug)]
pub enum Mode {
    Input,
    Output,
    InputCustomIo(StreamIo),
    OutputCustomIo(StreamIo),
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

// SAFETY: `Destructor` owns the `AVFormatContext` and, in the custom-IO
// modes, the `StreamIo` keep-alive (itself `Send`). `Drop` runs exactly once,
// with exclusive ownership, on whichever thread releases the last `Arc`;
// closing the context and dropping the `StreamIo` (which flushes the user
// stream) are safe from any thread because `StreamIo` and the wrapped stream
// are `Send` — hence `Send`. There are no `&self` methods and the fields are
// private, so a shared `&Destructor` gives another thread no way to touch the
// pointer or the embedded (non-`Sync`) `StreamIo` — hence `Sync`.
unsafe impl Send for Destructor {}
unsafe impl Sync for Destructor {}

impl Drop for Destructor {
    fn drop(&mut self) {
        unsafe {
            match self.mode {
                Mode::InputCustomIo(_) => {
                    // AVFMT_FLAG_CUSTOM_IO is set, so this leaves `pb` alone
                    // (demuxers' read_close() may still use it); the StreamIo
                    // in `mode` frees it when dropped after this body.
                    avformat_close_input(&mut self.ptr);
                }
                Mode::OutputCustomIo(_) => {
                    avformat_free_context(self.ptr);
                    // The StreamIo in `mode` is dropped afterwards; its Drop
                    // flushes buffered data to the stream before freeing the
                    // AVIOContext.
                }
                Mode::Input => avformat_close_input(&mut self.ptr),

                Mode::Output => {
                    avio_close((*self.ptr).pb);
                    avformat_free_context(self.ptr);
                }
            }
        }
    }
}
