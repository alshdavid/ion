#![allow(unused)]

use super::types::*;
use std::os::raw::c_char;
use std::os::raw::c_void;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn node_api_create_buffer_from_arraybuffer(
    env: napi_env,
    arraybuffer: napi_value,
    byte_offset: usize,
    byte_length: usize,
    result: *mut napi_value,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn node_api_post_finalizer(
    env: node_api_basic_env,
    finalize_cb: napi_finalize,
    finalize_data: *mut c_void,
    finalize_hint: *mut c_void,
) -> napi_status {
    todo!()
}
