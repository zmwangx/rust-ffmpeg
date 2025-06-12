use std::fmt;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};
use std::ptr;

use super::mutable;
use ffi::*;

pub struct Owned<'a> {
    inner: mutable::Ref<'a>,
}

impl Default for Owned<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Owned<'_> {
    pub unsafe fn own(ptr: *mut AVDictionary) -> Self {
        Owned {
            inner: mutable::Ref::wrap(ptr),
        }
    }

    pub unsafe fn disown(mut self) -> *mut AVDictionary {
        let result = self.inner.as_mut_ptr();
        self.inner = mutable::Ref::wrap(ptr::null_mut());

        result
    }
}

impl Owned<'_> {
    pub fn new() -> Self {
        unsafe {
            Owned {
                inner: mutable::Ref::wrap(ptr::null_mut()),
            }
        }
    }
}

impl<'b> FromIterator<(&'b str, &'b str)> for Owned<'_> {
    fn from_iter<T: IntoIterator<Item = (&'b str, &'b str)>>(iterator: T) -> Self {
        let mut result = Owned::new();

        for (key, value) in iterator {
            result.set(key, value);
        }

        result
    }
}

impl<'b> FromIterator<&'b (&'b str, &'b str)> for Owned<'_> {
    fn from_iter<T: IntoIterator<Item = &'b (&'b str, &'b str)>>(iterator: T) -> Self {
        let mut result = Owned::new();

        for &(key, value) in iterator {
            result.set(key, value);
        }

        result
    }
}

impl FromIterator<(String, String)> for Owned<'_> {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iterator: T) -> Self {
        let mut result = Owned::new();

        for (key, value) in iterator {
            result.set(&key, &value);
        }

        result
    }
}

impl<'b> FromIterator<&'b (String, String)> for Owned<'_> {
    fn from_iter<T: IntoIterator<Item = &'b (String, String)>>(iterator: T) -> Self {
        let mut result = Owned::new();

        for (key, value) in iterator {
            result.set(key, value);
        }

        result
    }
}

impl<'a> Deref for Owned<'a> {
    type Target = mutable::Ref<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// Remove explicit lifetime in DerefMut
impl DerefMut for Owned<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// Remove explicit lifetime in Clone
impl Clone for Owned<'_> {
    fn clone(&self) -> Self {
        let mut dictionary = Owned::new();
        dictionary.clone_from(self);

        dictionary
    }

    fn clone_from(&mut self, source: &Self) {
        unsafe {
            let mut ptr = self.as_mut_ptr();
            av_dict_copy(&mut ptr, source.as_ptr(), 0);
            self.inner = mutable::Ref::wrap(ptr);
        }
    }
}

// Remove explicit lifetime in Drop
impl Drop for Owned<'_> {
    fn drop(&mut self) {
        unsafe {
            av_dict_free(&mut self.inner.as_mut_ptr());
        }
    }
}

// Remove explicit lifetime in Debug
impl fmt::Debug for Owned<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(fmt)
    }
}

// Add missing methods for Owned
impl Owned<'_> {
    pub fn set(&mut self, key: &str, value: &str) {
        // ...implementation to set key/value in self.inner...
        self.inner.set(key, value);
    }
    pub fn as_ptr(&self) -> *const AVDictionary {
        unsafe { self.inner.as_ptr() }
    }
    pub fn as_mut_ptr(&mut self) -> *mut AVDictionary {
        unsafe { self.inner.as_mut_ptr() }
    }
}
