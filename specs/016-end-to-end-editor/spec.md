# Feature 016: 端到端 NES 游戏编辑器

## 目标

创建一个完整的网页应用，让用户可以：
1. 上传 NES ROM
2. 自动反编译为 NESL 代码
3. 在线编辑代码
4. 实时重新编译
5. 在集成的模拟器中测试
6. 下载修改后的 ROM

## 用户故事

**作为** NES 游戏爱好者/开发者  
**我想** 在浏览器中修改现有的 NES 游戏  
**以便** 快速实验和学习，无需安装任何工具

## 功能设计

### 1. 上传界面
```
┌─────────────────────────────────────┐
│  📁 拖放 NES ROM 文件到这里        │
│                                     │
│  或点击选择文件                     │
│                                     │
│  支持: .nes 文件                   │
└─────────────────────────────────────┘
```

### 2. 主界面（三栏布局）

```
┌──────────┬──────────────────┬──────────┐
│          │                  │          │
│  文件树  │   代码编辑器      │  预览区  │
│          │                  │          │
│  ROM     │  1  var x = 124; │  ┌────┐  │
│  ├─main  │  2  var y = 112; │  │NES │  │
│  ├─vars  │  3               │  │模拟│  │
│  └─tiles │  4  fn main() {  │  │器  │  │
│          │  5    ...        │  └────┘  │
│          │                  │          │
│          │  [▶ 编译运行]    │  [控制]  │
└──────────┴──────────────────┴──────────┘
```

### 3. 工作流程

```
1. 上传 ROM
   ↓
2. 自动反编译
   - 检查符号表
   - 如有符号表 → NESL 源码
   - 无符号表 → 高级伪代码
   ↓
3. 编辑器展示代码
   - 语法高亮
   - 自动补全
   - 错误提示
   ↓
4. 用户修改代码
   ↓
5. 实时编译
   - NESL → ROM
   - 显示编译状态
   ↓
6. 模拟器运行
   - 加载新 ROM
   - 可玩测试
   ↓
7. 导出
   - 下载修改后的 ROM
   - 下载源码
```

## 技术方案

### 前端架构

**纯 JavaScript + WASM（无框架）**
- 轻量级，加载快
- 全部在浏览器端运行
- 数据隐私保护

### 核心组件

#### 1. ROM 处理器
```javascript
class ROMProcessor {
  async decompile(romData) {
    // 调用 WASM 反编译
    const result = await wasmDecompile(romData);
    return result.code;
  }
  
  async compile(neslCode) {
    // 调用 WASM 编译
    const result = await wasmCompile(neslCode);
    return result.rom;
  }
}
```

#### 2. 代码编辑器
```javascript
class CodeEditor {
  constructor(container) {
    this.editor = createEditor(container);
    this.setupSyntaxHighlight();
    this.setupAutoComplete();
  }
  
  onChange(callback) {
    this.editor.on('change', callback);
  }
}
```

#### 3. NES 模拟器集成
```javascript
class EmulatorPanel {
  loadROM(romData) {
    this.emulator.load(romData);
  }
  
  play() { this.emulator.run(); }
  pause() { this.emulator.pause(); }
  reset() { this.emulator.reset(); }
}
```

### WASM 接口

需要添加到 `nesc-wasm`:

```rust
#[wasm_bindgen]
pub fn nesc_compile_full(src: &str) -> String {
    // 完整编译，返回 ROM + 调试信息
}

#[wasm_bindgen]
pub fn nesc_decompile_full(rom_base64: &str) -> String {
    // 完整反编译，返回源码 + 元数据
}
```

## 界面设计

### 颜色方案
- 主色: #667eea (紫色)
- 编辑器背景: #1e1e1e (VS Code 暗色)
- 代码文本: #d4d4d4
- 语法高亮: VS Code 主题

### 布局
- 响应式设计
- 可调整分栏大小
- 全屏模式

## 用户体验

### 快速入门
1. 提供示例 ROM（sprite.nes）
2. 点击"加载示例"
3. 查看代码
4. 修改一行（例如改变颜色）
5. 点击"编译运行"
6. 在模拟器中看到效果

### 错误处理
- 编译错误：高亮错误行，显示错误信息
- ROM 无效：友好提示
- 浏览器不支持：降级提示

### 性能优化
- 懒加载模拟器
- 增量编译
- 代码缓存

## 实现阶段

### Phase 1: 基础框架
- [x] 三栏布局
- [ ] 文件上传
- [ ] WASM 集成

### Phase 2: 编辑器
- [ ] 代码编辑器（简单 textarea 或 Monaco Editor）
- [ ] 语法高亮
- [ ] 自动缩进

### Phase 3: 编译集成
- [ ] 调用 WASM 编译
- [ ] 显示编译结果
- [ ] 错误提示

### Phase 4: 模拟器
- [ ] 集成 NES 模拟器（jsnes 或 nes-rust）
- [ ] 控制器映射
- [ ] 音频支持

### Phase 5: 完善
- [ ] 保存/加载项目
- [ ] 导出功能
- [ ] 键盘快捷键
- [ ] 教程/帮助

## 成功标准

- ✅ 可以上传 NESL ROM 并反编译
- ✅ 可以在线编辑代码
- ✅ 可以重新编译并运行
- ✅ 修改效果可在模拟器中验证
- ✅ 可以下载修改后的 ROM
- ✅ 完全浏览器端，无需服务器
- ✅ 流畅的用户体验（<1s 响应）

## 非目标

- ❌ 不支持复杂的项目管理
- ❌ 不支持多人协作
- ❌ 不是完整的 IDE（只是演示工具）
- ❌ 不需要账号系统

## 技术栈

- HTML5 + CSS3 + JavaScript (ES6+)
- WebAssembly (Rust)
- 代码编辑器: Monaco Editor 或简单 CodeMirror
- NES 模拟器: jsnes 或 nes-rust WASM

## 预计时间

- Phase 1-2: 2-3 小时
- Phase 3: 2 小时
- Phase 4: 3-4 小时
- Phase 5: 2 小时
- **总计: 9-11 小时**

## MVP

最小可行产品包括：
- Phase 1-3: 上传、编辑、编译
- 简单的模拟器集成（或使用外部模拟器链接）
- 下载功能

不包括：
- 复杂的 IDE 功能
- 项目管理
- 高级调试

## 示例场景

**用户小明想修改 sprite.nes 的精灵颜色：**

1. 打开网页，点击"加载示例"
2. 看到 sprite.nesl 源码
3. 找到 `tile 1` 的颜色定义
4. 将 `2` 改为 `3`（改变颜色）
5. 点击"编译运行"
6. 在右侧模拟器中看到精灵颜色变化 ✨
7. 满意后，点击"下载 ROM"

**整个过程 < 1 分钟！**
