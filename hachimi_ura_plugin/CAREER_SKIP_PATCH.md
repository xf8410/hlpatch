# Career Skip — lib.rs 接线说明（共 2 行改动）

分支 `workbench/career-skip-20260829`。新文件 `hachimi_ura_plugin/src/career_skip.rs` 已就位（自包含，只用 crate 根已有 helper，无新依赖）。

## 改动 1 — 注册模块（hachimi_ura_plugin/src/lib.rs）

在 `#![allow(dead_code)]`（文件头部，`const PLUGIN_VERSION` 之前）后加一行：

```rust
mod career_skip;
```

## 改动 2 — 路由分发

在 `handle_http` 的路由 if/else 链里加一个分支（建议放在 `/config.html` 分支之前；该链按 `path == "..."` 逐个匹配，最后一个 else 是 not_found）：

```rust
} else if let Some(body) = career_skip::handle(&path) {
    body
}
```

注意：现有链的分支形如 `} else if path == "/xxx" { <expr> }`，所以这个分支直接以 `body` 作为分支值即可，不要 return。

## 验证流程（SO 部署后，全本地零网络）

```bash
GET http://127.0.0.1:18765/skip/status
# 期望 enabled=false, choice_guard_101=false(在剧情中) / null(不在)
GET http://127.0.0.1:18765/skip/enable
# 期望 ok=true, readback_enabled=true, story/train_high_speed_type=2
# 重启游戏再查 /skip/status → enabled 仍为 true（SQLite 持久化生效）
```

然后开一局育成验证：
- 剧情以超高速滚动（官方 SUPER_HIGH_SPEED 链生效）
- 事件/支援卡**选项框照常弹出**（choice_guard_101 保持 false，模块从不写它）
- `/api/sniff/status` 的 last_id 对比调用前后无增量（纯本地设置）

## 兜底（P1，可选）

若个别场景读取的是另一条静态路径，再 hook 静态方法
`Gallop.StoryTimelineController.IsSettingSuperHighSpeedSkip`（0 参 → bool）
用现有 Interceptor API（参考 SNIFF 的 COMPRESS_REQUEST_ADDR 装钩方式）恒返 true。
本模块默认不需要。
