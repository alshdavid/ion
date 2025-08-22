#![allow(unused)]

use super::types::*;
use std::os::raw::c_void;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_add_async_cleanup_hook(
    env: napi_env,
    hook: napi_async_cleanup_hook,
    arg: *mut c_void,
    remove_handle: *mut napi_async_cleanup_hook_handle,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_remove_async_cleanup_hook(
    remove_handle: napi_async_cleanup_hook_handle
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_object_freeze(
    env: napi_env,
    object: napi_value,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_object_seal(
    env: napi_env,
    object: napi_value,
) -> napi_status {
    todo!()
}
