//! `nesc-wasm` — the NESL compiler exposed to the browser as WebAssembly.
//!
//! Every function returns exactly the JSON that `nesc serve` would answer
//! with, so the editor can switch between the local server and this
//! in-browser compiler without any change in shape.
//!
//! Built with `wasm-pack build --target web`, which produces `pkg/nesc_wasm.js`
//! (an ES module) plus `pkg/nesc_wasm_bg.wasm`.

use nesc::json::{builtins_json, check_json, compile_json, examples_json};
use wasm_bindgen::prelude::*;

/// `{"ok":true,"rom":"<base64>","prg_used":N,"asm":".."}` or `{"ok":false,"errors":[..]}`.
#[wasm_bindgen]
pub fn nesc_compile(src: &str) -> String {
    compile_json(src)
}

/// `{"ok":true,"errors":[]}` or `{"ok":false,"errors":[..]}` — same compile, no ROM.
#[wasm_bindgen]
pub fn nesc_check(src: &str) -> String {
    check_json(src)
}

/// `[{"name":..,"source":..}, ..]` — the bundled example programs.
#[wasm_bindgen]
pub fn nesc_examples() -> String {
    examples_json()
}

/// `{"functions":[..],"constants":[..],"keywords":[..]}` for completion and hover.
#[wasm_bindgen]
pub fn nesc_builtins() -> String {
    builtins_json()
}
