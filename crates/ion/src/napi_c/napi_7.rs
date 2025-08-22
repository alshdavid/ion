#![allow(unused)]

use super::types::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_detach_arraybuffer(
    env: napi_env,
    arraybuffer: napi_value,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn napi_is_detached_arraybuffer(
    env: napi_env,
    value: napi_value,
    result: *mut bool,
) -> napi_status {
    todo!()
}
