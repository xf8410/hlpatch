# 心情/逛街/伙伴/彩圈 — IL2CPP 元数据分析

**来源**: global-metadata.dat (v31) + libil2cpp.so + UmaAi 源码交叉验证
**分析日期**: 2026-07-12

---

## 一、心情倍率 (Motivation)

### 1.1 相关类和方法

metadata 中的心情相关符号：

```
AdjustByMotivation     — 按心情调整属性值
GetMotivationCoef      — 获取心情系数
motivationCoef         — 心情系数字段
Motivation             — 心情值 (1-5)
MotivationCoef         — 心情系数属性
STATUS_THRESHOLD       — 状态阈值常量
```

### 1.2 心情等级

游戏内心情 1-5 对应：

| 值 | 日文 | 中文 | 颜色 |
|----|------|------|------|
| 1 | 絶不調 | 极差 | 红 |
| 2 | 不調 | 差 | 橙 |
| 3 | 普通 | 普通 | 黄 |
| 4 | 好調 | 好 | 浅蓝 |
| 5 | 絶好調 | 极好 | 深蓝 |

### 1.3 倍率值

**未在 metadata 中找到浮点常量** (0.6/0.8/0.9/1.0/1.1/1.2)。

心情倍率的具体数值硬编码在 libil2cpp.so 的机器码中，不在 metadata 字符串表里。

社区公认倍率（未从代码确认）：
- 倍不调 (1) = 0.6
- 不调 (2) = 0.8  
- 普通 (3) = 1.0
- 好调 (4) = 1.1
- 绝好调 (5) = 1.2

**确认方法**: 反汇编 `AdjustByMotivation` 或 `GetMotivationCoef` 函数，查找 `fmov`/`fmov.d` 指令加载的浮点常量。

### 1.4 应用位置

`AdjustByMotivation` 出现在马匹参数类中（RawSpeed/RawStamina/...），用于**比赛**属性计算。

训练增益的心情倍率在 `calculateTrainingValueSingle` (UmaAi 对应函数) 中应用。libil2cpp.so 中对应函数需要通过 method dump 地址定位后反汇编。

---

## 二、逛街 (Card Distribution)

### 2.1 得意率 (GoodTrainingRateUp = type 19)

```
type 19 = GoodTrainingRateUp
```

得意率不在 `CardTrainingEffect::apply()` 中处理（UmaAi 代码注释 `// 得意率-不处理`）。

得意率用于构建 `std::discrete_distribution<>`——决定每张卡出现在5个训练中哪一个的概率分布。

### 2.2 逛街机制 (UmaAi 实现)

```cpp
// 每张卡有一个 distribution，根据得意率生成 0~5 的整数
// 0-4 = 速耐力根智，5 = 不出现
std::vector<int> probs = { 100, 100, 100, 100, 100, 200 }; // 默认概率
// 得意率高的卡，对应训练的概率增大
```

1. 游戏每回合为每张卡调用 `distribution(rand)` 决定它出现在哪个训练
2. 友人卡/理事长/记者也有独立的 distribution
3. 每个训练最多5个人头，超过随机选取
4. NPC 也有 distribution，可能出现在任何训练

### 2.3 逛街卡的效果

逛街卡（出现在非得意训练）**依然提供以下效果**：

| type | 效果 | 逛街生效 |
|------|------|:-------:|
| 2 | 干劲加成 (MotivationUp) | ✅ |
| 3-7 | 副属性加成 (TrainingSpeedUp~WizUp) | ✅ |
| 8 | 训练加成 (TrainingEffectUp) | ✅ |
| 27 | 失败率下降 (TrainingFailureRateDown) | ✅ |
| 28 | 体力消耗下降 (TrainingHPConsumptionDown) | ✅ |
| 1 | **友情加成 (SpecialTagEffectUp)** | ❌ 只彩圈 |

**结论**: 逛街卡不是白来的，它贡献干劲/副属性/训练加成/失败率/体力消耗下降，只有友情倍率不触发。

