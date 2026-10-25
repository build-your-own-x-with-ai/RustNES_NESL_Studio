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

// ============================================================================
// Advanced Decompiler (Feature 015)
// ============================================================================

use nesl::decompile::advanced;

/// Analyze a NES ROM and return variables, functions, and pseudocode as JSON.
///
/// Input: ROM bytes as base64 string
/// Output: JSON with structure:
/// ```json
/// {
///   "ok": true,
///   "rom_info": {"prg_banks": N, "chr_banks": N, "mapper": N},
///   "variables": [{"addr": N, "name": "...", "type": "...", "access_count": N, "pattern": "..."}],
///   "functions": [{"addr": N, "name": "...", "size": N, "called_count": N, "calls_made": N}],
///   "pseudocode": "..."
/// }
/// ```
/// or `{"ok": false, "error": "..."}` on failure
#[wasm_bindgen]
pub fn nesc_decompile(rom_base64: &str) -> String {
    // Decode base64
    let rom_bytes = match base64_decode(rom_base64) {
        Ok(bytes) => bytes,
        Err(e) => return json_error(&format!("Failed to decode base64: {}", e)),
    };

    // Parse ROM header
    if rom_bytes.len() < 16 || &rom_bytes[0..4] != b"NES\x1a" {
        return json_error("Invalid NES ROM format");
    }

    let prg_banks = rom_bytes[4] as usize;
    let chr_banks = rom_bytes[5] as usize;
    let mapper = (rom_bytes[6] >> 4) | (rom_bytes[7] & 0xF0);
    let prg_size = prg_banks * 16384;

    if rom_bytes.len() < 16 + prg_size {
        return json_error("ROM file truncated");
    }

    let prg_rom = &rom_bytes[16..16 + prg_size];

    // Phase 1: Variable analysis
    let accesses = advanced::scan_zeropage_accesses(prg_rom);
    let variables = advanced::infer_variables(&accesses);

    // Phase 2: Function analysis
    let functions = advanced::infer_functions(prg_rom);

    // Phase 3: Generate pseudocode
    let pseudocode = advanced::generate_pseudocode(prg_rom, &variables, &functions);

    // Build JSON response
    let mut json = String::from("{\"ok\":true,");

    // ROM info
    json.push_str(&format!(
        "\"rom_info\":{{\"prg_banks\":{},\"chr_banks\":{},\"mapper\":{}}},",
        prg_banks, chr_banks, mapper
    ));

    // Variables
    json.push_str("\"variables\":[");
    let mut first = true;
    for (addr, var) in variables.iter() {
        if !first { json.push(','); }
        first = false;
        json.push_str(&format!(
            "{{\"addr\":{},\"name\":\"{}\",\"type\":\"{:?}\",\"access_count\":{},\"read_count\":{},\"write_count\":{},\"pattern\":\"{:?}\"}}",
            addr,
            escape_json(&var.name),
            var.var_type,
            var.access_count,
            var.read_count,
            var.write_count,
            var.access_pattern
        ));
    }
    json.push_str("],");

    // Functions
    json.push_str("\"functions\":[");
    first = true;
    for (addr, func) in functions.iter() {
        if !first { json.push(','); }
        first = false;
        json.push_str(&format!(
            "{{\"addr\":{},\"name\":\"{}\",\"size\":{},\"called_count\":{},\"calls_made\":{}}}",
            addr,
            escape_json(&func.name),
            func.size_bytes,
            func.called_from.len(),
            func.calls_to.len()
        ));
    }
    json.push_str("],");

    // Pseudocode
    json.push_str(&format!("\"pseudocode\":\"{}\"", escape_json(&pseudocode)));

    json.push('}');
    json
}

// Helper functions

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    // Simple base64 decoder
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let input = input.trim().replace(['\n', '\r', ' '], "");
    let mut output = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0;

    for ch in input.bytes() {
        if ch == b'=' { break; }

        let val = CHARS.iter().position(|&c| c == ch)
            .ok_or_else(|| format!("Invalid base64 character: {}", ch as char))?;

        buffer = (buffer << 6) | (val as u32);
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            output.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }

    Ok(output)
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}

fn json_error(msg: &str) -> String {
    format!("{{\"ok\":false,\"error\":\"{}\"}}", escape_json(msg))
}

/// Decompile a NES ROM with debug symbols back to NESL source code.
///
/// Input: ROM bytes as base64 string
/// Output: JSON with structure:
/// ```json
/// {
///   "ok": true,
///   "code": "// NESL source code..."
/// }
/// ```
/// or `{"ok": false, "error": "..."}` on failure
#[wasm_bindgen]
pub fn nesc_decompile_to_nesl(rom_base64: &str) -> String {
    // Note: This function is for future use when we can decompile without filesystem access
    // For now, return an error message
    json_error("Decompiling ROM to NESL requires symbol files. Please upload a .nesl source file instead.")
}
