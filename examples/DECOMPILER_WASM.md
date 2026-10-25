# NES ROM 反编译器 - WASM 版

基于 Rust + WebAssembly 的高性能 NES ROM 反编译器，完全在浏览器中运行。

## 功能特性

- ✨ **完全浏览器端运行** - 无需服务器，数据不上传
- 🚀 **Rust 性能** - WebAssembly 提供接近原生的速度
- 🔒 **隐私保护** - ROM 文件仅在本地处理
- 📊 **真实分析** - 使用完整的 Rust 反编译引擎

## 分析能力

### Phase 1: 变量识别
- Zero Page 内存访问扫描
- 自动类型推断（U8/U16/数组）
- 访问模式识别（Counter/Pointer/Flag）
- 智能变量命名

### Phase 2: 函数识别
- JSR/RTS 调用图分析
- 中断向量识别（NMI/RESET/IRQ）
- 函数边界检测
- 调用关系图

### Phase 3: 伪代码生成
- 35+ 6502 指令翻译
- C-like 可读输出
- 语法高亮
- 下载功能

## 使用方法

### 在线使用（推荐）

1. 启动本地服务器：
   ```bash
   cd /path/to/RustNES
   python3 -m http.server 8000
   ```

2. 打开浏览器访问：
   ```
   http://localhost:8000/examples/decompiler_wasm.html
   ```

3. 上传 NES ROM 文件（.nes）

4. 查看分析结果：
   - 📊 概览 - 统计数据
   - 🔢 变量分析 - Zero Page 变量表
   - ⚙️ 函数分析 - 函数调用关系
   - 📝 伪代码 - C-like 输出

### 本地构建

如需重新构建 WASM 模块：

```bash
cd crates/nesc-wasm
wasm-pack build --target web --release
```

生成的文件在 `pkg/` 目录：
- `nesc_wasm.js` - JavaScript 绑定
- `nesc_wasm_bg.wasm` - WebAssembly 模块

## 测试案例

### sprite.nes (NESL 示例)
- 26 个变量识别
- 29 个函数识别
- player_x, player_y 自动命名

### Solomon's Key (商业 ROM)
- 243 个变量识别
- 183 个函数识别
- 12KB 伪代码生成

## 技术栈

- **Rust** - 核心反编译引擎
- **wasm-bindgen** - Rust ↔ JavaScript 绑定
- **WebAssembly** - 高性能运行时
- **纯 JavaScript** - 无框架，轻量级 UI

## API

WASM 导出的函数：

```javascript
import init, { nesc_decompile } from './pkg/nesc_wasm.js';

await init();

// 输入: ROM 字节的 base64 编码
// 输出: JSON 格式的分析结果
const result = nesc_decompile(romBase64);
const data = JSON.parse(result);

// data.variables - 变量列表
// data.functions - 函数列表
// data.pseudocode - 生成的伪代码
// data.rom_info - ROM 元数据
```

## 性能

- **WASM 加载**: ~100ms
- **分析 32KB ROM**: ~50-200ms
- **UI 渲染**: 即时

比纯 JavaScript 实现快 **5-10 倍**。

## 限制

- 需要现代浏览器（支持 WebAssembly）
- 需要本地服务器（CORS 限制）
- WASM 模块大小 ~540KB（首次加载）

## 对比

| 版本 | 引擎 | 性能 | 离线 |
|------|------|------|------|
| decompiler_web.html | JavaScript 模拟 | 慢 | ✅ |
| decompiler_wasm.html | Rust WASM | 快 | ✅ |
| nesc pseudocode | Rust 原生 | 最快 | ✅ |

## 未来改进

- [ ] 压缩 WASM 模块体积
- [ ] 增加控制流重建
- [ ] NES 特定模式识别
- [ ] 导出为 NESL 格式
- [ ] 多线程分析（Web Workers）

## 相关文档

- [使用指南](../docs/decompiler-guide.md)
- [Feature 015 规格](../specs/015-advanced-decompiler/spec.md)
- [NESL 语言参考](../docs/nesl-reference.md)
