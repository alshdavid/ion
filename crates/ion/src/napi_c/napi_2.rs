#![allow(unused)]

use super::types::*;
use std::os::raw::c_int;

#[unsafe(no_mangle)]

pub unsafe extern "C" fn napi_get_uv_event_loop(
    env: napi_env,
    loop_: *mut *mut uv_loop_s,
) -> napi_status {
    todo!()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn uv_run(
    loop_: *mut uv_loop_s,
    mode: uv_run_mode,
) -> c_int {
    todo!()
}
