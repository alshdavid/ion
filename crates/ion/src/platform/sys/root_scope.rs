use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;

pub(super) struct RootScopeInner {
    pub(super) scope: *mut v8::ScopeStorage<v8::HandleScope<'static, ()>>,
    pub(super) scope_pinned: *mut v8::PinnedRef<'static, v8::HandleScope<'static, ()>>,
    pub(super) context_scope: *mut v8::ContextScope<'static, 'static, v8::HandleScope<'static>>,
}

pub struct RootScope(pub(super) Rc<RootScopeInner>);

impl Clone for RootScope {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl Deref for RootScope {
    type Target = v8::ContextScope<'static, 'static, v8::HandleScope<'static>>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.context_scope }
    }
}

impl DerefMut for RootScope {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.context_scope }
    }
}

impl Drop for RootScopeInner {
    fn drop(&mut self) {
        unsafe {
            // Order matters or there will be a segfault
            drop(Box::from_raw(self.context_scope));
            drop(Box::from_raw(self.scope_pinned));
            drop(Box::from_raw(self.scope));
        };
    }
}
