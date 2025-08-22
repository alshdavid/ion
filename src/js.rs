use std::{cell::RefCell, rc::Rc, sync::OnceLock};

thread_local! {
  static ISOLATE: OnceLock<Rc<RefCell<v8::OwnedIsolate>>> = OnceLock::new();
}

pub fn get_isolate() -> Rc<RefCell<v8::OwnedIsolate>> {
    let i = ISOLATE.with(|v| {
        let i = v.get_or_init(|| {
            let v = v8::Isolate::new(v8::CreateParams::default());
            Rc::new(RefCell::new(v))
        });
        Rc::clone(i)
    });
    i
}

type DynV8Callback =
    Box<dyn 'static + Fn(&mut v8::HandleScope, v8::FunctionCallbackArguments, v8::ReturnValue)>;

struct DynV8CallbackWrapper(DynV8Callback);

impl Drop for DynV8CallbackWrapper {
    fn drop(&mut self) {
        println!("Dropping callback")
    }
}

pub fn v8_create_function_from_closure<'a>(
    scope: &mut v8::ContextScope<'a, v8::HandleScope>,
    closure: impl 'static + Fn(&mut v8::HandleScope, v8::FunctionCallbackArguments, v8::ReturnValue),
) -> v8::Local<'a, v8::Function> {
    println!("define fn");
    // let iso = scope.get
    let callback = Box::<DynV8CallbackWrapper>::new(DynV8CallbackWrapper(Box::new(closure)));
    let callback_ptr = Box::into_raw(callback);
    let js_external = v8::External::new(scope, callback_ptr as *mut std::ffi::c_void);

    let weak = v8::Weak::with_guaranteed_finalizer(
        scope,
        js_external,
        Box::new(|| println!("Called: with_guaranteed_finalizer")),
    );

    v8::Function::builder_raw(v8_create_function_from_closure_trampoline)
        .data(weak.to_local(scope).unwrap().into())
        .build(scope)
        .unwrap()
}

extern "C" fn v8_create_function_from_closure_trampoline(info: *const v8::FunctionCallbackInfo) {
    let callback_info = unsafe { &*info };
    let args = v8::FunctionCallbackArguments::from_function_callback_info(callback_info);
    let rv = v8::ReturnValue::from_function_callback_info(callback_info);
    let scope = unsafe { &mut v8::CallbackScope::new(callback_info) };

    // SAFETY: create_function guarantees that the data is a CallbackInfo external.
    let info_ptr: *mut DynV8CallbackWrapper = unsafe {
        let js_data = args.data();
        let external_value: v8::Local<'_, v8::External> =
            v8::Local::<v8::External>::cast_unchecked(js_data);

        external_value.value() as _
    };

    // SAFETY: pointer from Box::into_raw.
    let info = unsafe { &mut *info_ptr };
    info.0(scope, args, rv);
}

pub fn v8_set_obj_property<'a, S: AsRef<str>>(
    scope: &mut v8::ContextScope<'a, v8::HandleScope>,
    object: &v8::Local<'a, v8::Object>,
    key: S,
    value: v8::Local<'a, v8::Value>,
) {
    let key = v8::String::new(scope, key.as_ref()).unwrap();
    object.define_property(
        scope,
        key.into(),
        &v8::PropertyDescriptor::new_from_value(value),
    );
}

pub fn v8_get_obj_property<'a, S: AsRef<str>>(
    scope: &mut v8::ContextScope<'a, v8::HandleScope>,
    object: &v8::Local<'a, v8::Object>,
    key: S,
) -> Option<v8::Local<'a, v8::Value>> {
    let key = v8::String::new(scope, key.as_ref()).unwrap();
    object.get(scope, key.into())
}

pub fn v8_delete_obj_property<'a, S: AsRef<str>>(
    scope: &mut v8::ContextScope<'a, v8::HandleScope>,
    object: &v8::Local<'a, v8::Object>,
    key: S,
) {
    let key = v8::String::new(scope, key.as_ref()).unwrap();
    object.delete(scope, key.into());
}
