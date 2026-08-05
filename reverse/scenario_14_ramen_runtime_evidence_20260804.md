# Scenario 14 拉面：游戏内运行时实锤证据

**状态**：运行时/MDB已确认部分；未确认项单独列出。  
**日期**：2026-08-04  
**来源**：当前游戏进程的 IL2CPP 类/字段/方法、运行时协议元数据、当前进程加载的 MDB。

## 1. 吃面接口

当前协议观测到以下接口：

```text
POST /umamusume/single_mode_ramen/tasting
```

客户端运行时方法：

```text
Gallop.SingleModeRamenAPI.SendServingPractice
```

参数数：2，返回：`void`。

对应客户端场景方法：

```text
Gallop.WorkSingleModeScenarioRamen.ApplyRamenTasting
```

参数数：3，返回：`void`。

`WorkSingleModeScenarioRamen` 还提供：

```text
ApplyRamenDataSetCommon
ApplyRamenDataSetStart
ApplyRamenDataSetLoad
ApplyRamenDataSetCheckEvent
ApplyIsGaugeGained
ApplySelectRegion
ApplyCheckPoint
```

## 2. 吃面事务对象

运行时类：

```text
Gallop.SingleMode.ScenarioRamen.ServingPracticeTransactionEntity
```

字段：

```text
_singleModeId
ConsumedFeelingList
ConsumedSpecialFeelingNum
ActiveRegion
ActiveEffectList
IsTipsEffectActive
```

方法：

```text
get_ConsumedFeelingList
get_ConsumedSpecialFeelingNum
get_ActiveRegion
get_ActiveEffectList
get_HasActiveEffect
get_HasActiveRegion
get_IsTipsEffectActive
IsBonusEffectTraining
```

运行时类：

```text
Gallop.SingleMode.ScenarioRamen.ServingPracticeTransactionRepository
```

已确认的方法：

```text
Get
GetToUseLog
GetConsumedFeelingList
GetActiveEffectList
IsTipsEffect
IsUseLogEffectType
GetActiveComboEffectMasterDataList
GetActiveRegionEffectMasterDataList
GetActiveUrafEffectMasterDataList
CreateComboEffectList
CreateRegionEffectList
CreateUrafEffectList
```

## 3. 拉面效果对象

运行时类：

```text
Gallop.SingleMode.ScenarioRamen.ServingPracticeEffectVO
```

字段：

```text
_effectCategory
_needEquipSupportTypeCount
EffectTextId
ObtainParameterTypeList
EffectValue
```

方法：

```text
get_EffectTextId
get_ObtainParameterTypeList
get_IsBasicEffect
get_IsRegionEffect
get_IsUrafEffect
get_IsUrafCommonEffect
get_IsUrafUniqueEffect
get_EffectValue
CreateBasicEffect
CreateRegionEffect
CreateUrafEffect
Invalid
InvalidByEquipSupportTypeCount
```

这证明一次吃面交易的效果对象可以区分：

- Basic effect；
- Region effect；
- Uraf effect；
- 参数类型列表；
- 效果数值。

## 4. 吃面与训练是不同请求

当前协议元数据中同时观测到：

```text
/umamusume/single_mode_ramen/tasting
/umamusume/single_mode_ramen/exec_command
```

两者是不同 HTTP 请求。当前记录中 `/tasting` 之后才出现 `/exec_command`。

因此已确认：

```text
吃面事务 != 训练事务
```

不能把训练响应字段自动归因到吃面响应，也不能把吃面响应自动归因到支援卡故事事件。

## 5. 拉面运行时状态

运行时类：

```text
Gallop.WorkSingleModeScenarioRamenDataSet
```

关键属性：

```text
CommandInfoArray
EvaluationInfoArray
FeelingReduceTurnInfoArray
FeelingTurnInfoArray
FeelingInfoArray
SpecialFeelingNum
ActiveEffectArray
UrafEffectInfo
CommandFeelingInfoArray
TrainingExecInfoArray
AutoSelectInfo
AutoSelectSetInfo
RecommendType
SelectedRegionIdArray
ReduceBaseTurnInfoArray
CheckPointInfoArray
LastTastingInfo
CheckPointPt
ExpectedCheckPointPt
UsedTwinkleTextIdArray
AllSelectedRegionIdArray
IsUrafEffectSelectEventChecked
IsNotGainSpecialFeeling
IsGaugeGained
```

`LastTastingInfo` 字段：

```text
FeelingId1Num
FeelingId2Num
FeelingId3Num
SpecialFeelingNum
RegionId
```

`ActiveEffectInfo` 字段：

```text
EffectCategory
EffectId
EffectValue
```

