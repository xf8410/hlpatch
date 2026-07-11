---
AIGC:
    Label: "1"
    ContentProducer: 001191110102MACQD9K64018705
    ProduceID: 2380867232079196_0/project_7654253504034472235-files/逆向分析/il2cpp_full_analysis.md
    ReservedCode1: ""
    ContentPropagator: 001191110102MACQD9K64028705
    PropagateID: 2380867232079196#1783732980461
    ReservedCode2: ""
---
# 赛马娘 il2cpp 全量逆向分析报告

> 从 `global-metadata.dat`（44MB, v31）提取 **11,240 个类名** 和完整SQL表结构
> 结合 `master.mdb`（26MB SQLite3）交叉验证

---

## 一、剧本体系完整映射

### 1.1 剧本编号与 il2cpp 类对应

| 编号 | il2cpp 类 | 中文名 | 数据表前缀 |
|------|----------|--------|-----------|
| 1 | WorkSingleModeScenarioURA | URA决赛 | single_mode_training_effect(1) |
| 2 | WorkSingleModeScenarioArc | 凯旋门 | single_mode_training_effect(2) |
| 3 | WorkSingleModeScenarioLive | Grand Live | single_mode_training_effect(3) |
| 4 | WorkSingleModeScenarioMecha | 机甲杯 | single_mode_training_effect(4) |
| 5 | WorkSingleModeScenarioFree | 自由剧本 | single_mode_training_effect(5) |
| 6 | WorkSingleModeScenarioLegend | 传说杯 | single_mode_training_effect(6) |
| 7 | WorkSingleModeScenarioSport | 运动杯 | single_mode_training_effect(7) |
| 8 | WorkSingleModeScenarioCook | 料理杯 | single_mode_training_effect(8) |
| 9 | WorkSingleModeScenarioVenus | Venus | single_mode_09_* |
| 10 | WorkSingleModeScenarioLive | Live扩展 | single_mode_10_* |
| 11 | WorkSingleModeScenarioPioneer | 先驱杯 | single_mode_11_* |
| 12 | WorkSingleModeScenarioOnsen | 温泉杯 | single_mode_12_* |
| 13 | WorkSingleModeScenarioBreeders | **青春杯/种田杯** | single_mode_13_* |
| 14 | WorkSingleModeScenarioRamen | **拉面杯** | single_mode_14_* |
| — | WorkSingleModeScenarioTeamRace | 团队赛 | (独立) |

### 1.2 剧本14（拉面杯）数据表

```
single_mode_14_basic_effect        — 基础效果
single_mode_14_check_point         — 检查点
single_mode_14_check_point_effect  — 检查点效果
single_mode_14_check_point_pt      — 检查点Pt
single_mode_14_check_point_pt_effect — 检查点Pt效果
single_mode_14_deck_info           — 卡组信息
single_mode_14_feeling_bonus       — 心情加成
single_mode_14_finals_effect       — 决赛效果
single_mode_14_finals_gain_skill   — 决赛获得技能
single_mode_14_outing_effect       — 外出效果
single_mode_14_region_effect       — 地区效果
single_mode_14_region_effect_bonus — 地区效果加成
single_mode_14_region_feeling      — 地区心情
single_mode_14_region_select       — 地区选择
single_mode_14_special_gain_turn   — 特殊获得回合
single_mode_14_twinkle_ramen       — 闪亮拉面
```

### 1.3 剧本13（青春杯/种田杯）数据表

```
single_mode_13_add_dream_point        — 增加梦想点
single_mode_13_bc_program_flag        — BC节目标记
single_mode_13_member                 — 成员
single_mode_13_rank                   — 排名
single_mode_13_rank_bonus_effect_group — 排名奖励效果组
single_mode_13_schedule               — 日程
single_mode_13_team_rank              — 团队排名
single_mode_13_team_sp_effect         — 团队SP效果
single_mode_13_team_sp_level          — 团队SP等级
single_mode_13_top_bg_chara           — 顶部背景角色
```

