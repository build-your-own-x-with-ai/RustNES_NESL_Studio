//! Feature 020：SMB → C 静态重编译验收测试。
//!
//! 验收标准（spec FR-4）：
//! 1. generate_c() 产出，`cc` 编译零错误
//! 2. 标题/演示：C 版快照经 nes-sim 渲染，与原版第 30/60/600/700 帧
//!    逐像素一致；演示画面与标题差异 >1000 像素
//! 3. 操作脚本：Start→1-1→右+B+间歇跳，检查点帧逐像素一致；
//!    原版卷轴 ≥30 个不同值（运动真实发生）
//! 4. 可读性：switch 跳转表、≥100 个函数、经验证符号表被使用

use nes_sim::nes::TestNes;
use nes_sim::render;
use nesl::decompile::c_codegen::generate_c;
use nesl::disasm::roundtrip::parse_ines;
use std::path::{Path, PathBuf};
use std::process::Command;

const BTN_A: u8 = 0x80;
const BTN_B: u8 = 0x40;
const BTN_START: u8 = 0x10;
const BTN_RIGHT: u8 = 0x01;

fn smb_rom() -> Option<Vec<u8>> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../roms/FC_ROMS-main/ROM/0066/Super Mario Bros. (W) [!].nes");
    match std::fs::read(&p) {
        Ok(b) => Some(b),
        Err(_) => {
            eprintln!("跳过：找不到 SMB ROM：{}", p.display());
            None
        }
    }
}

