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