---

## 二、核心类体系

### 2.1 训练体系

| 类名 | 用途 |
|------|------|
| SingleModeTrainingCommandEntity | 训练指令实体 |
| SingleModeTrainingCommandListEntity | 训练指令列表 |
| SingleModeTrainingCommandService | 训练指令服务 |
| SingleModeTrainingPartnerEntity | 训练伙伴(支援卡)实体 |
| SingleModeTrainingPartnerRepository | 训练伙伴仓库 |
| SingleModeTrainingPartnerTipsEntity | 训练伙伴提示 |
| SingleModeTrainingBackGroundEntity | 训练背景 |
| SingleModeTrainingCharacterEntity | 训练角色 |
| SingleModeTrainingFailureRateService | 训练失败率服务 |
| SingleModeTrainingCutInHelper | 训练切入辅助 |
| SingleModeIsTagTrainingCheckService | 彩圈训练检查服务 |
| SingleModeEffectedTrainingCommandA | 生效训练指令 |
| SingleModeEffectedTrainingCommandInfo | 生效训练指令信息 |
| SingleModeGainPartnerSupportEffectInfo | 获得伙伴支援效果信息 |
| TrainingGainParameterEntity | 训练获得参数实体 |
| TrainingParamChangeA | 训练参数变化 |
| TrainingParamChangeSupportMemberA | 训练参数变化支援成员 |
| TrainingParamChangeUI | 训练参数变化UI |
| TrainingLevelInfo | 训练等级信息 |
| TrainingDefine | 训练定义 |
| TrainingEnvParam | 训练环境参数 |
| TrainingEnvParamHelper | 训练环境参数辅助 |

### 2.2 彩圈/Tag训练

| 类名 | 用途 |
|------|------|
| SingleModeIsTagTrainingCheckService | **彩圈判断服务** |
| SingleModeMainViewTagTrainingCutInPlayer | 彩圈CutIn播放器 |
| SingleModeMainViewTagTrainingFlashModifier | 彩圈闪光修改器 |
| PartsSingleModeTagTrainingStartParticleA | 彩圈粒子效果 |
| SingleModePioneerTagTrainingCutInPlayer | Pioneer版彩圈CutIn |
| SingleModePioneerTagTrainingFlashModifier | Pioneer版彩圈闪光 |

### 2.3 支援卡体系

| 类名 | 用途 |
|------|------|
| SingleModeEquipSupportCardEntity | 装备支援卡实体 |
| SingleModeEquipSupportCardListEntity | 装备支援卡列表 |
| SingleModeSupportCard | 支援卡 |
| SingleModeFriendSupportCard | 好友支援卡 |
| UserSupportCard | 用户支援卡 |
| MasterSupportCardData | 支援卡Master数据 |
| MasterSupportCardEffectTable | 支援卡效果表 |
| MasterSupportCardUniqueEffect | 支援卡固有效果 |
| MasterSupportCardLevel | 支援卡等级 |
| MasterSupportCardLimit | 支援卡上限 |
| MasterSupportCardLimitBreak | 支援卡突破 |
| SupportCardUtil | 支援卡工具类 |
| GainPartnerSupportEffect | 获得伙伴支援效果 |

### 2.4 NPC体系

| 类名 | 用途 |
|------|------|
| MasterSingleModeNpc | NPC Master数据 |
| SingleModeNpcResult | NPC结果 |
| SingleModeNpcTeamData | NPC团队数据 |
| SingleModeTwikleRaceNpcResult | 闪亮比赛NPC结果 |
| SingleModeFreeTwinkleRaceNpcInfo | 自由剧本闪亮比赛NPC信息 |
| PartsNpcMotivation | NPC干劲 |
| PartsTrainedCharacterDetailInnerNpc | 训练角色详情内部NPC |

### 2.5 羁绊/友情

