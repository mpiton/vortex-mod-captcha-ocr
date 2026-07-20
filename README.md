# vortex-mod-captcha-ocr

Official Vortex image CAPTCHA solver backed by the system Tesseract installation.

The WASM plugin never receives a generic subprocess capability. It can only call
Vortex's typed `run_tesseract` broker, which fixes the executable, arguments,
timeouts, input format, and output bounds. If Tesseract is absent, the plugin
returns `unavailable` so the configured solver cascade can continue.

```bash
cargo test
cargo build --target wasm32-wasip1 --release
```

GPL-3.0.
