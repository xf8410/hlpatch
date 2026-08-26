# 继承因子（种马因子）加成数值 — 二进制级证据

- 采集时间：2026-08-26，游戏运行中实时采集（hlpatch SO 3.27.15）
- 数据源：`/data/data/jp.co.cygames.umamusume/files/master/master.mdb`（本地 SQLite，经 `/mdb/raw?sql=` 直查）+ IL2CPP 元数据内省
- 会话：1787734430298-16890；场景：Ramen(scenario_id=14)
- 结论性质：设备上 master.mdb 实测行 + IL2CPP 类字段/方法签名；无网页来源
- 重要修正：原先将 `63` 解释为 `9×7` 的继承触发收益是错误的；`63` 属于育成开局的白参数因子初始加成。

## 1. IL2CPP 代码结构证据

### MasterSuccessionFactorEffect（master 表封装）
- `TABLE_NAME` 静态字段实际值 = **"succession_factor_effect"**
- 懒加载字典：`_lazyPrimaryKeyDictionary`、`_dictionaryWithFactorGroupId`、`_dictionaryWithFactorGroupIdAndEffectId`、`_dictionaryWithTargetType`
- 查询 API：`Get / GetWithFactorGroupId / GetListWithFactorGroupIdAndEffectId / GetListWithTargetType`
- 注意：拉面剧本下该单例所有字典为 null —— 表按需懒加载，育成外不进内存

### SuccessionFactorEffect（行结构 ORM，offset 为实例内偏移）
| 字段 | offset | 类型 |
|---|---|---|
| Id | 0x10 | i4 |
| FactorGroupId | 0x14 | i4 |
| EffectId | 0x18 | i4 |
| TargetType | 0x1C | i4 |
| Value1 | 0x20 | i4 |
| Value2 | 0x24 | i4 |

### SuccessionInitialFactor（开局初始加成表 ORM）
| 字段 | offset | 类型 |
|---|---|---|
| Id | 0x10 | i4 |
| FactorType | 0x14 | i4 |
| Value1 | 0x18 | i4 |
| Value2 | 0x1C | i4 |
| AddPoint | 0x20 | i4 |

### SuccessionBonusParams（运行时套用器）
- `_valueDictionary : Dictionary<MasterSuccessionFactorEffect.SuccessionFactorEffect.FactorTargetType, int>`
- 方法：`ApplyFactor(ref MasterSuccessionFactor.SuccessionFactor, ref MasterSkillData.SkillData, int factorLv)`、`GetBonusValueByType(FactorTargetType) => int`
- `SuccessionFactor` 还提供 `Name(int lv)` 和 `get_Description()`；因子 ID 到显示文本不能简单假定为 `text_data.id`。
- 静态常量：`FACTOR_TYPE_PARAM=1, FACTOR_TYPE_PROPER=2, FACTOR_TYPE_CHARA=3, FACTOR_TYPE_SKILL=4, FACTOR_TYPE_RACE=5, FACTOR_TYPE_SEANARIO=6, FACTOR_TYPE_MATCH_BONUS=7, FACTOR_TYPE_PARENT_STAR=8, FACTOR_TYPE_SPECIAL_CONDITION=9, FACTOR_TYPE_PARENT_STAR_AD1/AD2, MAIN_FACTOR_MAX`

### 相关枚举/辅助类
- `ParamFactorValue2Type { ParamUp, MaxUp }`（`succession_initial_factor.value_2` 用）
- `SingleMode12FactorBonus { Id, FactorGroupId, StratumType, BonusValue }`（场景通关因子奖励）

## 2. 数据库表与行数

| 表 | 行数 | 列 |
|---|---:|---|
| succession_factor | 2577 | factor_id PK, factor_group_id, rarity(1..3), grade, factor_type, effect_group_id, succession_search_hidden, start_date, end_date |
| succession_factor_effect | **6674** | id PK, factor_group_id, effect_id, target_type, value_1, value_2 |
| succession_factor_addon_effect | 2 | condition_type, condition_value_1..6, factor_id |
| succession_initial_factor | 13 | id PK, factor_type, value_1, value_2, add_point |
| single_mode_12_factor_bonus | 9 | factor_group_id, stratum_type, bonus_value |

`text_data` 表结构为 `id, category, index, text`。它不是可直接用 `factor_id` 等值连接的文本表。

## 3. target_type 映射（DISTINCT 实测 + 语义标注）

| target_type | 行数 | value_1 范围 | 语义 |
|---|---:|---:|---|
| 1 | 88 | 1..36 | 速度 |
| 2 | 73 | 1..36 | 耐力 |
| 3 | 88 | 1..36 | 力量 |
| 4 | 85 | 1..36 | 毅力 |
| 5 | 61 | 1..36 | 智力 |
| 6 | 52 | 1..15 | 技能点 |
| 7 | 120 | 6..18 | 技能提示（value_1=技能点数, value_2=1）|
| 11/12 | 各4 | 1..2 | 适性·场地 芝/泥 |
| 21..24 | 各4 | 1..2 | 适性·距离 短/英/中/长 |
| 31..34 | 各4 | 1..2 | 适性·脚质 逃/先/差/追 |
| 41 | 3414 | 200012..920771 | 比赛因子（race_id 编码）|
| 51 | 7 | 1000011..1000017 | 剧情因子 |
| 61..65 | 562/458/634/510/482 | 1..6 | 亲代★数联动 |

