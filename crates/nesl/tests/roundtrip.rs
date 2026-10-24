//! Round-trip tests: compile → decompile → recompile, verify semantic equivalence.

mod common;
use common::*;

/// Helper: compiles source, decompiles, recompiles, checks both ROMs have same behavior.
fn roundtrip(name: &str, src: &str) {
    // Phase 1: Compile original
    let out1 = nesl::compile(src).expect("First compilation failed");

    // Phase 2: Generate symbols (stub for now)
    let symbols = nesl::debug_symbols::generate_symbols(&out1);
    let symbols_json = serde_json::to_string_pretty(&symbols).expect("Symbol serialization failed");

    // Phase 3: Decompile (will be implemented in US1)
    // let decompiled_src = nesl::decompile::decompile(...);

    // Phase 4: Recompile
    // let out2 = nesl::compile(&decompiled_src).expect("Second compilation failed");

    // Phase 5: Verify semantic equivalence
    // assert_eq!(out1.mapper, out2.mapper);
    // assert_eq!(out1.prg_used, out2.prg_used);

    // For now, just verify symbols can serialize
    assert!(!symbols_json.is_empty());
    assert!(symbols_json.contains("\"version\""));

    eprintln!("{} symbols: {} bytes", name, symbols_json.len());
}

#[test]
fn hello_roundtrip() {
    let src = r#"
        fn main() {
            ppu_off();
            print(9, 3, "HELLO NES!");
            ppu_on();
            loop { wait_vblank(); }
        }
    "#;
    roundtrip("hello", src);
}

#[test]
fn simple_variable_roundtrip() {
    let src = r#"
        var x: u8 = 120;
        var y: u16 = 1000;

        fn main() {
            x += 5;
            y -= 100;
        }
    "#;
    roundtrip("vars", src);
}

#[test]
fn const_array_roundtrip() {
    let src = r#"
        const PAL: [u8] = [0x0F, 0x30, 0x16, 0x27];

        fn main() {
            load_palette(PAL);
        }
    "#;
    roundtrip("const", src);
}
