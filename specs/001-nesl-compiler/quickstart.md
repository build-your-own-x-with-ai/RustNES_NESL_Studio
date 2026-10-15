# Quickstart

```bash
cargo build --release
./target/release/nesc build examples/hello.nesl     # → examples/hello.nes
./target/release/nesc serve                          # 打开 http://127.0.0.1:8080
cargo test                                           # 全部测试
```

最小程序 `hello.nesl`：
```nesl
const PAL: [u8] = [0x0F,0x30,0x16,0x27, 0x0F,0x30,0x16,0x27, 0x0F,0x30,0x16,0x27, 0x0F,0x30,0x16,0x27,
                   0x0F,0x30,0x16,0x27, 0x0F,0x30,0x16,0x27, 0x0F,0x30,0x16,0x27, 0x0F,0x30,0x16,0x27];
fn main() {
    load_palette(PAL);
    print(10, 14, "HELLO NES!");
    ppu_on();
    loop { wait_vblank(); }
}
```
用任意 NES 模拟器打开 `hello.nes` 即可看到文字。
