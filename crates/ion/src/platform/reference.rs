use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::c_void;

use crate::Env;

// `v8::Weak` values must outlive the scope that they are created in for the finalizer callback to run.
// To ensure the Weak values outlive their scope, they are stored in a thread local HashMap where
// the reference count informs if a value should be removed from the map.
// The reference count looks at both calls to Rust's drop() and hooks into v8's GC
thread_local! {
    static STATICS: RefCell<HashSet<*mut c_void>> = Default::default();
}

fn cache_insert(entry: *mut std::ffi::c_void) {
    STATICS.with(|hm| hm.borrow_mut().insert(entry));
}

fn cache_remove(key: *mut std::ffi::c_void) -> bool {
    STATICS.with(|hm| hm.borrow_mut().remove(&key))
}

#[derive(Debug)]
pub enum ReferenceState {
    Strong(v8::Global<v8::Value>),
    Weak(v8::Weak<v8::Value>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReferenceOwnership {
    Runtime,
    Rust,
}

pub struct Reference {
    env: *mut Env,
    ref_count: u32,
    state: ReferenceState,
    ownership: ReferenceOwnership,
    finalize_cb: Option<Box<dyn 'static + FnOnce(Env)>>,
}

impl Reference {
    /// Register a callback to run when the value is GC'd by v8
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn register_global_finalizer<'a>(
        value: impl Into<v8::Local<'a, v8::Value>>,
        env: *mut Env,
        initial_ref_count: u32,
        ownership: ReferenceOwnership,
        finalize_cb: Option<Box<dyn 'static + FnOnce(Env)>>,
    ) {
        let reference = Self::new(
            value.into(),
            env,
            initial_ref_count,
            ownership,
            Box::new(|_| {}),
        );
        let ptr = Box::into_raw(reference);
        let address = ptr as *mut c_void;

        let reference = unsafe { &mut *ptr };

        reference.dec_ref();
        reference.finalize_cb = Some(Box::new(move |_| {
            drop(unsafe { Box::from_raw(address as *mut Reference) });
            cache_remove(address);
            if let Some(callback) = finalize_cb {
                callback(unsafe { *env });
            }
        }));

        cache_insert(address);
    }

    /// Note, the finalizer callback will not fire if the reference is dropped before
    /// the scope it is created within is. Use [`Reference::register_global_finalizer`]
    /// If you need a guaranteed clean-up callback regardless of scope.
    pub fn new<'a>(
        value: impl Into<v8::Local<'a, v8::Value>>,
        env: *mut Env,
        initial_ref_count: u32,
        ownership: ReferenceOwnership,
        finalize_cb: Box<dyn 'static + FnOnce(Env)>,
    ) -> Box<Self> {
        let isolate = Reference::isolate(env);

        let mut reference = Box::new(Reference {
            env,
            state: ReferenceState::Strong(v8::Global::new(isolate, value.into())),
            ref_count: initial_ref_count,
            ownership,
            finalize_cb: Some(finalize_cb),
        });

        if initial_ref_count == 0 {
            reference.set_weak();
        }

        reference
    }

    fn isolate(env: *mut Env) -> &'static mut v8::Isolate {
        unsafe { (*env).isolate() }
    }

    pub fn inc_ref(&mut self) -> u32 {
        self.ref_count += 1;
        if self.ref_count == 1 {
            self.set_strong();
        }
        self.ref_count
    }

    pub fn dec_ref(&mut self) -> u32 {
        let old_ref_count = self.ref_count;
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
        if old_ref_count == 1 && self.ref_count == 0 {
            self.set_weak();
        }
        self.ref_count
    }

    fn set_strong(&mut self) {
        if let ReferenceState::Weak(w) = &self.state {
            let isolate = Reference::isolate(self.env);
            if let Some(g) = w.to_global(isolate) {
                self.state = ReferenceState::Strong(g);
            }
        }
    }

    fn set_weak(&mut self) {
        let reference = self as *mut Reference;
        if let ReferenceState::Strong(g) = &self.state {
            let cb = Box::new(move || Reference::weak_callback(reference));
            let isolate = Reference::isolate(self.env);
            self.state = ReferenceState::Weak(v8::Weak::with_guaranteed_finalizer(isolate, g, cb));
        }
    }

    fn weak_callback(reference: *mut Reference) {
        let reference = unsafe { &mut *reference };

        let finalize_cb = &mut reference.finalize_cb;
        let ownership = reference.ownership;
        if let Some(finalize_cb) = finalize_cb.take() {
            finalize_cb(unsafe { *reference.env });
        }

        if ownership == ReferenceOwnership::Runtime {
            unsafe { drop(Reference::from_raw(reference)) }
        }
    }

    pub fn into_raw(r: Box<Reference>) -> *mut Reference {
        Box::into_raw(r)
    }

    /// # SAFETY
    ///
    /// This can only live for as long as the v8::Context
    pub unsafe fn from_raw(r: *mut Reference) -> Box<Reference> {
        unsafe { Box::from_raw(r) }
    }
}
