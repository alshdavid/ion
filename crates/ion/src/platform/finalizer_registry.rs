use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct FinalizerRegistery {
    callbacks: Rc<RefCell<HashMap<usize, (v8::Weak<v8::Value>, Box<dyn FnOnce()>)>>>,
    isolate: *mut v8::Isolate,
}

impl FinalizerRegistery {
    pub fn new(isolate: *mut v8::Isolate) -> Self {
        Self {
            callbacks: Default::default(),
            isolate,
        }
    }

    pub fn register(
        &self,
        value: &v8::Local<'_, v8::Value>,
        callback: impl 'static + FnOnce(),
    ) -> usize {
        let mut callback = Box::new(callback);
        let id = callback.as_mut() as *mut _ as usize;

        let weak = v8::Weak::with_guaranteed_finalizer(
            unsafe { &mut *self.isolate },
            value,
            Box::new({
                let id = id.clone();
                let callbacks = self.callbacks.clone();
                move || {
                    let id = id.clone();
                    let mut callbacks = callbacks.borrow_mut();
                    if let Some((_, callback)) = callbacks.remove(&id) {
                        callback();
                    };
                }
            }),
        );

        let mut callbacks = self.callbacks.borrow_mut();
        callbacks.insert(id.clone(), (weak, callback));
        id
    }

    pub fn clear(&self) {
        let mut callbacks = self.callbacks.borrow_mut();
        for (_, (_, callback)) in callbacks.drain() {
            callback();
        }
    }
}
