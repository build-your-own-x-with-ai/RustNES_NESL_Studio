# Implementation Plan: NESL Studio 编辑器增强

**Branch**: `008-studio` | **Date**: 2026-10-17 | **Spec**: [spec.md](./spec.md)

## Summary

可测试的逻辑放进 `crates/nesc/web/studio.js`（纯函数，UMD：浏览器挂到 `window.Studio`，Node 用 `require`），
页面 `index.html` 只负责界面。内建函数说明放在编译器里（`builtins::DOCS`），通过 `/api/builtins` 下发，
编辑器不再维护自己的函数名列表。

## Design

- `studio.js`：`findTiles / tileToCode / upsertTile`、`findMaps / setMapCell`、`findPalette / setPaletteEntry`、
  `wordAt`、`complete(prefix, builtins, source)`、`decodeChr(rom)`、`NES_RGB`（取自 jsnes 1.2.1 色表，与游戏画面一致）。
- 测试：`crates/nesc/web/studio.test.js`（`node --test`），由 `crates/nesc/tests/studio.rs` 调用；无 Node 时跳过。
  另有 Rust 测试用 Node 改写示例源码后编译，断言 CHR / 地图 / 调色板字节。
- 服务器：`/api/check` 只编译不返回 ROM；`/api/builtins` 返回 `{functions:[{name,sig,doc}], constants:[...], keywords:[...]}`。

## Project Structure

```text
crates/nesl/src/builtins.rs       DOCS + 一致性测试
crates/nesc/src/server.rs         新路由
crates/nesc/web/studio.js         编辑逻辑（纯函数）
crates/nesc/web/studio.test.js    Node 单元测试
crates/nesc/web/index.html        标签页、画板、补全、悬停、实时检查
crates/nesc/tests/studio.rs       运行 Node 测试 + 写回端到端
```
