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

        let ref_count = RefCounter::new(1);

        // Store both the data pointer AND the RefCounter in the V8 External
        let external_data = Box::into_raw(Box::new((ptr, ref_count.clone())));
        let value = v8::External::new(scope, external_data as _);

        env.finalizer_registry.register(&value.into(), {
            let ref_count = ref_count.clone();
            move || {
                if ref_count.dec() {
                    // Clean up both the data and the external_data tuple
                    drop(unsafe { Box::from_raw(ptr as *mut T) });
                    drop(unsafe { Box::from_raw(external_data) });
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
        let data = unsafe { &*(self.ptr as *mut T) };
        Ok(data)
    }
}

impl<T> Clone for JsExternal<T> {
    fn clone(&self) -> Self {
        self.ref_count.inc();
        Self {
            value: self.value,
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
        let external_data_ptr = external.value() as *const (*mut c_void, RefCounter);
        let (ptr, ref_count) = unsafe { &*external_data_ptr };

        // Increment the original RefCounter instead of creating a new one
        ref_count.inc();

        Ok(Self {
            value,
            env: env.clone(),
            ptr: *ptr,
            ref_count: ref_count.clone(),
            _data: Default::default(),
        })
    }
}

impl<T> ToJsValue for JsExternal<T> {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(val.value)
    }
}
