//! Ultimate decompile command - 生成完全可运行的 NESL

use std::path::Path;
use nesl::decompile::ultimate;

pub fn ultimate_decompile(rom_path: &Path, output: Option<&Path>) -> Result<(), String> {
    ultimate::ultimate_decompile(rom_path, output)
}
