use crate::Env;
use crate::ToJsValue;
use crate::platform::sys::Value;

pub trait JsValuesTupleIntoVec {
    fn into_vec(
        self,
        env: &Env,
    ) -> crate::Result<Vec<Value>>;
}

impl<T> JsValuesTupleIntoVec for T
where
    T: ToJsValue,
{
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn into_vec(
        self,
        env: &Env,
    ) -> crate::Result<Vec<Value>> {
        // allow call function with `()` and function's arguments should be empty array
        if std::mem::size_of::<T>() == 0 {
            Ok(vec![])
        } else {
            Ok(vec![{ <T as ToJsValue>::to_js_value(env, self)? }])
        }
    }
}

pub trait TupleFromSliceValues {
    #[allow(clippy::missing_safety_doc)]
    unsafe fn from_slice_values(
        env: &Env,
        values: &[Value],
    ) -> crate::Result<Self>
    where
        Self: Sized;
}

pub struct FnArgs<T> {
    pub data: T,
}

impl<T> From<T> for FnArgs<T> {
    fn from(value: T) -> Self {
        FnArgs { data: value }
    }
}

// TODO Use a macro to generate these
// impl JsValuesTupleIntoVec for () {
//     fn into_vec(self, env: &Env) -> crate::Result<Vec<Value>> {
//         Ok(vec![
//         ])
//     }
// }

// impl<A: ToJsValue> JsValuesTupleIntoVec for A {
//     fn into_vec(self, env: &Env) -> crate::Result<Vec<Value>> {
//         Ok(vec![
//             A::to_js_value(env, self)?,
//         ])
//     }
// }

impl<A: ToJsValue, B: ToJsValue> JsValuesTupleIntoVec for (A, B) {
    fn into_vec(
        self,
        env: &Env,
    ) -> crate::Result<Vec<Value>> {
        Ok(vec![
            A::to_js_value(env, self.0)?,
            B::to_js_value(env, self.1)?,
        ])
    }
}

impl<A: ToJsValue, B: ToJsValue, C: ToJsValue> JsValuesTupleIntoVec for (A, B, C) {
    fn into_vec(
        self,
        env: &Env,
    ) -> crate::Result<Vec<Value>> {
        Ok(vec![
            A::to_js_value(env, self.0)?,
            B::to_js_value(env, self.1)?,
            C::to_js_value(env, self.2)?,
        ])
    }
}

impl<A: ToJsValue, B: ToJsValue, C: ToJsValue, D: ToJsValue> JsValuesTupleIntoVec for (A, B, C, D) {
    fn into_vec(
        self,
        env: &Env,
    ) -> crate::Result<Vec<Value>> {
        Ok(vec![
            A::to_js_value(env, self.0)?,
            B::to_js_value(env, self.1)?,
            C::to_js_value(env, self.2)?,
            D::to_js_value(env, self.3)?,
        ])
    }
}

impl<A: ToJsValue, B: ToJsValue, C: ToJsValue, D: ToJsValue, E: ToJsValue> JsValuesTupleIntoVec
    for (A, B, C, D, E)
{
    fn into_vec(
        self,
        env: &Env,
    ) -> crate::Result<Vec<Value>> {
        Ok(vec![
            A::to_js_value(env, self.0)?,
            B::to_js_value(env, self.1)?,
            C::to_js_value(env, self.2)?,
            D::to_js_value(env, self.3)?,
            E::to_js_value(env, self.4)?,
        ])
    }
}

impl<A: ToJsValue, B: ToJsValue, C: ToJsValue, D: ToJsValue, E: ToJsValue, F: ToJsValue>
    JsValuesTupleIntoVec for (A, B, C, D, E, F)
{
    fn into_vec(
        self,
        env: &Env,
    ) -> crate::Result<Vec<Value>> {
        Ok(vec![
            A::to_js_value(env, self.0)?,
            B::to_js_value(env, self.1)?,
            C::to_js_value(env, self.2)?,
            D::to_js_value(env, self.3)?,
            E::to_js_value(env, self.4)?,
            F::to_js_value(env, self.5)?,
        ])
    }
}
