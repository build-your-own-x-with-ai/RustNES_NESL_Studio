# Quickstart: NESL Decompiler Validation

**Date**: 2026-10-24

**Purpose**: 端到端验证反编译器功能的可运行场景

---

## Prerequisites

- Rust toolchain（stable，edition 2021）
- NESL 项目完整构建（`cargo build --release`）
- 示例文件：`examples/hello.nesl`、`examples/quest.nesl`

---

## Scenario 1: Basic Roundtrip（基本往返测试）

**Goal**: 验证 `hello.nesl` 的编译 → 反编译 → 重新编译流程

### Setup

```bash
cd /Users/i/Code/RustNES
cargo build --release
```

### Step 1: 编译并生成符号表

```bash
./target/release/nesc build examples/hello.nesl --debug-symbols
```

**Expected Output**:
- `examples/hello.nes`（32KB ROM）
- `examples/hello.syms`（符号表 JSON 文件）

**Verify**:
```bash
ls -lh examples/hello.nes examples/hello.syms
# hello.nes 应该是 32784 字节（16 字节头 + 32KB PRG + 8KB CHR）
# hello.syms 应该是几 KB 的 JSON 文件

file examples/hello.nes
# 输出: NES ROM image (iNES format)
```

### Step 2: 反编译

```bash
./target/release/nesc decompile examples/hello.nes -o /tmp/hello.dec.nesl
```

**Expected Output**:
- `/tmp/hello.dec.nesl`（反编译的 NESL 源码）

**Verify**:
```bash
head -20 /tmp/hello.dec.nesl
# 应该看到:
# // Decompiled from examples/hello.nes
# // NESL Compiler: v0.1.0
# // Decompiled at: 2026-10-24 ...
#
# const PAL: [u8] = [0x0F, 0x30, ...];
# var x: u8 = 124;
# ...
```

### Step 3: 重新编译

```bash
./target/release/nesc build /tmp/hello.dec.nesl -o /tmp/hello2.nes
```

**Expected Output**:
- `/tmp/hello2.nes`

**Verify**:
```bash
# 检查 ROM 大小和 mapper 类型
xxd -l 16 /tmp/hello2.nes
# 前 16 字节（iNES 头）应该与 examples/hello.nes 的头部一致（mapper=0, 32KB PRG, 8KB CHR）
```

### Step 4: 语义等价性测试

```bash
cargo test --package nesl roundtrip_hello
```

**Expected Output**:
```
test roundtrip_hello ... ok
```

**What the test does**:
1. 在 `nes-sim` 中运行 `hello.nes`，执行 30 帧
2. 在 `nes-sim` 中运行 `hello2.nes`，执行 30 帧
3. 断言两个 ROM 的最终状态相同：
   - 变量 `x` 的值
   - VRAM 内容（显示的文字"HELLO NES!"）
   - OAM 内容（精灵位置）

---

## Scenario 2: UNROM Roundtrip（多 bank ROM）

**Goal**: 验证 UNROM（mapper 2）ROM 的反编译，包括 6 个 world 的地图数据

### Setup

```bash
./target/release/nesc build examples/quest.nesl --debug-symbols
```

**Expected Output**:
- `examples/quest.nes`（128KB，mapper 2）
- `examples/quest.syms`

### Decompile

```bash
./target/release/nesc decompile examples/quest.nes -o /tmp/quest.dec.nesl
```

**Verify**:
```bash
# 检查地图还原
grep -c "^map W" /tmp/quest.dec.nesl
# 应该输出 6（6 个 world 的地图）

grep "const WORLDS" /tmp/quest.dec.nesl
# 应该看到: const WORLDS: [map] = [W1, W2, W3, W4, W5, W6];

# 检查音乐还原
grep "^music " /tmp/quest.dec.nesl | head -3
# 应该看到 music MELODY = "...";
```

### Recompile and Test

```bash
./target/release/nesc build /tmp/quest.dec.nesl -o /tmp/quest2.nes
cargo test --package nesl roundtrip_quest
```

**Expected**:
- 测试通过
- `quest2.nes` 的 mapper 也是 2（UNROM）
- 游戏逻辑完全相同（hero 碰撞、金币收集、bank 切换）

---

## Scenario 3: Error Handling（错误处理）

**Goal**: 验证反编译器拒绝非 NESL ROM 和损坏的符号表

### Test 1: 非 NESL ROM

```bash
# 下载一个非 NESL 的 ROM（如 nestest.nes，测试 ROM）
# 或者创建一个空 ROM
echo "NES\x1a\x02\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00" > /tmp/fake.nes
dd if=/dev/zero bs=1024 count=40 >> /tmp/fake.nes

./target/release/nesc decompile /tmp/fake.nes
```

**Expected Output**:
```
错误：找不到符号表文件: /tmp/fake.syms
提示：使用 `nesc build --debug-symbols` 生成符号表
```

