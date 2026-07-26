# Hachimi URA Plugin — 赛马娘育成内存读取插件

基于 [Hachimi](https://github.com/akemiko/hachimi) 框架的赛马娘育成辅助插件，通过注入游戏进程读取 IL2CPP 内存数据，提供 HTTP 端点查询与自动推送能力。

> **2026-07-26（v3.24.68）**
> - 新增 `GET /ramen` 轻量端点：拉面杯数据（回合+盛況度/素材/隠し味/地区），浮窗轮询专用
> - 修正盛況度档位为 MDB `check_point_pt_effect` 真实 11 档（250/500/1000/1500/2000/2500/3000/3500/4000/5000），旧阈值为猜测值
>
> **2026-07-23（v3.24.32）**
> - HTTP 服务仅绑定 `127.0.0.1:18765`；局域网/电脑调试请用 `adb forward tcp:18765 tcp:18765`
> - 拉面杯候选地区池只在选择回合（3/24/48）以 `selectable_region_ids_derived` 输出；其余回合移至 `region_pool_for_latest_selection_phase_derived` 并标 `currently_selectable_status`
> - 体积数据导出（reverse/ 下 19 个 JSON/TXT，211MB）已迁移至 [uma-data exports/](https://github.com/xf8410/uma-data/tree/main/exports)（LFS）；仓库历史已重写瘦身，旧 commit SHA 失效，请按 tag 检索

## 功能亮点

- **实时属性读取**：速度/耐力/力量/毅力/贤、体力/体力上限/技能Pt/粉丝数/干劲
- **训练增益计算**：自动解析各训练项目的属性增益与体力消耗，含训练等级提升
- **Buff 状态映射**：CharaEffectId → 可读名称（夜鷹/怠け/肌荒れ/練習上手 等）
- **評価点计算**：per-stat独立查表+cubic外推，支持0-2300全量属性范围
- **技能评分**：计算skill_score合并属性评估点
- **HTTP 端点**：20+端点覆盖育成数据/类结构/字段偏移/方法枚举/配置
- **自动推送**：push_loop定时向浮窗App推送数据，内置错误冷却退避
- **IL2CPP自省**：`/fields`/`/methods`/`/classes` 端点可直接探查游戏类结构

## HTTP 端点总览

插件端口：`18765`（可通过 `/config` 修改）

### 育成数据端点

| 端点 | 用途 |
|------|------|
| `GET /summary` | 完整育成状态（属性/训练/Buff/支援卡/評価点/技能评分） |
| `GET /ramen` | 拉面杯轻量数据（回合/盛況度11档/素材槽/隠し味/已选地区） |
| `GET /data` | 原始训练数据 |
| `GET /scenario` | 剧本详情（Ramen等特殊剧本字段） |
| `GET /log` | 训练日志 |
| `GET /carddb` | 卡片数据库 |
| `GET /skilldata` | 技能数据 |
| `GET /hall` | 殿堂马数据（評価点+属性+rankScore） |
| `GET /ranking` | 排行榜数据（服务端拉取） |
| `GET /saddles` | 鞍一览 |
| `GET /saddles-dl` | 鞍数据下载 |

### IL2CPP 探查端点（逆向核心）

| 端点 | 用途 | 示例 |
|------|------|------|
| `GET /classes/search/{keyword}` | 按关键词搜索类名 | `/classes/search/SingleMode` |
| `GET /fields/{ClassName}` | 列出类所有字段+偏移量 | `/fields/SingleModeStoryData` |
| `GET /methods/{ClassName}` | 列出类所有方法 | `/methods/SingleModeStoryData` |
| `GET /scan` | 扫描所有IL2CPP类 | |
| `GET /singletons` | 查找所有单例 | |
| `GET /find_method/{name}` | 在所有类中搜索方法 | `/find_method/get_Title` |

### 调试与配置

| 端点 | 用途 |
|------|------|
| `GET /status` | 插件运行状态（game_initialized / last_phase） |
| `GET /health` | 健康检查+版本号 |
| `GET /debug/params` | 调试参数增减 |
| `GET /debug/breeders` | 调试训练师队伍 |
| `GET /config` | 当前配置 |
| `POST /config` | 更新配置（JSON body） |
| `GET /config.html` | 配置页面（浏览器直接打开） |

## IL2CPP 事件数据类结构（已验证偏移）

以下是赛马娘育成事件系统的完整类层级和字段偏移，通过 `/fields` 端点实测获取。

### 类层级关系

```
MasterStoryDatabase                    ← 总库（所有故事/事件容器）
├── MasterSingleModeStoryData          ← 事件元数据表（Master查询器）
│   └── SingleModeStoryData            ← 事件元数据行（33字段）
├── MasterSingleModeEventChoiceReward  ← 选择肢奖励表（Master查询器）
│   └── SingleModeEventChoiceReward    ← 选择肢奖励行（5字段）
├── EventChoiceBranchReward            ← 分支奖励（_gainParamArray）
│   └── EventChoiceRewardGainParam     ← 属性增益（4字段，ObscuredInt）
├── SingleModeEventConclusion          ← 事件结局（3字段，动画数据）
├── SingleModeEventPlayTiming          ← 事件触发时机（枚举）
├── MasterSingleModeStoryConditionSet  ← 事件触发条件表
└── SingleModeEventAccesor            ← 运行时读事件数据
```

### SingleModeStoryData — 事件元数据（33字段）

| 字段 | 偏移 | 说明 |
|------|------|------|
| Id | 16 | 主键 |
| StoryId | 20 | 事件ID |
| ShortStoryId | 24 | 短事件ID |
| CardId | 28 | 角色卡ID |
| CardCharaId | 32 | 角色卡角色ID |
| **SupportCardId** | **36** | **支援卡ID（匹配事件到卡的关键字段）** |
| SupportCharaId | 40 | 支援卡角色ID |
| ShowProgress1 | 44 | 出现时机1 |
| ShowProgress2 | 48 | 出现时机2 |
| ShowProgress3 | 52 | 出现时机3 |
| ShowClear | 56 | 通关时出现 |
| ShowSuccession | 60 | 继承时出现 |
| EventTitleStyle | 64 | 标题样式 |
| EventTitleDressIcon | 68 | 标题服装图标 |
| EventTitleCharaIcon | 72 | 标题角色图标 |
| SeChange | 76 | SE变化 |
| EndingType | 80 | 结局类型 |
| RaceEventFlag | 84 | 比赛事件标记 |
| MiniGameResult | 88 | 小游戏结果 |
| GalleryMainScenario | 92 | 画廊主剧本 |
| GalleryFlag | 96 | 画廊标记 |
| GalleryListId | 100 | 画廊列表ID |
| GalleryGruopId | 104 | 画廊分组ID（原文拼写） |
| GallerySort | 108 | 画廊排序 |
| GalleryCondition | 112 | 画廊条件 |
| GallerySuggestEvent | 116 | 画廊推荐事件 |
| AvailableGalleryKey | 120 | 可用画廊键 |
| PastRaceId | 124 | 过去比赛ID |
| PastRaceId2 | 128 | 过去比赛ID2 |
| PastRaceId3 | 132 | 过去比赛ID3 |
| PastRaceId4 | 136 | 过去比赛ID4 |
| ForceUseRaceDress | 140 | 强制比赛服装 |
| **EventCategory** | **141** | **事件分类（区分训练/友情/连续事件等）** |

**关键方法**（getter属性，不在字段中）：
- `get_Title()` — 事件标题
- `get_CategoryForSkip()` — 跳过用分类
- `IsHintEvent()` — 是否提示事件
- `IsOnsenEvent()` — 是否温泉事件
- `CheckEndingType()` — 检查结局类型

### SingleModeEventChoiceReward — 选择肢奖励（5字段）

| 字段 | 偏移 | 说明 |
|------|------|------|
| Id | 16 | 主键（关联StoryId） |
| DispType | 20 | 显示类型（byte） |
| EffectValueType0 | 21 | 效果值类型0（byte：Speed/Stamina/Power/Guts/Wisdom等枚举） |
| EffectValueType1 | 22 | 效果值类型1 |
| EffectValueType2 | 23 | 效果值类型2 |

**DispType + EffectValueType → 决定效果含义**：DispType标识奖励展示方式，EffectValueType标识具体属性类型。

### EventChoiceBranchReward — 分支奖励（1字段）

| 字段 | 偏移 | 说明 |
|------|------|------|
| _gainParamArray | 16 | 指向 EventChoiceRewardGainParam[] 数组 |

### EventChoiceRewardGainParam — 属性增益（4字段）

| 字段 | 偏移 | 说明 |
|------|------|------|
| _displayId | 16 | 显示ID |
| _effectValue0 | 36 | 效果值0（增益量） |
| _effectValue1 | 56 | 效果值1 |
| _effectValue2 | 76 | 效果值2 |

**⚠️ ObscuredInt加密**：offset 36/56/76间距20字节（正常int间距4字节），说明_effectValue0/1/2使用了ObscuredInt加密，每个占20字节。

### SingleModeEventConclusion — 事件结局（3字段）

| 字段 | 偏移 | 说明 |
|------|------|------|
| Id | 16 | 主键 |
| CharaId | 20 | 角色ID |
| CharaMotionSetId | 24 | 角色动作集ID（播放动画用） |

### SingleModeEventPlayTiming — 事件触发时机（枚举）

枚举值（value__偏移16），表示事件在什么时机触发：

| 枚举值 | 说明 |
|--------|------|
| None | 无 |
| TurnStart | 回合开始 |
| RaceStart | 比赛开始 |
| RaceEnd | 比赛结束 |
| TurnEnd | 回合结束 |
| ModeEnd | 育成结束 |
| CommandStart | 指令开始（训练选择后） |
| Continue | 继续 |
| MiniGameEnd | 小游戏结束 |
| TeamRaceEnd | 团队赛结束 |
| TeamRaceAfterEvent | 团队赛后事件 |
| TeamRaceAfterTeamParameterRankUp | 团队赛参数升级后 |
| LiveEnd | Live结束 |
| GRSEnd | GRS剧本结束 |
| ArcEnd | Arc剧本结束 |
| SportCompetitoionEnd | 运动竞技结束 |
| CookTastingEnd | 料理品尝结束 |
| MechaEnd | 机甲剧本结束 |
| LegendScenarioRaceEnd | 传说剧本比赛结束 |
| LegendLastScenarioRaceEnd | 传说最终剧本比赛结束 |
| PioneerCheckPointEnd | 先驱者检查点结束 |
| OnsenCheckPointEnd | 温泉检查点结束 |
| BreedersTeamReviewEnd | 训练师队伍审查结束 |
| BreedersBeforeBCRaceCard | 训练师BC赛马卡前 |
| **RamenCheckPointEnd** | **拉面剧本检查点结束** |

## IL2CPP 加速相关类（已识别）

| 类 | 用途 | 备注 |
|----|------|------|
| RaceTimeController | 比赛时间控制器 | 比赛内加速 |
| CourseTimescaleParam | 赛道TimeScale参数 | Hachimi已覆盖 |
| DialogReduceTimeJobsOne | 对话加速完成 | 游戏内原生加速 |
| DialogReduceTimeJobs | 对话加速批量 | 游戏内原生加速 |
| PartsReduceTimeJobsControl | 加速等待控制 | 读条/等待 |
| SingleModeResultPassBaseRemainTimeModel | 育成结果等待时间 | 结果界面 |
| CutinTimeScaleCurve | 过场TimeScale曲线 | 动画 |
| TimelineKeyTimeScaleData | 时间轴TimeScale数据 | Live/Cutin |

**⚠️ 注意**：Hachimi已有10倍速UI和TimeScale控制，加速插件不要碰全局TimeScale，重点做Hachimi没覆盖的：跳训练动画、快进读条、自动选事件。

## 逆向探查教程

### 1. 搜索类名

```
http://127.0.0.1:18765/classes/search/SingleMode
```

返回匹配的类列表（字符串匹配，不依赖metadata解密）。

### 2. 查看字段偏移

```
http://127.0.0.1:18765/fields/SingleModeStoryData
```

返回所有字段名、偏移量、所属类（含父类字段）。

**⚠️ 路径注意**：正确路径是 `/fields/ClassName`，**不是** `/classes/search/ClassName/fields`！

### 3. 查看方法列表

```
http://127.0.0.1:18765/methods/SingleModeStoryData
```

返回所有方法名和所属类。getter属性（如`get_Title`）不出现在fields里，但可以在methods里看到。

### 4. 类名规则

| 模式 | 含义 | 用途 |
|------|------|------|
| `MasterXxxData` | Master表查询器 | 用`Get()`查数据行，只有SQLite基类字段 |
| `XxxData` | 数据行 | **有业务字段和偏移，探查时用这个** |
| `XxxAccessor` | 运行时访问器 | 读当前游戏实例 |
| `IXxxData` | 接口 | 查实现类 |

### 5. 常用搜索关键词

| 搜索词 | 用途 |
|--------|------|
| `SingleMode` | 育成相关所有类 |
| `CharaStatus` | 角色属性 |
| `TimeScale` | 加速/时间缩放 |
| `EventChoice` | 事件选择肢 |
| `Rank` | 排行 |
| `Score` | 评分 |
| `Effect` | 效果/Buff |
| `WorkData` | 育成运行时数据 |

### 6. metadata加密说明

Cy加密了IL2CPP metadata，导致：
- ✅ `/classes/search/` 正常工作（字符串匹配）
- ✅ `/fields/` 正常工作（运行时反射，绕过metadata）
- ✅ `/methods/` 正常工作（同上）
- ❌ 静态il2cppdumper无法直接dump（需Zygisk-Il2CppDumper绕过加密）

**结论**：URA插件的运行时反射端点已绕过metadata加密，无需il2cppdumper即可获取字段偏移。

## 关联项目

- [uma-juece](https://github.com/xf8410/uma-juece) — 浮窗App（Java/Android），接收推送+AI决策推荐
- [uma-train](https://github.com/xf8410/uma-train) — 训练框架（Python），MCTS+神经网络
- [uma-data](https://github.com/xf8410/uma-data) — 事件数据（JSON），支援卡事件选择肢+奖励

## 环境要求

- Hachimi 框架（插件宿主环境）
- 赛马娘日服客户端
- GitHub Actions 自动编译（或本地 Rust + NDK）

## 构建

使用 GitHub Actions 自动编译，产物在 Releases 页面下载。

本地构建：
```bash
cd hachimi_ura_plugin
cargo build --release --target aarch64-linux-android
```

产物：`target/aarch64-linux-android/release/libhachimi_ura.so`

## 安装

1. 下载对应版本的 SO 文件，重命名为 `libhachimi_ura_vX.Y.Z.so`
2. 放入 Hachimi 插件目录
3. 启动游戏，插件自动加载
4. 浏览器访问 `http://127.0.0.1:18765/health` 确认运行

## 故障排查

- **游戏闪退**：访问 `/status`，查看 `last_phase` 定位崩溃阶段
- **浮窗无数据**：确认 `/status` 返回 `game_initialized: true`
- **属性全部为0**：确认插件版本 ≥ 3.14.1，Int32字段已修复
- **/fields 返回空**：确认用的是 `/fields/ClassName` 而非 `/classes/search/ClassName/fields`
- **类搜不到**：尝试不同关键词，部分类名可能拼写不同

## 许可证

本项目仅供学习研究使用，请勿用于商业用途。
