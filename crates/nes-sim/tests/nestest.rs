//! Runs nestest.nes in automation mode ($C000) and compares CPU state against the
//! reference log for every official-opcode line. Skips if the data files are missing.

use nes_sim::TestNes;
use std::path::PathBuf;

struct LogLine {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    sp: u8,
    cyc: u64,
    unofficial: bool,
}

fn hex8(line: &str, key: &str) -> u8 {
    let i = line.find(key).unwrap_or_else(|| panic!("missing {key} in {line}")) + key.len();
    u8::from_str_radix(&line[i..i + 2], 16).unwrap()
}

fn parse(line: &str) -> LogLine {
    let cyc_i = line.find("CYC:").unwrap() + 4;
    LogLine {
        pc: u16::from_str_radix(&line[0..4], 16).unwrap(),
        a: hex8(line, "A:"),
        x: hex8(line, "X:"),
        y: hex8(line, "Y:"),
        p: hex8(line, "P:"),
        sp: hex8(line, "SP:"),
        cyc: line[cyc_i..].trim().parse().unwrap(),
        unofficial: line.as_bytes().get(15) == Some(&b'*'),
    }
}

#[test]
fn nestest_official_opcodes() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data");
    let (rom, log) = match (
        std::fs::read(dir.join("nestest.nes")),
        std::fs::read_to_string(dir.join("nestest.log")),
    ) {
        (Ok(r), Ok(l)) => (r, l),
        _ => {
            println!("note: nestest data files missing in {}; skipping", dir.display());
            return;
        }
    };

    let mut nes = TestNes::from_ines(&rom).unwrap();
    nes.cpu.pc = 0xC000;
    nes.cpu.cycles = 7;

    let mut matched = 0;
    for (n, raw) in log.lines().enumerate() {
        let exp = parse(raw);
        if exp.unofficial {
            break;
        }
        let c = &nes.cpu;
        let got = (c.pc, c.a, c.x, c.y, c.p, c.sp, c.cycles);
        let want = (exp.pc, exp.a, exp.x, exp.y, exp.p, exp.sp, exp.cyc);
        assert_eq!(
            got,
            want,
            "mismatch at log line {} (after {matched} matching lines)\n  expected: {raw}\n  got: PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
            n + 1,
            got.0, got.1, got.2, got.3, got.4, got.5, got.6
        );
        matched += 1;
        nes.cpu.step(&mut nes.bus).unwrap();
    }
    println!("nestest: {matched} official-opcode lines matched");
    assert!(matched >= 5000, "only {matched} lines checked");
    // nestest reports official-opcode failures in $02 (0 = all passed so far).
    assert_eq!(nes.peek(0x02), 0, "nestest error code in $02");
}
