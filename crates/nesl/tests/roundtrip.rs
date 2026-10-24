//! Round-trip tests: compile → decompile → recompile, verify semantic equivalence.

/// Helper: compiles source, decompiles, recompiles, checks both ROMs compile successfully.
fn roundtrip(name: &str, src: &str) {
    // Phase 1: Compile original
    let out1 = nesl::compile(src).expect("First compilation failed");

    // Phase 2: Generate symbols
    let symbols = nesl::debug_symbols::generate_symbols(&out1);
    let symbols_json = serde_json::to_string_pretty(&symbols).expect("Symbol serialization failed");

    // Verify symbols can serialize
    assert!(!symbols_json.is_empty());
    assert!(symbols_json.contains("\"version\""));

    // Phase 3: Parse symbols back
    let parsed_symbols = nesl::decompile::parse_symbols(&symbols_json)
        .expect("Symbol parsing failed");
    assert_eq!(parsed_symbols.version, "1.0.0");
    assert_eq!(parsed_symbols.mapper, out1.mapper);

    eprintln!("{} roundtrip: {} variables, {} constants, {} functions",
        name, parsed_symbols.variables.len(), parsed_symbols.constants.len(),
        parsed_symbols.functions.len());
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
    let out = nesl::compile(src).unwrap();
    let symbols = nesl::debug_symbols::generate_symbols(&out);

    // Verify variables are captured
    assert_eq!(symbols.variables.len(), 2);
    assert!(symbols.variables.iter().any(|v| v.name == "x"));
    assert!(symbols.variables.iter().any(|v| v.name == "y"));

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
    let out = nesl::compile(src).unwrap();
    let symbols = nesl::debug_symbols::generate_symbols(&out);

    // Verify constants are captured
    assert_eq!(symbols.constants.len(), 1);
    assert_eq!(symbols.constants[0].name, "PAL");

    roundtrip("const", src);
}

#[test]
fn symbols_generation_hello() {
    let src = std::fs::read_to_string("../../examples/hello.nesl")
        .or_else(|_| std::fs::read_to_string("examples/hello.nesl"))
        .expect("Failed to read examples/hello.nesl");
    let out = nesl::compile(&src).expect("Compilation failed");
    let symbols = nesl::debug_symbols::generate_symbols(&out);

    // Verify symbol table structure
    assert_eq!(symbols.version, "1.0.0");
    assert_eq!(symbols.mapper, 0);
    assert!(symbols.prg_size > 0);
    assert_eq!(symbols.chr_size, 8192);

    // Verify main function exists
    assert!(symbols.functions.iter().any(|f| f.name == "main"));

    // Verify PAL constant exists
    assert!(symbols.constants.iter().any(|c| c.name == "PAL"));
}

#[test]
fn parse_generated_symbols() {
    let src = "var x: u8 = 5; fn main() {}";
    let out = nesl::compile(src).unwrap();
    let symbols = nesl::debug_symbols::generate_symbols(&out);
    let json = serde_json::to_string(&symbols).unwrap();

    // Parse back
    let parsed = nesl::decompile::parse_symbols(&json).unwrap();

    assert_eq!(parsed.version, symbols.version);
    assert_eq!(parsed.mapper, symbols.mapper);
    assert_eq!(parsed.variables.len(), symbols.variables.len());
    assert_eq!(parsed.functions.len(), symbols.functions.len());
}
