pub(crate) mod js;
pub(crate) mod utils;

use std::{collections::HashMap, ptr::NonNull, rc::Rc, time::Duration};

use crate::utils::{Bytes, DropDetector, MemoryStats, v8_eval_code};

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

static BYTES: &[u8] = include_bytes!("../../../Cargo.lock");

fn main() -> anyhow::Result<()> {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::set_flags_from_string(
        "--no_freeze_flags_after_init --expose_gc --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls --js-source-phase-imports",
    );
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    // main2()?;
    // println!("Next {})", MemoryStats::current());

    let mut prev = MemoryStats::current();
    for i in 0..100 {
        for _ in 0..10000 {
            main2()?;
        }
        let res = MemoryStats::current();
        let diff = res.resident.0 - prev.resident.0;
        prev = res;
        println!("Next {}, {} ({})", i, prev, Bytes(diff));
        // std::thread::sleep(Duration::from_secs(1));
    }

    // std::thread::sleep(Duration::from_secs(60));
    Ok(())
}

static CODE: &str = r#"
    "Hello World"
"#;

fn main2() -> anyhow::Result<()> {
    let mut isolate = v8::Isolate::new(v8::CreateParams::default());

    let handle_scope = &mut v8::HandleScope::new(&mut isolate);

    let context = v8::Context::new(handle_scope, Default::default());
    let context_scope = &mut v8::ContextScope::new(handle_scope, context);
    let context_scope_ptr = Box::into_raw(Box::new(context_scope));

    let callback = {
        let scope = &mut v8::HandleScope::new(unsafe { *context_scope_ptr });

        v8_create_function_from_closure(scope, |_,_,_| {
            // println!("Hello")
        })
    };

    {
        let scope = &mut v8::HandleScope::new(unsafe { *context_scope_ptr });
        let recv = v8::undefined(scope);
        let cb = callback;
        // let callback = v8::Local::new(scope, callback);
        cb.call(scope, recv.into(), &[]);
    }

    {
        let scope = &mut v8::HandleScope::new(unsafe { *context_scope_ptr });
        drop(unsafe {Box::from_raw(context_scope_ptr)});
        scope.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
    }
    Ok(())
}

type DynV8Callback =
    Box<dyn 'static + Fn(&mut v8::HandleScope, v8::FunctionCallbackArguments, v8::ReturnValue)>;

pub fn v8_create_function_from_closure<'a>(
    scope: &mut v8::HandleScope<'a>,
    closure: impl 'static + Fn(&mut v8::HandleScope, v8::FunctionCallbackArguments, v8::ReturnValue),
) -> v8::Local<'a, v8::Function> {
    let callback = Box::<DynV8Callback>::new(Box::new(closure));
    let callback_ptr = Box::into_raw(callback);
    let js_external = v8::External::new(scope, callback_ptr as *mut std::ffi::c_void);

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

    v8::Weak::with_finalizer(
        scope,
        handle,
        Box::new(move |_| {
            drop(unsafe { Box::from_raw(callback_ptr) });
        }),
    ).to_local(scope).unwrap()
}
