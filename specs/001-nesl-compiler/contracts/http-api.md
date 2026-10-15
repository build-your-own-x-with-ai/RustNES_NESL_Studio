# Contract: 编辑器 HTTP API（`nesc serve`）

| 方法 | 路径 | 请求 | 响应 |
|---|---|---|---|
| GET | `/` | – | `text/html` 编辑器页面 |
| GET | `/api/examples` | – | `application/json`：`[{"name":"hello","source":"..."}]` |
| POST | `/api/compile` | body = NESL 源码（`text/plain; utf-8`） | 见下 |

`POST /api/compile` 成功：
```json
{"ok": true, "rom": "<base64 iNES>", "prg_used": 1234, "asm": "<汇编清单>"}
```
失败（仍为 HTTP 200）：
```json
{"ok": false, "errors": [{"line": 3, "col": 5, "message": "unknown variable `y`"}]}
```
其他路径返回 404；请求体上限 1 MiB，超出返回 413。
