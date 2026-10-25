//! Feature 019：SMB 字节级往返反编译验收测试。
//!
//! 验收标准（spec FR-4）：
//! 1. 指令表互逆，官方指令数 = 151
//! 2. SMB 反汇编 → 重汇编，.nes 文件逐字节一致
//! 3. 标题画面：原版与重建版帧逐像素一致，且画面非空白
//! 4. Attract 演示：检查点帧一致，且画面随时间变化（游戏在动）
//! 5. 操作脚本：Start 进入 1-1，右+B+间歇跳跃，帧一致且卷轴持续前进

use nes_sim::nes::TestNes;
use nes_sim::render;
use nesl::asm::{encode_opcode, opcode_table};
use nesl::disasm::asm_text::assemble_text;
use nesl::disasm::roundtrip::{disassemble, parse_ines};
use std::path::PathBuf;

/// 手柄按键位（nes-sim 移位序：A 先读出）。
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

/// 反汇编 + 重汇编，返回重建的 .nes 字节。
fn rebuild(rom: &[u8]) -> Vec<u8> {
    let img = parse_ines(rom).expect("解析 iNES");
    let asm = disassemble(&img).expect("反汇编");
    assemble_text(&asm, Some(&img.chr)).expect("重汇编")
}

/// 原版与重建版两台机器。
fn pair(rom: &[u8]) -> (TestNes, TestNes) {
    let rebuilt = rebuild(rom);
    let a = TestNes::from_ines(rom).expect("加载原版");
    let b = TestNes::from_ines(&rebuilt).expect("加载重建版");
    (a, b)
}

/// 两台机器同步输入并运行 `frames` 帧。
fn step_both(a: &mut TestNes, b: &mut TestNes, buttons: u8, frames: u64) {
    a.set_buttons(0, buttons);
    b.set_buttons(0, buttons);
    a.run_frames(frames).expect("原版运行");
    b.run_frames(frames).expect("重建版运行");
}

/// 对比两台机器当前帧，逐像素一致；返回原版帧。
fn assert_frames_equal(a: &TestNes, b: &TestNes, label: &str) -> Vec<u8> {
    let fa = render::frame(&a.bus, a.frames - 1);
    let fb = render::frame(&b.bus, b.frames - 1);
    let diff = fa.iter().zip(fb.iter()).filter(|(x, y)| x != y).count();
    assert_eq!(diff, 0, "{label}：原版与重建版帧有 {diff} 个像素不一致");
    fa
}

fn distinct_colors(frame: &[u8]) -> usize {
    let mut seen = [false; 64];
    for &p in frame {
        seen[p as usize] = true;
    }
    seen.iter().filter(|&&s| s).count()
}

// ---------------------------------------------------------------------------
// 测试 1：指令表互逆
// ---------------------------------------------------------------------------

#[test]
fn opcode_table_is_complete_and_bijective() {
    let table = opcode_table();
    let official = table.iter().filter(|e| e.is_some()).count();
    assert_eq!(official, 151, "6502 官方指令数应为 151");
    for (byte, entry) in table.iter().enumerate() {
        if let Some((op, mode)) = entry {
            assert_eq!(
                encode_opcode(*op, *mode),
                Some(byte as u8),
                "opcode ${byte:02X} 解码为 {op:?} {mode:?} 后应编码回同一字节"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 测试 2：往返逐字节一致
// ---------------------------------------------------------------------------

#[test]
fn smb_roundtrip_is_byte_identical() {
    let Some(rom) = smb_rom() else { return };
    let rebuilt = rebuild(&rom);
    assert_eq!(rebuilt.len(), rom.len(), "重建 ROM 大小");
    let first_diff = rom.iter().zip(rebuilt.iter()).position(|(a, b)| a != b);
    assert_eq!(first_diff, None, "重建 ROM 与原版应逐字节一致");
}

// ---------------------------------------------------------------------------
// 测试 3：标题画面帧一致
// ---------------------------------------------------------------------------

#[test]
fn smb_title_screen_frames_match() {
    let Some(rom) = smb_rom() else { return };
    let (mut a, mut b) = pair(&rom);
    step_both(&mut a, &mut b, 0, 30);
    assert_frames_equal(&a, &b, "第 30 帧");
    step_both(&mut a, &mut b, 0, 30);
    let title = assert_frames_equal(&a, &b, "第 60 帧（标题画面）");
    assert!(
        distinct_colors(&title) >= 5,
        "标题画面应有 ≥5 种颜色（非空白），实际 {}",
        distinct_colors(&title)
    );
}

// ---------------------------------------------------------------------------
// 测试 4：Attract 演示帧一致（实测：演示从第 557 帧起开始卷轴移动）
// ---------------------------------------------------------------------------

#[test]
fn smb_attract_demo_frames_match_and_animate() {
    let Some(rom) = smb_rom() else { return };
    let (mut a, mut b) = pair(&rom);
    step_both(&mut a, &mut b, 0, 60);
    let title = assert_frames_equal(&a, &b, "第 60 帧（标题）");
    step_both(&mut a, &mut b, 0, 540);
    assert_frames_equal(&a, &b, "第 600 帧（演示中）");
    step_both(&mut a, &mut b, 0, 100);
    let demo = assert_frames_equal(&a, &b, "第 700 帧（演示中）");
    let moved = title.iter().zip(demo.iter()).filter(|(x, y)| x != y).count();
    assert!(
        moved > 1000,
        "演示画面应与标题画面明显不同（游戏在动），实际差异 {moved} 像素"
    );
}

// ---------------------------------------------------------------------------
// 测试 5：操作脚本帧一致 + 卷轴前进（Mario 运动一致）
// ---------------------------------------------------------------------------

#[test]
fn smb_gameplay_script_frames_match_and_scroll_advances() {
    let Some(rom) = smb_rom() else { return };
    let (mut a, mut b) = pair(&rom);
    // 标题画面 → 按 Start → 等待 1-1 加载
    step_both(&mut a, &mut b, 0, 60);
    step_both(&mut a, &mut b, BTN_START, 4);
    step_both(&mut a, &mut b, 0, 200);
    assert_frames_equal(&a, &b, "第 264 帧（1-1 开场）");
    // 按住 右+B 跑动，每 60 帧按 A 跳 10 帧；记录原版卷轴值
    let mut scrolls = std::collections::BTreeSet::new();
    for f in 0..600u64 {
        let mut buttons = BTN_RIGHT | BTN_B;
        if f % 60 < 10 {
            buttons |= BTN_A;
        }
        step_both(&mut a, &mut b, buttons, 1);
        scrolls.insert(a.bus.scroll.0);
        if f == 200 || f == 400 || f == 599 {
            assert_frames_equal(&a, &b, &format!("操作第 {f} 帧"));
        }
    }
    assert!(
        scrolls.len() >= 30,
        "按住右跑动 600 帧，水平卷轴应持续前进（≥30 个不同值），实际 {}",
        scrolls.len()
    );
}
