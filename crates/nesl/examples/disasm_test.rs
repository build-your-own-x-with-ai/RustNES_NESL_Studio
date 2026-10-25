#!/usr/bin/env rust-script
//! 6502 反汇编器测试工具
//!
//! 使用方法：
//! ```bash
//! cargo run --bin disasm-test /tmp/game.nes
//! ```

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: {} <rom.nes>", args[0]);
        eprintln!("示例: {} /tmp/hello.nes", args[0]);
        process::exit(1);
    }

    let rom_path = &args[1];
    let rom = fs::read(rom_path).expect("无法读取 ROM 文件");

    // 跳过 iNES 头部（16 字节）
    if rom.len() < 16 || &rom[0..4] != b"NES\x1a" {
        eprintln!("错误：不是有效的 iNES ROM");
        process::exit(1);
    }

    let prg_banks = rom[4] as usize;
    let chr_banks = rom[5] as usize;

    println!("=== NES ROM 信息 ===");
    println!("PRG-ROM: {} x 16KB = {} KB", prg_banks, prg_banks * 16);
    println!("CHR-ROM: {} x 8KB = {} KB", chr_banks, chr_banks * 8);
    println!();

    // PRG-ROM 从字节 16 开始
    let prg_start = 16;
    let prg_size = prg_banks * 16384;
    let prg_rom = &rom[prg_start..prg_start + prg_size];

    println!("=== 反汇编 PRG-ROM ===");
    println!();

    // NES 程序通常从 $8000 开始（NROM mapper）
    let base_addr = 0x8000u16;

    let mut disasm = nesl::disasm::Disassembler::new(prg_rom);
    let mut addr = 0usize;
    let mut count = 0;

    // 反汇编前 100 条指令
    while addr < prg_rom.len() && count < 100 {
        if let Some((instr, size)) = disasm.decode_at(addr) {
            let rom_addr = base_addr.wrapping_add(addr as u16);

            // 显示字节码
            let bytes: Vec<String> = (0..size)
                .map(|i| format!("{:02X}", prg_rom[addr + i]))
                .collect();
            let bytes_str = bytes.join(" ");

            // 显示反汇编
            println!("{:04X}:  {:<12}  {}",
                rom_addr,
                bytes_str,
                instr.to_asm(rom_addr)
            );

            addr += size;
            count += 1;
        } else {
            break;
        }
    }

    println!();
    println!("反汇编了 {} 条指令", count);

    // 检查 NES 中断向量（ROM 的最后 6 字节）
    if prg_rom.len() >= 6 {
        let nmi_vector = u16::from_le_bytes([
            prg_rom[prg_rom.len() - 6],
            prg_rom[prg_rom.len() - 5],
        ]);
        let reset_vector = u16::from_le_bytes([
            prg_rom[prg_rom.len() - 4],
            prg_rom[prg_rom.len() - 3],
        ]);
        let irq_vector = u16::from_le_bytes([
            prg_rom[prg_rom.len() - 2],
            prg_rom[prg_rom.len() - 1],
        ]);

        println!();
        println!("=== 中断向量 ===");
        println!("NMI:   ${:04X}", nmi_vector);
        println!("RESET: ${:04X}", reset_vector);
        println!("IRQ:   ${:04X}", irq_vector);
    }
}
