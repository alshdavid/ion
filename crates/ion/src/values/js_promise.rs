use std::cell::RefCell;
use std::rc::Rc;

use crate::Env;
use crate::JsFunction;
use crate::JsUnknown;
use crate::ToJsUnknown;
use crate::platform::sys;
use crate::platform::sys::Value;
use crate::utils::v8::v8_create_string;
use crate::values::FromJsValue;
use crate::values::JsValue;
use crate::values::ToJsValue;

#[derive(Clone)]
pub struct JsPromise {
    pub(crate) value: Value,
    pub(crate) env: Env,
}

impl JsPromise {
    pub fn new(env: &Env) -> crate::Result<JsPromise> {
        let scope = &mut env.scope();
        let object = v8::Object::new(scope);
        Ok(Self {
            value: sys::v8_from_value(object),
            env: env.clone(),
        })
    }

    /// Non blocking call to then/catch
    pub fn settled<Resolved: FromJsValue + 'static>(
        self,
        settled_callback: impl 'static
        + Send
        + Sync
        + FnOnce(&Env, JsPromiseResult<Resolved>) -> crate::Result<()>,
    ) -> crate::Result<()> {
        // RefCell<Option> ensures callback is only called once
        #[allow(clippy::type_complexity)]
        let settled_callback: Rc<
            RefCell<
                Option<
                    Box<
                        dyn 'static
                            + Send
                            + Sync
                            + FnOnce(&Env, JsPromiseResult<Resolved>) -> crate::Result<()>,
                    >,
                >,
            >,
        > = Rc::new(RefCell::new(Some(Box::new(settled_callback))));

        let scope = &mut self.env.scope();

        // Promise
        let promise = sys::v8_value_cast::<v8::Object, _>(self.value);

        // Promise.then
        let then_key = v8_create_string(scope, "then")?;
        let Some(then) = promise.get(scope, sys::v8_value_cast::<v8::Value, _>(then_key)) else {
            return Err(crate::Error::ValueGetError);
        };
        let then_fn = sys::v8_value_cast::<v8::Function, _>(then);

        let then_fn_recv = JsFunction::new(&self.env, {
            let settled_callback = Rc::clone(&settled_callback);
            move |env, ctx| {
                let settled_callback = {
                    let mut settled_callback = settled_callback.borrow_mut();
                    settled_callback.take().unwrap()
                };
                let result = ctx.arg::<JsUnknown>(0)?;
                let result = Resolved::from_js_value(env, *result.value())?;
                let result = JsPromiseResult::<Resolved>::Resolved(result);
                settled_callback(env, result)
            }
        })?;

        // Promise.catch
        let catch_key = v8_create_string(scope, "catch")?;
        let Some(catch) = promise.get(scope, sys::v8_value_cast::<v8::Value, _>(catch_key)) else {
            return Err(crate::Error::ValueGetError);
        };
        let catch_fn = sys::v8_value_cast::<v8::Function, _>(catch);

        let catch_fn_recv = JsFunction::new(&self.env, move |env, ctx| {
            let settled_callback = {
                let mut settled_callback = settled_callback.borrow_mut();
                settled_callback.take().unwrap()
            };
            let result = ctx.arg::<JsUnknown>(0)?;
            let result = JsPromiseResult::Rejected(result);
            settled_callback(env, result)
        })?;

        // Call Promise.then & Promise.catch
        if then_fn
            .call(scope, promise.into(), &[then_fn_recv.value])
            .is_none()
        {
            return Err(crate::Error::FunctionCallError);
        };
        if catch_fn
            .call(scope, promise.into(), &[catch_fn_recv.value])
            .is_none()
        {
            return Err(crate::Error::FunctionCallError);
        };

        Ok(())
    }
}

pub enum JsPromiseResult<Result: FromJsValue> {
    Resolved(Result),
    Rejected(JsUnknown),
}

impl JsValue for JsPromise {
    fn value(&self) -> &Value {
        &self.value
    }

    fn env(&self) -> &Env {
        &self.env
    }
}

impl ToJsUnknown for JsPromise {}

impl FromJsValue for JsPromise {
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

impl ToJsValue for JsPromise {
    fn to_js_value(
        _env: &Env,
        val: Self,
    ) -> crate::Result<Value> {
        Ok(val.value)
    }
}
