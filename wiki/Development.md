# 参与开发

## 运行测试

```bash
cargo test --workspace        # 全部测试
cargo clippy --workspace --all-targets   # 保持零警告
```

## 测试怎么做

编译器测试不是"断言输出字符串"，而是**真的跑一遍**：

1. 把 NESL 源码编译成 ROM
2. 放进 `crates/nes-sim` 这台软件 NES 里执行若干帧
3. 断言 RAM / VRAM / OAM / APU 的最终状态

`nes-sim` 的 CPU 通过了 nestest 全部 5003 行官方指令校验，所以执行结果是可信的。

涉及浏览器的部分（`crates/nesc/tests/studio.rs`）会把页面里的内联脚本抽出来用 `node --check`
做语法校验，并跑 `web/studio.test.js` 的单元测试；没装 node 时自动跳过。

## 交叉验证

除了自己的测试机，还会拿 **jsnes**（另一个独立实现的模拟器）跑同一份 ROM：

```bash
cd tools/xcheck && npm ci
cargo test
```

逐像素比较最后一帧，截图写到 `target/xcheck/`。
两套独立实现结果一致，能挡掉"编译器和测试机一起错"这类问题。
没有 Node 或没装依赖时，这些测试自动跳过。

## TDD 与提交风格

git 历史里每个模块都是**先提交失败的测试**（`test:` 前缀），再提交实现（`feat:`）。
新功能建议沿用：

1. `test(xxx): failing checks for …`
2. `feat(xxx): …`
3. 需要时补 `docs(xxx): …`

## Spec Kit

`specs/` 下每个特性一个目录：

```
specs/013-mmc3/
  spec.md             需求（用户故事 + 验收标准）
  plan.md             实现计划
  tasks.md            任务清单
  contracts/          语言 / API 契约（长期有效的参考文档）
  IMPLEMENTATION.md   实现说明（部分特性有）
```

`.specify/` 里是 Spec Kit 配置。新特性可以直接用 `/speckit-*` 命令起头。

## 改动编辑器前端

前端资源在 `crates/nesc/web/`：

- `index.html` —— 页面 + 编辑器逻辑 + 模拟器集成
- `studio.js` —— 纯编辑函数（画像素、改地图、改调色板），同时被页面和 Node 测试复用

写完记得重新生成静态站：

```bash
bash tools/build-pages.sh
```

## 改编译器后要同步什么

- 新增内建函数：在 `crates/nesl/src/builtins.rs` 的 `DOCS` 表里加一行，
  编辑器的补全和悬浮提示会自动跟着更新，[内建函数参考](Builtins) 也是从这张表生成的。
- 新增语法：同时更新 `specs/` 里对应的 `contracts/language.md`。
