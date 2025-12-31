use crate::Env;
use crate::ToJsUnknown;
use crate::platform::sys;
use crate::platform::sys::Value;
use crate::values::FromJsValue;
use crate::values::JsObjectValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Clone)]
pub struct JsArrayBuffer {
    pub(crate) value: Value,
    pub(crate) env: Env,
}

impl JsArrayBuffer {
    pub fn new(
        env: &Env,
        length: usize,
    ) -> crate::Result<Self> {
        let scope = &mut env.scope();
        let array_buffer = v8::ArrayBuffer::new(scope, length);
        Ok(Self {
            value: sys::Value::new(array_buffer.into()),
            env: env.clone(),
        })
    }

    pub fn from_bytes(
        env: &Env,
        data: &[u8],
    ) -> crate::Result<Self> {
        let scope = &mut env.scope();
        let array_buffer = v8::ArrayBuffer::new(scope, data.len());

        // Copy data into the ArrayBuffer
        let backing_store = array_buffer.get_backing_store();
        if let Some(buffer_data) = backing_store.data() {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    buffer_data.as_ptr() as *mut u8,
                    data.len(),
                );
            }
        }

        Ok(Self {
            value: sys::Value::new(array_buffer.into()),
            env: env.clone(),
        })
    }

    pub fn with_length(
        env: &Env,
        length: usize,
    ) -> crate::Result<Self> {
        let scope = &mut env.scope();
        let array_buffer = v8::ArrayBuffer::new(scope, length);
        Ok(Self {
            value: sys::Value::new(array_buffer.into()),
            env: env.clone(),
        })
    }

    pub fn byte_length(&self) -> crate::Result<usize> {
        let _scope = &mut self.env.scope();
        let Ok(array_buffer) = self.value.try_cast::<v8::ArrayBuffer>() else {
            return Err(crate::Error::ValueCastError);
        };
        Ok(array_buffer.byte_length())
    }

    pub fn is_detached(&self) -> crate::Result<bool> {
        let _scope = &mut self.env.scope();
        let Ok(array_buffer) = self.value.try_cast::<v8::ArrayBuffer>() else {
            return Err(crate::Error::ValueCastError);
        };
        Ok(array_buffer.was_detached())
    }

    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        let _scope = &mut self.env.scope();
        let Ok(array_buffer) = self.value.try_cast::<v8::ArrayBuffer>() else {
            return Err(crate::Error::ValueCastError);
        };

        if array_buffer.was_detached() {
            return Err(crate::Error::ValueCastError);
        }

        let byte_length = array_buffer.byte_length();
        let mut result = vec![0u8; byte_length];

        let backing_store = array_buffer.get_backing_store();
        if let Some(buffer_data) = backing_store.data()
            && byte_length > 0
        {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    buffer_data.as_ptr() as *const u8,
                    result.as_mut_ptr(),
                    byte_length,
                );
            }
        }

        Ok(result)
    }

    pub fn as_slice(&self) -> crate::Result<&[u8]> {
        let _scope = &mut self.env.scope();
        let Ok(array_buffer) = self.value.try_cast::<v8::ArrayBuffer>() else {
            return Err(crate::Error::ValueCastError);
        };

        if array_buffer.was_detached() {
            return Err(crate::Error::ValueCastError);
        }

        let byte_length = array_buffer.byte_length();

        let backing_store = array_buffer.get_backing_store();
        if let Some(buffer_data) = backing_store.data() {
            unsafe {
                return Ok(std::slice::from_raw_parts(
                    buffer_data.as_ptr() as *const u8,
                    byte_length,
                ));
            }
        }

        Ok(&[])
    }

    pub fn as_mut_slice(&mut self) -> crate::Result<&mut [u8]> {
        let _scope = &mut self.env.scope();
        let Ok(array_buffer) = self.value.try_cast::<v8::ArrayBuffer>() else {
            return Err(crate::Error::ValueCastError);
        };

        if array_buffer.was_detached() {
            return Err(crate::Error::ValueCastError);
        }

        let byte_length = array_buffer.byte_length();

        let backing_store = array_buffer.get_backing_store();
        if let Some(buffer_data) = backing_store.data() {
            unsafe {
                return Ok(std::slice::from_raw_parts_mut(
                    buffer_data.as_ptr() as *mut u8,
                    byte_length,
                ));
            }
        }

        Ok(&mut [])
    }
}

impl JsValue for JsArrayBuffer {
    fn value(&self) -> &Value {
        &self.value
    }

    fn env(&self) -> &Env {
        &self.env
    }
}

impl ToJsUnknown for JsArrayBuffer {}
impl JsObjectValue for JsArrayBuffer {}

impl FromJsValue for JsArrayBuffer {
    fn from_js_value(
        env: &Env,
        value: Value,
    ) -> crate::Result<Self> {
        Ok(Self {
            value,
            env: env.clone(),
        })
    }
}

impl ToJsValue for JsArrayBuffer {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(val.value)
    }
}

impl ToJsValue for Vec<u8> {
    fn to_js_value(
        env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(JsArrayBuffer::from_bytes(env, &val)?.value().clone())
    }
}

impl ToJsValue for &[u8] {
    fn to_js_value(
        env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(JsArrayBuffer::from_bytes(env, val)?.value().clone())
    }
}

impl Env {
    pub fn create_array_buffer(
        &self,
        length: usize,
    ) -> crate::Result<JsArrayBuffer> {
        JsArrayBuffer::new(self, length)
    }

    pub fn create_array_buffer_from_bytes(
        &self,
        data: &[u8],
    ) -> crate::Result<JsArrayBuffer> {
        JsArrayBuffer::from_bytes(self, data)
    }
}
