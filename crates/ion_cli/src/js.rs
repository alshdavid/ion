use std::{cell::RefCell, marker::PhantomData, ptr::NonNull, rc::Rc};

pub struct JsFunction {

}

type DynV8Callback =
    Box<dyn 'static + Fn(&mut v8::HandleScope, v8::FunctionCallbackArguments, v8::ReturnValue)>;

pub fn v8_create_function_from_closure<'a>(
    isolate_ptr: *mut v8::Isolate,
    context: NonNull<v8::Context>,
    closure: impl 'static + Fn(&mut v8::HandleScope, v8::FunctionCallbackArguments, v8::ReturnValue),
) -> v8::Global<v8::Function> {
    todo!()
    //     let weak_handle = {
    //         let scope = &mut v8_global_scope(isolate_ptr, context);

    //         let callback = Box::<DynV8Callback>::new(Box::new(closure));
    //         let callback_ptr = Box::into_raw(callback);

    //         let js_external = v8::External::new(scope, callback_ptr as *mut std::ffi::c_void);
    //         let value = v8::Global::new(scope, js_external);

    //         let weak = v8::Weak::with_guaranteed_finalizer(
    //             scope, //v8_isolate_ptr(isolate_ptr),
    //             value,
    //             Box::new(|| println!("Called: with_guaranteed_finalizer")),
    //         );

    //         // weak.clone_with_finalizer(finalizer)
    //         // value
    //     };

    //     {
    //         let scope = &mut v8_global_scope(isolate_ptr, context);
    //         let value_l = v8::Local::new(scope, weak_handle);

    //         let handle = v8::Function::builder_raw(v8_create_function_from_closure_trampoline)
    //             .data(value_l.into())
    //             .build(scope)
    //             .unwrap();

    //         v8::Global::new(scope, handle)
    //     }
    // }

    // extern "C" fn v8_create_function_from_closure_trampoline(info: *const v8::FunctionCallbackInfo) {
    //     let callback_info = unsafe { &*info };
    //     let args = v8::FunctionCallbackArguments::from_function_callback_info(callback_info);
    //     let rv = v8::ReturnValue::from_function_callback_info(callback_info);
    //     let scope = unsafe { &mut v8::CallbackScope::new(callback_info) };

    //     // SAFETY: create_function guarantees that the data is a CallbackInfo external.
    //     let info_ptr: &mut DynV8Callback = unsafe {
    //         let js_data = args.data();
    //         let external_value = v8::Local::<v8::External>::cast_unchecked(js_data);
    //         let info_ptr = external_value.value() as *mut DynV8Callback;
    //         &mut *info_ptr
    //     };

    //     // SAFETY: pointer from Box::into_raw.
    //     // let info = unsafe { &mut *info_ptr };
    //     info_ptr(scope, args, rv);
}

// extern "C" fn v8_create_function_from_closure_trampoline(info: *const v8::FunctionCallbackInfo) {
//     let callback_info = unsafe { &*info };
//     let args = v8::FunctionCallbackArguments::from_function_callback_info(callback_info);
//     let rv = v8::ReturnValue::from_function_callback_info(callback_info);
//     let scope = unsafe { &mut v8::CallbackScope::new(callback_info) };

//     // SAFETY: create_function guarantees that the data is a CallbackInfo external.
//     let js_data = args.data();
//     dbg!(&js_data.is_external());
//     let js_external = js_data.try_cast::<v8::External>().unwrap();
//     let external_ptr = js_external.value() as *const DynV8Callback;
//     let callback: &DynV8Callback = unsafe { &*external_ptr };

//     // SAFETY: pointer from Box::into_raw.
//     callback(scope, args, rv);
// }