fn workdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("smb_c_{name}_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 生成 C 源码并用 cc 编译，返回可执行文件路径。
fn build_c(rom: &[u8], dir: &Path) -> PathBuf {
    let img = parse_ines(rom).expect("解析 iNES");
    let c = generate_c(&img).expect("生成 C 源码");
    let src = dir.join("smb.c");
    std::fs::write(&src, &c).unwrap();
    let bin = dir.join("smb_bin");
    let out = Command::new("cc")
        .arg("-O1")
        .arg("-o")
        .arg(&bin)
        .arg(&src)
        .env("DEVELOPER_DIR", "/Library/Developer/CommandLineTools")
        .output()
        .expect("无法运行 cc");
    assert!(
        out.status.success(),
        "cc 编译失败：\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    bin
}

/// C 侧快照（与生成的 C 运行时的 dump 格式对应）。
struct Snap {
    frames: u64,
    vram: Vec<u8>,
    oam: Vec<u8>,
    ppuctrl: u8,
    ppumask: u8,
    reg_log: Vec<(u64, u8, u8)>,
}

fn read_snap(path: &Path) -> Snap {
    let b = std::fs::read(path).unwrap_or_else(|e| panic!("读取快照 {}: {e}", path.display()));
    assert!(b.len() >= 4 + 8 + 8 + 0x800 + 0x4000 + 256 + 2 + 4, "快照太小：{}", b.len());
    assert_eq!(&b[0..4], b"SMBC", "快照 magic");
    let u64_at = |o: usize| u64::from_le_bytes(b[o..o + 8].try_into().unwrap());
    let frames = u64_at(4);
    let mut o = 4 + 8 + 8; // magic + frames + cyc
    o += 0x800; // ram（渲染不需要，跳过）
    let vram = b[o..o + 0x4000].to_vec();
    o += 0x4000;
    let oam = b[o..o + 256].to_vec();
    o += 256;
    let ppuctrl = b[o];
    let ppumask = b[o + 1];
    o += 2;
    let nlog = u32::from_le_bytes(b[o..o + 4].try_into().unwrap()) as usize;
    o += 4;
    let mut reg_log = Vec::with_capacity(nlog);
    for i in 0..nlog {
        let e = o + i * 10;
        reg_log.push((u64_at(e), b[e + 8], b[e + 9]));
    }
    Snap { frames, vram, oam, ppuctrl, ppumask, reg_log }
}

/// 用 nes-sim 的渲染器渲染 C 侧快照。
fn render_snap(rom: &[u8], s: &Snap) -> Vec<u8> {
    let mut nes = TestNes::from_ines(rom).expect("加载 ROM（取镜像/CHR 配置）");
    nes.bus.vram = s.vram.clone();
    nes.bus.oam.copy_from_slice(&s.oam);
    nes.bus.ppuctrl = s.ppuctrl;
    nes.bus.ppumask = s.ppumask;
    nes.bus.reg_log = s.reg_log.clone();
    render::frame(&nes.bus, s.frames - 1)
}

/// 原版 ROM 按同一脚本运行：在帧 F（NMI 体执行前）先采样再更新按键。
/// 返回各检查点的渲染帧，以及游戏期间的卷轴值集合。
fn run_original(
    rom: &[u8],
    btns: &[(u64, u8)],
    checkpoints: &[u64],
) -> (Vec<(u64, Vec<u8>)>, std::collections::BTreeSet<u8>) {
    let mut nes = TestNes::from_ines(rom).expect("加载原版");
    let last = *checkpoints.iter().max().unwrap();
    let mut shots = Vec::new();
    let mut scrolls = std::collections::BTreeSet::new();
    for f in 0..=last {
        if checkpoints.contains(&f) {
            shots.push((f, render::frame(&nes.bus, f - 1)));
        }
        if let Some(&(_, mask)) = btns.iter().find(|&&(bf, _)| bf == f) {
            nes.set_buttons(0, mask);
        }
        if f < last {
            nes.run_frames(1).expect("原版运行");
            scrolls.insert(nes.bus.scroll.0);
        }
    }
    (shots, scrolls)
}

/// 生成 C 侧脚本文本。
fn script_text(btns: &[(u64, u8)], checkpoints: &[u64], dir: &Path) -> (String, Vec<PathBuf>) {
    let mut lines = Vec::new();
    let mut paths = Vec::new();
    for &(f, m) in btns {
        lines.push(format!("btn {f} {m}"));
    }
    for &f in checkpoints {
        let p = dir.join(format!("snap_{f}.bin"));
        lines.push(format!("dump {f} {}", p.display()));
        paths.push(p);
    }
    let last = *checkpoints.iter().max().unwrap();
    lines.push(format!("end {}", last + 1));
    (lines.join("\n") + "\n", paths)
}

/// 完整对比流程：构建 C 版，双方跑同一脚本，检查点逐像素对比。
fn compare_run(
    rom: &[u8],
    dir: &Path,
    btns: &[(u64, u8)],
    checkpoints: &[u64],
) -> (Vec<(u64, Vec<u8>)>, std::collections::BTreeSet<u8>) {
    let bin = build_c(rom, dir);
    let (script, snap_paths) = script_text(btns, checkpoints, dir);
    let script_path = dir.join("script.txt");
    std::fs::write(&script_path, script).unwrap();
    let out = Command::new(&bin).arg(&script_path).output().expect("运行 C 版");
    assert!(
        out.status.success(),
        "C 版运行失败：\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let (orig_shots, scrolls) = run_original(rom, btns, checkpoints);
    for ((f, orig), snap_path) in orig_shots.iter().zip(&snap_paths) {
        let snap = read_snap(snap_path);
        assert_eq!(snap.frames, *f, "快照帧号");
        let c_frame = render_snap(rom, &snap);
        let diff = orig.iter().zip(c_frame.iter()).filter(|(a, b)| a != b).count();
        assert_eq!(diff, 0, "第 {f} 帧：C 版与原版有 {diff} 个像素不一致");
    }
    (orig_shots, scrolls)
}

fn distinct_colors(frame: &[u8]) -> usize {
    let mut seen = [false; 64];
    for &p in frame {
        seen[p as usize] = true;
    }
    seen.iter().filter(|&&s| s).count()
}

// ---------------------------------------------------------------------------
// 测试 1：生成 + cc 编译零错误
// ---------------------------------------------------------------------------

#[test]
fn smb_c_generates_and_compiles() {
    let Some(rom) = smb_rom() else { return };
    let dir = workdir("gen");
    build_c(&rom, &dir);
}

// ---------------------------------------------------------------------------
// 测试 2：标题 + attract 演示帧一致
// ---------------------------------------------------------------------------

#[test]
fn smb_c_title_and_demo_frames_match() {
    let Some(rom) = smb_rom() else { return };
    let dir = workdir("title");
    let (shots, _) = compare_run(&rom, &dir, &[], &[30, 60, 600, 700]);
    let title = &shots.iter().find(|(f, _)| *f == 60).unwrap().1;
    let demo = &shots.iter().find(|(f, _)| *f == 700).unwrap().1;
    assert!(distinct_colors(title) >= 5, "标题画面应非空白");
    let moved = title.iter().zip(demo.iter()).filter(|(a, b)| a != b).count();
    assert!(moved > 1000, "演示应与标题明显不同（游戏在动），实际差异 {moved} 像素");
}

// ---------------------------------------------------------------------------
// 测试 3：操作脚本帧一致 + 卷轴前进
// ---------------------------------------------------------------------------

#[test]
fn smb_c_gameplay_frames_match_and_scroll_advances() {
    let Some(rom) = smb_rom() else { return };
    let dir = workdir("play");
    // Start → 等待 1-1 → 按住右+B，每 60 帧按 A 跳 10 帧
    let mut btns: Vec<(u64, u8)> = vec![(60, BTN_START), (64, 0)];
    let mut prev = 0u8;
    for f in 264..864u64 {
        let mut m = BTN_RIGHT | BTN_B;
        if (f - 264) % 60 < 10 {
            m |= BTN_A;
        }
        if m != prev {
            btns.push((f, m));
            prev = m;
        }
    }
    let (_, scrolls) = compare_run(&rom, &dir, &btns, &[264, 464, 664, 863]);
    assert!(
        scrolls.len() >= 30,
        "按住右跑动期间水平卷轴应持续前进（≥30 个不同值），实际 {}",
        scrolls.len()
    );
}

// ---------------------------------------------------------------------------
// 测试 4：可读性
// ---------------------------------------------------------------------------

#[test]
fn smb_c_output_is_readable() {
    let Some(rom) = smb_rom() else { return };
    let img = parse_ines(&rom).unwrap();
    let c = generate_c(&img).expect("生成 C 源码");
    let funcs = c.matches("\nvoid sub_").count();
    assert!(funcs >= 100, "应生成 ≥100 个函数，实际 {funcs}");
    assert!(c.contains("switch ("), "JumpEngine 跳转表应翻译为 switch");
    for sym in ["FrameCounter", "OperMode", "Player_X_Position", "Player_Y_Position"] {
        assert!(c.contains(&format!("#define {sym}")), "缺少符号定义 {sym}");
        assert!(c.contains(&format!("ram[{sym}]")), "符号 {sym} 未被使用");
    }
}
