use std::ffi::{CStr, CString};
use std::ptr;
use std::str::from_utf8_unchecked;

use super::{Context, Filter};
use ffi::*;
use libc::c_int;
use Error;

pub struct Graph {
    ptr: *mut AVFilterGraph,
}

unsafe impl Send for Graph {}
unsafe impl Sync for Graph {}

impl Graph {
    pub unsafe fn wrap(ptr: *mut AVFilterGraph) -> Self {
        Graph { ptr }
    }

    pub unsafe fn as_ptr(&self) -> *const AVFilterGraph {
        self.ptr as *const _
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut AVFilterGraph {
        self.ptr
    }
}

impl Graph {
    pub fn new() -> Self {
        unsafe {
            let ptr = avfilter_graph_alloc();

            if ptr.is_null() {
                panic!("out of memory");
            }

            Graph::wrap(ptr)
        }
    }

    pub fn validate(&mut self) -> Result<(), Error> {
        unsafe {
            match avfilter_graph_config(self.as_mut_ptr(), ptr::null_mut()) {
                0 => Ok(()),
                e => Err(Error::from(e)),
            }
        }
    }

    pub fn add<'a, 'b>(
        &'a mut self,
        filter: &Filter,
        name: &str,
        args: &str,
    ) -> Result<Context<'b>, Error>
    where
        'a: 'b,
    {
        unsafe {
            let name = CString::new(name).unwrap();
            let args = CString::new(args).unwrap();
            let mut context = ptr::null_mut();

            match avfilter_graph_create_filter(
                &mut context as *mut *mut AVFilterContext,
                filter.as_ptr(),
                name.as_ptr(),
                args.as_ptr(),
                ptr::null_mut(),
                self.as_mut_ptr(),
            ) {
                n if n >= 0 => Ok(Context::wrap(context)),
                e => Err(Error::from(e)),
            }
        }
    }

    pub fn get<'a, 'b>(&'b mut self, name: &str) -> Option<Context<'b>>
    where
        'a: 'b,
    {
        unsafe {
            let name = CString::new(name).unwrap();
            let ptr = avfilter_graph_get_filter(self.as_mut_ptr(), name.as_ptr());

            if ptr.is_null() {
                None
            } else {
                Some(Context::wrap(ptr))
            }
        }
    }

    pub fn dump(&self) -> String {
        unsafe {
            let ptr = avfilter_graph_dump(self.as_ptr() as *mut _, ptr::null());
            let cstr = from_utf8_unchecked(CStr::from_ptr(ptr).to_bytes());
            let string = cstr.to_owned();

            av_free(ptr as *mut _);

            string
        }
    }

    pub fn input(&mut self, name: &str, pad: usize) -> Result<Parser, Error> {
        Parser::new(self).input(name, pad)
    }

    pub fn output(&mut self, name: &str, pad: usize) -> Result<Parser, Error> {
        Parser::new(self).output(name, pad)
    }

    pub fn parse(&mut self, spec: &str) -> Result<(), Error> {
        Parser::new(self).parse(spec)
    }

    /// Links one filter in the graph with another. Filters are specified by name. If you want to
    /// link two [Contexts] together, see the [`crate::filters::context::Context::link`] method.
    ///
    /// [Contexts]: crate::filters::context::Context
    pub fn link(&mut self, from: &str, to: &str) -> Result<(), Error> {
        unsafe {
            let from_s = CString::new(from).unwrap();
            let ff_ptr = sys::avfilter_graph_get_filter(self.as_mut_ptr(), from_s.as_ptr());

            let ff = if ff_ptr.is_null() {
                return Err(Error::FilterNotFound);
            } else {
                ff_ptr
            };
            let to_s = CString::new(to).unwrap();
            let tf_ptr = sys::avfilter_graph_get_filter(self.as_mut_ptr(), to_s.as_ptr());

            let tf = if tf_ptr.is_null() {
                return Err(Error::FilterNotFound);
            } else {
                tf_ptr
            };
            match sys::avfilter_link(ff, 0, tf, 0) {
                s if s >= 0 => Ok(()),
                e => Err(Error::from(e)),
            }
        }
    }

    /// Links multiple filters in the graph together. The first filter will be link to the second
    /// one and then the second one to the third one, and so on.
    ///
    /// All filters must already exist within the graph and sshould be specified by name.
    pub fn chain_link<'a, N: AsRef<[&'a str]>>(&mut self, filters: N) -> Result<(), Error> {
        for (from, to) in filters.as_ref().iter().zip(filters.as_ref().iter().skip(1)) {
            self.link(from, to)?;
        }
        Ok(())
    }
}

impl Drop for Graph {
    fn drop(&mut self) {
        unsafe {
            avfilter_graph_free(&mut self.as_mut_ptr());
        }
    }
}

pub struct Parser<'a> {
    graph: &'a mut Graph,
    inputs: *mut AVFilterInOut,
    outputs: *mut AVFilterInOut,
}

