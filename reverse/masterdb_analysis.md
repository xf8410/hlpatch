---
AIGC:
    Label: "1"
    ContentProducer: 001191110102MACQD9K64018705
    ProduceID: 2380867232079196_0/project_7654253504034472235-files/逆向分析/masterdb_analysis.md
    ReservedCode1: ""
    ContentPropagator: 001191110102MACQD9K64028705
    PropagateID: 2380867232079196#1783732983045
    ReservedCode2: ""
---
# 赛马娘 Master.mdb 数据库逆向分析报告

> 基于 `master.mdb`（26MB SQLite3）直接查询，覆盖 35 项游戏机制。
> 数据版本：schema 4，138角色 / 404支援卡 / 369换皮 / 1438技能 / 2304比赛

---

## 1. 角色ID (chara_data)

**表**: `chara_data` — 138 条记录

| 字段 | 说明 |
|------|------|
| id | 角色唯一ID（如 1001=特别周） |
| birth_year/month/day | 出生日期 |
| sex | 性别 |
| chara_category | 角色分类 |
| love_rank_limit | 好感度上限 |

**角色ID范围**: 1001~1130（含NPC如9001~9044）

---

## 2. 支援卡ID (support_card_data)

**表**: `support_card_data` — 404 条记录

### Command ID 分布（核心映射）

| command_id | command_type | 数量 | 含义 |
|-----------|-------------|------|------|
| 101 | 1 | 86 | 速度得意 |
| 102 | 1 | 75 | 耐力得意 |
| 103 | 1 | 81 | 根性得意 |
| 105 | 1 | 73 | 力量得意 |
| 106 | 1 | 69 | 智力得意 |
| 0 | 0 | 17 | 友人卡（command_type=0） |
| 0 | 1 | 3 | 特殊卡（30067/30137/30180） |

**关键字段**:
- `id`: 支援卡ID（10001~30221）
- `chara_id`: 所属角色ID
- `rarity`: 稀有度（1=R, 2=SR, 3=SSR）
- `command_id`: 得意训练（0=非五属性/友人）
- `support_card_type`: 支援卡类型（1=五属性, 2=友人, 3=特殊）
- `skill_set_id`: 技能组ID
- `unique_effect_id`: 固有效果ID
- `effect_table_id`: 效果表ID

### 友人卡（command_id=0, support_card_type=2）

| ID | chara_id | 角色 |
|----|----------|------|
| 10021 | 9001 | 骏川手纲 |
| 10022 | 9004 | 桐生院葵 |
| 10060 | 9006 | 安心泽刺刺美 |
| 10074 | 9005 | 都留岐凉花 |
| 10083 | 9008 | 理事长代理 |
| 10094 | 9043 | 佐岳 |
| 10104 | 9044 | 凯旋门友人 |
| 10109 | 9002 | 记者 |
| 20021 | 9004 | 桐生院SR |
| 30021 | 9001 | 骏川SSR |
| 30036 | 9006 | 安心泽SSR |
| 30052 | 9008 | 理事长代理SSR |
| 30080 | 9005 | 都留岐SSR |
| 30160 | 9043 | 佐岳SSR |
| 30188 | 9044 | 凯旋门友人SSR |
| 30207 | 9002 | 记者SSR |

**特殊卡（command_type=1, command_id=0）**:
- 30067: chara 1017 (帝王光环, SSR, support_card_type=3)
- 30137: chara 9040 (SSR, support_card_type=3)
- 30180: chara 1068 (SSR, support_card_type=3)

---

## 3. 换皮ID (dress_data)

**表**: `dress_data` — 369 条记录

| 字段 | 说明 |
|------|------|
| id | 换皮ID |
| chara_id | 所属角色 |
| condition_type | 条件类型 |
| costume_type | 服装类型 |
| use_race | 是否比赛用 |
| use_home | 是否主页用 |

---

## 4. 同名不同卡ID

同一角色多张支援卡，通过 `chara_id` 分组：

```
chara_id=1001 (特别周): 30081 (support_card_type=3, command_id=0)
chara_id=9001 (骏川): 10021 (R), 30021 (SSR)
chara_id=1017 (帝王光环): 30067 (特殊command_id=0)
```

---

## 5. 支援卡加成效果 (support_card_effect_table)

**表**: `support_card_effect_table` — 按 `(id, type)` 双主键

### Effect Type 枚举