| 类名 | 用途 |
|------|------|
| SingleModeScenarioLegendGainedFriendGaugeInfo | 传说杯友情槽信息 |
| SingleModeScenarioCookFriendsPowerModel | 料理杯友情力量模型 |
| SingleModeScenarioCookGainCookingFriendPowerA | 料理杯获得料理友情力量 |
| CookGainCookingFriendPowerParameterInfo | 料理获得友情力量参数 |
| PartsSingleModeScenarioCookFriendsPowerMissionInfo | 料理杯友情力量任务 |
| PartsSingleModeScenarioCookGainFriendsPower | 料理杯获得友情力量 |
| PartsSingleModeScenarioCookMainViewFriendsPowerInfo | 料理杯主页友情力量 |

### 2.6 比赛/赛程

| 类名 | 用途 |
|------|------|
| SingleModeScenarioRacePaddockView | 剧本比赛围场视图 |
| SingleModeScenarioRacePaddockViewController | 剧本比赛围场控制器 |
| SingleModeScenarioVenusScenarioRacePaddockView | Venus剧本比赛围场 |
| SingleModeScenarioVenusScenarioRaceTop | Venus剧本比赛顶部 |
| SingleModeMainViewStartShimaTrainingCuttRunner | 岛训练CutIn |

### 2.7 剧本效果参数变化

每个剧本有独立的 `WorkSingleModeChangeParameterInfo`:
- WorkSingleModeChangeParameterInfoScenarioArc
- WorkSingleModeChangeParameterInfoScenarioBreeders
- WorkSingleModeChangeParameterInfoScenarioCook
- WorkSingleModeChangeParameterInfoScenarioFree
- WorkSingleModeChangeParameterInfoScenarioLegend
- WorkSingleModeChangeParameterInfoScenarioLive
- WorkSingleModeChangeParameterInfoScenarioMecha
- WorkSingleModeChangeParameterInfoScenarioOnsen
- WorkSingleModeChangeParameterInfoScenarioPioneer
- WorkSingleModeChangeParameterInfoScenarioRamen
- WorkSingleModeChangeParameterInfoScenarioSport
- WorkSingleModeChangeParameterInfoScenarioTeamRace
- WorkSingleModeChangeParameterInfoScenarioVenus

### 2.8 训练数据备份

每个剧本有独立的 `TrainingBackupData`:
- TrainingBackupDataDefault
- TrainingBackupDataScenarioBreeders
- TrainingBackupDataScenarioLegend
- TrainingBackupDataScenarioMecha
- TrainingBackupDataScenarioPioneer
- TrainingBackupDataScenarioSport
- TrainingBackupDataScenarioVenus

---

## 三、剧本14 拉面杯核心类

### 3.1 WorkSingleModeScenarioRamen

```
WorkSingleModeScenarioRamen  — 拉面杯剧本数据
├── 地区选择
│   ├── SingleModeScenarioRamenRegionSelectView
│   ├── SingleModeScenarioRamenRegionSelectViewController
│   └── SingleModeScenarioRamenRegionSelectViewModel
├── 地区地图
│   ├── SingleModeScenarioRamenRegionMapView
│   ├── SingleModeScenarioRamenRegionMapViewController
│   └── SingleModeScenarioRamenRegionMapViewViewModel
├── 检查点
│   ├── SingleModeScenarioRamenCheckPointTopView
│   ├── SingleModeScenarioRamenCheckPointTopViewController
│   ├── SingleModeScenarioRamenCheckPointTopViewModel
│   ├── SingleModeScenarioRamenFinalCheckPointTopView
│   ├── SingleModeScenarioRamenFinalCheckPointTopViewController
│   └── SingleModeScenarioRamenFinalCheckPointTopViewViewModel
├── 训练执行
│   ├── SingleModeRamenTrainingExecInfo
│   ├── ObscuredSingleModeRamenTrainingExecInfo
│   └── SingleModeMainTrainingDecideConfirmScenarioRamen
├── CutIn
│   ├── SingleModeScenarioRamenCutInHelper
│   ├── SingleModeScenarioRamenMiniCharaController
│   ├── SingleModeScenarioRamenMiniCharaParam
│   ├── SingleModeScenarioRamenMiniCharaFinalTopParam
│   └── SingleModeScenarioRamenExtraEditionImageController
├── 剧情
│   ├── SingleModeScenarioRamenStoryOriginalRaceA
│   └── SingleModeScenarioRamenSettingParam
└── 定义
    └── SingleModeScenarioRamenDefine
```

