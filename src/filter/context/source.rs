use std::ptr;

use ffi::*;
use ::{Error, Frame};
use super::Context;

pub struct Source<'a> {
	ctx: &'a mut Context<'a>,
}

impl<'a> Source<'a> {
	pub unsafe fn wrap<'b>(ctx: &'b mut Context<'b>) -> Source<'b> {
		Source { ctx: ctx }
	}
}

impl<'a> Source<'a> {
	pub fn failed_requests(&self) -> usize {
		unsafe {
			av_buffersrc_get_nb_failed_requests(self.ctx.as_ptr()) as usize
		}
	}

	pub fn add(&mut self, frame: &Frame) -> Result<(), Error> {
		unsafe {
			match av_buffersrc_add_frame(self.ctx.as_mut_ptr(), frame.as_ptr() as *mut _) {
				0 => Ok(()),
				e => Err(Error::from(e)),
			}
		}
	}

	pub fn flush(&mut self) -> Result<(), Error> {
		unsafe {
			self.add(&Frame::wrap(ptr::null_mut()))
		}
	}
}
