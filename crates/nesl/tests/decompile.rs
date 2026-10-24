//! Round-trip decompilation tests (compile → decompile → recompile).

#[test]
fn decompile_error_display() {
    use nesl::decompile::DecompileError;

    let e = DecompileError::RomNotFound {
        path: "test.nes".into(),
    };
    let msg = format!("{}", e);
    assert!(msg.contains("找不到 ROM 文件"));
    assert!(msg.contains("test.nes"));

    let e = DecompileError::UnsupportedMapper { mapper: 7 };
    let msg = format!("{}", e);
    assert!(msg.contains("不支持的 mapper: 7"));
}

#[test]
fn rom_parser_rejects_invalid_files() {
    use nesl::decompile::rom_parser;
    use std::io::Write;
    use std::path::PathBuf;

    let tmp = std::env::temp_dir();
    let path = tmp.join("invalid.nes");

    // Too small
    std::fs::write(&path, b"NES").unwrap();
    assert!(rom_parser::load_rom(&path).is_err());

    // Wrong magic
    std::fs::write(&path, b"WRONG_MAGIC_NUM!").unwrap();
    assert!(rom_parser::load_rom(&path).is_err());

    std::fs::remove_file(&path).ok();
}

#[test]
fn symbol_loader_discovers_files() {
    use nesl::decompile::symbol_loader;
    use std::path::PathBuf;

    let tmp = std::env::temp_dir();
    let rom_path = tmp.join("test.nes");
    let syms_path = tmp.join("test.syms");

    // Create dummy .syms file
    std::fs::write(&syms_path, "{}").unwrap();

    let discovered = symbol_loader::discover_symbols(&rom_path);
    assert_eq!(discovered, Some(syms_path.clone()));

    std::fs::remove_file(&syms_path).ok();
}
