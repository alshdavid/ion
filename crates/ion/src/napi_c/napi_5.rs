#![allow(unused)]

use super::types::*;
use std::os::raw::c_void;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_create_date(
    env: napi_env,
    time: f64,
    result: *mut napi_value,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_is_date(
    env: napi_env,
    value: napi_value,
    is_date: *mut bool,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_get_date_value(
    env: napi_env,
    value: napi_value,
    result: *mut f64,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_add_finalizer(
    env: napi_env,
    js_object: napi_value,
    native_object: *mut c_void,
    finalize_cb: napi_finalize,
    finalize_hint: *mut c_void,
    result: *mut napi_ref,
) -> napi_status {
    todo!()
}
