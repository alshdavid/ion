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
    let callback = Box::<DynV8CallbackWrapper>::new(DynV8CallbackWrapper(Box::new(closure)));
    let callback_ptr = Box::into_raw(callback);
    let js_external = v8::External::new(scope, callback_ptr as *mut std::ffi::c_void);
    

    v8::Function::builder(
        |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, rv: v8::ReturnValue| {
            let js_data = args.data();
            let js_external = js_data.try_cast::<v8::External>().unwrap();
            let external_ptr = js_external.value() as *const DynV8CallbackWrapper;
            let callback: &DynV8CallbackWrapper = unsafe { &*external_ptr };
            callback.0(scope, args, rv)
        },
    )
    .data(js_external.into())
    .build(scope)
    .unwrap()

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
    object.get(
        scope,
        key.into(),
    )
}