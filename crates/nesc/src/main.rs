//! `nesc` — NESL command line compiler and browser editor.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
nesc — compile NESL programs into NES ROMs

USAGE:
    nesc build <file.nesl> [-o <out.nes>] [--asm]
    nesc check <file.nesl>
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
            let mut it = rest.iter();
            while let Some(a) = it.next() {
                match a.as_str() {
                    "-o" | "--output" => output = Some(PathBuf::from(it.next().ok_or("`-o` needs a path")?)),
                    "--asm" => asm = true,
                    s if s.starts_with('-') => return Err(format!("unknown option `{s}`\n\n{USAGE}")),
                    s => input = Some(PathBuf::from(s)),
                }
            }
            let input = input.ok_or_else(|| format!("missing input file\n\n{USAGE}"))?;
            build(&input, output, asm, cmd == "check")
        }
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

fn build(input: &Path, output: Option<PathBuf>, asm: bool, check_only: bool) -> Result<(), String> {
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
    println!("wrote {} ({} bytes PRG used)", path.display(), out.prg_used);
    Ok(())
}