### 3.2 WorkSingleModeScenarioCook（料理杯）

```
WorkSingleModeScenarioCook  — 料理杯剧本数据
├── CookInfoData (料理信息)
├── DishInfoData (菜品信息)
├── EvaluationInfo (评价信息)
├── ResultInfoData (结果信息)
├── FacilityInfoData (设施信息)
├── MaterialInfoData (材料信息)
├── DishSkillInfoData (菜品技能信息)
├── PowerEffectInfoData (力量效果信息)
├── GainMaterialInfoData (获得材料信息)
├── MaterialHarvestInfoData (材料收获信息)
├── CommandMaterialCareInfoData (指令材料关注信息)
└── MaterialCareHistoryInfoData (材料关注历史信息)
```

---

## 四、MasterDB 剧本数据表映射

### 4.1 single_mode_scenario（8个基础剧本）

| id | sort_id | 推测剧本 | 属性上限(速/耐/力/根/智) |
|----|---------|---------|------------------------|
| 1 | 1 | URA | 200/200/200/200/200 |
| 2 | 2 | 青春杯 | 100/100/100/100/600 |
| 3 | 4 | 巅峰杯 | 400/100/100/300/100 |
| 4 | 3 | 凯旋门 | 0/700/0/0/300 |
| 5 | 5 | 大师杯 | 300/200/300/100/100 |
| 6 | 6 | L'Arc | 400/400/300/300/100 |
| 7 | 7 | 种田杯 | 500/300/300/300/100 |
| 8 | 8 | 拉面杯 | 550/-200/500/500/150 |

### 4.2 single_mode_scenario_group（剧本组）

| group_id | 包含剧本 |
|----------|---------|
| 1 | 1-8 (基础) |
| 100 | 101-108 (扩展) |
| 601 | 601-602 |
| 701 | 701-706 |
| 801 | 801-808 |
| 901 | 901-905 |
| 1901 | 1901-1907 |

---

## 五、剧本14 数据库表结构（从 metadata SQL 提取）

### single_mode_14_basic_effect
```sql
SELECT id, effect_type, effect_value_1, effect_value_2, effect_value_3, effect_value_4
FROM single_mode_14_basic_effect
```

### single_mode_14_check_point
```sql
-- 检查点定义
SELECT id, check_point_type, check_point_value, ...
FROM single_mode_14_check_point
```

### single_mode_14_finals_effect
```sql
SELECT text_group_id, select_type, effect_type, effect_value,
       condition_type_1, condition_value_1, ...
FROM single_mode_14_finals_effect
```

### single_mode_14_twinkle_ramen
```sql
SELECT text_group, text_number, check_point_type, result_state, text_type, text_type_value
FROM single_mode_14_twinkle_ramen
```

### single_mode_14_region_effect
```sql
-- 地区效果
SELECT id, region_id, effect_type, effect_value, ...
FROM single_mode_14_region_effect
```

### single_mode_14_region_select
```sql
-- 地区选择
SELECT id, region_id, select_condition, ...
FROM single_mode_14_region_select
```

### single_mode_14_feeling_bonus
```sql
-- 心情加成
SELECT id, feeling_type, bonus_type, bonus_value, ...
FROM single_mode_14_feeling_bonus
```

### single_mode_14_special_gain_turn
```sql
SELECT turn, gain_id
FROM single_mode_14_special_gain_turn
```

---

## 六、关键发现与修正

### 6.1 剧本映射修正

