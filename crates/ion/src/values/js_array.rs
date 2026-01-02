use crate::Env;
use crate::ToJsUnknown;
use crate::platform::sys;
use crate::platform::sys::Value;
use crate::values::FromJsValue;
use crate::values::JsObjectValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Clone)]
pub struct JsArray {
    pub(crate) value: Value,
    pub(crate) env: Env,
}

impl JsArray {
    pub fn new(env: &Env) -> crate::Result<Self> {
        let scope = &mut env.scope();
        let array = v8::Array::new(scope, 0);
        Ok(Self {
            value: sys::Value::new(array.into()),
            env: env.clone(),
        })
    }

    pub fn with_length(
        env: &Env,
        length: i32,
    ) -> crate::Result<Self> {
        let scope = &mut env.scope();
        let array = v8::Array::new(scope, length);
        Ok(Self {
            value: sys::Value::new(array.into()),
            env: env.clone(),
        })
    }

    pub fn from_vec<T>(
        env: &Env,
        vec: Vec<T>,
    ) -> crate::Result<Self>
    where
        T: ToJsValue,
    {
        let scope = &mut env.scope();
        let array = v8::Array::new(scope, vec.len() as i32);

        for (index, item) in vec.into_iter().enumerate() {
            let js_value = T::to_js_value(env, item)?;
            let v8_value = sys::v8_into_static_value::<v8::Value, v8::Value>(js_value.as_inner());
            array.set_index(scope, index as u32, v8_value);
        }

        Ok(Self {
            value: sys::Value::new(array.into()),
            env: env.clone(),
        })
    }

    pub fn length(&self) -> crate::Result<u32> {
        let _scope = &mut self.env.scope();
        let Ok(array) = self.value.try_cast::<v8::Array>() else {
            return Err(crate::Error::ValueCastError);
        };
        Ok(array.length())
    }

    pub fn get<I>(
        &self,
        index: I,
    ) -> crate::Result<Value>
    where
        I: TryInto<u32>,
        I::Error: std::fmt::Debug,
    {
        let context = self.env.context.as_local();
        v8::callback_scope!(unsafe scope, context);

        let Ok(array) = self.value.try_cast::<v8::Array>() else {
            return Err(crate::Error::ValueCastError);
        };

        let index_u32 = index.try_into().map_err(|_| crate::Error::ValueCastError)?;
        if let Some(value) = array.get_index(scope, index_u32) {
            Ok(sys::Value::new(value))
        } else {
            // Return undefined for out-of-bounds access
            let undefined = v8::undefined(scope);
            Ok(sys::Value::new(undefined.into()))
        }
    }

    pub fn set<I, T>(
        &self,
        index: I,
        value: T,
    ) -> crate::Result<()>
    where
        I: TryInto<u32>,
        I::Error: std::fmt::Debug,
        T: ToJsValue,
    {
        let scope = &mut self.env.scope();
        let Ok(array) = self.value.try_cast::<v8::Array>() else {
            return Err(crate::Error::ValueCastError);
        };

        let index_u32 = index.try_into().map_err(|_| crate::Error::ValueCastError)?;
        let js_value = T::to_js_value(&self.env, value)?;
        let v8_value = sys::v8_into_static_value::<v8::Value, v8::Value>(js_value.as_inner());
        array.set_index(scope, index_u32, v8_value);
        Ok(())
    }

    pub fn push<T>(
        &self,
        value: T,
    ) -> crate::Result<u32>
    where
        T: ToJsValue,
    {
        let current_length = self.length()?;
        self.set(current_length, value)?;
        Ok(current_length + 1)
    }

    pub fn to_vec<T>(&self) -> crate::Result<Vec<T>>
    where
        T: FromJsValue,
    {
        let length = self.length()?;
        let mut vec = Vec::with_capacity(length as usize);

        for i in 0..length {
            let value = self.get(i)?;
            let item = T::from_js_value(&self.env, value)?;
            vec.push(item);
        }

        Ok(vec)
    }
}

impl JsValue for JsArray {
    fn value(&self) -> &Value {
        &self.value
    }

    fn env(&self) -> &Env {
        &self.env
    }
}

impl ToJsUnknown for JsArray {}
impl JsObjectValue for JsArray {}

impl FromJsValue for JsArray {
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

impl ToJsValue for JsArray {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(val.value)
    }
}

impl<T> ToJsValue for Vec<T>
where
    T: ToJsValue,
{
    fn to_js_value(
        env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(JsArray::from_vec(env, val)?.value().clone())
    }
}

impl Env {
    pub fn create_array(&self) -> crate::Result<JsArray> {
        JsArray::new(self)
    }

    pub fn create_array_with_length(
        &self,
        length: i32,
    ) -> crate::Result<JsArray> {
        JsArray::with_length(self, length)
    }

    pub fn create_array_from_vec<T>(
        &self,
        vec: Vec<T>,
    ) -> crate::Result<JsArray>
    where
        T: ToJsValue,
    {
        JsArray::from_vec(self, vec)
    }
}
