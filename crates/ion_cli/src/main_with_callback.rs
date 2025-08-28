pub(crate) mod js;

use std::{collections::HashMap, ptr::NonNull, rc::Rc, time::Duration};

use crate::js::CallbackInfo;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

static CODE: &str = r#"
    "Hello World"
"#;

static BYTES: &[u8] = include_bytes!("../../../Cargo.lock");

fn main() -> anyhow::Result<()> {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::set_flags_from_string(
        "--no_freeze_flags_after_init --expose_gc --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls --js-source-phase-imports",
    );
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    main2()?;
    println!("Next {})", MemoryStats::current());


    // let mut prev = MemoryStats::current();
    // for i in 0..100 {
    //     for _ in 0..10000 {
    //         main2()?;
    //     }
    //     let res = MemoryStats::current();
    //     let diff = res.resident.0 - prev.resident.0;
    //     prev = res;
    //     println!("Next {}, {} ({})", i,  prev, Bytes(diff));
    //     // std::thread::sleep(Duration::from_secs(1));
    // }

    // std::thread::sleep(Duration::from_secs(60));
    Ok(())
}

fn main2() -> anyhow::Result<()> {


    // Globals
    let mut isolate = v8::Isolate::new(v8::CreateParams::default());
    let isolate_ptr = isolate.as_mut() as *mut v8::Isolate;

    let context_ptr = {
        let mut scope = v8::HandleScope::new(&mut isolate);
        let context = v8::Context::new(&mut scope, Default::default());
        context
    };

    // let callback = {
    //     let scope = &mut v8::HandleScope::with_context(unsafe { &mut *isolate_ptr }, context_ptr);

    //     let value_ref = CallbackInfo::new(|| {});
    //     let external = v8::External::new(scope, value_ref.into_ptr());

    //     let callback = v8::Function::builder(
    //         move |scope: &mut v8::HandleScope,
    //               args: v8::FunctionCallbackArguments,
    //               rv: v8::ReturnValue| {
    //             // println!("Running callback");
    //             let js_data = args.data();
    //             let js_external = unsafe { v8::Local::<v8::External>::cast_unchecked(js_data) };

    //             let external_ptr = js_external.value() as *mut Vec<u8>;
    //             let callback: &Vec<u8> = unsafe { &*external_ptr };
    //             // callback.hello();

    //             let weak = {
    //                 let scope = &mut v8::HandleScope::new(scope);
    //                 let weak = v8::Weak::with_guaranteed_finalizer(
    //                     scope, //v8_isolate_ptr(isolate_ptr),
    //                     js_external,
    //                     Box::new(move || {
    //                         println!("Called: with_guaranteed_finalizer");
    //                         unsafe { let _ = Box::from_raw(external_ptr); };
    //                     }),
    //                 );

    //                 weak.into_raw().unwrap()
    //             };
    //         },
    //     )
    //     .data(external.into())
    //     .build(scope)
    //     .unwrap();

    //     // v8::Global::new(scope, callback)
    //     callback
    // };

    {
        // let scope = &mut v8_global_scope(isolate_ptr, context_ptr);
    
        let scope = &mut v8::HandleScope::with_context(unsafe { &mut *isolate_ptr }, context_ptr);
        // let recv = v8::undefined(scope);
        // let callback = v8::Local::new(scope, callback);
        // callback.call(scope, recv.into(), &[]);
        v8_eval_code(scope, CODE);

        // let v = unsafe { v8::Global::from_raw(v8_isolate_ptr(isolate_ptr), context_ptr) };
        // drop(v)
    };

    isolate.request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
    Ok(())
}

pub fn v8_eval_code<'a, S: AsRef<str>>(
    scope: &mut v8::HandleScope<'a>,
    code: S,
) -> v8::Global<v8::Value> {
    let code = v8::String::new(scope, code.as_ref()).unwrap();
    let script = v8::Script::compile(scope, code, None).unwrap();
    let value = script.run(scope).unwrap();
    v8::Global::new(scope, value)
}

pub fn v8_global_context(context_ptr: NonNull<v8::Context>) -> v8::Local<'static, v8::Context> {
    // SAFETY: `v8::Local` is always non-null pointer; the `HandleScope` is
    // already on the stack, but we don't have access to it.
    unsafe { std::mem::transmute::<NonNull<v8::Context>, v8::Local<v8::Context>>(context_ptr) }
}

pub fn v8_global_scope(
    isolate_ptr: *mut v8::Isolate,
    context_ptr: NonNull<v8::Context>,
) -> v8::HandleScope<'static> {
    v8::HandleScope::with_context(v8_isolate_ptr(isolate_ptr), v8_global_context(context_ptr))
}

pub fn v8_isolate_ptr(isolate_ptr: *mut v8::Isolate) -> &'static mut v8::Isolate {
    unsafe { &mut *isolate_ptr }
}

struct DropDetector {}

impl DropDetector {
    fn hello(&self) {
        // println!("Hello")
    }
}

impl Drop for DropDetector {
    fn drop(&mut self) {
        // println!("Dropped")
    }
}

struct MemoryStats {
    allocated: Bytes,
    resident: Bytes,
}

impl MemoryStats {
    fn current() -> MemoryStats {
        jemalloc_ctl::epoch().unwrap();
        MemoryStats {
            allocated: Bytes(jemalloc_ctl::stats::allocated().unwrap()),
            resident: Bytes(jemalloc_ctl::stats::resident().unwrap()),
        }
    }
}

impl std::fmt::Display for MemoryStats {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{} allocated {} resident",
            self.allocated, self.resident,
        )
    }
}

#[derive(Default)]
struct Bytes(usize);

impl std::fmt::Display for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let bytes = self.0;
        if bytes < 4096 {
            return write!(f, "{} bytes", bytes);
        }
        let kb = bytes / 1024;
        if kb < 4096 {
            return write!(f, "{}kb", kb);
        }
        let mb = kb / 1024;
        write!(f, "{}mb", mb)
    }
}

impl std::ops::AddAssign<usize> for Bytes {
    fn add_assign(&mut self, x: usize) {
        self.0 += x;
    }
}