**之前错误**: 将 master.mdb 的 scenario_id 1-8 直接映射为所有剧本
**正确映射**: 
- master.mdb 的 scenario 1-8 是基础剧本
- scenario 9-14 在 il2cpp 中有独立数据表（single_mode_09_* ~ single_mode_14_*）
- **剧本13 = 青春杯/种田杯 (Breeders)**，不是"凯旋门"
- **剧本14 = 拉面杯 (Ramen/TwinkleRamen)**，是我们需要的

### 6.2 训练效果隔离

不同剧本的训练效果**不共享**：
- 剧本 1-8: 使用通用的 `single_mode_training_effect`（按 scenario_id 区分）
- 剧本 9-14: 使用专用的 `single_mode_XX_*` 表

**所以在 hlpatch 中，剧本14的训练效果不会从 `single_mode_training_effect(scenario_id=8)` 读取，而是从 `single_mode_14_*` 系列表读取。**

### 6.3 彩圈判断核心类

`SingleModeIsTagTrainingCheckService` — 这是彩圈判断的入口服务类。
`SingleModeEffectedTrainingCommandA` — 生效训练指令，包含当前训练类型和支援卡匹配逻辑。

### 6.4 支援卡效果类型完整枚举

从 `MasterSupportCardEffectTable` 和 `support_card_effect_table` 确认：

| type | 名称 | 说明 |
|------|------|------|
| 1 | 速度加成 | 训练速度属性增加 |
| 2 | 耐力加成 | 训练耐力属性增加 |
| 3 | 初期羁绊 | 支援卡初始羁绊值 |
| 5 | 友情训练 | 达到条件后触发彩圈 |
| 8 | 训练效果 | 训练效果整体提升 |
| 9 | 干劲效果 | 干劲对训练的影响 |
| 14 | 得意率 | 支援卡出现在得意训练的概率 |
| 15 | 技能Pt加成 | 训练获得技能点增加 |
| 16 | 比赛加成 | 比赛后属性增加 |
| 17 | 粉丝数加成 | 比赛后粉丝数增加 |
| 18 | 体力消费下降 | 训练体力消耗减少 |
| 19 | 初始属性 | 育成开始时属性增加 |
| 25 | 干劲效果(高级) | 高等级干劲效果 |
| 26 | 智力训练回复 | 智力训练体力回复 |
| 27 | 事件回复 | 事件中体力回复 |
| 28 | 事件效果 | 事件中效果提升 |
| 30 | 启发率 | 技能启发概率 |
| 31 | 启发事件 | 技能启发事件 |

---

## 七、完整类列表（11,240个类）

所有 Gallop 命名空间下的类已从 metadata 提取。关键模块：

| 模块 | 类数量(约) | 说明 |
|------|-----------|------|
| SingleMode | 500+ | 育成核心逻辑 |
| Scenario | 200+ | 各剧本专用逻辑 |
| Training | 150+ | 训练系统 |
| SupportCard | 100+ | 支援卡系统 |
| Race | 200+ | 比赛系统 |
| Skill | 80+ | 技能系统 |
| Master | 300+ | Master数据库 |
| Work | 100+ | 运行时数据 |
| Dialog | 400+ | UI对话框 |
| Parts | 500+ | UI组件 |
| Network | 200+ | 网络请求 |

---

## 八、后续工作建议

1. **剧本14数据表查询**: 需从手机端获取 `single_mode_14_*` 表数据（不在 master.mdb 中，可能在运行时下载）
2. **彩圈判断逻辑**: 重点分析 `SingleModeIsTagTrainingCheckService` 和 `SingleModeEffectedTrainingCommandA`
3. **地区效果系统**: 剧本14的 `region_effect` 表是关键
4. **训练效果系统**: 剧本14使用独立的 `basic_effect` 而非通用 `training_effect`

---

*报告生成时间: 2026-07-11*
*数据来源: libil2cpp.so (209MB) + global-metadata.dat (44MB) v31 + master.mdb (26MB)*

---

> 本内容由 Coze AI 生成，请遵循相关法律法规及《人工智能生成合成内容标识办法》使用与传播。
