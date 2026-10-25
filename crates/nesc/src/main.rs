//! `nesc` — NESL command line compiler and browser editor.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
nesc — compile NESL programs into NES ROMs

USAGE:
    nesc build <file.nesl> [-o <out.nes>] [--asm] [--debug-symbols]
    nesc check <file.nesl>
    nesc decompile <file.nes> [-s <symbols.syms>] [-o <out.nesl>]
    nesc disasm <file.nes> [-o <out.asm>]
    nesc pseudocode <file.nes> [-o <out.c>]
    nesc enhanced-pseudocode <file.nes> [-o <out.c>]
    nesc complete-decompile <file.nes> [-o <out.nesl>]
    nesc final-decompile <file.nes> [-o <out.nesl>]
    nesc analyze-data <file.nes>
    nesc generate-nesl <file.nes> [-o <out.nesl>]
    nesc serve [--port <N>]
    nesc help";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let Some(cmd) = args.first() else {
        return Err(USAGE.into());
    };
    let rest = &args[1..];
    match cmd.as_str() {
        "build" | "check" => {
            let mut input = None;
            let mut output = None;
            let mut asm = false;
            let mut debug_symbols = false;
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "-o" | "--output" => output = Some(PathBuf::from(it.next().ok_or("`-o` needs a path")?)),
                    "--asm" => asm = true,
                    "--debug-symbols" => debug_symbols = true,
                    s if s.starts_with('-') => return Err(format!("unknown option `{s}`\n\n{USAGE}")),
                    s => input = Some(PathBuf::from(s)),
                }
            }
            let input = input.ok_or_else(|| format!("missing input file\n\n{USAGE}"))?;
            build(&input, output, asm, debug_symbols, cmd == "check")
        }
        "decompile" => {
            let mut input = None;
            let mut symbols = None;
            let mut output = None;
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "-s" | "--symbols" => symbols = Some(PathBuf::from(it.next().ok_or("`-s` needs a path")?)),
                    "-o" | "--output" => output = Some(PathBuf::from(it.next().ok_or("`-o` needs a path")?)),
                    s if s.starts_with('-') => return Err(format!("unknown option `{s}`\n\n{USAGE}")),
                    s => input = Some(PathBuf::from(s)),
                }
            }
            let input = input.ok_or_else(|| format!("missing ROM file\n\n{USAGE}"))?;
            nesc::cmd::decompile::decompile(&input, symbols.as_deref(), output.as_deref())
        }
        "disasm" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "-o" | "--output" => output = Some(PathBuf::from(it.next().ok_or("`-o` needs a path")?)),
                    s if s.starts_with('-') => return Err(format!("unknown option `{s}`\n\n{USAGE}")),
                    s => input = Some(PathBuf::from(s)),
                }
            }
            let input = input.ok_or_else(|| format!("missing ROM file\n\n{USAGE}"))?;
            nesc::cmd::disasm::disasm(&input, output.as_deref())
        }
        "pseudocode" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "-o" | "--output" => output = Some(PathBuf::from(it.next().ok_or("`-o` needs a path")?)),
                    s if s.starts_with('-') => return Err(format!("unknown option `{s}`\n\n{USAGE}")),
                    s => input = Some(PathBuf::from(s)),
                }
            }
            let input = input.ok_or_else(|| format!("missing ROM file\n\n{USAGE}"))?;
            nesc::cmd::pseudocode::pseudocode(&input, output.as_deref())
        }
        "enhanced-pseudocode" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "-o" | "--output" => output = Some(PathBuf::from(it.next().ok_or("`-o` needs a path")?)),
                    s if s.starts_with('-') => return Err(format!("unknown option `{s}`\n\n{USAGE}")),
                    s => input = Some(PathBuf::from(s)),
                }
            }
            let input = input.ok_or_else(|| format!("missing ROM file\n\n{USAGE}"))?;
            nesc::cmd::enhanced_pseudocode::enhanced_pseudocode(&input, output.as_deref())
        }
        "complete-decompile" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "-o" | "--output" => output = Some(PathBuf::from(it.next().ok_or("`-o` needs a path")?)),
                    s if s.starts_with('-') => return Err(format!("unknown option `{s}`\n\n{USAGE}")),
                    s => input = Some(PathBuf::from(s)),
                }
            }
            let input = input.ok_or_else(|| format!("missing ROM file\n\n{USAGE}"))?;
            nesc::cmd::complete_decompile::complete_decompile(&input, output.as_deref())
        }
        "final-decompile" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "-o" | "--output" => output = Some(PathBuf::from(it.next().ok_or("`-o` needs a path")?)),
                    s if s.starts_with('-') => return Err(format!("unknown option `{s}`\n\n{USAGE}")),
                    s => input = Some(PathBuf::from(s)),
                }
            }
            let input = input.ok_or_else(|| format!("missing ROM file\n\n{USAGE}"))?;
            nesc::cmd::final_decompile::final_decompile(&input, output.as_deref())
        }
        "analyze-data" => {
            let mut input: Option<PathBuf> = None;
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    s if s.starts_with('-') => return Err(format!("unknown option `{s}`\n\n{USAGE}")),
                    s => input = Some(PathBuf::from(s)),
                }
            }
            let input = input.ok_or_else(|| format!("missing ROM file\n\n{USAGE}"))?;
            nesc::cmd::analyze_data::analyze_data(&input)
        }
        "generate-nesl" => {
            let mut input: Option<PathBuf> = None;
            let mut output: Option<PathBuf> = None;
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "-o" | "--output" => output = Some(PathBuf::from(it.next().ok_or("`-o` needs a path")?)),
                    s if s.starts_with('-') => return Err(format!("unknown option `{s}`\n\n{USAGE}")),
                    s => input = Some(PathBuf::from(s)),
                }
            }
            let input = input.ok_or_else(|| format!("missing ROM file\n\n{USAGE}"))?;
            nesc::cmd::generate_nesl::generate_nesl(&input, output.as_deref())
        }
        #[cfg(not(target_arch = "wasm32"))]
        "serve" => {
            let mut port = 8080u16;
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "--port" | "-p" => {
                        port = it.next().and_then(|p| p.parse().ok()).ok_or("`--port` needs a number")?;
                    }
                    s => return Err(format!("unknown option `{s}`\n\n{USAGE}")),
                }
            }
            nesc::server::serve(port).map_err(|e| format!("server error: {e}"))
        }
        "help" | "--help" | "-h" => {
            println!("{USAGE}");
            Ok(())
        }
        other => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    }
}

