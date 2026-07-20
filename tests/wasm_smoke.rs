use std::path::PathBuf;

use extism::{Function, UserData, Val, PTR};

fn wasm_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/wasm32-wasip1/release/vortex_mod_captcha_ocr.wasm")
}

fn run_tesseract() -> Function {
    Function::new(
        "run_tesseract",
        [PTR],
        [PTR],
        UserData::<()>::default(),
        |plugin, inputs, outputs, _user_data: UserData<()>| {
            let request: String = plugin.memory_get_val(&inputs[0])?;
            let request: serde_json::Value = serde_json::from_str(&request)?;
            if request != serde_json::json!({ "image_data": "aW1hZ2U=" }) {
                return Err(extism::Error::msg("unexpected typed OCR request"));
            }
            let response = r#"{"status":"solved","solution":"abc123"}"#;
            let handle = plugin.memory_new(response)?;
            outputs[0] = Val::I64(handle.offset() as i64);
            Ok(())
        },
    )
}

#[test]
fn wasm_exports_solve_through_the_typed_tesseract_broker() {
    let path = wasm_path();
    assert!(
        path.is_file(),
        "release WASM must be built before smoke tests"
    );
    let manifest = extism::Manifest::new([extism::Wasm::file(path)]);
    let mut plugin = extism::Plugin::new(&manifest, [run_tesseract()], true).expect("load WASM");
    let input = r#"{"challenge_id":"captcha-1","challenge_type":"image","challenge_url":"https://example.test","image_data":"aW1hZ2U="}"#;

    let supported: String = plugin.call("can_solve", input).expect("can_solve");
    let solved: String = plugin.call("solve", input).expect("solve");

    assert_eq!(supported, "true");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&solved).unwrap(),
        serde_json::json!({ "status": "solved", "solution": "abc123" })
    );
}
