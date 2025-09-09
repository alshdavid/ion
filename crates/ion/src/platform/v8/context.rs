use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;

use crate::platform::v8::RawIsolate;

#[derive(Debug)]
pub struct RawContext(*mut v8::Local<'static, v8::Context>);

impl RawContext {
    pub fn new(
        isolate: &RawIsolate,
        scope: &mut v8::HandleScope<'_, ()>,
    ) -> Rc<Self> {
        // Note: [`v8::Global::into_raw`] appears to have a memory leak
        let context_local = v8::Context::new(scope, Default::default());
        let context_global = v8::Global::new(isolate.as_mut(), context_local);
        let context = Box::into_raw(Box::new(context_global));
        Rc::new(Self(context as *mut v8::Local<'static, v8::Context>))
    }

    pub fn as_inner(&self) -> v8::Local<'static, v8::Context> {
        unsafe { *self.0 }
    }

    pub fn address(&self) -> usize {
        self.0 as usize
    }
}

impl Deref for RawContext {
    type Target = v8::Local<'static, v8::Context>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for RawContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

impl Drop for RawContext {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.0 as *mut v8::Global<v8::Context>) })
    }
}
