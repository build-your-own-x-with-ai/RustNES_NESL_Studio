# Super Mario Bros 完整反编译项目

## 目标

将 Super Mario Bros ROM 反编译为完全可编译的 NESL 代码

## 策略

### Phase 1: 深度分析（1-2天）

**分析 Mario 的架构：**

1. **识别核心系统**
   - 游戏状态机
   - 物理引擎
   - 碰撞检测
   - 关卡数据
   - 精灵管理

2. **识别关键变量**
   ```
   $00-$0F: 临时变量
   $10-$1F: 玩家状态
   $20-$4F: 敌人数据
   $50-$6F: 关卡数据
   $70-$7F: 其他
   ```

3. **识别关键函数**
   - 主循环
   - 玩家移动
   - 跳跃逻辑
   - 碰撞检测
   - 敌人 AI
   - 关卡加载

### Phase 2: 模式识别（2-3天）

**识别常见模式：**

1. **控制流模式**
   ```asm
   CMP #$02
   BCC skip
   ; code
   skip:
   ```
   ↓
   ```c
   if (value >= 2) {
       // code
   }
   ```

2. **循环模式**
   ```asm
   LDX #$10
   loop:
   ; code
   DEX
   BNE loop
   ```
   ↓
   ```c
   for (int i = 16; i > 0; i--) {
       // code
   }
   ```

3. **状态机模式**
   ```asm
   LDA state
   CMP #$00
   BEQ state_0
   CMP #$01
   BEQ state_1
   ```
   ↓
   ```c
   switch (state) {
       case 0: /* state_0 */ break;
       case 1: /* state_1 */ break;
   }
   ```

### Phase 3: 手工标注（3-5天）

**标注关键部分：**

1. **变量命名**
   ```
   $70 → player_x
   $71 → player_y
   $1A → player_state
   ```

2. **函数命名**
   ```
   sub_8182 → check_collision
   sub_80EA → update_player
   sub_8040 → irq_handler
   ```

3. **控制流注释**
   ```c
   // This is the main game loop
   // This handles player input
   // This updates enemy AI
   ```

### Phase 4: 自动化工具（5-7天）

**开发 Mario 专用反编译器：**

```rust
struct MarioDecompiler {
    rom: Vec<u8>,
    variables: HashMap<u8, String>,  // 地址 → 名称
    functions: HashMap<u16, String>, // 地址 → 名称
    patterns: Vec<Pattern>,          // 识别模式
}

impl MarioDecompiler {
    fn decompile(&self) -> String {
        // 1. 识别所有函数
        // 2. 对每个函数：
        //    - 重建控制流
        //    - 识别变量用途
        //    - 生成高级代码
        // 3. 生成可编译的 NESL
    }
    
    fn rebuild_control_flow(&self, func: &Function) -> ControlFlow {
        // goto → if/while/for
    }
    
    fn infer_variable_types(&self, func: &Function) -> Types {
        // 推断类型
    }
}
```

### Phase 5: 测试验证（3-5天）

**验证流程：**

1. 编译生成的 NESL
2. 运行两个 ROM（原版 vs 重编译）
3. 对比行为
4. 修复差异

---

## 预期成果

### 生成代码示例

```nesl
// Super Mario Bros - 完整可编译版本

// ========== 变量定义 ==========

var player_x: u8;          // 玩家 X 坐标
var player_y: u8;          // 玩家 Y 坐标
var player_vx: i8;         // 玩家 X 速度
var player_vy: i8;         // 玩家 Y 速度
var player_state: u8;      // 0=小, 1=大, 2=火
var on_ground: bool;       // 是否在地面

// ========== 游戏主循环 ==========

fn nmi() {
    // VBlank 中断
    update_input();
    update_physics();
    update_enemies();
    check_collisions();
    render_frame();
}

// ========== 玩家逻辑 ==========

fn update_player() {
    // 处理输入
    if (button_left) {
        player_vx = -2;
    }
    if (button_right) {
        player_vx = 2;
    }
    if (button_a && on_ground) {
        player_vy = -8;  // 跳跃
        on_ground = false;
    }
    
    // 应用重力
    if (!on_ground) {
        player_vy += 1;
    }
    
    // 更新位置
    player_x += player_vx;
    player_y += player_vy;
    
    // 减速
    if (player_vx > 0) {
        player_vx--;
    } else if (player_vx < 0) {
        player_vx++;
    }
}

// ========== 碰撞检测 ==========

fn check_collision(x: u8, y: u8) -> bool {
    // 检查地图碰撞
    var tile = get_tile(x / 16, y / 16);
    return tile != 0;
}

// ========== 敌人 AI ==========

fn update_goomba(id: u8) {
    var x = enemy_x[id];
    var vx = enemy_vx[id];
    
    x += vx;
    
    // 碰到墙壁转向
    if (check_collision(x, enemy_y[id])) {
        vx = -vx;
    }
    
    enemy_x[id] = x;
    enemy_vx[id] = vx;
}

// ========== 主函数 ==========

fn main() {
    init_hardware();
    load_level_1();
    
    loop {
        wait_vblank();
    }
}
```

---

## 时间估算

| 阶段 | 工作量 | 时间 |
|------|--------|------|
| Phase 1: 深度分析 | 手工 | 1-2 天 |
| Phase 2: 模式识别 | 手工 | 2-3 天 |
| Phase 3: 标注 | 手工 | 3-5 天 |
| Phase 4: 自动化 | 编程 | 5-7 天 |
| Phase 5: 测试 | 测试 | 3-5 天 |
| **总计** | | **14-22 天** |

---

## 现实挑战

### 仍然困难的部分

1. **精确时序**
   - Mario 依赖精确的 CPU 周期
   - NESL 编译器可能生成不同的时序

2. **硬件寄存器**
   - PPU 滚动
   - APU 声音
   - 精灵 DMA

3. **查找表和数据**
   - 关卡数据嵌入代码
   - 物理表
   - 图形数据

4. **边界情况**
   - 溢出
   - 边界检查
   - 未文档化的行为

---

## 成功标准

✅ **最低标准：**
- 编译通过
- 游戏可以启动
- 基本移动有效

✅ **理想标准：**
- 玩法完全相同
- 无明显 bug
- 性能相当

🏆 **完美标准：**
- 逐帧相同
- 字节级相同
- （几乎不可能）

---

## 下一步

你想要：

1. **我现在开始** Phase 1（深度分析 Mario）
2. **先开发工具** Phase 4（通用改进）
3. **混合方案**（同时进行）

选择一个，我开始！
