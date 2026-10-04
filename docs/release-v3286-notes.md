# v3.28.6 — 崩溃归因版（pc/lr + 线程标签 + 细窗口标记）

基线沿革：v3.28.5 同款链（v3.28.0 + cumulative 3.27.23 + SIGSEGV 防护 + 崩溃日志修复 + 摘 AI + 动态版本 + meta 直查 + 崩溃取证/恢复窗口修复）。
**本版只增强崩溃归因，不改任何数据行为**（数据端点、嗅探、发收包落盘原样保留）。

## v3.28.5 的战果（首次拿到真判词）

v3.28.5 装机后的新日志尾巴：

```
[92] S:json
[93] S:json_built
[94] S:obs_done
[95] S:cache_done
CRASH at step 95 sig=11 addr=0x72bf999f80 tid=486810748160 FATAL
```

三条结论：
1. **JSON 组装段无罪**——`S:json_built` 已打出，100KB 的 format! 完整跑完；
2. **FATAL = 崩溃发生在"武装窗口"之外**（`S:cache_done` 之后，或发生在未武装的线程上）——所以 SIGSEGV 防护没有生效不是防护失灵，是崩点不在它的射程内；
3. **addr=0x72bf999f80 位于游戏堆段（0x72…）**——这是"对象生死/线程时序"层面的问题（指针拿到时对象还活着，读取时已被释放/复用），**不是"字段偏移错位"层面的问题**（若偏移错位，会读垃圾值或在读取当下就崩，不会全部读完、JSON 组完才崩）。

## 本版新增（全部只在崩溃/流水日志里追加信息，不改控制流）

1. **pc= / lr= 模块归因**：崩溃日志追加 `pc=<模块>+<十六进制偏移> lr=<模块>+<偏移>`。
   模块表在启动时从 `/proc/self/maps` 解析（hlpatch / il2cpp / native / libc / main / 其他），信号处理器只读静态表、零分配。
   → 下一份日志直接指认"故障指令在哪条 SO、偏移多少"。
2. **线程标签**：push / http / init 三线程自注册（环形表，最近 16 个永不丢）；崩溃行追加 `thr=push | http | init | ?`。
3. **细窗口标记（7 个）**：`S:enter`、`S:exit`、`H:ret`、`H:saved`、`P:read_done`、`P:push_start`、`P:push_done`。
   把"S:cache_done 之后到下个请求"这段黑盒切成单语句级网格。
4. 崩溃消息缓冲 200→320 字节（容纳新字段）。

## 下一次崩溃怎么读（对照表）

| 日志尾巴 | 含义 |
|---|---|
| `S:cache_done` → `CRASH`（无 `S:exit`） | 崩在 read_summary 收尾（observe 之后 / 写缓存之后 / 返回之前） |
| `S:exit` → `CRASH`（无 `H:ret`） | 崩在 /summary 路由返回与 handle_http 之间 |
| `H:ret` → `CRASH`（无 `H:saved`） | 崩在 save_endpoint_log 或响应写出 |
| `P:read_done` → `CRASH`（无 `P:push_start`） | 崩在 push 线程读后处理 |
| 行内 `thr=http` / `thr=push` | 锁定故障线程；`thr=?` = 未注册线程（多半是游戏线程上执行的钩子代码） |
| `pc=il2cpp+0x…` | 故障指令在游戏引擎里（若经我们钩子 trampoline 进入，会同时看到 lr=hlpatch+…） |
| `pc=hlpatch+0x…` | 故障指令在插件里——结合 `lr=` 反推调用者 |

## 保留项与排除项（与 v3.28.5 一致）

保留：SIGSEGV 真判词、恢复窗口修复、4MiB 深栈、meta 直查控制台、纯数据管道（摘 AI）、动态版本号、全部数据端点/嗅探/发收包落盘。
不含：画面映射（截屏）、training_anim_skip、签名明文观测。

SHA256 见 SHA256SUMS / BUILD-MANIFEST.txt。
