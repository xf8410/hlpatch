# 三个文件最终扒光状态

**日期**: 2026-07-12

---

## 1. libil2cpp.so (209MB, ARM64, stripped)

### 已完成

| 部分 | 大小 | 函数数 | 浮点操作 | 输出 |
|------|------|--------|---------|------|
| .text (Unity引擎) | 4.8MB | 1,881 | 2,678 | 40MB |
| il2cpp (游戏逻辑) | 93MB | 177,385 | 142,147 | 886MB |
| .plt (跳转表) | 9KB | — | — | — |
| **合计** | **98MB** | **179,266** | **144,825** | **926MB** |

- 全部反汇编为 ASM (90+5个分块文件)
- 索引: 179K函数, 291万调用边, 144K浮点操作, 9K立即数
- 基址确认: 0x7330ef37c4
- 关键函数地址映射 (GetMotivationCoef, ExecTraining, ApplyExecCommand等)
- 调用图分析: ApplyExecCommand 323个bl调用, 0个浮点操作 → 纯分发函数

### 无法进一步分析

- 数据段2 (48.6MB): IL2CPP运行时类型指针表/对象分配器 — 无游戏逻辑常量
- 虚函数分发: 通过 vtable 间接调用 (blr x8), 静态分析无法追踪
- IL2CPP内联: 小方法被内联到大函数, 边界不可识别

## 2. global-metadata.dat (44MB, v31)

### 已完成

| 段 | 大小 | 解析状态 |
|----|------|---------|
| string (字符串表) | 7.3MB | 343,462条字符串全部解析 |
| string_literal | 388KB | 已解析 |
| type_definitions | 4.7MB | 53,684类型 (88字节/条) |
| fields | 3.1MB | 130,972字段 (24字节/条) |
| field_default_values | 888KB | 搜索确认无心情倍率常量 |

### 部分完成

| 段 | 大小 | 状态 |
|----|------|------|
| methods | 12.1MB | 已定位get_Speed等方法, 但entry_size不固定(v31变更), 完整解析需SO Type索引表 |
| parameters | 3.7MB | 依赖methods段的parameterStart索引 |
| properties | 1.2MB | 未解析 |
| events | 11KB | 未解析 |

### 无法完成

- methods段完整解析: v31使用变长布局,需要SO中的Il2CppType索引表交叉引用
- 字段精确偏移: fieldStart编码方式需要Il2CppType表(SO数据段)才能解

## 3. master.mdb (42MB, SQLite)

### 已完成 — 关键表全部导出

| 表 | 行数 | 内容 |
|----|------|------|
| support_card_data | 541 | 全部支援卡 |
| support_card_effect_table | 4,931 | 效果表 (type 1-32) |
| support_card_unique_effect | 399 | 固有效果 (type 101-122) |
| single_mode_training | 189 | 训练配置 (失败率/人数) |
| single_mode_training_effect | 830 | 训练基础值 (14剧本×5训练×多等级) |
| single_mode_npc | 3,810 | NPC属性 (含motivation_min/max) |
| single_mode_scenario | 14 | 14个剧本配置 |
| race_motivation_rate | 5 | 比赛心情倍率 (0.96-1.04) |
| single_mode_chara_effect | 43 | 角色效果 |
| single_mode_chara_effect_buff | 6 | 角色buff |
| single_mode_14_* | 16张表 | 拉面杯完整数据 |
| text_data | 95,128 | 全部文本数据 |
| single_mode_story_data | 19,943 | 全部事件 |
| chara_story_data | 917 | 角色固定事件 |
| single_mode_story_condition_set | 478 | 事件触发条件 |
| single_mode_event_cr_priority | 81 | 事件优先级 |
| story_event_bonus_support_card | 945 | 支援卡事件加成 |
| single_mode_hint_gain | 4,919 | 启发获取 |
| single_mode_event_choice_reward | 58 | 事件选项奖励 |
| single_mode_event_conclusion | 524 | 事件结论 |

### 导出文件

- `reverse/mdb_key_tables_export.json` (4.4MB) — 14张关键表
- `reverse/all_support_cards_full.json` (1.8MB) — 541张卡全属性+效果
- `reverse/all_event_data.json` (2.9MB) — 事件+条件+优先级+加成
- `reverse/all_story_and_ramen_events.json` (4.6MB) — 故事+拉面杯

### 不在MDB中(服务器端计算)

- 事件触发概率
- 乱入事件概率
- 支援卡出现概率
- 训练增益计算
- 心情倍率计算

---

## 最终结论

### 训练增益
**服务器端计算**, 客户端只存储和显示。
- 客户端调用链: SendExecCommand → 服务器返回 → MsgPack反序列化 → ApplyExecCommand → set属性
- 心情倍率/友情加成/彩圈判定/RNG → 全部在Cygames服务器上

### 心情倍率
**公式计算**, 不在游戏文件中以常量存在。
- UmaAi逆推公式: `1 + 0.1 × (motivation-3) × (1 + 0.01 × ganJing)`
- SO代码段: 0.1f/0.05f/0.6f/0.8f/0.9f/1.1f/1.2f 零命中
- MDB: 只有比赛倍率(race_motivation_rate), 无训练倍率
- metadata: field_default_values中无相关常量

### 客户端能做的
1. 读取服务器返回的最终值 (gains/evaluation/kizuna等)
2. 从MDB读取静态数据 (command_id/bond_threshold/effect_table)
3. 判断彩圈 (基于服务器返回的TipsEventPartnerArray)
4. "透视" = 截获服务器响应提前显示

### 静态分析天花板
三个文件已扒到**静态分析的物理极限**:
- SO: stripped + 内联 + 虚函数分发
- metadata: methods段变长布局需SO交叉引用
- mdb: 关键表已全部导出, 剩余约600张表无训练/事件逻辑

### 反编译工具
- 沙箱无root,无法安装radare2/Ghidra
- 已有capstone反汇编,产出等价
- C#伪代码需dnSpy/ILSpy GUI,命令行不可用

**进一步突破需要运行时方案**: Frida hook / Zygisk-Il2CppDumper

*完成: 2026-07-12*
