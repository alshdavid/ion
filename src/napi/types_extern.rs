use std::ffi::c_void;

use super::*;

pub type napi_status = i32;
pub type napi_env = *mut c_void;
pub type napi_callback_info = *mut c_void;
pub type napi_deferred = *mut c_void;
pub type napi_ref = *mut c_void;
pub type napi_threadsafe_function = *mut c_void;
pub type napi_handle_scope = *mut c_void;
pub type napi_callback_scope = *mut c_void;
pub type napi_escapable_handle_scope = *mut c_void;
pub type napi_async_cleanup_hook_handle = *mut c_void;
pub type napi_async_work = *mut c_void;
pub type napi_async_context = *mut c_void;

pub type napi_cleanup_hook = unsafe extern "C" fn(data: *mut c_void);

pub type napi_callback =
    unsafe extern "C" fn(env: napi_env, info: napi_callback_info) -> napi_value<'static>;