fn build(input: &Path, output: Option<PathBuf>, asm: bool, debug_symbols: bool, check_only: bool) -> Result<(), String> {
    let src = std::fs::read_to_string(input).map_err(|e| format!("cannot read {}: {e}", input.display()))?;
    let out = nesl::compile(&src).map_err(|errs| {
        errs.iter()
            .map(|d| format!("{}:{}:{}: error: {}", input.display(), d.line, d.col, d.message))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    if check_only {
        println!("{}: ok ({} bytes PRG used)", input.display(), out.prg_used);
        return Ok(());
    }
    let path = output.unwrap_or_else(|| input.with_extension("nes"));
    std::fs::write(&path, &out.rom).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    if asm {
        let lst = path.with_extension("s");
        std::fs::write(&lst, &out.listing).map_err(|e| format!("cannot write {}: {e}", lst.display()))?;
        println!("wrote {}", lst.display());
    }
    if debug_symbols {
        let syms = path.with_extension("nes.syms");
        let table = nesl::debug_symbols::generate_symbols(&out);
        let json = serde_json::to_string_pretty(&table)
            .map_err(|e| format!("cannot serialize symbols: {}", e))?;
        std::fs::write(&syms, json)
            .map_err(|e| format!("cannot write {}: {e}", syms.display()))?;
        println!("wrote {}", syms.display());
    }
    println!("wrote {} ({} bytes PRG used)", path.display(), out.prg_used);
    Ok(())
}