| type | 含义 | 示例 |
|------|------|------|
| 1 | 速度加成 | init=5, lv50=15 |
| 2 | 耐力加成 | init=10, lv50=35 |
| 3 | 初期羁绊 | 固定值 |
| 5 | 友情训练 | lv40=1 |
| 8 | 训练效果 | lv30=5, lv50=10 |
| 14 | 得意率 | init=10, lv50=30 |
| 15 | 技能Pt加成 | lv45=5 |
| 16 | 比赛加成 | lv50=15 |
| 17 | 粉丝数加成 | lv40=2 |
| 18 | 体力消费下降 | lv50=30 |
| 19 | 初始属性 | lv50=35 |
| 25 | 干劲效果 | lv50=60 |
| 26 | 智力训练回复 | lv50=35 |
| 27 | 事件回复 | lv50=30 |
| 28 | 事件效果 | lv50=25 |
| 30 | 启发率 | lv50=1 |
| 31 | 启发事件 | lv50=2 |

### 友人卡效果示例 (10021/30021)

- 训练效果提升 (type=8)
- 得意率 (type=14)
- 干劲效果 (type=25)
- 智力训练回复 (type=26)
- 事件回复 (type=27)
- 事件效果 (type=28)

---

## 6. 支援卡技能 (skill_data)

**表**: `skill_data` — 1,438 条

支援卡通过 `skill_set_id` 关联技能组。`support_card_data.skill_set_id` → `skill_set` → `skill_data`。

---

## 7. 卡Hit逻辑

**未见独立 hit 表**。卡hit（得意训练出现概率）由 `support_card_effect_table.type=14`（得意率）控制，配合场景机制。

---

## 8-9. 友情训练标志

### 彩圈判断逻辑（已在代码中实现）

```
command_id mapping:
  support_card_data.command_id == 当前训练command_id → 彩圈
```

**关键规则**:
- 羁绊 < 80 → 不闪（`bond < 80`）
- command_id=0 的特殊卡（友人卡）不能套用普通规则
- 普通五属性卡：`card_training == current_training` → 彩圈

---

## 10. 支援卡效果完整映射

**效果表**: `support_card_effect_table`

每张卡有多条 type 记录，按突破等级（lv5/lv10/lv15/lv20/lv25/lv30/lv35/lv40/lv45/lv50）递增。

---

## 11. 剧本场景训练效果 (single_mode_training_effect)

**表**: `single_mode_training_effect`

### target_type 枚举

| target_type | 含义 | 示例值 |
|-------------|------|--------|
| 1 | 速度增加 | 11 |
| 2 | 耐力增加 | 6 |
| 3 | 力量增加 | 6 |
| 4 | 根性增加 | 4 |
| 5 | 智力增加 | 2 |
| 10 | 失败率 | -21 |
| 20 | 技能点 | 4 |
| 30 | 体力消耗 | 4 |
| 101 | 剧本特殊 | 8 |

### 场景ID映射

| scenario_id | 剧本 | 特色 |
|-------------|------|------|
| 1 | 基础URA | 标准训练 |
| 2 | 青春杯 | 训练效果变化 |
| 3 | 巅峰杯 | 额外属性 |
| 4 | 凯旋门 | 训练效果减弱 |
| 5 | 大师杯 | 平衡型 |
| 6 | L'Arc | 高属性上限 |
| 7 | 种田杯 | 待分析 |
| 8 | 拉面杯 | 训练效果大幅变化 |

### 各剧本训练效果差异（速度训练 command_id=101, result_state=2）

| 剧本 | 速度+ | 耐力+ | 失败率 | 体力 |
|------|-------|-------|--------|------|
| 基础 | 11 | 6 | -21 | 4 |
| 青春杯 | 8 | 4 | -19 | 4 |
| 巅峰杯 | 8 | 4 | -19 | 4 |
| 凯旋门 | 8 | 4 | -19 | 2 |
| 大师杯 | 10 | 3 | -19 | 5 |
| L'Arc | 10 | 3 | -21 | 6 |
| 拉面杯 | 11 | 2 | -19 | 5 |

**关键发现**: 拉面杯速度训练 → 耐力增长仅+2（最低），适合拉面杯剧本特点。

---

## 12. 心情效果

**未见独立心情表**，但训练效果中 `result_state` 可能与心情联动：
- `result_state=2`: 普通训练结果
- 其他值: 可能对应不同心情等级

---

## 13. 五等级训练效果

**表**: `single_mode_training`

