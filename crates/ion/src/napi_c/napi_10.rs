#![allow(unused)]

use super::types::*;
use std::os::raw::c_char;
use std::os::raw::c_void;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn node_api_create_external_string_latin1(
    env: napi_env,
    str_: *const c_char,
    length: isize,
    napi_finalize: node_api_basic_finalize,
    finalize_hint: *mut c_void,
    result: *mut napi_value,
    copied: *mut bool,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn node_api_create_external_string_utf16(
    env: napi_env,
    str_: *const u16,
    length: isize,
    napi_finalize: node_api_basic_finalize,
    finalize_hint: *mut c_void,
    result: *mut napi_value,
    copied: *mut bool,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn node_api_create_property_key_utf16(
    env: napi_env,
    str_: *const u16,
    length: isize,
    result: *mut napi_value,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn node_api_create_property_key_utf8(
    env: napi_env,
    str_: *const c_char,
    length: isize,
    result: *mut napi_value,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn node_api_create_property_key_latin1(
    env: napi_env,
    str_: *const c_char,
    length: isize,
    result: *mut napi_value,
) -> napi_status {
    todo!()
}
