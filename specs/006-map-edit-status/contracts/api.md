# Contract: 可修改地图与状态栏（v6 增量）

## 改写地图
```nesl
if map_tile(WORLD, cx, cy) == '$' {
    set_map_tile(WORLD, cx, cy, ' ');   // 读取、补画、屏幕都会看到新值
    score += 1;
}
reset_map(WORLD);                        // 撤销 WORLD 的全部修改（重画需再 draw_map）
```
- `set_map_tile(M, tx, ty, tile) -> u8`：返回 1 表示已记录，0 表示修改记录已满（64 处，所有地图共用）。
  记录满时可见的格子仍会更新，但卷走后再卷回来会恢复原样。
- 列号 tx 为 u8（0-254）。修改不会改变 ROM 里的地图本身。

## 状态栏
```nesl
status_bar(4);                 // 顶部 4 行不随卷轴滚动（2-8 行；0 关闭）
print(1, 1, "SCORE");          // 写在状态栏里，固定显示
print_u8(8, 1, score);
```
- 地图从第 n 行以下开始绘制和卷动；`map` 的前 n 行不会显示。
- 状态栏占用 **精灵 0** 和状态栏最后一行**最右边一格**（第 31 列）；请从 1 号精灵开始使用，不要 `tile 0x7E` / `tile 0x7F`。
- 状态栏使用背景调色板 0。
