use super::*;

#[repr(C)]
#[derive(Debug)]
/// Env that is shared between all contexts in same native module.
pub struct EnvShared {
    pub instance_data: Option<InstanceData>,
    pub napi_wrap: v8::Global<v8::Private>,
    pub type_tag: v8::Global<v8::Private>,
    // pub finalize: Option<napi_finalize>,
    // pub finalize_hint: *mut c_void,
    pub filename: String,
}
