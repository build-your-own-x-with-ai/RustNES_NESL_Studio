//! 008: the editor's JavaScript logic, tested with Node (skipped when Node is missing).
use std::path::Path;
use std::process::Command;

fn node_available() -> bool {
    Command::new("node").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

#[test]
fn studio_js_unit_tests() {
    if !node_available() {
        eprintln!("studio.js tests skipped: `node` not found");
        return;
    }
    let web = Path::new(env!("CARGO_MANIFEST_DIR")).join("web");
    let out = Command::new("node").current_dir(&web).args(["--test", "studio.test.js"]).output().unwrap();
    assert!(
        out.status.success(),
        "studio.test.js failed:\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Runs `expr` (with `S` = studio.js and `src` = the example's source) in Node and
/// returns (original source, edited source).
fn edit(example: &str, expr: &str) -> Option<(String, String)> {
    if !node_available() {
        eprintln!("studio edit test skipped: `node` not found");
        return None;
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let inp = root.join(format!("../../examples/{example}.nesl"));
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("studio");
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join(format!("{example}-{}.nesl", expr.len()));
    let status = Command::new("node")
        .arg(root.join("tests/studio_edit.js"))
        .arg(root.join("web/studio.js"))
        .arg(&inp)
        .arg(&out)
        .arg(expr)
        .output()
        .unwrap();
    assert!(status.status.success(), "{}", String::from_utf8_lossy(&status.stderr));
    Some((std::fs::read_to_string(inp).unwrap(), std::fs::read_to_string(out).unwrap()))
}

fn rom_byte(out: &nesl::CompileOutput, label: &str, off: u16) -> u8 {
    out.rom[16 + (out.symbols[label] + off - 0x8000) as usize]
}

const CHR: usize = 16 + 32768;

#[test]
fn painting_a_tile_pixel_changes_one_line_and_the_chr() {
    let Some((before, after)) = edit(
        "sprite",
        "(() => { const t = S.findTiles(src).find((t) => t.index === 1); const p = t.pixels.map((r) => r.slice()); p[0][0] = 3; return S.upsertTile(src, t.indexText, p); })()",
    ) else { return };
    let changed = before.lines().zip(after.lines()).filter(|(a, b)| a != b).count();
    assert_eq!((changed, before.lines().count()), (1, after.lines().count()));
    let old = nesl::compile(&before).unwrap().rom;
    let new = nesl::compile(&after).unwrap().rom;
    assert_eq!((old[CHR + 16] & 0x80, old[CHR + 24] & 0x80), (0, 0));
    assert_eq!((new[CHR + 16] & 0x80, new[CHR + 24] & 0x80), (0x80, 0x80), "colour 3 in both planes");
}

#[test]
fn a_new_tile_is_appended_and_compiles() {
    let Some((before, after)) = edit("hello", "S.upsertTile(src, '7', [[3,0,0,0,0,0,0,1], ...Array(7).fill([0,0,0,0,0,0,0,0])])") else { return };
    assert!(after.starts_with(&before));
    let rom = nesl::compile(&after).unwrap().rom;
    assert_eq!((rom[CHR + 7 * 16], rom[CHR + 7 * 16 + 8]), (0x81, 0x80));
}

#[test]
fn painting_a_map_cell_changes_the_map_data() {
    let Some((before, after)) = edit("maze", "S.setMapCell(src, 'LEVEL', 2, 5, '#')") else { return };
    // maps are stored column-major: column c, row r at c * 30 + r
    assert_eq!(rom_byte(&nesl::compile(&before).unwrap(), "m_LEVEL", 2 * 30 + 5), b' ');
    assert_eq!(rom_byte(&nesl::compile(&after).unwrap(), "m_LEVEL", 2 * 30 + 5), b'#');
}

#[test]
fn picking_a_palette_colour_changes_the_palette_data() {
    let Some((before, after)) = edit("hello", "S.setPaletteEntry(src, 1, 0x2A)") else { return };
    assert_eq!(rom_byte(&nesl::compile(&before).unwrap(), "d_PAL", 1), 0x30);
    assert_eq!(rom_byte(&nesl::compile(&after).unwrap(), "d_PAL", 1), 0x2A);
    assert_eq!(before.lines().count(), after.lines().count());
}
