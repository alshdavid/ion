use std::sync::Arc;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    IO(Arc<std::io::Error>),
    Generic(String),
    PlatformInitializeError,
    IsolateNotInitializedError,
    EventLoopNotInitializedError,
    WorkerInitializeError,
    StringCreateError,
    ScriptCompileError,
    ScriptRunError,
    ExecError,
    TaskSpawnError,
}

impl std::fmt::Display for Error {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IO(Arc::new(value))
    }
}
