#![allow(non_camel_case_types)]
use std::ffi::c_void;
use std::ops::Deref;

#[derive(Debug)]
pub struct Value(*mut c_void);

impl Value {
    pub fn inner(&self) -> v8::Local<'static, v8::Value> {
        unsafe { *(self.0 as *mut v8::Local<'static, v8::Value>) }
    }

    pub fn address(&self) -> usize {
        self.0 as usize
    }
}

impl From<v8::Local<'_, v8::Value>> for Value {
    fn from(value: v8::Local<'_, v8::Value>) -> Self {
        Self(Box::into_raw(Box::new(value)) as _)
    }
}

impl Deref for Value {
    type Target = v8::Local<'static, v8::Value>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.0 as *mut v8::Local<'static, v8::Value>) }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.0 as *mut v8::Local<'static, v8::Value>) })
    }
}