**Exit code**: 1

### Test 2: 损坏的符号表

```bash
# 复制 hello.syms 并破坏 JSON 格式
cp examples/hello.syms /tmp/broken.syms
echo "{BROKEN JSON" > /tmp/broken.syms

./target/release/nesc decompile examples/hello.nes --symbols /tmp/broken.syms
```

**Expected Output**:
```
错误：符号表格式错误: expected value at line 1 column 2
```

**Exit code**: 3

### Test 3: Mapper 不匹配

```bash
# 手动修改符号表的 mapper 字段
jq '.mapper = 99' examples/hello.syms > /tmp/wrong-mapper.syms

./target/release/nesc decompile examples/hello.nes --symbols /tmp/wrong-mapper.syms
```

**Expected Output**:
```
错误：符号表与 ROM 不一致: mapper 不匹配（ROM: 0, symbols: 99）
```

**Exit code**: 4

---

## Scenario 4: Partial Decompilation（部分反编译）

**Goal**: 验证遇到无法识别的模式时，反编译器仍能输出有效的 NESL 代码

### Setup

（这个场景需要在实现时人工构造一个包含"未知模式"的 ROM，或者修改符号表让某些函数的源码映射缺失）

### Test

```bash
# 假设 special.nes 有一个函数缺少完整的符号表
./target/release/nesc decompile special.nes -o /tmp/special.dec.nesl -v
```

**Expected Output**:
```
警告: 函数 'unknown_func' at 0x8100 缺少符号信息，使用占位符
反编译完成: /tmp/special.dec.nesl
```

**Verify**:
```bash
grep "UNKNOWN CODE" /tmp/special.dec.nesl
# 应该看到:
# fn unknown_func() {
#     // UNKNOWN CODE at $8100
# }
```

---

## Automated Test Suite

### Run All Tests

```bash
cargo test --package nesl --test decompile
```

**Expected Output**:
```
running 8 tests
test decompile::basic_roundtrip ... ok
test decompile::unrom_roundtrip ... ok
test decompile::tile_restoration ... ok
test decompile::map_restoration ... ok
test decompile::music_restoration ... ok
test decompile::error_missing_symbols ... ok
test decompile::error_invalid_rom ... ok
test decompile::error_version_mismatch ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Individual Test Examples

```bash
# 只测试基本往返
cargo test --package nesl roundtrip_hello

# 只测试错误处理
cargo test --package nesl decompile::error

# 详细输出
cargo test --package nesl roundtrip -- --nocapture
```

---

## Performance Validation

**Goal**: 验证反编译速度符合性能目标（<5 秒处理 64KB ROM）

```bash
time ./target/release/nesc decompile examples/quest.nes -o /tmp/quest.dec.nesl
```

**Expected**:
```
real    0m1.234s
user    0m1.100s
sys     0m0.050s
```

（实际时间应远小于 5 秒）

---

## Manual Verification Checklist

After running the automated tests, manually verify:

- [ ] 反编译输出的 NESL 代码可读性好（变量名、函数名、注释）
- [ ] tile 数据还原正确（用肉眼看像素矩阵）
- [ ] map 数据还原正确（字符串格式，列宽正确）
- [ ] music MML 字符串完整
- [ ] 错误信息清晰且有用（指出问题和解决方法）
- [ ] CLI 帮助文本完整（`nesc decompile --help`）

---

## Success Criteria

反编译器通过验证当且仅当：

1. ✅ 所有 `examples/*.nesl` 都能往返成功（编译 → 反编译 → 重新编译 → 语义等价）
2. ✅ UNROM 和 MMC3 ROM 正确处理（多 bank、IRQ handler）
3. ✅ 资源（tile/map/music）完整还原
4. ✅ 非 NESL ROM 被正确拒绝（清晰错误信息）
5. ✅ 性能符合目标（<5 秒处理 64KB ROM）
6. ✅ 所有单元测试和集成测试通过

---

## Troubleshooting

### Problem: 符号表文件不存在

**Solution**: 重新编译时加上 `--debug-symbols` 选项
```bash
nesc build examples/hello.nesl --debug-symbols
```

### Problem: 反编译输出的代码无法编译

**Solution**: 检查符号表是否完整，查看详细错误
```bash
nesc decompile hello.nes -v -o output.nesl
nesc check output.nesl
```

### Problem: 往返测试失败（语义不等价）

**Solution**: 运行模拟器比较两个 ROM 的行为差异
```bash
cargo test --package nesl roundtrip_hello -- --nocapture
# 查看测试输出，找到第一个不同的内存地址或 VRAM 位置
```

### Problem: 反编译太慢

**Solution**: 检查是否是 debug 模式编译，切换到 release 模式
```bash
cargo build --release
./target/release/nesc decompile ...
```
