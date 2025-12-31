use std::cell::Cell;
use std::cell::RefCell;
use std::ptr::NonNull;
use std::rc::Rc;

use super::RootScope;
use super::RootScopeInner;

// Create a root scope the first time a call to GlobalContext.scope() is called,
// reusing that scope for subsequent calls. If the context is changed, tear down the
// root scope and replace it with the scope of the new context.
// This enables context multiplexing
thread_local! {
    static CURRENT_CONTEXT: Cell<Option<usize>> = Cell::new(None);
    static CURRENT_SCOPE: RefCell<Option<RootScope>> = RefCell::new(None);
}

struct GlobalContextInner(*mut v8::Isolate, NonNull<v8::Context>);

impl Drop for GlobalContextInner {
    fn drop(&mut self) {
        CURRENT_CONTEXT.replace(None);
        CURRENT_SCOPE.replace(None);
        unsafe { v8::Global::from_raw(&mut *self.0, self.1) };
    }
}

pub struct GlobalContext(usize, Rc<GlobalContextInner>);

impl Clone for GlobalContext {
    fn clone(&self) -> Self {
        Self(self.0.clone(), Rc::clone(&self.1))
    }
}

impl GlobalContext {
    pub fn new(isolate: &mut v8::Isolate) -> Self {
        let isolate_ptr = { &mut *isolate } as *mut v8::Isolate;
        v8::scope!(let handle_scope, isolate.as_mut());
        let context_local = v8::Context::new(handle_scope, Default::default());
        let context_global = v8::Global::new(unsafe { &mut *isolate_ptr }, context_local);

        let inner = Rc::new(GlobalContextInner(isolate_ptr, context_global.into_raw()));
        let addr = Rc::as_ptr(&inner) as usize;
        Self(addr, inner)
    }

    pub fn as_local(&self) -> v8::Local<'static, v8::Context> {
        unsafe {
            std::mem::transmute::<NonNull<v8::Context>, v8::Local<'static, v8::Context>>(self.1.1)
        }
    }

    pub fn isolate(&self) -> &mut v8::Isolate {
        unsafe { &mut *self.1.0 }
    }

    /// Get or init root scope
    pub fn scope(&self) -> RootScope {
        if CURRENT_CONTEXT.get().is_none() {
            self.init_root_scope();
        }
        if let Some(id) = CURRENT_CONTEXT.get()
            && id != self.0
        {
            self.init_root_scope();
        }
        CURRENT_SCOPE.with(|root_scope| {
            let root_scope = root_scope.borrow();
            let root_scope = root_scope
                .as_ref()
                .expect("Critical Internal Error, unable to get root scope");
            root_scope.clone()
        })
    }

    fn init_root_scope(&self) {
        // Clear current scope and context first so they can be torn down
        CURRENT_CONTEXT.replace(None);
        CURRENT_SCOPE.replace(None);

        let scope = Box::new(v8::HandleScope::new(unsafe { &mut *self.1.0 }));
        let scope_ptr = Box::into_raw(scope) as *mut v8::ScopeStorage<v8::HandleScope<'static, ()>>;

        let scope_pinned = {
            let scope_pinned = unsafe { std::pin::Pin::new_unchecked(&mut *(scope_ptr)) };
            Box::new(scope_pinned.init())
        };
        let scope_pinned_ptr = Box::into_raw(scope_pinned);

        let context_scope = Box::new(v8::ContextScope::new(
            unsafe { &mut *(scope_pinned_ptr) },
            self.as_local(),
        ));
        let context_scope_ptr = Box::into_raw(context_scope);

        let root_scope = RootScope(Rc::new(RootScopeInner {
            scope: scope_ptr,
            scope_pinned: scope_pinned_ptr,
            context_scope: context_scope_ptr,
        }));

        CURRENT_CONTEXT.replace(Some(self.0.clone()));
        CURRENT_SCOPE.replace(Some(root_scope));
    }
}
