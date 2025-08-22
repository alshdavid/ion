use std::ffi::c_void;

#[repr(C)]
#[derive(Debug)]
pub struct InstanceData {
    pub data: *mut c_void,
    // pub finalize_cb: Option<napi_finalize>,
    // pub finalize_hint: *mut c_void,
}
