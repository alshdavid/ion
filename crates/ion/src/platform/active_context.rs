use std::rc::Rc;

use crate::platform::v8::RawContext;
use crate::platform::v8::RawContextScope;
use crate::platform::v8::RawIsolate;
use crate::platform::v8::RawIsolateScope;

pub struct ActiveContext {
    id: Option<usize>,
    context_scope: Option<Rc<RawContextScope>>,
    handle_scope: Option<Rc<RawIsolateScope>>,
    isolate: Rc<RawIsolate>,
}

impl ActiveContext {
    pub fn new(isolate: Rc<RawIsolate>) -> Self {
        Self {
            id: None,
            handle_scope: None,
            context_scope: None,
            isolate,
        }
    }

    pub fn set(
        &mut self,
        context: &RawContext,
    ) -> bool {
        if let Some(id) = self.id
            && id == context.address()
        {
            return false;
        }

        // Drop the current context first so v8 can do clean up
        self.id = None;
        drop(self.context_scope.take());
        drop(self.handle_scope.take());

        // Create new context and put it on the stack
        let handle_scope = RawIsolateScope::new(v8::HandleScope::new(self.isolate.as_mut()));
        let context_scope = RawContextScope::new(v8::ContextScope::new(
            handle_scope.as_mut(),
            context.as_inner(),
        ));

        self.id.replace(context.address());
        self.context_scope.replace(context_scope);
        self.handle_scope.replace(handle_scope);

        return true;
    }

    pub fn take(&mut self) -> Option<(Rc<RawContextScope>, Rc<RawIsolateScope>)> {
        if self.id.is_none() {
            return None;
        }
        // SAFETY, if id is set, everything else is too
        self.id = None;
        return Some((
            self.context_scope.take().unwrap(),
            self.handle_scope.take().unwrap(),
        ));
    }
}