上述为静态表的语义映射；ID 到日文名称/描述需要结合游戏文本资源和实现逻辑验证。

## 4. 属性/技能点因子的星级阶梯

### 蓝参数因子：继承事件触发收益

五维属性（`target_type 1..5`）的标准阶梯按 `effect_id` 档位为：

| effect_id（档） | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| value_1 | 1 | 4 | 7 | 10 | 13 | 16 | 19 | 22 | 25 | 28 |

换算为通常显示的继承触发收益：**1★=+3、2★=+6、3★=+9……10★=+28**。这是蓝参数因子的继承事件收益，和下节的白因子开局加成不是同一机制。

存在少量特殊/活动因子变体；不能仅凭主流阶梯覆盖所有因子。

### 技能点、技能提示、适性与亲代星级

- 技能点因子（`target_type=6`）：主流 1/2/3★ 为 `+3/+6/+9 PT`，存在 `[4,5] / [8,10] / [12,15]` 变体。
- 技能 hint 因子（`target_type=7`）：`(value_1,value_2)=(6,1)/(12,1)/(18,1)`。
- 适性因子（target 11..34）：1★=+1、2★=+2。
- 亲代★联动（target 61..65）：以 target 61 为例，e1=[1]、e2=[1,2]、e3=[1,2,4]、e4=[2,4,6]、e5..e10=[2,3,4]。

## 5. 白参数因子初始加成（开局，不是继承触发）

`succession_initial_factor` 实测 13 行。核心参数段：

| factor_type | value_1 | value_2 | add_point | 含义 |
|---|---:|---:|---:|---|
| 1（参数） | 1 | 0（ParamUp） | **+5** | 白参数因子1★开局属性 |
| 1 | 2 | 0 | **+12** | 白参数因子2★开局属性 |
| 1 | 3 | 0 | **+21** | 白参数因子3★开局属性 |
| 1 | 1 | 1（MaxUp） | +4 | 上限变体 |
| 1 | 2 | 1 | +9 | 上限变体 |
| 1 | 3 | 1 | +16 | 上限变体 |

其余 7 行为适性段和技能 hint 段：适性按 `1/4/7/10` 与 `3/6/9/999` 分段，技能段为 `1/2/3` 与 `1/2/4`，`add_point=0`。

### 63 的正确解释

数据库明确给出：

```text
factor_type=1, value_1=3, value_2=0 => add_point=21
```

因此三个适用的三星白参数因子合计：

```text
21 × 3 = 63
```

**63 是育成开始时的属性加成，不是 `9×7`，也不是继承事件触发次数、相性乘区或场景奖励造成的蓝因子收益。**

## 6. 场景通关因子额外奖励

`single_mode_12_factor_bonus` 共 9 行，分别为：

```text
(1,1,10) (5,1,6) (2,1,3)
(4,2,10) (1,2,6) (3,2,3)
(3,3,10) (5,3,6) (2,3,3)
```

`succession_factor_addon_effect` 仅 2 行，为条件型附加效果。

## 7. 因子 ID 与文本资源映射

逆向/网络数据中的种马因子首先是 ID。静态数值解析应使用：

```text
factor_id
  -> succession_factor
  -> factor_group_id / factor_type / rarity / grade / effect_group_id
  -> succession_factor_effect
  -> target_type / value_1 / value_2
```

显示名称和技能描述还需接入文本资源。`WinSaddleAnalyzer` 的实现读取 `factor_effects.br`，按资源中的 `index -> text` 建立效果文本映射；因此不能把 `factor_id` 直接当作 `text_data.id`。`deserializedb5` 的完整实现可作为 ID→文本资源映射的参考实现。

## 8. 内存地址说明

此前记录的 libil2cpp 进程地址是 ASLR 下的会话瞬时值；持久化时应使用文件 offset，而不是运行时虚拟地址。当前静态结论不依赖这些地址。

## 9. SO 端点状态

当前 SO 3.27.15 已能通过 `/mdb/raw?sql=<URL编码SQL>` 查看上述静态表，并能读取 IL2CPP 类、字段和方法签名；本次修改不需要调用游戏工具，也不需要更新 SO。专用端点的参数别名问题（`name`/`table`、query 偶发丢失）属于便利性问题，可后续处理。

## 复现 SQL

```sql
SELECT * FROM succession_initial_factor;
SELECT * FROM succession_factor_effect WHERE target_type=1 ORDER BY value_1;
SELECT sql FROM sqlite_master WHERE name IN ('succession_factor','succession_factor_effect','succession_initial_factor','text_data');
```