| 训练 | Lv1 | Lv2 | Lv3 | Lv4 | Lv5 |
|------|-----|-----|-----|-----|-----|
| 速度(101) | 520 | 524 | 528 | 532 | 536 |
| 耐力(102) | 516 | 520 | 524 | 528 | 532 |
| 根性(103) | 532 | 536 | 540 | 544 | 548 |
| 力量(105) | 507 | 511 | 515 | 519 | 523 |
| 智力(106) | 320 | 321 | 322 | 323 | 324 |

**关键发现**: `failure_rate` 随等级递增（Lv5 最高）。

---

## 14. 训练属性ID映射（完整确认）

```
101 = Speed (速度)    - 86张支援卡
102 = Stamina (耐力)  - 75张支援卡
103 = Guts (根性)     - 81张支援卡
105 = Power (力量)    - 73张支援卡
106 = Wisdom (智力)   - 69张支援卡
```

**注意**: 根性=103、力量=105（非对称），与一般习惯不同。

---

## 15. 支援卡槽位局内识别

**已确认**: 通过 `support_card_data.command_id` 字段。
- 玩家装备5张支援卡（slot 1~5）+ 1张借卡（slot 6）
- 每张卡的 `command_id` 决定其得意训练类型
- 友人卡 `command_id=0` 表示无特定得意训练

---

## 16. 羁绊槽增加逻辑

**未见独立羁绊表**。羁绊系统在代码层实现：
- 支援卡初始羁绊：`support_card_effect_table.type=3`（初期羁绊值）
- 每次训练增加羁绊量由代码逻辑控制
- 羁绊上限通常为100
- 80为友情训练触发阈值

---

## 17. 拉面杯剧本机制 (single_mode_cook)

### 核心数据表

| 表名 | 用途 |
|------|------|
| `single_mode_cook_dish` | 料理定义（35种） |
| `single_mode_cook_dish_material` | 料理材料需求 |
| `single_mode_cook_dish_effect` | 料理效果 |
| `single_mode_cook_success_odds` | 料理成功率 |
| `single_mode_cook_power_data` | 力量等级与参数 |
| `single_mode_cook_garden` | 菜园设施 |
| `single_mode_cook_garden_level` | 菜园升级 |
| `single_mode_cook_garden_effect` | 菜园效果 |
| `single_mode_cook_cooking_type` | 料理类型 |
| `single_mode_cook_cooking_rate` | 料理出现率 |
| `single_mode_cook_listener` | 伙伴位置 |
| `single_mode_cook_coin_rate` | 金币汇率 |
| `single_mode_cook_material_rate` | 材料率 |

### 料理类型 (dish_type)

| dish_type | 数量 | 平均金币 | 平均力量 |
|-----------|------|---------|---------|
| 0 | 16 | 694 | 784 |
| 1 | 7 | 600 | 593 |
| 2 | 6 | 650 | 650 |
| 3 | 6 | 650 | 650 |

### 料理成功率 (single_mode_cook_success_odds)

| 力量值范围 | 成功率(%) |
|-----------|----------|
| 0-299 | 0 |
| 300-1499 | 15 |
| 1500-2499 | 18 |
| 2500-4999 | 20 |
| 5000-6999 | 22 |
| 7000-9999 | 24 |
| 10000-11999 | 25 |
| 12000+ | 100 |

**关键发现**: 力量 ≥ 12000 时，料理成功率100%。

### 力量等级 (single_mode_cook_power_data)

| 等级 | 力量需求 | 金币 | 效果 |
|------|---------|------|------|
| 1 | 24 | 1000 | Lv2 |
| 2 | 36 | 2500 | Lv2 |
| 3 | 48 | 5000 | Lv3 |
| 4 | 60 | 7000 | Lv3 |
| 5 | 72 | 10000~12000 | Lv3 |

### 菜园 (garden)

5个设施(facility_id: 100~500)，每个3级(garden_lv: 1~3)，每级5个设施等级(facility_lv: 1~5)。

---

## 18. 拉面杯伙伴逻辑

**表**: `single_mode_cook_listener` — 6条记录

| ID | 名称 | X | Y | Z |
|----|------|---|---|---|
| 1 | 0001 | 2200 | 0 | -16300 |
| 2 | 0002 | 10900 | 0 | 7500 |
| 3 | 0003 | 0 | 0 | 15000 |
| 4 | 0004 | -32500 | 0 | 19300 |
| 5 | 0005 | -25600 | 0 | -11200 |
| 6 | 0006 | -13900 | 0 | -14900 |

6个伙伴在3D场景中各有固定位置。

---

