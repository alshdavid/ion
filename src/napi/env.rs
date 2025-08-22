use std::cell::RefCell;
use std::ffi::c_void;
use std::ptr::NonNull;
use std::rc::Rc;

use super::*;

#[repr(C)]
pub struct Env {
    context: NonNull<v8::Context>,
    pub isolate_ptr: *mut v8::Isolate,
    pub open_handle_scopes: usize,
    // pub shared: *mut EnvShared,
    // cleanup_hooks: Rc<RefCell<Vec<(napi_cleanup_hook, *mut c_void)>>>,
    // pub last_error: napi_extended_error_info,
    pub last_exception: Option<v8::Global<v8::Value>>,
    pub global: v8::Global<v8::Object>,
    pub buffer_constructor: v8::Global<v8::Function>,
    pub report_error: v8::Global<v8::Function>,
}

unsafe impl Send for Env {}
unsafe impl Sync for Env {}

impl Env {
    pub fn new(
        isolate_ptr: *mut v8::Isolate,
        context: v8::Global<v8::Context>,
        global: v8::Global<v8::Object>,
        buffer_constructor: v8::Global<v8::Function>,
        report_error: v8::Global<v8::Function>,
        // sender: V8CrossThreadTaskSpawner,
        // cleanup_hooks: Rc<RefCell<Vec<(napi_cleanup_hook, *mut c_void)>>>,
        // external_ops_tracker: ExternalOpsTracker,
    ) -> Self {
        Self {
            isolate_ptr,
            context: context.into_raw(),
            global,
            buffer_constructor,
            report_error,
            // shared: std::ptr::null_mut(),
            open_handle_scopes: 0,
            // async_work_sender: sender,
            // cleanup_hooks,
            // external_ops_tracker,
            // last_error: napi_extended_error_info {
            //     error_message: std::ptr::null(),
            //     engine_reserved: std::ptr::null_mut(),
            //     engine_error_code: 0,
            //     error_code: napi_ok,
            // },
            last_exception: None,
        }
    }

    pub fn shared(&self) -> &EnvShared {
        // SAFETY: the lifetime of `EnvShared` always exceeds the lifetime of `Env`.
        // unsafe { &*self.shared }
        todo!()
    }

    pub fn shared_mut(&mut self) -> &mut EnvShared {
        // SAFETY: the lifetime of `EnvShared` always exceeds the lifetime of `Env`.
        // unsafe { &mut *self.shared }
        todo!()
    }

    pub fn add_async_work(&mut self, async_work: impl FnOnce() + Send + 'static) {
        // self.async_work_sender.spawn(|_| async_work());
        todo!()
    }

    #[inline]
    pub fn isolate(&mut self) -> &mut v8::Isolate {
        // SAFETY: Lifetime of `Isolate` is longer than `Env`.
        // unsafe { &mut *self.isolate_ptr }
        todo!()
    }

    #[inline]
    pub fn scope(&self) -> v8::CallbackScope<'_> {
        // SAFETY: `v8::Local` is always non-null pointer; the `HandleScope` is
        // already on the stack, but we don't have access to it.
        let context = unsafe {
            std::mem::transmute::<NonNull<v8::Context>, v8::Local<v8::Context>>(self.context)
        };
        // SAFETY: there must be a `HandleScope` on the stack, this is ensured because
        // we are in a V8 callback or the module has already opened a `HandleScope`
        // using `napi_open_handle_scope`.
        unsafe { v8::CallbackScope::new(context) }
    }

    pub fn threadsafe_function_ref(&mut self) {
        // self.external_ops_tracker.ref_op();
    }

    pub fn threadsafe_function_unref(&mut self) {
        // self.external_ops_tracker.unref_op();
    }

    pub fn add_cleanup_hook(&mut self, hook: napi_cleanup_hook, data: *mut c_void) {
        // let mut hooks = self.cleanup_hooks.borrow_mut();
        // if hooks
        //     .iter()
        //     .any(|pair| std::ptr::fn_addr_eq(pair.0, hook) && pair.1 == data)
        // {
        //     panic!("Cannot register cleanup hook with same data twice");
        // }
        // hooks.push((hook, data));
        todo!()
    }

    pub fn remove_cleanup_hook(&mut self, hook: napi_cleanup_hook, data: *mut c_void) {
        // let mut hooks = self.cleanup_hooks.borrow_mut();
        // match hooks
        //     .iter()
        //     .rposition(|&pair| std::ptr::fn_addr_eq(pair.0, hook) && pair.1 == data)
        // {
        //     Some(index) => {
        //         hooks.remove(index);
        //     }
        //     None => panic!("Cannot remove cleanup hook which was not registered"),
        // }
        todo!()
    }
}
