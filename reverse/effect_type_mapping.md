# 支援卡效果类型完整映射

**来源**: global-metadata.dat → SupportCardEffectType 枚举 + UmaAi CardEffect.cpp 交叉验证
**验证日期**: 2026-07-12
**游戏版本**: v2.28.5

## support_card_effect_table.type → 效果名映射

| type | 枚举名 | 中文 | UmaAi变量 | 计算方式 |
|:----:|--------|------|----------|---------|
| 1 | SpecialTagEffectUp | 友情加成 | youQing | 乘算 `(100+youQing)*(100+value)/100-100` |
| 2 | MotivationUp | 干劲加成 | ganJing | 加算 `+=value` |
| 3 | TrainingSpeedUp | 速度训练加成 | bonus[0] | 加算（副属性Speed） |
| 4 | TrainingStaminaUp | 耐力训练加成 | bonus[1] | 加算（副属性Stamina） |
| 5 | TrainingPowerUp | 力量训练加成 | bonus[2] | 加算（副属性Power） |
| 6 | TrainingGutzUp | 根性训练加成 | bonus[3] | 加算（副属性Guts） |
| 7 | TrainingWizUp | 智力训练加成 | bonus[4] | 加算（副属性Wiz） |
| 8 | TrainingEffectUp | 训练加成(全) | xunLian | 加算 `+=value` |
| 9 | InitialSpeedUp | 初始Speed | initialBonus[0] | — |
| 10 | InitialStaminaUp | 初始Stamina | initialBonus[1] | — |
| 11 | InitialPowerUp | 初始Power | initialBonus[2] | — |
| 12 | InitialGutzUp | 初始Guts | initialBonus[3] | — |
| 13 | InitialWizUp | 初始Wiz | initialBonus[4] | — |
| 14 | InitialEvaluationUp | 初始羁绊 | initialJiBan | — |
| 15 | RaceStatusUp | 赛后Status | saiHou | — |
| 16 | RaceFanUp | 赛后Fan | — | — |
| 17 | SkillTipsLvUp | Hint等级 | hintLevel | — |
| 18 | SkillTipsEventRateUp | Hint发生率 | hintProbIncrease | — |
| 19 | GoodTrainingRateUp | 得意率 | deYiLv | 不处理（分布用） |
| 20 | SpeedLimitUp | Speed上限 | — | — |
| 21 | StaminaLimitUp | Stamina上限 | — | — |
| 22 | PowerLimitUp | Power上限 | — | — |
| 23 | GutzLimitUp | Guts上限 | — | — |
| 24 | WizLimitUp | Wiz上限 | — | — |
| 25 | EventRecoveryAmountUp | 事件体力加成 | eventRecoveryAmountUp | — |
| 26 | EventEffectUp | 事件属性加成 | eventEffectUp | — |
| 27 | TrainingFailureRateDown | 失败率下降 | failRateDrop | 减乘 `100-(100-x)*(100-value)/100` |
| 28 | TrainingHPConsumptionDown | 体力消耗下降 | vitalCostDrop | 减乘 `100-(100-x)*(100-value)/100` |
| 29 | MinigameEffectUp | 小游戏效果 | — | — |
| 30 | (未确认) | 智力回体? | vitalBonus? | — |
| 31 | (未确认) | 智力回体UP | vitalBonus | 加算 `+=value` |
| 32 | (未确认) | ? | — | — |

## support_card_unique_effect.type_0 映射

| type_0 | 含义 | 备注 |
|:------:|------|------|
| 1-8 | 同上表效果 | 自定义固有效果 |
| 10-11 | 初始属性 | 自定义固有效果 |
| 27 | 失败率下降 | 自定义固有效果 |
| 30 | 特殊标记 | 可能是激活条件 |
| 101 | **羁绊阈值标记** | value_0 = bond 阈值 (60/80/100) |
| 102-122 | 自定义固有效果 | type_1 字段为实际效果值 |

## 关键结论

### 彩圈（友情训练）机制
- **type 1 (SpecialTagEffectUp)** = 友情加成，**只在彩圈时乘算**
- 逛街卡不触发 type 1，但 type 2-8（干劲/副属性/训练加成）照常生效
- 这与 UmaAi 的 `if (isCardShining) { totalYouqingMultiplier *= (1+0.01*eff.youQing); }` 一致

### 固有效果机制
- **type_0=101** 是 bond 阈值标记，不是效果类型
- bond ≥ threshold 后，type_1 字段的效果生效
- type_1 的值同上表映射（1=友情, 2=干劲, 8=训练, etc.）

### 得意率 (type 19)
- 不在 `apply()` 中处理，而是用于 `Person.distribution` 概率分布
- 得意率越高 → 卡出现在对应训练的概率越高
- 逛街 = 得意率分布把卡分到了非得意训练

## 逛街卡的效果（不需要彩圈）
- type 2: 干劲加成 → 当前训练全属性 ×(1+0.01*ganJing)
- type 3-7: 副属性加成 → 对应属性 +=value（不管当前训练类型）
- type 8: 训练加成 → 当前训练全属性 +=value
- type 27: 失败率下降
- type 28: 体力消耗下降

## 彩圈额外效果（需要 bond≥阈值 + 训练匹配）
- type 1: 友情加成 → 当前训练全属性 ×(1+0.01*youQing)
- 智力彩圈: 体力消耗额外减少 vitalBonus

*由 global-metadata.dat SupportCardEffectType 枚举 + UmaAi CardEffect.cpp 交叉验证*
