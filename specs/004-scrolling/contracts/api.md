# Contract: 横向卷轴（v4 增量）

在 [003 游戏开发 API](../../003-game-api/contracts/api.md) 基础上：

## 宽地图
```nesl
map WORLD = [
    "................................................................",   // 最多 255 列
    ...                                                                     // 30 行
];
draw_map(WORLD);                          // 画出最左两屏，镜头归零
if map_tile(WORLD, hero.x >> 3, hero.y >> 3) == '#' { ... }   // 列号可为 u16
```

## 镜头
```nesl
camera_follow(hero.x, WORLD);             // 每帧调用：跟随、在两端停住、自动补画新列
let sx = hero.x - camera_x();             // 世界坐标 → 屏幕坐标（u16）
if sx < 256 { metasprite(0, sx as u8, hero.y, HERO); }   // 在屏幕外时不画
```
- 主角保持在屏幕 x≈120；镜头每帧最多移动 8 像素。
- 只支持横向卷轴。背景上的文字会随之滚动；固定的分数请用 `status_bar(n)`（见 [006](../../006-map-edit-status/contracts/api.md)），
  或用精灵绘制（字体图块就是 ASCII 码：`sprite(i, x, y, '0' + d, 0)`）。

## 调色板分区
```nesl
palette_zone WORLD = [        // 每个字符 = 一个 16×16 区域使用的背景调色板 0-3（`.` = 0）
    "0000000000000000",
    "1111222211112222",
    ...                        // 最多 15 行，每行最多 ⌈宽度/2⌉ 个字符
];
```
