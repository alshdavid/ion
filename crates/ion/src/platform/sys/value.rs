use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;

pub(super) struct RawValueInner(*mut v8::Local<'static, v8::Value>);

impl Drop for RawValueInner {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.0));
        };
    }
}

pub struct Value(pub(super) Rc<RawValueInner>);

impl Clone for Value {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl Value {
    pub fn new<'a>(value: v8::Local<'a, v8::Value>) -> Self {
        let value = Box::new(value.into());
        let value_ptr = Box::into_raw(value) as *mut v8::Local<'static, v8::Value>;
        Self(Rc::new(RawValueInner(value_ptr)))
    }

    pub fn as_inner(&self) -> v8::Local<'static, v8::Value> {
        unsafe { *(self.0.0) }
    }
}

impl Deref for Value {
    type Target = v8::Local<'static, v8::Value>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.0 }
    }
}

impl DerefMut for Value {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.0 }
    }
}

pub fn v8_into_static_value<'a, V, T>(value: v8::Local<'a, T>) -> v8::Local<'static, V> {
    unsafe { std::mem::transmute(value) }
}

pub fn v8_value_cast<'a, V, T>(value: v8::Local<'a, T>) -> v8::Local<'static, V> {
    unsafe { std::mem::transmute(value) }
}
