# v3.28.5 — 崩溃取证升级 + 恢复窗口修复（稳定性版）

基线沿革：**v3.28.4 同款链**（v3.28.0 提交 721c086 + cumulative 3.27.23 + SIGSEGV 防护 + 崩溃日志落盘修复 + 摘 AI + 动态版本 + meta 直查）。
本版**只带稳定性修复**，不动任何数据端点行为。

## 现场（v3.28.4 两次闪退的日志尾巴）

```
[100] S:json
CRASH at step 100 sig=11 RECOVERED
[1] S:wdm            ← 新进程（闪退后重启）
...
[82] S:json
CRASH at step 82 sig=11 RECOVERED
```

## 审计发现（读生成的源码，只读 CI）

1. **`RECOVERED` 是假话**：崩溃处理器在检查恢复标志**之前**就把 `" RECOVERED"`
   无条件拼进日志。两次崩溃后计数归 `[1]`（新进程）→ longjmp 兜底那条路
   **并没有救回进程**（或崩点落在了无保护窗口）。
2. **恢复窗口有洞**：`read_summary` 在 `catch_unwind` 返回后**立刻**清掉
   `SIGSEGV_RECOVERY`，然后才跑 `observe_ramen_transition` / 写缓存 / 返回。
   尾巴这一段裸奔——在那里崩，handler 只能 re-raise 杀进程。
   日志"`S:json` 之后没下一条"恰好落在这个窗口（S:json → 下一轮 `S:wdm` 之间）。

## 本版修正

1. **崩溃日志说真话**
   - 改用 `sigaction(SA_SIGINFO | SA_ONSTACK)` + `sigaltstack`；
   - 日志格式升级为：
     `CRASH at step N sig=11 addr=0x<故障地址> tid=<故障线程> RECOVERED|FATAL`
   - **只有 longjmp 真正兜住时才写 `RECOVERED`；进程将死时写 `FATAL`**；
   - 全部手工拼接、零分配（沿用裸 syscall 纪律）。
2. **收尾切 3 个细诊断点**：`S:json_built`（JSON 组装完成）、`S:obs_done`、
   `S:cache_done`。下一次崩，日志尾巴直接告诉你崩在"组装内 / 尾巴 / 下一轮前导"。
3. **补恢复窗口**：`SIGSEGV_RECOVERY` 只在 `read_summary` 最末端清除，
   覆盖 observe / cache / return 全尾段。
4. **深栈**：push 线程与 HTTP 每请求线程改为 4 MiB 栈（`/summary` 就跑在这两类线程上；
   巨型 JSON 组装也在其中）。

## 怎么读下一次的崩溃日志

| 日志尾巴 | 含义 |
|---|---|
| `S:json` → `CRASH ... FATAL` | 崩在 JSON 组装**内部**（format! 读取某片段时） |
| `S:json_built` → `CRASH ... FATAL` | 崩在组装完成→收尾之间 |
| `S:obs_done` / `S:cache_done` → `CRASH ... FATAL` | 崩在收尾 tail（本版已把这段纳入恢复窗口，若仍 FATAL 说明连窗口机制都没能拦住） |
| `CRASH ... RECOVERED`（且进程活着） | 真兜住了（60s 冷却后自愈） |
| `addr=0x0` 或极小值 | 空指针/野指针读；`addr` 位于线程栈区间 | 栈溢出（本版已加 4MiB 栈，仍溢才是真凶） |

## 验证

安装后：
1. `/health` 应报 `version=3.28.5`；
2. 正常玩一场，若曾必崩的场景不再崩 → 窗口/栈修复生效；
3. 若仍崩：把 `Android/media/jp.co.cygames.umamusume/hachimi/uma_predict.log` 的**最后 50 行**给我，新格式会直接给出 地址+线程+真伪判词。

数据端点、嗅探、发收包落盘全部原样保留。
