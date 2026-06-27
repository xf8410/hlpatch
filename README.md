# Hachimi URA Plugin — 赛马娘育成内存读取插件

基于 [Hachimi](https://github.com/akemiko/hachimi) 框架的赛马娘育成辅助插件，通过注入游戏进程读取 IL2CPP 内存数据，提供 HTTP 端点查询与自动推送能力。

## 功能亮点

- **实时属性读取**：速度/耐力/力量/毅力/贤、体力/体力上限/技能Pt/粉丝数/干劲，一键获取当前育成状态。
- **训练增益计算**：自动解析各训练项目的属性增益与体力消耗，含训练等级提升信息。
- **Buff 状态映射**：从 CharaEffectId 直接映射可读名称（夜鷹/怠け/肌荒れ/練習上手 等），区分 Good/Bad 类型，全剧本通用。
- **生病检测**：基于 CharaEffectId 1-6 判定不良状态，不再依赖无效的 State 字段。
- **HTTP 端点**：`/summary` 返回完整育成数据 JSON，`/status` 返回插件运行状态与崩溃阶段追踪。
- **自动推送**：push_loop 定时向浮窗 App 推送数据，内置错误冷却退避防止高频 IL2CPP 调用导致崩溃。
- **崩溃防护**：catch_unwind 包裹核心调用，/status 端点暴露 LAST_PHASE 变量辅助定位崩溃阶段。

## API 端点

| 端点 | 端口 | 说明 |
|------|------|------|
| `GET /summary` | 18765 | 返回完整育成状态 JSON（属性/训练/Buff/支援卡等） |
| `GET /status` | 18765 | 返回插件运行状态（game_initialized / last_phase） |

## /summary 响应示例

```json
{
  "version": "3.14.2",
  "month": 6, "half": 2, "scenario": "URA",
  "stats": {
    "speed": 300, "stamina": 200, "power": 150,
    "guts": 100, "wiz": 80, "vital": 70, "max_vital": 100,
    "motivation": "Best", "skill_point": 50, "fan": 10000
  },
  "trainings": [...],
  "support_cards": [...],
  "buffs": [{"name": "練習上手", "level": 0, "desc": "練習上手", "type": "Good"}],
  "chara_effect_ids": [10, 7]
}
```

## 目录结构

- `hachimi_ura_plugin/src/lib.rs`：插件核心逻辑（约 2500 行 Rust）。
- `hachimi_ura_plugin/Cargo.toml`：构建配置，目标 `aarch64-linux-android`。
- `.github/workflows/`：GitHub Actions 自动编译 SO 文件。

## 环境要求

- Rust 工具链（nightly，含 `aarch64-linux-android` target）。
- Android NDK r25+。
- Hachimi 框架（插件宿主环境）。
- 赛马娘日服客户端。

## 构建

```bash
# 安装 target
rustup target add aarch64-linux-android

# 编译 release
cd hachimi_ura_plugin
cargo build --release --target aarch64-linux-android
```

产物位于 `target/aarch64-linux-android/release/libhachimi_ura.so`，重命名为 `libhachimi_ura_vX.Y.Z.so` 即可使用。

也可直接使用 GitHub Actions 自动构建。

## 安装

1. 将编译好的 SO 文件放入 Hachimi 插件目录。
2. 启动游戏，插件自动加载。
3. 浏览器访问 `http://localhost:18765/status` 确认运行状态。

## 技术栈

- **语言**：Rust（no_std + il2cpp bindings）
- **构建**：Cargo + NDK cross-compile
- **注入**：Hachimi native hook 框架
- **通信**：HTTP Server（tiny_http），JSON 序列化

## 故障排查

- **游戏闪退**：浏览器访问 `18765/status`，查看 `last_phase` 值定位崩溃阶段。
- **浮窗无数据**：确认 `/status` 返回 `game_initialized: true`；插件使用 probe 探测机制，游戏启动后自动初始化。
- **属性全部为 0**：确认插件版本 ≥ 3.14.1，Int32 字段已修复读取方式。
- **Buff 为空**：确认插件版本 ≥ 3.14.2，CharaEffectId 映射已启用。

## 许可证

本项目仅供学习研究使用，请勿用于商业用途。
