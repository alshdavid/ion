use flume::Sender;
use tokio_util::task::TaskTracker;

use crate::DynResolver;
use crate::Env;
use crate::fs::FileSystem;
use crate::platform::background_worker::BackgroundWorkerEvent;
use crate::platform::module_map::ModuleMap;
use crate::utils::channel::oneshot;

// Container that constructs a V8 context and preserves the internals until dropped
pub struct JsRealm {
    pub(crate) resolvers: Vec<DynResolver>,
    pub(crate) fs: FileSystem,
    background_tasks: *mut Sender<BackgroundWorkerEvent>,
    id: usize,
    env: Box<Env>,
    modules: *mut ModuleMap,
    context: *mut v8::Local<'static, v8::Context>,
    global_this: *mut std::ffi::c_void, // v8::Global<v8::Object>,
    async_tasks: *mut TaskTracker,
    handle_scope: *mut v8::HandleScope<'static, ()>,
    context_scope: *mut v8::ContextScope<'static, v8::HandleScope<'static>>,
}

impl JsRealm {
    pub(crate) fn new(
        isolate_ptr: *mut v8::Isolate,
        fs: FileSystem,
        resolvers: Vec<DynResolver>,
        background_tasks: Sender<BackgroundWorkerEvent>,
    ) -> Box<Self> {
        let handle_scope = Box::new(v8::HandleScope::new(unsafe { &mut *isolate_ptr }));
        let handle_scope_ptr = Box::into_raw(handle_scope);
        let handle_scope = unsafe { &mut *handle_scope_ptr };

        let context = Box::new(v8::Context::new(&mut *handle_scope, Default::default()));
        let context_ptr = Box::into_raw(context);
        let context = unsafe { *context_ptr };

        let global_this = Box::new(v8::Global::new(
            unsafe { &mut *isolate_ptr },
            context.global(&mut *handle_scope),
        ));
        let global_this_ptr = Box::into_raw(global_this);

        let context_scope = Box::new(v8::ContextScope::new(handle_scope, context));
        let context_scope_ptr = Box::into_raw(context_scope);

        let async_tasks = Box::new(TaskTracker::new());
        let async_tasks_ptr: *mut TaskTracker = Box::into_raw(async_tasks);

        let background_tasks = Box::into_raw(Box::new(background_tasks));

        let env = Env::new(
            isolate_ptr,
            context_ptr,
            global_this_ptr,
            async_tasks_ptr,
            background_tasks,
        );

        let modules = Box::into_raw(Box::new(ModuleMap::default()));

        let mut realm = Box::new(JsRealm {
            id: 0,
            env,
            fs,
            background_tasks,
            modules,
            resolvers,
            context: context_ptr,
            global_this: global_this_ptr as _,
            async_tasks: async_tasks_ptr,
            handle_scope: handle_scope_ptr,
            context_scope: context_scope_ptr,
        });

        let realm_ptr = realm.as_mut() as *mut JsRealm;
        let realm_id = realm_ptr as usize;

        {
            // TODO use slot or data
            let scope = unsafe { &mut *context_scope_ptr };
            let key = v8::String::new(scope, "__data").unwrap();
            let value = v8::External::new(scope, realm_ptr as _);
            let global_this = context.global(scope);
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
        for on_before_exit in unsafe { &mut *self.env.on_before_exit }.iter_mut() {
            drop(on_before_exit());
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
        drop(unsafe { Box::from_raw(self.global_this as *mut v8::Global<v8::Object>) });
        drop(unsafe { Box::from_raw(self.context_scope) });
        drop(unsafe { Box::from_raw(self.context) });
        drop(unsafe { Box::from_raw(self.handle_scope) });
        drop(unsafe { Box::from_raw(self.modules) });
    }
}
