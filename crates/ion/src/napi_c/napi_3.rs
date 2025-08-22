#![allow(unused)]

use super::types::*;
use std::os::raw::c_void;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_fatal_exception(
    env: napi_env,
    err: napi_value,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_add_env_cleanup_hook(
    env: napi_env,
    fun: Option<unsafe extern "C" fn(arg: *mut c_void)>,
    arg: *mut c_void,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_remove_env_cleanup_hook(
    env: napi_env,
    fun: Option<unsafe extern "C" fn(arg: *mut c_void)>,
    arg: *mut c_void,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_open_callback_scope(
    env: napi_env,
    resource_object: napi_value,
    context: napi_async_context,
    result: *mut napi_callback_scope,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_close_callback_scope(
    env: napi_env,
    scope: napi_callback_scope,
) -> napi_status {
    todo!()
}
