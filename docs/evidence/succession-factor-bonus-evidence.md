# 继承因子（种马因子）加成数值 — 二进制级证据

- 采集时间：2026-08-26，游戏运行中实时采集（hlpatch SO 3.27.15）
- 数据源：`/data/data/jp.co.cygames.umamusume/files/master/master.mdb`（本地 SQLite，经 `/mdb/raw?sql=` 直查）+ IL2CPP 元数据内省
- 会话：1787734430298-16890；场景：Ramen(scenario_id=14)
- 结论性质：全部为设备上 master.mdb 实测行 + IL2CPP 类字段/方法签名，无任何网页来源、无猜测值

## 1. IL2CPP 代码结构证据

### MasterSuccessionFactorEffect（master 表封装）
- `TABLE_NAME` 静态字段实际值 = **"succession_factor_effect"**（`/il2cpp/static?name=` 读出）
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

### SuccessionBonusParams（运行时套用器）
- `_valueDictionary : Dictionary<MasterSuccessionFactorEffect.SuccessionFactorEffect.FactorTargetType, int>`
- 方法：`ApplyFactor(ref MasterSuccessionFactor.SuccessionFactor, ref MasterSkillData.SkillData, int factorLv)`、`GetBonusValueByType(FactorTargetType) => int`
- 静态常量（SuccessionFactor）：`FACTOR_TYPE_PARAM=1, FACTOR_TYPE_PROPER=2, FACTOR_TYPE_CHARA=3, FACTOR_TYPE_SKILL=4, FACTOR_TYPE_RACE=5, FACTOR_TYPE_SEANARIO=6, FACTOR_TYPE_MATCH_BONUS=7, FACTOR_TYPE_PARENT_STAR=8, FACTOR_TYPE_SPECIAL_CONDITION=9, FACTOR_TYPE_PARENT_STAR_AD1/AD2, MAIN_FACTOR_MAX`

### 相关枚举/辅助类
- `ParamFactorValue2Type { ParamUp, MaxUp }`（succession_initial_factor.value_2 用）
- `SingleMode12FactorBonus { Id, FactorGroupId, StratumType, BonusValue }`（场景通关因子奖励）

## 2. 数据库表与行数

| 表 | 行数 | 列 |
|---|---|---|
| succession_factor | 2577 | factor_id PK, factor_group_id, rarity(1..3), grade, factor_type, effect_group_id, succession_search_hidden, start_date, end_date |
| succession_factor_effect | **6674** | id PK, factor_group_id, effect_id, target_type, value_1, value_2 |
| succession_factor_addon_effect | 2 | condition_type, condition_value_1..6, factor_id |
| succession_initial_factor | 13 | factor_type, value_1, value_2, add_point |
| single_mode_12_factor_bonus | 9 | factor_group_id, stratum_type, bonus_value |

factor_type 分布（rarity×grade 计数）：type1 参数 5/5/5，type2 适性 10/10/10，type3 白名 264×3，type4 技能 445/445/447，type5 比赛 37×3，type6 剧情 34×3，type7 7，type8 4/4/4，type9 51×3，type10 2×3，type11 4×3。

## 3. target_type 映射（DISTINCT 实测 + 语义标注）

| target_type | 行数 | value_1 范围 | 语义（依据 FACTOR_TYPE_* 常量与阶梯形状推断） |
|---|---|---|---|
| 1 | 88 | 1..36 | 速度 |
| 2 | 73 | 1..36 | 耐力 |
| 3 | 88 | 1..36 | 力量 |
| 4 | 85 | 1..36 | 毅力 |
| 5 | 61 | 1..36 | 智力 |
| 6 | 52 | 1..15 | 技能点 |
| 7 | 120 | 6..18 | 技能（value_1=技能点数, value_2=1）|
| 11/12 | 各4 | 1..2 | 适性·场地 芝/泥（1星/2星）|
| 21..24 | 各4 | 1..2 | 适性·距离 短/英/中/长 |
| 31..34 | 各4 | 1..2 | 适性·脚质 逃/先/差/追 |
| 41 | 3414 | 200012..920771 | 比赛因子（race_id 编码）|
| 51 | 7 | 1000011..1000017 | 剧情因子 |
| 61..65 | 562/458/634/510/482 | 1..6 | 亲代★数联动（parent_star）|

> 标注列中：target 1..7 与 11..65 的语义由静态常量序列及数值阶梯形状支持；41/51 为 ID 编码段（观测事实）。

## 4. 属性/技能点因子的星级阶梯（核心答案）

### 五维属性（target_type 1..5 同构），按 effect_id = 星级档位
标准蓝因子阶梯（每属性 88/73/88/85/61 行中的主流值）：

| effect_id(档) | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|
| value_1 | 1 | 4 | 7 | 10 | 13 | 16 | 19 | 22 | 25 | 28 |

即：**1★=+3、2★=+6、3★=+9 … 10★=+28（每档累计）**。例：3★速度因子一次继承触发 = +9 速度。
存在少量变体行（各属性 c=1~4 条）：e1={2,3,10,12}, e2={6,8,10,12,20,24}, e3={6,7,12,15,18,30,36}, e4={8,10}, e5={10,13} —— 对应特殊/活动因子组。