### 2.4 当前 hlpatch 处理

`HomeInfoData.ParamsIncDecInfoArray` 是游戏引擎算好的**最终训练收益值**，已包含所有出现卡的效果（逛街+彩圈）。当前代码不需要额外计算逛街加成——gains 里已经有了。

---

## 三、剧本 NPC 和伙伴

### 3.1 伙伴类型

metadata 中的 TrainingPartner 体系：

```
ISingleModeTrainingPartnerEntity
  ├── get_PartnerId           — 伙伴 ID (1-6=支援卡, 6=理事长, 7=记者, 8=NPC)
  ├── get_EvaluationValue      — 羁绊值
  ├── get_TrainingBaseCommandId — 训练命令 ID
  ├── get_IsNoneTrainingCommand — 是否无训练命令
  ├── GetMasterSupportCardData  — 获取支援卡 MasterDB 数据
  ├── get_Exp                   — 经验值
  └── get_LimitBreakCount       — 突破数

子类:
  ├── SingleModeTrainingPartnerUniqueCharaEntity  — 剧本独有角色
  ├── SingleModeTrainingPartnerScoutEntity        — 剧本 scout 角色
  └── SingleModeTrainingPartnerEtcCharaEntity     — 其他角色
```

### 3.2 剧本专用伙伴

每个剧本有专属的 TrainingPartner 类：

| 剧本 | 伙伴类 |
|------|--------|
| 先锋 (Pioneer/11) | SingleModeScenarioPioneerShimaTrainingPartnerRepository |
| 温泉 (Onsen/12) | SingleModeScenarioOnsenTrainingPartnerIsTagTrainingChecker |
| 团队赛 (TeamRace/2) | SingleModeScenarioTeamRaceTrainingPartnerEntity |
| TeamRace 伙伴 | 含 InterestState / SoulEventState / SoulThresholdId |

TeamRace 伙伴独有字段：
- `interestState` — 兴趣状态
- `soulEventState` — 灵魂事件状态
- `soulThresholdId` — 灵魂阈值 ID
- `trainingBaseCommandId` — 训练基础命令 ID

### 3.3 NPC 伙伴

```
PartnerId = 8 (PSID_npc) — NPC 伙伴
PartnerId = 6 (PSID_noncardYayoi) — 非卡理事长
PartnerId = 7 (PSID_noncardReporter) — 非卡记者
```

- NPC 每回合通过 `distribution_npc(rand)` 随机分配到训练
- 记者第13回合才出现（`turn < 12` 时不出现）
- 夏合宿期间记者不出现
- NPC 不提供支援卡效果，但占用训练人头位

### 3.4 剧本独有角色

某些剧本有特殊 NPC/角色伙伴：
- `SingleModeTrainingPartnerUniqueCharaEntity` — 剧本独有角色（如拉面杯的伙伴）
- `SingleModeTrainingPartnerScoutEntity` — scout 角色
- 这些伙伴有自己的 `_masterSingleModeUniqueChara` 或 `_masterSingleModeScoutChara` 引用

---

## 四、彩圈 (Tag Training)

### 4.1 彩圈判定体系

metadata 中的彩圈检查体系非常完整：

