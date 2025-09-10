use crate::platform::v8::__v8_context;
use crate::platform::v8::__v8_context_scope;
use crate::platform::v8::__v8_root_scope;
use crate::platform::v8::v8_drop_context_scope;
use crate::platform::v8::v8_drop_root_scope;
use crate::platform::v8::v8_get_context;
use crate::platform::v8::v8_get_context_address;
use crate::platform::v8::v8_get_root_scope;
use crate::platform::v8::v8_new_context_scope;
use crate::platform::v8::v8_new_root_scope;

pub struct ActiveContext {
    current: Option<(usize, __v8_context_scope, __v8_root_scope)>,
    isolate: *mut v8::Isolate,
}

impl std::fmt::Debug for ActiveContext {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        if let Some((id, _, _)) = &self.current {
            write!(f, "ActiveContext {:?}", id)
        } else {
            write!(f, "ActiveContext (none)",)
        }
    }
}

impl ActiveContext {
    pub fn new(isolate: *mut v8::Isolate) -> Self {
        Self {
            current: None,
            isolate,
        }
    }

    pub fn set(
        &mut self,
        context: __v8_context,
    ) -> bool {
        if let Some((id, _, _)) = self.current
            && id == v8_get_context_address(context)
        {
            return false;
        }

        // Drop the current context first so v8 can do clean up
        self.unset();

        // Create new context and put it on the stack
        let handle_scope = v8_new_root_scope(v8::HandleScope::new(unsafe { &mut *self.isolate }));
        let context_scope = v8_new_context_scope(v8::ContextScope::new(
            v8_get_root_scope(handle_scope),
            v8_get_context(context),
        ));

        self.current
            .replace((v8_get_context_address(context), context_scope, handle_scope));

        return true;
    }

    pub fn take(
        &mut self
    ) -> Option<(
        v8::ContextScope<'static, v8::HandleScope<'static>>,
        v8::HandleScope<'static, ()>,
    )> {
        let Some((_id, context_scope, handle_scope)) = self.current.take() else {
            return None;
        };
        Some((
            v8_drop_context_scope(context_scope),
            v8_drop_root_scope(handle_scope),
        ))
    }

    pub fn unset(&mut self) {
        if let Some((_id, context_scope, handle_scope)) = self.current.take() {
            v8_drop_context_scope(context_scope);
            v8_drop_root_scope(handle_scope);
        }
    }
}
