use std::rc::Rc;

use flume::Sender;
use tokio_util::task::TaskTracker;

use crate::DynResolver;
use crate::Env;
use crate::fs::FileSystem;
use crate::platform::background_worker::BackgroundWorkerEvent;
use crate::platform::module_map::ModuleMap;
use crate::platform::v8::RawContext;
use crate::platform::v8::RawContextScope;
use crate::platform::v8::RawGlobal;
use crate::platform::v8::RawIsolate;
use crate::platform::v8::RawIsolateScope;
use crate::utils::channel::oneshot;

// Container that constructs a V8 context and preserves the internals until dropped
pub struct JsRealm {
    pub(crate) resolvers: Vec<DynResolver>,
    pub(crate) fs: FileSystem,
    pub(crate) id: usize,
    pub(crate) env: Box<Env>,
    // TODO make these RefCells
    pub(crate) background_tasks: *mut Sender<BackgroundWorkerEvent>,
    pub(crate) modules: *mut ModuleMap,
    pub(crate) async_tasks: *mut TaskTracker,
    // SAFETY: Changing the order of these properties
    // will affect their drop order and break the isolate
    #[allow(unused)]
    pub(crate) global_this: Rc<RawGlobal>,
    #[allow(unused)]
    pub(crate) context: Rc<RawContext>,
}

impl JsRealm {
    pub(crate) fn new(
        isolate: Rc<RawIsolate>,
        fs: FileSystem,
        resolvers: Vec<DynResolver>,
        background_tasks: Sender<BackgroundWorkerEvent>,
    ) -> Box<Self> {
        let handle_scope = RawIsolateScope::new(v8::HandleScope::new(isolate.as_mut()));

        let context = RawContext::new(isolate.as_ref(), handle_scope.as_mut());
        let context_scope = RawContextScope::new(v8::ContextScope::new(
            handle_scope.as_mut(),
            context.as_inner(),
        ));

        let global_this = RawGlobal::new(&context, &context_scope);

        // TODO make these RefCells
        let async_tasks = Box::new(TaskTracker::new());
        let async_tasks_ptr: *mut TaskTracker = Box::into_raw(async_tasks);
        let modules = Box::into_raw(Box::new(ModuleMap::default()));
        let background_tasks = Box::into_raw(Box::new(background_tasks));

        let env = Env::new(
            isolate,
            Rc::clone(&context),
            Rc::clone(&global_this),
            async_tasks_ptr,
            background_tasks,
        );

        let mut realm = Box::new(JsRealm {
            id: 0,
            env,
            fs,
            background_tasks,
            modules,
            resolvers,
            async_tasks: async_tasks_ptr,
            // v8 internals
            global_this,
            context,
        });

        let realm_ptr = realm.as_mut() as *mut JsRealm;
        let realm_id = realm_ptr as usize;

        {
            // TODO use slot or data
            let scope = &mut unsafe { v8::CallbackScope::new(realm.context.as_inner()) };
            let key = v8::String::new(scope, "__data").unwrap();
            let value = v8::External::new(scope, realm_ptr as _);
            let global_this = realm.context.global(scope);
            global_this.set(scope, key.into(), value.into());
        }

        realm.id = realm_id;

        realm
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn async_tasks(&self) -> &TaskTracker {
        unsafe { &mut *self.async_tasks }
    }

    pub(crate) fn background_tasks(&self) -> &Sender<BackgroundWorkerEvent> {
        unsafe { &mut *self.background_tasks }
    }

    pub async fn drain_async_tasks(&self) {
        self.async_tasks().close();
        self.async_tasks().wait().await;
    }

    pub fn env(&self) -> &Env {
        &self.env
    }

    pub fn notify_shutdown(&self) {
        let mut on_before_exit = self.env.on_before_exit.borrow_mut();
        while let Some(on_before_exit) = on_before_exit.pop() {
            on_before_exit().unwrap();
        }
    }

    pub fn background_blocking<Return: 'static + Send + Sync>(
        &self,
        fut: impl 'static + Send + Sync + Future<Output = crate::Result<Return>>,
    ) -> crate::Result<Return> {
        let (tx, rx) = oneshot();
        self.background_tasks()
            .try_send(BackgroundWorkerEvent::ExecFut(Box::pin(async move {
                tx.try_send(fut.await).unwrap();
                Ok(())
            })))?;
        rx.recv()?
    }

    pub fn background_async(
        &self,
        fut: impl 'static + Send + Sync + Future<Output = crate::Result<()>>,
    ) -> crate::Result<()> {
        Ok(self
            .background_tasks()
            .try_send(BackgroundWorkerEvent::ExecFut(Box::pin(fut)))?)
    }

    #[allow(clippy::mut_from_ref)]
    pub(crate) fn module_map(&self) -> &mut ModuleMap {
        unsafe { &mut *self.modules }
    }

    pub(crate) fn v8_revive<'a>(scope: &mut v8::HandleScope<'_>) -> &'a mut JsRealm {
        let context = scope.get_current_context();
        let global_this = context.global(scope);
        let data_key = v8::String::new(scope, "__data").unwrap();
        let data = global_this.get(scope, data_key.into()).unwrap();
        let data = data.cast::<v8::External>();
        unsafe { &mut *(data.value() as *mut JsRealm) }
    }
}

impl Drop for JsRealm {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.async_tasks) });
        drop(unsafe { Box::from_raw(self.background_tasks) });
        drop(unsafe { Box::from_raw(self.modules) });
    }
}
