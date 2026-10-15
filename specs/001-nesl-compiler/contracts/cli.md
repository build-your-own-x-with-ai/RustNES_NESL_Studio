# Contract: `nesc` 命令行

```text
nesc build <file.nesl> [-o <out.nes>] [--asm]
    编译源码。默认输出与源文件同名的 .nes。
    --asm  额外输出 <out>.s 汇编清单（含地址与机器码）。
    成功：退出码 0，stdout 打印 "wrote <path> (<N> bytes PRG used)"。
    失败：退出码 1，stderr 每条诊断一行："<file>:<line>:<col>: error: <message>"。

nesc check <file.nesl>
    仅检查，不写文件。退出码同上。

nesc serve [--port <N>]     (默认 8080，仅绑定 127.0.0.1)
    启动浏览器编辑器，stdout 打印 URL。

nesc help | --help | -h
```
