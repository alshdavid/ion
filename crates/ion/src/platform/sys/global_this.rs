use std::ptr::NonNull;
use std::rc::Rc;

struct GlobalThisInner(*mut v8::Isolate, NonNull<v8::Object>);

impl Drop for GlobalThisInner {
    fn drop(&mut self) {
        unsafe { v8::Global::from_raw(&mut *self.0, self.1) };
    }
}

pub struct GlobalThis(Rc<GlobalThisInner>);

impl Clone for GlobalThis {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl GlobalThis {
    pub fn new(context: &super::GlobalContext) -> Self {
        let isolate_ptr = { context.isolate() } as *mut v8::Isolate;
        let scope = &mut context.scope();
        let global_this = context.as_local().global(scope);
        let global_this_global = v8::Global::new(unsafe { &mut *isolate_ptr }, global_this);
        Self(Rc::new(GlobalThisInner(
            isolate_ptr,
            global_this_global.into_raw(),
        )))
    }

    pub fn as_local(&self) -> v8::Local<'static, v8::Object> {
        unsafe {
            std::mem::transmute::<NonNull<v8::Object>, v8::Local<'static, v8::Object>>(self.0.1)
        }
    }
}
