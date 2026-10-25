# Feature 017: 集成 NES 模拟器

## 目标

在端到端编辑器中集成一个完整的 NES 模拟器，实现：
1. 编译后立即在浏览器中运行
2. 完整的游戏手柄支持
3. 保存/加载状态
4. 实时调试功能

## 技术选型

### 方案对比

| 方案 | 优点 | 缺点 | 选择 |
|------|------|------|------|
| **jsnes** | 纯 JS，轻量，易集成 | 性能中等 | ✅ |
| nes-rust WASM | 性能最好 | 需要编译，体积大 | ❌ |
| embed.nes | 功能完整 | 依赖多，复杂 | ❌ |

**选择 jsnes**：
- 纯 JavaScript，无需编译
- CDN 加载，快速
- API 简单
- 社区活跃

## 功能设计

### 1. 基本模拟器

```javascript
class NESEmulator {
  constructor(canvas) {
    this.nes = new jsnes.NES({
      onFrame: (frameBuffer) => {
        this.renderFrame(frameBuffer);
      },
      onAudioSample: (left, right) => {
        this.audioQueue.push(left, right);
      }
    });
  }
  
  loadROM(romData) {
    this.nes.loadROM(romData);
  }
  
  start() {
    this.nes.frame();
    requestAnimationFrame(() => this.start());
  }
}
```

### 2. 控制器映射

| 键盘 | NES 按钮 |
|------|----------|
| ↑↓←→ | 方向键 |
| Z | A |
| X | B |
| Enter | Start |
| Shift | Select |

### 3. UI 布局

```
┌────────────────────────────────┐
│  NES 模拟器                     │
├────────────────────────────────┤
│                                │
│        [256x240 Canvas]        │
│                                │
├────────────────────────────────┤
│  [▶ 运行] [⏸ 暂停] [🔄 重置]  │
│  [💾 保存] [📂 加载]           │
└────────────────────────────────┘
```

### 4. 功能集成

**编译 → 运行流程**：
1. 用户点击 "编译运行"
2. WASM 编译 NESL → ROM
3. 自动加载 ROM 到模拟器
4. 模拟器开始运行
5. 用户可以用键盘玩游戏

## 实现步骤

### Phase 1: 基础模拟器
- [ ] 引入 jsnes 库
- [ ] 实现基本渲染
- [ ] 加载 ROM

### Phase 2: 控制器
- [ ] 键盘映射
- [ ] 按键事件处理
- [ ] 手柄支持（可选）

### Phase 3: 高级功能
- [ ] 保存/加载状态
- [ ] 速度控制
- [ ] 截图功能

### Phase 4: 调试功能
- [ ] 暂停/单步
- [ ] 内存查看
- [ ] CPU 状态显示

## API 示例

```javascript
// 创建模拟器
const emulator = new NESEmulator(canvas);

// 加载 ROM
emulator.loadROM(romBytes);

// 运行
emulator.start();

// 暂停
emulator.pause();

// 重置
emulator.reset();

// 保存状态
const state = emulator.saveState();

// 加载状态
emulator.loadState(state);
```

## 性能要求

- 60 FPS 稳定运行
- 音频同步
- 延迟 < 100ms

## 兼容性

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## 未来改进

- [ ] 网络多人游戏
- [ ] 录像回放
- [ ] 作弊码支持
- [ ] 自定义调色板