impl<'a> Parser<'a> {
    pub fn new(graph: &mut Graph) -> Parser {
        Parser {
            graph,
            inputs: ptr::null_mut(),
            outputs: ptr::null_mut(),
        }
    }

    pub fn input(mut self, name: &str, pad: usize) -> Result<Self, Error> {
        unsafe {
            let mut context = self.graph.get(name).ok_or(Error::InvalidData)?;
            let input = avfilter_inout_alloc();

            if input.is_null() {
                panic!("out of memory");
            }

            let name = CString::new(name).unwrap();

            (*input).name = av_strdup(name.as_ptr());
            (*input).filter_ctx = context.as_mut_ptr();
            (*input).pad_idx = pad as c_int;
            (*input).next = ptr::null_mut();

            if self.inputs.is_null() {
                self.inputs = input;
            } else {
                (*self.inputs).next = input;
            }
        }

        Ok(self)
    }

    pub fn output(mut self, name: &str, pad: usize) -> Result<Self, Error> {
        unsafe {
            let mut context = self.graph.get(name).ok_or(Error::InvalidData)?;
            let output = avfilter_inout_alloc();

            if output.is_null() {
                panic!("out of memory");
            }

            let name = CString::new(name).unwrap();

            (*output).name = av_strdup(name.as_ptr());
            (*output).filter_ctx = context.as_mut_ptr();
            (*output).pad_idx = pad as c_int;
            (*output).next = ptr::null_mut();

            if self.outputs.is_null() {
                self.outputs = output;
            } else {
                (*self.outputs).next = output;
            }
        }

        Ok(self)
    }

    pub fn parse(mut self, spec: &str) -> Result<(), Error> {
        unsafe {
            let spec = CString::new(spec).unwrap();

            let result = avfilter_graph_parse_ptr(
                self.graph.as_mut_ptr(),
                spec.as_ptr(),
                &mut self.inputs,
                &mut self.outputs,
                ptr::null_mut(),
            );

            avfilter_inout_free(&mut self.inputs);
            avfilter_inout_free(&mut self.outputs);

            match result {
                n if n >= 0 => Ok(()),
                e => Err(Error::from(e)),
            }
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::find;

    fn make_graph() -> Graph {
        let mut graph = Graph::new();
        graph.add(
            &find("buffer").unwrap(), "in",
            "width=320:height=240:pix_fmt=yuv410p:time_base=1/24:sar=1").unwrap();
        graph.add(
            &find("scale").unwrap(), "scale", "w=50:h=50:eval=frame:flags=fast_bilinear").unwrap();
        graph.add(&find("buffersink").unwrap(), "out", "").unwrap();
        graph
    }

    #[test]
    fn test_link() {
        let mut graph = make_graph();
        assert!(matches!(graph.link("in", "scale"), Ok(_)));
    }

    #[test]
    fn test_chain_link() {
        let mut graph = make_graph();
        assert!(matches!(graph.chain_link(&["in", "scale", "out"]), Ok(_)));
    }
}
