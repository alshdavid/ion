#![allow(unused)]

use super::types::*;
use std::os::raw::c_char;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn node_api_symbol_for(
    env: napi_env,
    utf8name: *const c_char,
    length: isize,
    result: *mut napi_value,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn node_api_get_module_file_name(
    env: napi_env,
    result: *mut *const c_char,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn node_api_create_syntax_error(
    env: napi_env,
    code: napi_value,
    msg: napi_value,
    result: *mut napi_value,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn node_api_throw_syntax_error(
    env: napi_env,
    code: *const c_char,
    msg: *const c_char,
) -> napi_status {
    todo!()
}
