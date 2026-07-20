use extism_pdk::*;

use crate::{build_tesseract_request, handle_can_solve, normalize_tesseract_response};

#[host_fn]
extern "ExtismHost" {
    fn run_tesseract(request: String) -> String;
}

#[plugin_fn]
pub fn can_solve(input: String) -> FnResult<String> {
    handle_can_solve(&input).map_err(plugin_error)
}

#[plugin_fn]
pub fn solve(input: String) -> FnResult<String> {
    let request = build_tesseract_request(&input).map_err(plugin_error)?;
    // SAFETY: Vortex registers this exact typed host function only for the
    // registry-verified OCR plugin. Extism owns the String ABI buffers.
    let response = unsafe { run_tesseract(request)? };
    normalize_tesseract_response(&response).map_err(plugin_error)
}

fn plugin_error(message: String) -> WithReturnCode<extism_pdk::Error> {
    extism_pdk::Error::msg(message).into()
}