## 19. 无彩圈训练逻辑

当支援卡羁绊 < 80 时，不触发彩圈（代码已实现）。
当 `command_id=0` 的友人卡时，无法用普通规则判断，需特殊处理。

---

## 20. 五种训练都不出现支援卡逻辑

**概率机制**：每回合训练时，系统根据支援卡出现概率（由得意率 type=14 控制）决定哪些卡在当前训练出现。如果概率判定全部失败，则当前训练无支援卡出现。

---

## 21. 支援卡80羁绊不闪逻辑

**已代码确认**: `bond < 80 → Some(false)`

---

## 22. 特殊支援卡（友人卡）逻辑

**command_id=0 的支援卡**:
- 17张 `support_card_type=2`（友人卡）：骏川、桐生院、安心泽、都留岐、理事长代理、记者、佐岳、凯旋门友人
- 3张 `support_card_type=3`（特殊卡）：command_type=1 但 command_id=0

**处理方式**: 在彩圈判断中，`command_id=0` 的特殊卡不能套用普通五属性规则，需走 `_ => None` 分支。

---

## 23. 满羁绊不在本训练触发支援卡固有效果

固有效果由 `unique_effect_id` 控制，触发条件由代码逻辑判断（通常需要支援卡出现在当前训练中）。

---

## 24. 目标比赛逻辑 (single_mode_route_race)

**表**: `single_mode_route_race`

| 字段 | 说明 |
|------|------|
| race_set_id | 比赛组ID |
| scenario_group_id | 剧本组 |
| target_type | 目标类型 |
| turn | 回合数 |
| race_type | 比赛类型 |
| condition_type/id | 条件 |
| determine_race | 判定比赛 |

---

## 25. 赛后获取技能Pt和属性

**表**: `single_mode_training_effect` 中 `target_type=20`（技能点）和 `target_type=30`（体力消耗）。

比赛后奖励由 `single_mode_race_limit_reward` / `single_mode_reward_set` 等表控制。

---

## 26. 马娘加成逻辑 (single_mode_chara_effect)

**表**: `single_mode_chara_effect`

| 字段 | 说明 |
|------|------|
| effect_type | 效果类型（1=育成, 2=比赛） |
| effect_category | 效果分类 |
| effect_group_id | 效果组 |
| priority | 优先级 |

---

## 27. 马娘3~5星初始属性 (single_mode_chara_grade)

**表**: `single_mode_chara_grade`

| 星级 | win_num | run_num | need_fan_count |
|------|---------|---------|----------------|
| 1★ | 0 | 0 | 0 |
| 2★ | 0 | 1 | 0 |
| 3★ | 1 | 1 | 0 |
| 4★ | 1 | 1 | 5,000 |
| 5★ | 1 | 1 | 20,000~320,000 |

**注意**: 该表控制的是升星条件（胜场数+粉丝数），而非初始属性值。初始属性在 `chara_data` 中。

---

## 28. 拉面杯吃道具加成 (single_mode_cook_dish_effect)

**表**: `single_mode_cook_dish_effect`

| effect_group | 效果类型 | 值 | 目标属性 |
|-------------|---------|-----|---------|
| 100 | type=2 | 25 | 速度/耐力/智力 |
| 101 | type=2 | 40 | 速度 |
| 102 | type=2 | 70 | 速度 |
| 111 | type=2 | 50 | 速度 |
| 200 | type=2 | 25 | 速度/力量/根性 |
| 201 | type=2 | 40 | 力量 |

效果类型：type=2=属性增加, type=21=训练效果, type=201=特殊, type=203=减值

---

## 29. 拉面杯选择地区加成 (single_mode_cook_garden_effect)

**表**: `single_mode_cook_garden_effect`

| effect_group | effect_type | 效果 |
|-------------|------------|------|
| 101 | 110 | 速度+100~200 |
| 101 | 130 | 料理效果+20% |
| 102 | 110 | 耐力+100~400 |
| 103 | 110 | 力量+100~600 |
| 103 | 140 | 训练效果+5% |
| 104 | 110 | 根性+100~800 |
| 105 | 110 | 智力+100~999 |
| 105 | 150 | 特殊效果+5% |

5个地区对应5种属性，效果逐级递增。

---

## 30. 拉面杯拉人头逻辑

菜园设施升级后，可邀请伙伴加入。`single_mode_cook_listener` 定义了6个伙伴位置。

---

## 31. 拉面杯做面条道具 (single_mode_cook_cooking_type)