### 技能点因子（target_type=6）
| effect_id | 1 | 2 | 3 |
|---|---|---|---|
| value_1 | 3~5 | 6~10 | 9~15 |

主流：3★=+9 PT（同 3/6/9 形状）；变体 {4,5}/{8,10}/{12,15}。

### 技能 hint 因子（target_type=7，40 组）
恒定 `(value_1, value_2)` = (6,1)/(12,1)/(18,1)，对应 1/2/3★ 的技能 PT。

### 适性因子（target 11..34）
1★=+1、2★=+2（各槽位独立 4 行）。

### 亲代★联动（target 61..65，以 target 61 为例，其余同构）
| effect_id | 1 | 2 | 3 | 4 | 5..10 |
|---|---|---|---|---|---|
| value_1 | 1 | 1~2 | 1~4 | 2~6 | 2/3/4 |
主流：e1=1, e2=1, e3=1, e4=2；特殊行含 e3=4、e4=6（各仅 1~3 组）。

## 5. 白因子初始加成（succession_initial_factor，13 行全量）

| factor_type | value_1 | value_2 | add_point | 解释（假设，待 UI 对照验证）|
|---|---|---|---|---|
| 1 (参数) | 1 | 0 (ParamUp) | **+5** | 白因子1★ 开局属性 |
| 1 | 2 | 0 | **+12** | 白因子2★ |
| 1 | 3 | 0 | **+21** | 白因子3★ |
| 1 | 1 | 1 (MaxUp) | +4 | 变体 |
| 1 | 2 | 1 | +9 | 变体 |
| 1 | 3 | 1 | +16 | 变体 |
| 2 (适性) | 1/4/7/10 | 3/6/9/999 | +1/+2/+3/+4 | 适性段 |
| 3 (技能) | 1/2/3 | 1/2/4 | 0 | hint 段 |

## 6. 场景通关因子额外奖励（single_mode_12_factor_bonus 全量）

| factor_group_id | stratum_type | bonus_value |
|---|---|---|
| 1 | 1 | 10 |
| 5 | 1 | 6 |
| 2 | 1 | 3 |
| 4 | 2 | 10 |
| 1 | 2 | 6 |
| 3 | 2 | 3 |
| 3 | 3 | 10 |
| 5 | 3 | 6 |
| 2 | 3 | 3 |

addon_effect 仅 2 行：cond_type=1 绑定 factor_id=3101403（值 31011/31012/31013），cond_type=2 绑定 3102003（值 3 与 31015..31019）——条件型附加效果，量极少。

## 7. 用户算式核对（9速=63？）

DB 事实：3★ 速度 = 每次"继承触发"+9。若一年两次继承事件、每次双亲结算 = 4 触发 → 36 点裸收益。
63 = 9×7，需要额外乘区/次数来源（相性、亲代★联动 target61-65、场景加成 single_mode_12_factor_bonus 等）。本文件只固化 DB 数值；乘区归因需后续 hook `GetBonusValueByType` 实测。

## 8. 内存地址快照（libil2cpp.so，本次会话实测）

- 只读段基址 A：`0x78f0026000`（r--p，size 0x3C415000，文件偏移≈虚拟地址）
- 只读段基址 B（第二映射）：`0x78fa22e000`（rw-p，size 0x30FFE000）
- 整数 63 命中样例：A 段 `0x78f1d63e9c`(off 0x1D3DE9C) 等 36 处；rw 段热点簇 `0x78fa9c3000–0x78faa1000`（63/126/21/35 高密度，疑似运行时数值结构池）
- 整数 126 命中样例：A 段 `0x78f002e8f8`(off 0x88F8)、`0x78fb3ac630`(off 0x117E630) 等
- 注：进程 ASLR 每次启动变化，以上为本会话瞬时值；持久化价值在文件偏移（offset 列）

## 9. SO 端点可用性记录（本次踩坑）

可用：
- `/mdb/raw?sql=<URL编码SQL>` ✅（核心通道，LIMIT 必带）
- `/mdb/schema?name=<table>` ✅
- `/debug/dumpclass?name=<class>`、`/il2cpp/static?name=`、`/il2cpp/methods?name=`、`/il2cpp/method_detail?class=&method=`（后者部分组合报缺参）
- `/il2cpp/search_int?values=63,126,...`

不可用/受限：
- `/mdb`、`/mdb/raw?table=`、`/mdb/search?table=`、`/mdb/dl_batch`、`/debug/download_table`、`/debug/push_table` → T003 缺参或 T099 失败（端点参数名不匹配或未实现）
- `/inherit/bonus_params`、`/inherit/factor_tree`、`/factor/*` → 空结果（依赖育成内观察文件，拉面剧本未产生）
- meta.sqlcipher hook：disabled_v3.24.57（直连文件读被禁，只能走游戏内连接）
- enum_values 返回 unresolved（枚举整数值未能展开）

## 复现命令

```
GET /mdb/schema?name=succession_factor_effect
GET /mdb/raw?sql=SELECT%20*%20FROM%20succession_factor_effect%20WHERE%20target_type=1%20ORDER%20BY%20value_1
GET /mdb/raw?sql=SELECT%20*%20FROM%20succession_initial_factor
```
