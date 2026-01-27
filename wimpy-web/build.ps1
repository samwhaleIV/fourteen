$env:RUSTFLAGS = "--cfg=web_sys_unstable_apis"

cargo build --target wasm32-unknown-unknown

wasm-bindgen ../target/wasm32-unknown-unknown/debug/wimpy_web.wasm --no-typescript --out-dir html/ `--target web
