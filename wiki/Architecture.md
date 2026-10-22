# 架构

## Crate 结构

```
crates/nes-sim     6502 CPU（通过 nestest 全部 5003 行官方指令校验）+ 简化 NES 测试机
crates/nesl        编译器库：lexer → parser → codegen → assembler → CHR / iNES
crates/nesc        CLI 与本地编辑器服务（std-only HTTP）+ web/ 前端资源
crates/nesc-wasm   把 nesl 编译成 WebAssembly，供浏览器编辑器使用
tools/             render-pages.mjs（生成静态站）、xcheck（交叉验证）
docs/              生成的静态站（GitHub Pages 产物）
specs/             Spec Kit 文档，每个特性一个目录
```

`nesl` **零外部依赖**，`nes-sim` 同理。整个项目只依赖标准库，`nesc-wasm` 额外依赖 `wasm-bindgen`。

## 编译流水线

```
NESL 源码
   │
   ├─ lexer.rs        词法：标识符、数字、字符串、注释
   ├─ parser.rs       语法 → AST（ast.rs），含类型检查
   ├─ builtins.rs     内建函数展开（37 个）+ 常量表
   ├─ codegen.rs      生成 6502 汇编（AST → asm）
   ├─ asm.rs          汇编器：标签解析、寻址模式、机器码
   ├─ chr.rs          tile / map / 字体 → CHR 图块数据
   ├─ mml.rs          MML 音乐 → APU 数据
   └─ rom.rs          组装 iNES 头 + PRG + CHR
   │
   └─→ .nes ROM
```

`nesl::compile()` 返回 ROM 字节、PRG 用量和汇编清单，同时给出带行列位置的诊断信息。

## 两个前端，一个编译器

编辑器的全部能力都通过 4 个 JSON 接口提供：

| 接口 | 作用 |
|---|---|
| `compile` | 编译 → ROM（base64）+ PRG 用量 + 汇编清单 |
| `check` | 只检查，返回诊断 |
| `examples` | 8 个内置示例源码 |
| `builtins` | 函数 / 常量 / 关键字，用于补全与悬浮提示 |

这套 payload 的构造代码在 `crates/nesc/src/json.rs`，**不含任何 I/O**，因此：

- 本地 `nesc serve` 走 HTTP（`crates/nesc/src/server.rs`，`std::net` 实现的最小 HTTP/1.1 服务）
- 浏览器走 `crates/nesc-wasm` 导出的 4 个 wasm 函数

两条路径返回**字节完全相同**的 JSON，前端用同一套代码消费。

`web/index.html` 里的后端选择逻辑：页面旁有 `pkg/` 就用 wasm，否则回落 HTTP。
所以本地 `nesc serve` 不需要构建 wasm，线上 Pages 也不需要服务器。

## 浏览器与 GitHub Pages

构建静态站：

```bash
bash tools/build-pages.sh
```

1. `wasm-pack build crates/nesc-wasm --release --target web` → `crates/nesc-wasm/pkg`
2. `node tools/render-pages.mjs` → 生成 `docs/`
   - 把 `studio.js` 内联进 `index.html`，页面自包含
   - 注入 wasm 加载器（动态 `import()`，失败自动回落 HTTP）
   - 资产路径全部相对化，保证在 `/<repo>/` 子路径下可用
   - 写 `.nojekyll`

产物约 596 KB（`index.html` 52 KB + wasm 493 KB）。

部署有两条路，二选一：

- **分支部署**：Pages 设置里选 "Deploy from a branch" → `main` / `/docs`，仓库里已提交 `docs/`
- **Actions 部署**：`.github/workflows/pages.yml` 每次 push 重新构建并 `upload-pages-artifact`

`configure-pages` 带了 `enablement: true`，仓库没开过 Pages 时也会自动启用。

## 数据流向图

```
                  ┌──────────────── 本地 ─────────────────┐
                  │  nesc serve ──→ nesc::json ──→ nesl   │
                  └───────────────────────────────────────┘
浏览器 ─── HTTP ──┘                                 │
                                                    │ 同一个 nesl
浏览器 ─── wasm ──┐                                 │
                  └── nesc-wasm ──→ nesc::json ─────┘
```