```
SingleModeIsTagTrainingCheckService (入口)
  ├── IsTagTrainingPartner(partnerEntity)     — 该 partner 是否彩圈
  ├── IsTagTrainingCommand(commandId)         — 该训练是否有彩圈
  ├── ExistsTagTrainingPartner(partnerList)   — 是否存在彩圈伙伴
  └── CreateTagTrainingChecker()              — 创建剧本专用检查器

ITrainingPartnerIsTagTrainingChecker (接口)
  └── IsTagTraining()                         — 核心判定方法

实现类:
  ├── SingleModeTrainingPartnerIsTagTrainingChecker (通用)
  │   ├── IsTagTrainingSupportCardTypeChara()    — 普通卡彩圈
  │   ├── IsTagTrainingSupportCardTypeFriend()   — 友人卡彩圈
  │   ├── IsTagTrainingSupportCardTypeGroup()    — 团体卡彩圈
  │   ├── IsEnoughTagTrainingEvaluation()        — 羁绊是否足够
  │   ├── IsTagTrainingCommandPosition()         — 训练位置是否匹配
  │   └── HasGroupSupportCardCharaEffectId()     — 团体卡效果ID检查
  │
  ├── SingleModeScenarioOnsenTagTrainingChecker (温泉剧本)
  │   └── IsTagTrainingScenarioEffect()
  │
  ├── SingleModeScenarioPioneerTagTrainingChecker (先锋剧本)
  │   ├── IsTagTrainingSupportCardTypeGroup()
  │   └── ExistsShimaTrainingCommand()
  │
  └── SingleModeTrainingPartnerIsTagTrainingCheckerLegacy (旧版)
      ├── IsTagTrainingByVenusSpirit()    — 维纳斯
      ├── IsTagTrainingBySportEffectedStance() — 运动
      ├── IsTagTrainingByMecha()          — 机甲
      └── IsTagTrainingByLegend()         — 传奇
```

### 4.2 彩圈判定的完整逻辑

从方法名推断的判定流程：

```
IsTagTraining(partnerEntity, commandId):
  1. IsTagTrainingSupportCardTypeChara()  — 普通卡?
     → IsEnoughTagTrainingEvaluation()    — bond >= 阈值?
     → IsTagTrainingCommandPosition()     — 训练匹配?
  
  2. IsTagTrainingSupportCardTypeFriend() — 友人卡?
     → (不彩圈，返回 false)
  
  3. IsTagTrainingSupportCardTypeGroup()  — 团体卡?
     → HasGroupSupportCardCharaEffectId() — 触发了特殊效果?
  
  4. IsTagTrainingScenarioEffect()        — 剧本特殊效果?
     → 各剧本独立检查
  ```

### 4.3 TipsEvent (启示事件)

```
SingleModeTrainingPartnerHasTipsCheckerService
  ├── HasTipsEvent(trainingPartner)      — 该 partner 是否有启示事件
  └── ExistsTipsEvent(trainingPartnerList) — 列表中是否有启示事件

SingleModeTrainingPartnerTipsRepository
  ├── get_WorkSingleModeHomeInfoData      — 从 HomeInfoData 获取
  └── get_SingleModeCommandInfoDataArray   — 从 CommandInfoData 获取

ISingleModeTrainingPartnerTipsEntity
  ├── get_PartnerId
  └── get_CommandId
```

`TipsEventPartnerArray` 在 `SingleModeCommandInfoData` 偏移 0x58 (88)：
- 存储当前训练中触发了启示事件的 partner_id 列表
- 团体卡在此列表中 = 触发了特殊启示 = 彩圈
- 这与 v3.24.15 的实现一致

### 4.4 彩圈时的额外效果

彩圈触发时，以下效果才生效：
- **type 1 (SpecialTagEffectUp/友情加成)**: `totalYouqingMultiplier *= (1 + 0.01 * youQing)`
- **智力彩圈体力减少**: `vitalCostBasic -= vitalBonus`
- **剧本特殊加成**: 如机甲剧本的 `friendshipBonus = 2 * mecha_upgrade[1][2]`

---

## 五、待反汇编确认

以下问题需要反汇编 libil2cpp.so 才能确认：

1. **心情倍率具体数值** — `AdjustByMotivation` / `GetMotivationCoef` 函数中的浮点常量
2. **训练增益完整公式** — `calculateTrainingValueSingle` 对应函数
3. **得意率→分布概率的转换公式** — 得意率数值如何映射到 discrete_distribution 概率
4. **团体卡 HasGroupSupportCardCharaEffectId 的具体判定** — 检查什么字段
5. **NPC 出现概率** — `distribution_npc` 的概率分布参数

*分析: 2026-07-12, 基于 global-metadata.dat v31 + UmaAi 源码*
