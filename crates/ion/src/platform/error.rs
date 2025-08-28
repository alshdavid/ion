pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    Generic(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Generic(msg) => write!(f, "GenericError: {}", msg),
        }
    }
}
