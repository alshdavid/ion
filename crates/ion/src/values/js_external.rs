use std::ffi::c_void;
use std::marker::PhantomData;

use crate::Env;
use crate::ToJsUnknown;
use crate::platform::sys;
use crate::platform::sys::Value;
use crate::utils::RefCounter;
use crate::values::FromJsValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

pub struct JsExternal<T> {
    pub(crate) value: Value,
    pub(crate) env: Env,
    ptr: *mut c_void,
    ref_count: RefCounter,
    _data: PhantomData<T>,
}

impl<T> JsExternal<T> {
    pub fn new(
        env: &Env,
        data: T,
    ) -> crate::Result<Self> {
        let ptr = Box::into_raw(Box::new(data)) as *mut c_void;
        let scope = &mut env.scope();

        let ref_count = RefCounter::new(2);

        let value = v8::External::new(scope, ptr as _);
        env.finalizer_registry.register(&value.into(), {
            let ref_count = ref_count.clone();
            move || {
                if ref_count.dec() {
                    drop(unsafe { Box::from_raw(ptr as *mut T) });
                }
            }
        });

        Ok(Self {
            value: sys::v8_from_value(value),
            env: env.clone(),
            ptr,
            ref_count,
            _data: Default::default(),
        })
    }

    pub fn as_inner(&self) -> crate::Result<&T> {
        let value = self.value.cast::<v8::External>();
        let ptr = value.value();
        let data = unsafe { &*(ptr as *mut T) };
        Ok(data)
    }
}

impl<T> Clone for JsExternal<T> {
    fn clone(&self) -> Self {
        self.ref_count.inc();
        println!("cloned Rust {}", self.ref_count.count() - 1);
        Self {
            value: self.value.clone(),
            env: self.env.clone(),
            ptr: self.ptr,
            ref_count: self.ref_count.clone(),
            _data: self._data,
        }
    }
}

impl<T> Drop for JsExternal<T> {
    fn drop(&mut self) {
        if self.ref_count.dec() {
            drop(unsafe { Box::from_raw(self.ptr as *mut T) });
        }
    }
}

impl<T> JsValue for JsExternal<T> {
    fn value(&self) -> &Value {
        &self.value
    }

    fn env(&self) -> &Env {
        &self.env
    }
}

impl<T> ToJsUnknown for JsExternal<T> {}

impl<T> FromJsValue for JsExternal<T> {
    fn from_js_value(
        env: &Env,
        value: Value,
    ) -> crate::Result<Self> {
        let external = value.cast::<v8::External>();
        let ptr = external.value();
        Ok(Self {
            value,
            env: env.clone(),
            ptr,
            ref_count: Default::default(),
            _data: Default::default(),
        })
    }
}

impl<T> ToJsValue for JsExternal<T> {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(val.value.clone())
    }
}
