use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

thread_local! {
    static EXTERNAL_CACHE: RefCell<HashMap<*mut std::ffi::c_void, Rc<v8::Weak<v8::External>>>> = Default::default();
}

fn cache_insert(key: *mut std::ffi::c_void, entry: Rc<v8::Weak<v8::External>>) {
    EXTERNAL_CACHE.with(|hm| hm.borrow_mut().insert(key, entry));
}

fn cache_remove(key: *mut std::ffi::c_void) -> bool {
    EXTERNAL_CACHE
        .with(|hm| hm.borrow_mut().remove(&key))
        .is_some()
}

#[derive(Clone)]
struct Counter(Rc<RefCell<usize>>);

impl Counter {
    pub fn new(i: usize) -> Self {
      Self(Rc::new(RefCell::new(i)))
    }

    pub fn inc(&self) -> usize {
        let mut i = self.0.borrow_mut();
        (*i) += 1;
        *i
    }

    pub fn dec(&self) -> usize {
      let mut i = self.0.borrow_mut();
        (*i) -= 1;
        *i
    }
}

/// Reference counted external v8 value.
/// The value cannot be sent between threads.
pub struct JsExternal<T> {
    count: Counter,
    handle: Rc<v8::Weak<v8::External>>,
    ptr: *mut std::ffi::c_void,
    _inner: PhantomData<T>,
}

impl<T> JsExternal<T> {
    pub fn new(scope: &mut v8::HandleScope, value: T) -> Self {
        let count = Counter::new(1);

        let boxed = Box::new(value);
        let raw = Box::into_raw(boxed) as *mut std::ffi::c_void;
        let external = v8::External::new(scope, raw);
        let weak = Rc::new(v8::Weak::with_guaranteed_finalizer(
            scope,
            external,
            Box::new({
                let count = count.clone();
                move || {
                    if count.dec() != 0 {
                        return;
                    }
                    if !cache_remove(raw) {
                        return;
                    }
                    drop(unsafe { Box::from_raw(raw as *mut T) })
                }
            }),
        ));

        cache_insert(raw, weak.clone());

        Self {
            count,
            handle: weak,
            ptr: raw,
            _inner: Default::default(),
        }
    }

    pub fn r#ref(&self) {
      self.count.inc();
    }

    pub fn unref(&self) {
      self.count.dec();
    }

    /// Get the internal value as a handle. Does not increment the reference count.
    /// Clone the value before consuming it as a v8 handle.
    pub fn as_local<'a>(&self, scope: &mut v8::HandleScope<'a, ()>) -> v8::Local<'a, v8::External> {
        self.handle.to_local(scope).unwrap()
    }
}

impl<T> Clone for JsExternal<T> {
    fn clone(&self) -> Self {
        self.r#ref();
        Self {
            count: self.count.clone(),
            handle: self.handle.clone(),
            ptr: self.ptr,
            _inner: self._inner,
        }
    }
}

impl<T> Drop for JsExternal<T> {
    fn drop(&mut self) {
        if self.count.dec() != 0 {
            return;
        }

        if EXTERNAL_CACHE
            .with(|hm| hm.borrow_mut().remove(&self.ptr))
            .is_some()
        {
            drop(unsafe { Box::from_raw(self.ptr as *mut T) })
        }
    }
}

impl<T> Deref for JsExternal<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.ptr as *mut T) }
    }
}
