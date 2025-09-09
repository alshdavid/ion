use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;

#[derive(Debug)]
pub struct RawContextScope(*mut v8::ContextScope<'static, v8::HandleScope<'static>>);

impl RawContextScope {
    pub fn new(scope: v8::ContextScope<'_, v8::HandleScope<'_>>) -> Rc<Self> {
        Rc::new(Self(Box::into_raw(Box::new(scope)) as _))
    }

    pub fn as_mut(&self) -> &mut v8::ContextScope<'static, v8::HandleScope<'static>> {
        unsafe { &mut *self.0 }
    }
}

impl Deref for RawContextScope {
    type Target = v8::ContextScope<'static, v8::HandleScope<'static>>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for RawContextScope {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

impl Drop for RawContextScope {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.0) })
    }
}
