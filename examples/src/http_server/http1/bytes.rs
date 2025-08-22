use std::borrow::Cow;
use std::convert::Infallible;

use http_body_util::Full;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes as HyperBytes;

pub struct Bytes(Vec<u8>);

impl From<Vec<u8>> for Bytes {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<&[u8]> for Bytes {
    fn from(value: &[u8]) -> Self {
        Self(value.to_vec())
    }
}

impl<'a> From<Cow<'a, [u8]>> for Bytes {
    fn from(value: Cow<'a, [u8]>) -> Self {
        Self(value.to_vec())
    }
}

impl From<&str> for Bytes {
    fn from(value: &str) -> Self {
        Self(value.as_bytes().to_vec())
    }
}

impl From<String> for Bytes {
    fn from(value: String) -> Self {
        Self(value.as_bytes().to_vec())
    }
}

impl From<Bytes> for Full<HyperBytes> {
    fn from(val: Bytes) -> Self {
        Full::new(HyperBytes::from(val.0))
    }
}

impl From<Bytes> for BoxBody<HyperBytes, Infallible> {
    fn from(val: Bytes) -> Self {
        BoxBody::new(Full::new(HyperBytes::from(val.0)))
    }
}
