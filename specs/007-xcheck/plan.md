# Implementation Plan: 第二模拟器交叉验证

**Branch**: `007-xcheck` | **Date**: 2026-10-17 | **Spec**: [spec.md](./spec.md)

## Summary

`nes-sim` 新增 `render` 模块：按逐扫描线的滚动状态（由 `reg_log` 重放）和当前 VRAM/OAM 画出颜色索引帧。
`tools/xcheck/run.js` 用 jsnes 跑 ROM 并输出最后一帧（RGB 整数）与 jsnes 的 64 色表；Rust 测试把我们的索引
经同一张色表转成 RGB 后逐像素比较，不一致时写 PNG。

## Design

- 水平滚动（`$2000` 第 0 位 + `$2005` x）：第 y 行使用「上一行第 257 点之前」的最后一次写入（硬件在该点复制）。
- 垂直滚动与命名表纵向位：取预渲染线（帧开始后 vblank 末）时的值。
- 帧 k = `TestNes::run_frames(N)` 后的 `N − 1`（最后一个画完的帧）；jsnes 调用 `frame()` N 次后的缓冲区。
- 依赖：jsnes 1.2.1（Apache-2.0，无传递依赖），精确版本 + `package-lock.json`，仅测试使用；`node_modules` 不入库。
- 宪章 V（零依赖）：Rust 侧仍零依赖；Node 依赖只在测试工具中，缺失时测试跳过。

## Project Structure

```text
crates/nes-sim/src/render.rs     渲染器 + 单元测试
tools/xcheck/{package.json,package-lock.json,run.js}
crates/nesl/tests/xcheck.rs      交叉验证（含最小 PNG 写入）
```