`CommandInfo` 字段：

```text
CommandType
CommandId
ParamsIncDecInfoArray
```

`TrainingExecInfo` 字段：

```text
BaseCommandId
ExecCount
```

## 6. 当前 MDB 中已实锤的拉面表

当前进程 MDB 中存在：

```text
single_mode_14_basic_effect              15 rows
single_mode_14_check_point               3 rows
single_mode_14_check_point_effect        21 rows
single_mode_14_check_point_pt             3 rows
single_mode_14_check_point_pt_effect     33 rows
single_mode_14_region_effect              98 rows
single_mode_14_region_effect_bonus       180 rows
single_mode_14_region_feeling             60 rows
single_mode_14_region_select                3 rows
single_mode_14_special_gain_turn           11 rows
single_mode_14_twinkle_ramen              347 rows
single_mode_14_finals_effect               19 rows
single_mode_14_finals_gain_skill            3 rows
```

检查点表的当前值：

```text
check_point_type=1: turn=24, success_pt=1500, great_success_pt=0
check_point_type=2: turn=48, success_pt=3000, great_success_pt=0
check_point_type=3: turn=72, success_pt=3500, great_success_pt=5000
```

最终技能表的当前值：

```text
select_type=1 -> skill_id=200761, normal=1, success=1, great_success=2
select_type=2 -> skill_id=201652, normal=1, success=1, great_success=2
select_type=3 -> skill_id=203851, normal=1, success=1, great_success=2
```

这三条是最终结算技能配置；本文件不把它们解释为普通吃面即时技能。

## 7. 地区效果与Tips候选

当前运行牌组所在地区示例中，MDB存在以下记录：

```text
region_id=1: effect_type=2, effect_value=20, condition_value_1=101
a             effect_type=19, effect_value=1, condition_value_1=101
region_id=2: effect_type=2, effect_value=20, condition_value_1=105
a             effect_type=19, effect_value=1, condition_value_1=105
region_id=5: effect_type=2, effect_value=20, condition_value_1=106
a             effect_type=19, effect_value=1, condition_value_1=106
```

其中 `101/105/106` 是训练命令ID。运行时同时存在：

```text
ServingPracticeTransactionEntity.get_IsTipsEffectActive
ServingPracticeTransactionEntity.IsBonusEffectTraining
```

因此 `effect_type=19` 是Tips相关机制的高可信候选，但**本文件不把19的枚举名称写成已确认事实**；仅凭当前可读结构还没有直接取得该枚举的同构建方法体语义。

## 8. 技能本体与技能Hint的独立容器

运行时类：

```text
Gallop.WorkSingleModeCharaData
```

字段：

```text
_acquiredSkillList
_skillTipsList
```

方法：

```text
get_AcquiredSkillList
get_SkillTipsList
GetSkillTips
GetTipsSkillPointDiscount
GetTipsSkillLevel
```

运行时类：

```text
Gallop.WorkSingleModeChangeParameterInfo
```

字段：

```text
_addSkillList
_addSkillLevelList
_addSkillMatchBonusList
_addSkillTipsList
_currentSkillTipsLevelDict
```

方法：

```text
SetSkillChange
SetSkillTipsChange
GetSkillIdBySkillTips
```

这部分可以确认：

```text
直接新增技能 != 新增技能Hint/折扣
```

## 9. 支援卡事件来源判定

事件快照中出现：

```text
story_id=830127001
```

当前 MDB：

```text
single_mode_story_data.story_id=830127001
support_card_id=30127
event_category=4
```

所以：

```text
830127001 -> 支援卡30127事件
```

这不是拉面事件。30127的三段事件链为：

```text
830127001 -> 830127002 -> 830127003
```

## 10. 明确未确认，禁止写成结论

以下内容当前仍未被游戏内响应字段或同构建方法体直接证明：

1. `effect_type=19` 的正式枚举名称；
2. `/tasting` 响应是否直接包含 `add_skill_tips`；
3. `/exec_command` 响应是否才包含 `add_skill_tips`；
4. `IsTipsEffectActive` 对一次具体效果记录的布尔结果；
5. `hint_gain_type=1` 的正式业务枚举；
6. “吃面直接获得技能”是否指 `add_skill`，还是指 `add_skill_tips`；
7. `single_mode_14_finals_gain_skill` 是否与用户所说的普通吃面技能完全无关——目前只能确认它是最终技能配置。

## 结论

本文件只收录当前游戏进程/MDB已经实锤的结构、接口、字段和来源关联。对于尚未解析的 MessagePack 响应字段，保留 UNKNOWN，不把调用链推断冒充结算事实。
