// TODO: Replace with JsFunction type

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// `v8::Weak` values must outlive the scope that they are created in for the finalizer callback to run.
// To ensure the Weak values outlive their scope, they are stored in a thread local HashMap where
// the reference count informs if a value should be removed from the map.
// The reference count looks at both calls to Rust's drop() and hooks into v8's GC
thread_local! {
    static EXTERNAL_CACHE: RefCell<HashMap<*mut std::ffi::c_void, Rc<v8::Weak<v8::Function>>>> = Default::default();
}

fn cache_insert(
    key: *mut std::ffi::c_void,
    entry: Rc<v8::Weak<v8::Function>>,
) {
    EXTERNAL_CACHE.with(|hm| hm.borrow_mut().insert(key, entry));
}

fn cache_remove(key: *mut std::ffi::c_void) -> bool {
    EXTERNAL_CACHE
        .with(|hm| hm.borrow_mut().remove(&key))
        .is_some()
}

type DynV8Callback =
    Box<dyn 'static + Fn(&mut v8::HandleScope, v8::FunctionCallbackArguments, v8::ReturnValue)>;

pub fn v8_create_function_from_closure<'a>(
    scope: &mut v8::HandleScope<'a>,
    closure: impl 'static + Fn(&mut v8::HandleScope, v8::FunctionCallbackArguments, v8::ReturnValue),
) -> Rc<v8::Weak<v8::Function>> {
    let callback = Box::<DynV8Callback>::new(Box::new(closure));
    // SAFETY: The supplied closure is owned by the current function and the value
    // is "moved" into the v8 function. However, v8 does not support Rust "move" semantics
    // and requires passing values in via the "data" property. This requires leaking the supplied
    // closure and dropping is up manually when v8 eventually GCs the function.
    let callback_ptr = Box::into_raw(callback) as *mut std::ffi::c_void;
    let js_external = v8::External::new(scope, callback_ptr);

    let handle = v8::Function::builder(
        |scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, rv: v8::ReturnValue| {
            let js_data = args.data();
            let js_external = js_data.try_cast::<v8::External>().unwrap();
            let external_ptr = js_external.value() as *const DynV8Callback;
            let callback: &DynV8Callback = unsafe { &*external_ptr };
            callback(scope, args, rv)
        },
    )
    .data(js_external.into())
    .build(scope)
    .unwrap();

    // SAFETY: This callback will fire when the v8 function is dropped by the
    // JS runtime and allows manually dropping the associated supplied externals
    let weak = Rc::new(v8::Weak::with_guaranteed_finalizer(
        scope,
        handle,
        Box::new(move || {
            cache_remove(callback_ptr);
            drop(unsafe { Box::from_raw(callback_ptr as *mut DynV8Callback) });
        }),
    ));

    cache_insert(callback_ptr, Rc::clone(&weak));
    weak
}