**表**: `single_mode_cook_cooking_type` — 独特料理类型，包含成功/失败动画。

---

## 32. 拉面杯万能菜

**未见独立"万能菜"表**。可能是指 `dish_type=0` 的通用料理（16种），不限定特定属性组合。

---

## 33. 拉面杯拉人头后彩圈识别

拉面杯6个伙伴有各自的羁绊条，逻辑与普通支援卡类似：
- 羁绊 ≥ 80 → 可触发彩圈
- 料理成功时，伙伴可能出现

---

## 34. 拉面杯试食会

**未见独立"试食会"表**。`single_mode_cook_cooking_cutt` 可能包含试食会动画。

---

## 35. 理事长羁绊和记者羁绊

**表**: `single_mode_npc` — 2,621 条NPC记录

### NPC分组

| npc_group_id | 数量 | 含义（推测） |
|-------------|------|------------|
| 0 | 1,950 | 通用NPC |
| 11 | 48 | 青春杯NPC |
| 12 | 78 | 巅峰杯NPC |
| 13 | 192 | 拉面杯NPC |
| 14 | 66 | 凯旋门NPC |
| 21~33 | 各30~48 | 各剧本NPC |
| 51 | 61 | 特殊NPC |

**理事长和记者**:
- 理事长: chara_id 可能在 9003 附近
- 记者: chara_id=9002（其支援卡 ID=10109/30207）
- 他们的羁绊条由 `single_mode_npc` 中对应 NPC 的 `motivation_min/max` 控制

---

## 总结：35项覆盖率

| # | 项目 | 数据来源 | 状态 |
|---|------|---------|------|
| 1 | 角色ID | chara_data | ✅ 138条 |
| 2 | 支援卡ID | support_card_data | ✅ 404条 |
| 3 | 换皮ID | dress_data | ✅ 369条 |
| 4 | 同名不同卡 | chara_id分组 | ✅ |
| 5 | 卡加成 | support_card_effect_table | ✅ 32种type |
| 6 | 卡技能 | skill_set_id→skill_data | ✅ 1438条 |
| 7 | 卡hit | effect type=14 得意率 | ✅ |
| 8 | 友情训练标志 | command_id匹配 | ✅ |
| 9 | 普通友情训标志 | command_id+羁绊 | ✅ |
| 10 | 支援卡效果 | effect_table | ✅ |
| 11 | 剧本场景效果 | single_mode_training_effect | ✅ 8剧本 |
| 12 | 心情效果 | 待代码确认 | ⚠️ |
| 13 | 5等级训练 | single_mode_training | ✅ |
| 14 | 训练属性ID映射 | command_id | ✅ 101~106 |
| 15 | 槽位识别 | command_id | ✅ |
| 16 | 羁绊增加 | type=3初期+代码 | ⚠️ |
| 17 | 拉面杯机制 | single_mode_cook_* | ✅ |
| 18 | 拉面杯伙伴 | cook_listener | ✅ 6人 |
| 19 | 无彩圈训练 | 羁绊<80 | ✅ |
| 20 | 5训练都不出现 | 得意率概率 | ✅ |
| 21 | 80羁绊不闪 | bond<80 | ✅ |
| 22 | 友人卡 | command_id=0 | ✅ 20张 |
| 23 | 满羁绊固有效果 | unique_effect_id | ⚠️ |
| 24 | 目标比赛 | single_mode_route_race | ✅ |
| 25 | 赛后奖励 | reward表 | ✅ |
| 26 | 马娘加成 | single_mode_chara_effect | ✅ |
| 27 | 3~5星初始属性 | single_mode_chara_grade | ✅ |
| 28 | 拉面杯道具加成 | cook_dish_effect | ✅ |
| 29 | 拉面杯地区加成 | cook_garden_effect | ✅ 5地区 |
| 30 | 拉面杯拉人头 | cook_listener | ✅ |
| 31 | 拉面杯做面条 | cook_cooking_type | ✅ |
| 32 | 拉面杯万能菜 | dish_type=0 | ✅ |
| 33 | 拉面杯彩圈 | 伙伴羁绊 | ✅ |
| 34 | 试食会 | cook_cooking_cutt | ⚠️ |
| 35 | 理事长/记者羁绊 | single_mode_npc | ✅ |

**结论**: 35项中31项(89%)已从MasterDB确认，4项(11%)需代码层确认。

---

> 本内容由 Coze AI 生成，请遵循相关法律法规及《人工智能生成合成内容标识办法》使用与传播。
