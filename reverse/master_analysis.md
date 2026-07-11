# 赛马娘 v2.28.5 全量逆向工程分析报告

**游戏版本**: v2.28.5 (日服)  
**libil2cpp.so**: 209MB (ARM64)  
**IL2CPP版本**: v31
**数据源**: IL2CPP方法转储 (27695类, 160909方法) + hlpatch插件源码 (1856行)
**生成时间**: 2026-07-11

---

## 一、剧本系统总览

共 **14个剧本ID**：

| ID | 剧本名 | 专用类数 | Obscured类数 |
|---|---|---|---|
| 1 | URA (育成シナリオ) | 66 | 0 |
| 2 | TeamRace (チームレース) | 0 | 0 |
| 3 | Live (ライブ) | 0 | 0 |
| 4 | Free (フリー) | 0 | 0 |
| 5 | Venus (ヴィーナス) | 0 | 0 |
| 6 | Arc (アーク) | 0 | 0 |
| 7 | Sport (スポーツ) | 0 | 0 |
| 8 | Cook (クック) | 0 | 0 |
| 9 | Mecha (メカ) | 0 | 0 |
| 10 | Legend (レジェンド) | 14 | 2 |
| 11 | Pioneer (パイオニア/青春杯) | 11 | 0 |
| 12 | Onsen (温泉) | 15 | 24 |
| 13 | Breeders (ブリーダーズ/種田杯) | 10 | 28 |
| 14 | Ramen (ラーメン/トゥインクル・ラーメン杯) | 16 | 40 |

## 二、核心数据访问路径

```
WorkSingleModeData (育成根节点)
  └→ WorkSingleModeCharaData (角色育成数据)
       ├→ support_card_array → SupportCardEntry[] (支援卡槽位)
       ├→ evaluation_info_array → EvaluationInfo[] (羁绊信息)
       ├→ training_level_info_array → TrainingLevelInfo[] (训练等级)
       └→ get_HomeInfoData() → WorkSingleModeHomeInfoData
            └→ CommandInfoArray → SingleModeCommandInfoData[] (训练命令)
                 ├→ get_CommandId() → ObscuredInt (101-106)
                 ├→ get_IsEnable() → ObscuredInt (0/1)
                 ├→ get_FailureRate() → ObscuredInt (%)
                 ├→ get_TrainingPartnerArray() → 伙伴列表
                 ├→ get_TipsEventPartnerArray() → 彩圈伙伴列表
                 └→ get_ParamsIncDecInfoArray() → 属性增减列表
                      ├→ get_TargetType() → ObscuredInt (1-30)
                      └→ get_Value() → ObscuredInt (增减值)
```

### WorkSingleModeCharaData — 211 methods, 81 getters

  - `get_Id`
  - `get_CardId`
  - `get_CharaId`
  - `get_CardData`
  - `get_CardRarityData`
  - `get_SuccessionTrainedCharaInfoFirst`
  - `get_SuccessionTrainedCharaInfoSecond`
  - `get_TalentLevel`
  - `get_LimitBreakCount`
  - `get_EquipSupportCardArray`
  - `get_ChangedModelDressId`
  - `get_CharaGrade`
  - `get_Hp`
  - `get_MaxHp`
  - `get_Speed`
  - `get_Stamina`
  - `get_Power`
  - `get_Guts`
  - `get_Wiz`
  - `get_MaxSpeed`
  - `get_MaxStamina`
  - `get_MaxPower`
  - `get_MaxGuts`
  - `get_MaxWiz`
  - `get_DefaultMaxSpeed`
  - `get_DefaultMaxStamina`
  - `get_DefaultMaxPower`
  - `get_DefaultMaxGuts`
  - `get_DefaultMaxWiz`
  - `get_EntryProgramId`
  - `get_ScenarioId`
  - `get_ScenarioImageId`
  - `get_DifficultyId`
  - `get_Difficulty`
  - `get_IsDifficultyTpBoost`
  - `get_RouteId`
  - `get_StartTime`
  - `get_TrainingEventType`
  - `get_AcquiredSkillList`
  - `get_DisableSkillIdList`
  - `get_SkillTipsList`
  - `get_SkillPoint`
  - `get_TrainingLevelInfoArray`
  - `get_ProperDistanceShort`
  - `get_ProperDistanceMile`
  - `get_ProperDistanceMiddle`
  - `get_ProperDistanceLong`
  - `get_ProperRunningStyleNige`
  - `get_ProperRunningStyleSenko`
  - `get_ProperRunningStyleSashi`
  - `get_ProperRunningStyleOikomi`
  - `get_ProperGroundTurf`
  - `get_ProperGroundDirt`
  - `get_RunningStyle`
  - `get_EventShortcutType`
  - `get_IsShortCutAllEvent`
  - `get_FanCount`
  - `get_EvaluationList`
  - `get_ReservedRaceProgramId`
  - `get_UpdateReservedPaceProgramId`
  - `get_Motivation`
  - `get_CharaEffectIdArray`
  - `get_RouteRaceIdArray`
  - `get_IsShortRace`
  - `get_ScenarioProgress`
  - `get_Race`
  - `get_WorkScenarioURA`
  - `get_TeamRace`
  - `get_RaceReserveContext`
  - `get_WorkScenarioFree`
  - `get_ScenarioLive`
  - `get_ScenarioVenus`
  - `get_ScenarioArc`
  - `get_ScenarioSport`
  - `get_ScenarioCook`
  - `get_ScenarioBreeders`
  - `get_ScenarioLegend`
  - `get_ScenarioMecha`
  - `get_ScenarioOnsen`
  - `get_ScenarioPioneer`

## 三、ID映射系统

### 3.1 CommandId → 训练类型
| CommandId | 训练 |
|---|---|
| 101 | Speed |
| 102 | Stamina |
| 103 | Guts |
| 105 | Power |
| 106 | Wisdom |

> ⚠️ CommandId 非对称：Guts=103, Power=105（跳过104）

### 3.2 TargetType → 属性类型
| TargetType | 属性 |
|---|---|
| 1 | Speed |
| 2 | Stamina |
| 3 | Guts |
| 4 | Power |
| 5 | Wiz |
| 10 | HP |
| 20 | Motivation |
| 30 | SkillPt |

> ⚠️ TargetType 与 CommandId 完全独立：Guts=3, Power=4

### 3.3 Motivation (心情)
| 等级 | 心情 |
|---|---|
| 5 | Best |
| 4 | Good |
| 3 | Normal |
| 2 | Bad |
| 1 | Worst |

## 四、支援卡系统

### 4.1 内存布局 (SupportCardEntry)
```
+0x10  position               i32   槽位 (1-8)
+0x14  support_card_id        i32   支援卡ID
+0x18  limit_break_count      i32   凸数 (0-4)
+0x20  training_partner_state i32   训练伙伴状态
``
- IL2CPP Array: length@+0x18, elements@+0x20 (8-byte ptrs)
- 访问: `WorkSingleModeCharaData.get_SupportCardArray()`

### 4.2 支援卡相关类 (173个)

- `.PartsSupportCardLimitBreakUnlockIcon` (1m)
- `.CampaignRentalSupportCard` (0m)
- `Gallop.MasterCampaignRentalSupportCard` (9m)
- `.SupportCardData` (20m)
- `Gallop.MasterSupportCardData` (16m)
- `.SupportCardEffectFilter` (1m)
- `Gallop.MasterSupportCardEffectFilter` (9m)
- `.SupportCardEffectFilterGroup` (0m)
- `Gallop.MasterSupportCardEffectFilterGroup` (5m)
- `.SupportCardEffectTable` (7m)
- `Gallop.MasterSupportCardEffectTable` (11m)
- `.SupportCardGroup` (0m)
- `Gallop.MasterSupportCardGroup` (15m)
- `.SupportCardLevel` (0m)
- `Gallop.MasterSupportCardLevel` (11m)
- `.SupportCardLimit` (2m)
- `Gallop.MasterSupportCardLimit` (5m)
- `.SupportCardLimitBreak` (0m)
- `Gallop.MasterSupportCardLimitBreak` (11m)
- `.SupportCardTeamScoreBonus` (0m)
- `Gallop.MasterSupportCardTeamScoreBonus` (8m)
- `.SupportCardUniqueEffect` (2m)
- `Gallop.MasterSupportCardUniqueEffect` (3m)
- `.AnnounceSupportCard` (1m)
- `Gallop.MasterAnnounceSupportCard` (4m)
- `.StoryEventBonusGroupSupportCard` (2m)
- `Gallop.MasterStoryEventBonusGroupSupportCard` (15m)
- `.StoryEventBonusSupportCard` (2m)
- `Gallop.MasterStoryEventBonusSupportCard` (9m)
- `Gallop.ISingleModeEquipSupportCardEntity` (6m)
- `Gallop.SingleModeEquipSupportCardEntity` (6m)
- `Gallop.ISingleModeEquipSupportCardListEntity` (1m)
- `Gallop.SingleModeEquipSupportCardListEntity` (1m)
- `Gallop.SingleModeEquipSupportCardListRepository` (1m)
- `Gallop.ObscuredIdleSingleModeSupportCardGainInfo` (4m)
- `Gallop.ObscuredIdleSingleModeSupportCardGainInfoExtensions` (3m)
- `Gallop.ObscuredRewardAddSupportCardNum` (6m)
- `Gallop.ObscuredRewardAddSupportCardNumExtensions` (3m)
- `Gallop.ObscuredUserSupportCard` (14m)
- `Gallop.ObscuredUserSupportCardExtensions` (3m)

### 4.3 支援卡类型枚举 (2个)
- `Gallop.SingleMode.ScenarioRamen.PartsSingleModeScenarioRamenRegionSelectSupportCardTypeListViewModel` (enum=False)
- `Gallop.SingleMode.ScenarioRamen.PartsSingleModeScenarioRamenRegionSelectSupportCardTypeList` (enum=False)

## 五、羁绊系统

### 5.1 内存布局 (EvaluationInfo)
```
+0x10  target_id    i32  目标ID (支援卡ID / 特殊NPC ID)
+0x14  evaluation   i32  羁绊值 (0-100)
+0x20  is_appear    i32  是否出现 (0/1)
``
- 访问: `WorkSingleModeCharaData.get_EvaluationInfoArray()`
- target_id 对应 support_card_id；理事长/记者等特殊NPC使用固定ID

### 5.2 羁绊相关类 (28个)

- `.ChampionsEvaluationRate` (3m)
  - `GetProperType`
  - `GetRank`
  - `GetRate`
- `Gallop.MasterChampionsEvaluationRate` (6m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `CalcScore`
- `.SingleModeEvaluation` (0m)
- `Gallop.MasterSingleModeEvaluation` (11m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithCharaIdOrderByIdAsc`
  - `_SelectWithCharaIdOrderByIdAsc`
  - `GetListWithCharaIdOrderByIdAsc`
  - `MaybeListWithCharaIdOrderByIdAsc`
  - `_ListSelectWithCharaIdOrderByIdAsc`
- `.TeamStadiumEvaluationRate` (0m)
- `Gallop.MasterTeamStadiumEvaluationRate` (8m)
  - `get_dictionary`
  - `GetKey`
  - `Get`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `GetRate`
- `Gallop.ObscuredSingleModeOnsenEvaluationInfo` (4m)
  - `get_TargetId`
  - `set_TargetId`
  - `get_CharaId`
  - `set_CharaId`
- `Gallop.ObscuredSingleModeOnsenEvaluationInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`
- `Gallop.ObscuredSingleModeRamenEvaluationInfo` (4m)
  - `get_TargetId`
  - `set_TargetId`
  - `get_CharaId`
  - `set_CharaId`
- `Gallop.ObscuredSingleModeRamenEvaluationInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`
- `.EvaluationData` (6m)
  - `get_CharaId`
  - `set_CharaId`
  - `get_CharacterName`
  - `set_CharacterName`
  - `get_DeltaValue`
  - `set_DeltaValue`
- `.Evaluation` (17m)
  - `get_TargetId`
  - `get_Value`
  - `get_IsOuting`
  - `get_StoryStep`
  - `get_IsAppear`
  - `get_GuestCharaId`
  - `get_InterestState`
  - `get_SoulEventState`
- `.EvaluationInfo` (3m)
  - `get_CharaId`
  - `get_TargetId`
  - `get_IsCookingFriend`
- `.PioneerEvaluationInfo` (0m)
- `Gallop.DialogSingleModeEvaluationMessage` (4m)
  - `GetFormType`
  - `GetParentType`
  - `Setup`
  - `Open`
- `.<PlayShimaTrainingEvaluationUp>d__20` (5m)
  - `System.IDisposable.Dispose`
  - `MoveNext`
  - `System.Collections.Generic.IEnumerator<System.Object>.get_Current`
  - `System.Collections.IEnumerator.Reset`
  - `System.Collections.IEnumerator.get_Current`
- `.EvaluationIcon` (5m)
  - `SetCharaIcon`
  - `SetEvalutionGauge`
  - `SetActiveMaxIcon`
  - `SetObtainIcon`
  - `GetObtainIconTexture`
- `Gallop.PartsSingleModeScenarioFreeShopUseItemConfirmEvaluationBase` (3m)
  - `SetIcon`
  - `SetEvaluationGauge`
  - `SetObtainIcon`
- `Gallop.PartsSingleModeScenarioFreeShopUseItemConfirmEvaluationSupportCardModel` (4m)
  - `get_SingleModeFreeShopEffectList`
  - `get_EffectTargetEvaluationList`
  - `GetPreviewEvaluationValue`
  - `GetEffectValue`
- `Gallop.PartsSingleModeScenarioFreeShopUseItemConfirmEvaluationSupportCard` (2m)
  - `Create`
  - `Setup`
- `Gallop.PartsSingleModeScenarioFreeShopUseItemConfirmEvaluationUniqueCharaModel` (4m)
  - `get_SingleModeFreeShopEffectList`
  - `get_EffectTargetEvaluationList`
  - `GetPreviewEvaluationValue`
  - `GetEffectValue`
- `Gallop.PartsSingleModeScenarioFreeShopUseItemConfirmEvaluationUniqueChara` (2m)
  - `Create`
  - `Setup`
- `Gallop.DialogTeamEvaluationPoint` (25m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `Open`
  - `RegisterDownload`
  - `Setup`
  - `PlayIn`
  - `FlameIn`
- `Gallop.DialogTeamEvaluationRewardList` (5m)
  - `CreateDialogData`
  - `GetFormType`
  - `GetParentType`
  - `Open`
  - `SetUp`
- `Gallop.DialogUpdateTeamEvaluationPoint` (51m)
  - `get_CloseButton`
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `Open`
  - `RegisterDownload`
  - `Setup`
  - `PlayRankUp`
- `Gallop.PartsTeamEvaluationRankupRewardsRow` (5m)
  - `Setup`
  - `UpdateIcons`
  - `PlayIcon`
  - `ShowIcons`
  - `IsContainReward`
- `Gallop.PartsTeamEvaluationRewardItem` (2m)
  - `Setup`
  - `Setup`
- `Gallop.SingleMode.ScenarioPioneer.PartsSingleModeScenarioPioneerShimaTrainingEvaluationUp` (9m)
  - `get_TrainingParamChangeUI`
  - `RegisterDownload`
  - `Create`
  - `Play`
  - `Setup`
  - `Play`
  - `PlayOut`
  - `CreateMessageWindow`

## 六、训练系统


### 6.2 训练数据结构
```
SingleModeCommandInfoData:
  get_CommandId()             → ObscuredInt (101-106)
  get_IsEnable()              → ObscuredInt (0/1)
  get_FailureRate()           → ObscuredInt (失败率%)
  get_TrainingPartnerArray()  → Array (训练伙伴)
  get_TipsEventPartnerArray() → Array (彩圈伙伴)
  get_ParamsIncDecInfoArray() → Array<SingleModeParamsIncDecInfoData>

SingleModeParamsIncDecInfoData:
  get_TargetType() → ObscuredInt
  get_Value()      → ObscuredInt
```

### 6.3 训练等级 (TrainingLevelInfo)
```
+0x10  command_id  i32  训练类型 (101-106)
+0x14  level       i32  等级 (1-5)
``
- 访问: `WorkSingleModeCharaData.get_TrainingLevelInfoArray()`

### 6.4 训练执行相关类 (40个)

- `Gallop.SingleModeScenarioPioneerShimaTrainingCommandEntity` (4m)
  - `get_FacilityId`
  - `set_FacilityId`
  - `get_TrainingCommandId`
  - `set_TrainingCommandId`
- `Gallop.ISingleModeScenarioPioneerShimaTrainingCommandEntity` (2m)
  - `get_FacilityId`
  - `get_TrainingCommandId`
- `Gallop.ISingleModeScenarioPioneerShimaTrainingCommandRepository` (1m)
  - `Get`
- `Gallop.SingleModeScenarioPioneerShimaTrainingCommandRepository` (1m)
  - `Get`
- `Gallop.SingleModeScenarioPioneerShimaTrainingCommandService` (2m)
  - `GetShimaTrainingData`
  - `GetExecutableShimaTrainingDataArray`
- `Gallop.SingleModeScenarioOnsenAssistantTrainingCommandEntity` (11m)
  - `get_DegreeType`
  - `set_DegreeType`
  - `get_CommandType`
  - `get_CommandId`
  - `get_IsEnable`
- `Gallop.SingleModeScenarioOnsenTrainingCommandListEntity` (4m)
  - `get_TrainingCommandList`
  - `set_TrainingCommandList`
  - `GetTrainingCommand`
  - `GetOnsenAssistantTraining`
- `Gallop.SingleModeScenarioOnsenTrainingCommandListRepository` (6m)
  - `Get`
  - `GetTrainingGainBonusParameterList`
  - `get_WorkSingleModeScenarioOnsen`
  - `get_OnsenAssistantCommandInfo`
  - `get_TrainingCommandInfoDataList`
- `Gallop.SingleModeScenarioOnsenTrainingCommandService` (3m)
  - `ExecTraining`
  - `ExecOnsenAssistant`
  - `ExecTrainingBase`
- `Gallop.SingleModeScenarioPioneerTrainingCommandListRepository` (4m)
  - `Get`
  - `GetTrainingGainBonusParameterList`
  - `get_TrainingCommandInfoDataList`
  - `get_TrainingLevelInfoArray`
- `Gallop.SingleModeScenarioPioneerTrainingCommandService` (3m)
  - `ExecTraining`
  - `SendExecCommand`
  - `SendShimaTrainingExec`
- `Gallop.SingleModeTrainingCommandService` (1m)
  - `ExecTraining`
- `Gallop.ISingleModeTrainingCommandEntity` (5m)
  - `get_BaseCommandId`
  - `get_TrainingLevel`
  - `get_TrainingFailureRate`
  - `get_TrainingGainParameterList`
  - `get_TrainingGainBonusParameterList`
- `Gallop.SingleModeTrainingCommandEntity` (15m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_CommandId`
  - `set_CommandId`
  - `get_IsEnable`
- `Gallop.ISingleModeTrainingCommandListEntity` (2m)
  - `get_TrainingCommandList`
  - `GetTrainingCommand`
- `Gallop.SingleModeTrainingCommandListEntity` (3m)
  - `get_TrainingCommandList`
  - `set_TrainingCommandList`
  - `GetTrainingCommand`
- `Gallop.SingleModeTrainingCommandListRepository` (3m)
  - `Get`
  - `get_TrainingCommandInfoDataList`
  - `get_TrainingLevelInfoArray`
- `.<PlayExecTrainingBeforePlayCut>d__94` (5m)
  - `System.IDisposable.Dispose`
  - `MoveNext`
  - `System.Collections.Generic.IEnumerator<System.Object>.get_Current`
  - `System.Collections.IEnumerator.Reset`
  - `System.Collections.IEnumerator.get_Current`
- `.<PlayExecTrainingBeforePlayCutDefault>d__95` (5m)
  - `System.IDisposable.Dispose`
  - `MoveNext`
  - `System.Collections.Generic.IEnumerator<System.Object>.get_Current`
  - `System.Collections.IEnumerator.Reset`
  - `System.Collections.IEnumerator.get_Current`
- `.<PlayExecTrainingBeforePlayCutPioneerShimaTraining>d__10` (5m)
  - `System.IDisposable.Dispose`
  - `MoveNext`
  - `System.Collections.Generic.IEnumerator<System.Object>.get_Current`
  - `System.Collections.IEnumerator.Reset`
  - `System.Collections.IEnumerator.get_Current`
- `.<PlayExecTrainingBeforePlayCutScenarioPioneer>d__9` (5m)
  - `System.IDisposable.Dispose`
  - `MoveNext`
  - `System.Collections.Generic.IEnumerator<System.Object>.get_Current`
  - `System.Collections.IEnumerator.Reset`
  - `System.Collections.IEnumerator.get_Current`
- `.<PlayExecTrainingBeforePlayCutScenarioSportLinkTraining>d__19` (5m)
  - `System.IDisposable.Dispose`
  - `MoveNext`
  - `System.Collections.Generic.IEnumerator<System.Object>.get_Current`
  - `System.Collections.IEnumerator.Reset`
  - `System.Collections.IEnumerator.get_Current`
- `Gallop.SingleModeScenarioCookTrainingCommandModel` (12m)
  - `get_WorkCook`
  - `get_ExistMaterialInfo`
  - `get_CommandMaterialCareInfoData`
  - `get_CommandId`
  - `get_MaterialId`
- `Gallop.SingleModeScenarioSportTrainingCommandModel` (14m)
  - `get_CommandId`
  - `get_SportType`
  - `get_MasterSingleModeTraining`
  - `get_BaseCommandId`
  - `get_TrainingName`
- `Gallop.PartsSingleModeScenarioSportUseItemTrainingCommandButtonForGUI` (6m)
  - `Setup`
  - `SetupTrainingIcon`
  - `SetupSportTypeIcon`
  - `SetupTrainingTypeIcon`
  - `SetupSportRankText`
- `Gallop.PartsSingleModeScenarioSportUseItemTrainingCommandButtonForGUIAccessory` (7m)
  - `RegisterDownload`
  - `SetActiveTagTrainingEffect`
  - `SetTagTrainingBackEffectParent`
  - `SetupTrainingPartnerNum`
  - `SetupGainSportRank`
- `Gallop.SingleModeMainTrainingResultMassageServiceFactory` (1m)
  - `Create`
- `Gallop.ISingleModeMainTrainingResultMassageService` (1m)
  - `GetTrainingResultMassage`
- `Gallop.AbstractSingleModeMainTrainingResultMassageService` (29m)
  - `GetTrainingResultMassage`
  - `IsAddHp`
  - `IsAddMotivation`
  - `IsAddParameterMax`
  - `IsAddLimitCharaEffect`
- `Gallop.SingleModeMainTrainingResultMassageService` (0m)

## 七、彩圈(Shining)系统

```
彩圈 = TipsEventPartnerArray.Length > 0

判定链:
  1. 支援卡出现在训练 → TrainingPartnerArray 包含该卡
  2. 支援卡闪彩 → TipsEventPartnerArray 包含该卡
  3. 闪彩条件 (推断):
     a. bond ≥ 80
     b. support_card_type 匹配 CommandId
     c. specialty 匹配训练类型
```

### 7.1 彩圈相关类 (2个)
- `.<PlayTrainingTipsEventText>d__51` (5m)
- `.<PlayTrainingTipsEventWipe>d__53` (5m)

## 八、角色系统

### 8.1 角色数据类 (80个)

- `Gallop.IMiniCharaData` (7m)
  - `get_UniqueId`
  - `get_Name`
  - `get_CharaId`
  - `get_DressId`
  - `get_DressColorId`
- `Gallop.MiniCharaData` (11m)
  - `get_UniqueId`
  - `set_UniqueId`
  - `get_Name`
  - `set_Name`
  - `get_CharaId`
- `Gallop.CookingMiniCharaData` (13m)
  - `get_UniqueId`
  - `set_UniqueId`
  - `get_Name`
  - `set_Name`
  - `get_CharaId`
- `Gallop.CookingResultMiniCharaData` (13m)
  - `get_UniqueId`
  - `set_UniqueId`
  - `get_Name`
  - `set_Name`
  - `get_CharaId`
- `Gallop.SingleModeScenarioPioneerEndingSecondCutMiniCharaData` (12m)
  - `get_UniqueId`
  - `set_UniqueId`
  - `get_Name`
  - `set_Name`
  - `get_CharaId`
- `.CardData` (13m)
  - `get_Name`
  - `get_Titlename`
  - `get_Charaname`
  - `get_CharaFurigana`
  - `get_IsDummyCard`
- `Gallop.MasterCardData` (21m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithCharaIdOrderByIdAsc`
  - `_SelectWithCharaIdOrderByIdAsc`
- `.CharaData` (26m)
  - `get_Name`
  - `get_FormalName`
  - `get_Furigana`
  - `get_Adana`
  - `get_Voice`
- `Gallop.MasterCharaData` (9m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`
- `.CharaDataGroup` (0m)
- `Gallop.MasterCharaDataGroup` (11m)
  - `Get`
  - `_SelectOne`
  - `GetWithGroupId`
  - `_SelectWithGroupId`
  - `_CreateOrmByQueryResultWithGroupId`
- `.SupportCardData` (20m)
  - `get_Name`
  - `get_Titlename`
  - `get_Charaname`
  - `get_CharaNameFurigana`
  - `get_GroupName`
- `Gallop.MasterSupportCardData` (16m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithCharaIdOrderByIdAsc`
  - `_SelectWithCharaIdOrderByIdAsc`
- `.LegendRaceCuttCharaData` (0m)
- `Gallop.MasterLegendRaceCuttCharaData` (11m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithSubIdAndCharaNumOrderByCharaIdAsc`
  - `_SelectWithSubIdAndCharaNumOrderByCharaIdAsc`
- `Gallop.MasterCardDatabase` (249m)
  - `get_masterCardData`
  - `set_masterCardData`
  - `get_masterCardRarityData`
  - `set_masterCardRarityData`
  - `get_masterCardTalentLevelUpgradeItem`
- `.MainStoryRaceCharaData` (0m)
- `Gallop.MasterMainStoryRaceCharaData` (15m)
  - `Get`
  - `_SelectOne`
  - `GetWithGroupIdOrderByBracketNumberAsc`
  - `_SelectWithGroupIdOrderByBracketNumberAsc`
  - `GetListWithGroupIdOrderByBracketNumberAsc`
- `.TrainingCuttCharaData` (1m)
  - `IsMatch`
- `Gallop.MasterTrainingCuttCharaData` (13m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithCommandIdAndSubIdOrderByCharaIdAsc`
  - `_SelectWithCommandIdAndSubIdOrderByCharaIdAsc`
- `.IMasterCharaDataAccessor` (17m)
  - `get_CharaId`
  - `get_BirthYear`
  - `get_BirthMonth`
  - `get_BirthDay`
  - `get_ImageColorMain`
- `Gallop.WorkSingleModeCharaData` (211m)
  - `get_Id`
  - `set_Id`
  - `get_CardId`
  - `set_CardId`
  - `get_CharaId`
- `Gallop.WorkCampaignRentalSupportCardData` (12m)
  - `get_IsUnlockCampaignRentalSupportCard`
  - `get_IsActive`
  - `get_CurrentCampaignRentalSupportDataId`
  - `get_RentalNumToDay`
  - `get_RemainRentalNumToDay`
- `.CardData` (73m)
  - `GetMasterCard`
  - `GetMasterRarityCard`
  - `GetMasterChara`
  - `GetCardId`
  - `GetTitleName`
- `Gallop.WorkCardData` (28m)
  - `get_ReleaseCardIdList`
  - `set_ReleaseCardIdList`
  - `UpdateAll`
  - `UpdateChangedModelDressId`
  - `AddCardList`
- `.ChampionsCharaData` (19m)
  - `get_UniqueId`
  - `get_Name`
  - `get_IsVisibleMessage`
  - `get_IsTrialCharacter`
  - `get_DressId`
- `.CharaData` (22m)
  - `Update`
  - `Update`
  - `get_Id`
  - `set_Id`
  - `get_TrainingNum`
- `Gallop.WorkCharaData` (9m)
  - `GetList`
  - `GetListShuffle`
  - `AddFan`
  - `UpdateLovePoint`
  - `Update`
- `.DirectoryCardData` (2m)
  - `get_Id`
  - `get_TrainedCharaData`
- `.WorkGalleryModeCharaData` (16m)
  - `get_CharaId`
  - `get_IsValidChara`
  - `get_CardId`
  - `get_CardData`
  - `get_RaceDressId`

## 九、技能系统

### 9.1 技能类 (19个)

- `Gallop.SkillUpgradeModel` (14m)
- `Gallop.SkillListItemAdjusterData` (6m)
- `Gallop.SkillBaseContentsHolder` (10m)
- `Gallop.SkillIcon` (15m)
- `Gallop.SkillIconLongTapInfoPop` (4m)
- `Gallop.SkillItem` (1m)
- `Gallop.SkillBuildInfo` (0m)
- `Gallop.SkillBase` (9m)
- `Gallop.SkillManager` (23m)
- `Gallop.SkillManagerReplay` (3m)
- `Gallop.SkillEffect` (26m)
- `Gallop.SkillEffectPlayer` (15m)
- `Gallop.SkillEffectPool` (17m)
- `Gallop.SkillEffectPoolNull` (8m)
- `Gallop.SkillModifierReceiver` (3m)
- `Gallop.SkillSEPlayer` (8m)
- `Gallop.SkillView` (14m)
- `Gallop.SkillViewNull` (14m)
- `Gallop.SkillDefine` (0m)

## 十、比赛系统

### 10.1 比赛类 (443个)

- `Gallop.RacePlayableAnimation` (45m)
- `Gallop.RaceImageEffect` (13m)
- `Gallop.RaceAudioAisacDistanceParameterAction` (1m)
- `Gallop.RaceAudioAisacSpeedParameterAction` (2m)
- `Gallop.RaceResultSwapMotionCySpringBoneDisableData` (1m)
- `Gallop.RaceResultSwapMotionData` (9m)
- `Gallop.RaceSwapMotionData` (7m)
- `Gallop.RaceModelController` (269m)
- `Gallop.MasterDailyLegendRaceDatabase` (9m)
- `Gallop.MasterDailyRaceDatabase` (16m)
- `Gallop.MasterLegendRaceDatabase` (28m)
- `Gallop.MasterRaceDatabase` (171m)
- `Gallop.MasterRatingRaceDatabase` (24m)
- `Gallop.MasterUltimateRaceDatabase` (33m)
- `.RatingRaceData` (0m)
- `Gallop.MasterRatingRaceData` (1m)
- `.SingleModeVenusSpraceData` (0m)
- `Gallop.MasterSingleModeVenusSpraceData` (9m)
- `.MainStoryRaceData` (0m)
- `Gallop.MasterMainStoryRaceData` (4m)
- `.TeamStadiumRaceResultMotion` (0m)
- `Gallop.MasterTeamStadiumRaceResultMotion` (8m)
- `.UltimateRaceData` (0m)
- `Gallop.MasterUltimateRaceData` (3m)
- `Gallop.ObscuredRaceHorseDataRaceResult` (6m)
- `Gallop.ObscuredRaceHorseDataRaceResultExtensions` (3m)
- `Gallop.ObscuredSingleModeBreedersMemberBCRaceResult` (4m)
- `Gallop.ObscuredSingleModeBreedersMemberBCRaceResultExtensions` (3m)
- `Gallop.ObscuredUltimateRaceData` (10m)
- `Gallop.ObscuredUltimateRaceDataExtensions` (3m)

## 十一、事件系统

### 11.1 事件类 (12个)

- `.SingleModeEventChoiceReward` (0m)
- `Gallop.MasterSingleModeEventChoiceReward` (3m)
- `.SingleModeEventConclusion` (0m)
- `Gallop.MasterSingleModeEventConclusion` (5m)
- `.SingleModeEventCrPriority` (2m)
- `Gallop.MasterSingleModeEventCrPriority` (10m)
- `.SingleModeEventItemDetail` (0m)
- `Gallop.MasterSingleModeEventItemDetail` (7m)
- `.SingleModeEventProduction` (1m)
- `Gallop.MasterSingleModeEventProduction` (4m)
- `Gallop.SingleModeEventAccesor` (7m)
- `Gallop.AbstractMiniEventButtonEntity` (13m)

## 十二、ObscuredInt加密

```
ObscuredInt (20 bytes inline):
  +0x00: key        (i32) XOR密钥
  +0x04: hidden     (i32) 加密值 (actual = key ^ hidden)
  +0x08: inited     (i32) 初始化标志
  +0x0c: fake       (i32) 伪装值
  +0x10: fakeActive (i32) 伪装激活

读取: call_getter_obscured_int(class, obj, "get_XXX") 或 value = key ^ hidden
```

### 12.1 Obscured类型 (233种)

- `ObscuredCharaEffectLog`
- `ObscuredCharaEffectLogExtensions`
- `ObscuredCharaProfileData`
- `ObscuredCharaProfileDataExtensions`
- `ObscuredCircleInfoAtFriend`
- `ObscuredCircleInfoAtFriendExtensions`
- `ObscuredCircleUser`
- `ObscuredCircleUserExtensions`
- `ObscuredConstWalkingRelease`
- `ObscuredConstWalkingReleaseExtensions`
- `ObscuredConstWalkingResult`
- `ObscuredConstWalkingResultExtensions`
- `ObscuredDefaultRunningStyle`
- `ObscuredDefaultRunningStyleExtensions`
- `ObscuredDeletedFriendTrainedCharaFavorite`
- `ObscuredDeletedFriendTrainedCharaFavoriteExtensions`
- `ObscuredEffectTypeValue`
- `ObscuredEffectTypeValueExtensions`
- `ObscuredFactorExtend`
- `ObscuredFactorExtendExtensions`
- `ObscuredFactorInfo`
- `ObscuredFactorInfoExtensions`
- `ObscuredFriendTrainedCharaFavorite`
- `ObscuredFriendTrainedCharaFavoriteExtensions`
- `ObscuredGainPartnerSupportEffect`
- `ObscuredGainPartnerSupportEffectExtensions`
- `ObscuredGenerateSuccessionPreset`
- `ObscuredGenerateSuccessionPresetExtensions`
- `ObscuredGenerateSuccessionPriorityFactorInfo`
- `ObscuredGenerateSuccessionPriorityFactorInfoExtensions`
- `ObscuredGenerateSuccessionScenarioFactorInfo`
- `ObscuredGenerateSuccessionScenarioFactorInfoExtensions`
- `ObscuredGenerateSuccessionStartChara`
- `ObscuredGenerateSuccessionStartCharaExtensions`
- `ObscuredHonorData`
- `ObscuredHonorDataExtensions`
- `ObscuredIdleSingleModeEndInfo`
- `ObscuredIdleSingleModeEndInfoExtensions`
- `ObscuredIdleSingleModeGainInfo`
- `ObscuredIdleSingleModeGainInfoExtensions`
- `ObscuredIdleSingleModeLoadInfo`
- `ObscuredIdleSingleModeLoadInfoExtensions`
- `ObscuredIdleSingleModePrioritySkill`
- `ObscuredIdleSingleModePrioritySkillExtensions`
- `ObscuredIdleSingleModeProgressInfo`
- `ObscuredIdleSingleModeProgressInfoExtensions`
- `ObscuredIdleSingleModeProgressLogInfo`
- `ObscuredIdleSingleModeProgressLogInfoExtensions`
- `ObscuredIdleSingleModeSignedInt`
- `ObscuredIdleSingleModeSignedIntExtensions`
- `ObscuredIdleSingleModeStartInfo`
- `ObscuredIdleSingleModeStartInfoExtensions`
- `ObscuredIdleSingleModeSuccessionFactorGainInfo`
- `ObscuredIdleSingleModeSuccessionFactorGainInfoExtensions`
- `ObscuredIdleSingleModeSupportCardGainInfo`
- `ObscuredIdleSingleModeSupportCardGainInfoExtensions`
- `ObscuredLovePointInfo`
- `ObscuredLovePointInfoExtensions`
- `ObscuredNoteDataForDisplay`
- `ObscuredNoteDataForDisplayExtensions`

## 十三、属性变化系统

### 13.1 WorkSingleModeChangeParameterInfo (120m, 63 getters)

  - `get_Speed`
  - `get_Stamina`
  - `get_Power`
  - `get_Guts`
  - `get_Wiz`
  - `get_Hp`
  - `get_Motivation`
  - `get_SkillPoint`
  - `get_FanNum`
  - `get_MaxSpeed`
  - `get_MaxStamina`
  - `get_MaxPower`
  - `get_MaxGuts`
  - `get_MaxWiz`
  - `get_MaxHp`
  - `get_ProperGroundTurf`
  - `get_ProperGroundDirt`
  - `get_ProperDistanceShort`
  - `get_ProperDistanceMile`
  - `get_ProperDistanceMiddle`
  - `get_ProperDistanceLong`
  - `get_ProperRunStyleNige`
  - `get_ProperRunStyleSenko`
  - `get_ProperRunStyleSashi`
  - `get_ProperRunStyleOikomi`
  - `get_CharaGrade`
  - `get_DisableCommandInfoList`
  - `get_DisableCommandExceptHoliday`
  - `get_DisableCommandExceptRace`
  - `get_AddSkillList`
  - `get_ResolveSkillList`
  - `get_AddSkillLevelList`
  - `get_AddSkillMatchBonusList`
  - `get_AddSkillTipsList`
  - `get_CurrentSkillTipsLevelDict`
  - `get_EvaluationChangeList`
  - `get_PrevCharaParamDic`
  - `get_PrevCharaMaxParamDic`
  - `get_AddCharaEffectIdList`
  - `get_RemoveCharaEffectIdList`
  - `get_ParamGuardList`
  - `get_ConditionGuardList`
  - `get_TrainingLevelupList`
  - `get_SuccessionDataList`
  - `get_OutingCharaNameList`
  - `get_OutingGroupNameList`
  - `get_AppearCharaIdList`
  - `get_DeletedRouteRaceId`
  - `get_ForcedRunStyle`
  - `get_GainParameterInfo`
  - `get_LimitStatusTypeList`
  - `get_LimitCharaEffectIdList`
  - `get_HasCharaEffectIdList`
  - `get_InvalidCharaEffectIdList`
  - `get_LimitTipsSkillIdList`
  - `get_LimitGetSkillIdList`
  - `get_LimitLevelSkillIdList`
  - `get_LimitEvaluationCharaIdList`
  - `get_NotUpEvaluationCharaIdList`
  - `get_NotUpEvaluationSupportCardIdList`

### 13.2 各剧本独立属性变化类

**Scenario 2 - TeamRace** (10m, 4 getters):
  - `get_RankingUp`
  - `get_RankingDown`
  - `get_TeamStatusUpDictionary`
  - `get_TotalPower`

**Scenario 3 - Live** (13m, 4 getters):
  - `get_Performance`
  - `get_PerformanceMax`
  - `get_LimitPerformanceTypeList`
  - `get_LiveGetMusicId`

**Scenario 4 - Free** (12m, 4 getters):
  - `get_UseItemIdList`
  - `get_EventShopItemAddList`
  - `get_TscRankingUpDown`
  - `get_CommandInfo`

**Scenario 5 - Venus** (10m, 4 getters):
  - `get_SpiritInfoList`
  - `get_AppearVenus`
  - `get_UsedSpirit`
  - `get_InfoLevelUp`

**Scenario 6 - Arc** (10m, 4 getters):
  - `get_PotentialIdList`
  - `get_GlobalExp`
  - `get_RivalBoostList`
  - `get_IsAllRivalBoostNotUp`

**Scenario 7 - Sport** (6m, 2 getters):
  - `get_GainSportRankDic`
  - `get_UseItemId`

**Scenario 8 - Cook** (32m, 7 getters):
  - `get_CookedDishModel`
  - `get_GainMaterialIconInfo`
  - `get_GainMaterialDictionary`
  - `get_GainCarePoint`
  - `get_GainFriendsPowerPoint`
  - `get_EventGainMaterialNumDictionary`
  - `get_SubCommandCharaInfo`

**Scenario 9 - Mecha** (22m, 9 getters):
  - `get_RivalData`
  - `get_PrevRivalData`
  - `get_NotUpStatusTypeArray`
  - `get_SortedNotUpStatusTypeArray`
  - `get_PrevOverdriveInfo`
  - `get_OverdriveRemainNum`
  - `get_OverdriveNumMaxFlag`
  - `get_OverdriveEnergyNum`
  - `get_TuningPoint`

**Scenario 10 - Legend** (27m, 18 getters):
  - `get_BackupData`
  - `get_ActivatedBuffIds`
  - `get_BuffGaugeNotUpLegendIds`
  - `get_FriendGaugeNotUpPartnerIds`
  - `get_FriendGaugeNotUpAtRandomPartnerIds`
  - `get_FriendGaugeNotDownPartnerIds`
  - `get_FriendLevelNotUpPartnerIds`
  - `get_NotExtendSuperMaxMotivationRemainCount`
  - `get_IsTransitionedZoneEndWaiting`
  - `get_AllFriendGaugeGainInfo`
  - `get_BadConditionGuardInfoList`
  - `get_NotMigraineByMasterly`
  - `get_NotLazyByMasterly`
  - `get_ParamGuardInfo`
  - `get_NotDownMotivationBySuperMaxMotivation`
  - `get_NotUpStatusTypeListOnBuffActivationEffect`
  - `get_NotUpCharaEffectIdListOnBuffActivationEffect`
  - `get_IsAppearLegend`

**Scenario 11 - Pioneer** (14m, 6 getters):
  - `get_PioneerInfo`
  - `get_EvaluationList`
  - `get_PioneerEvaluationInfoList`
  - `get_ShimaTrainingInfo`
  - `get_FacilityInfoList`
  - `get_PlanningInfoList`

**Scenario 12 - Onsen** (15m, 11 getters):
  - `get_BathingInfo`
  - `get_EvaluationInfoArray`
  - `get_AssistantCommandInfo`
  - `get_OnsenInfoArray`
  - `get_CommandInfoArray`
  - `get_DugOnsenIdArray`
  - `get_EffectedOnsenIdArray`
  - `get_DigEffectInfoArray`
  - `get_RyokanRank`
  - `get_BackupDataTurnPeriod`
  - `get_BackupDataDegreeType`

**Scenario 13 - Breeders** (15m, 12 getters):
  - `get_CommandInfoArray`
  - `get_TeamMemberInfoArray`
  - `get_TeamRank`
  - `get_TeamUnionProgress`
  - `get_HavingEnhancePoint`
  - `get_PredictDreamsPoint`
  - `get_TeamSpLevelLimit`
  - `get_TeamReviewResultArray`
  - `get_TeamSpTrainingInfo`
  - `get_EnhanceGroupArray`
  - `get_NotUpExpCharaIdList`
  - `get_IsOverflowTeamSpTrainingStock`

**Scenario 14 - Ramen** (9m, 7 getters):
  - `get_EvaluationInfoArray`
  - `get_CommandInfoArray`
  - `get_FeelingTurnInfoArray`
  - `get_FeelingInfoArray`
  - `get_SpecialFeelingNum`
  - `get_CheckPointPt`
  - `get_UrafEffectInfo`

## 十四、AI评价系统

### 14.1 评价分数表
- 总共 2801 个值 (索引=总修正属性值, 值=评价分数)
- 范围: 0 ~ 14280

### 14.2 基础五维上限
```
BASIC_FIVE_STATUS_LIMIT = [2300, 2200, 1800, 1400, 1400]
// [Speed, Stamina, Power, Guts, Wisdom]
```

### 14.3 各剧本总回合数
```rust
  1 => 78,  // URA
  _ => 72,
```

## 十五、特殊NPC (理事长/记者)

相关类 (74个):

- `Gallop.MiniDirectorCommand` (2m)
- `Gallop.MiniDirector` (52m)
- `Gallop.MiniDirectorBgParam` (6m)
- `Gallop.MiniDirectorCameraParam` (0m)
- `Gallop.CookingMinidirectorCameraParam` (0m)
- `Gallop.CollectRaidMiniDirectorCameraParam` (0m)
- `Gallop.PioneerEndingMiniDirectorCameraParam` (0m)
- `Gallop.MiniDirectorCharaParam` (13m)
- `Gallop.MiniDirectorControllerBase` (6m)
- `Gallop.MiniDirectorDefines` (7m)
- `Gallop.MiniDirectorUIParam` (6m)
- `Gallop.MiniDirectorUtil` (6m)
- `Gallop.MiniDirectorUI` (28m)
- `Gallop.StoryCharaPropController` (39m)
- `.Directory` (0m)
- `Gallop.MasterDirectory` (11m)
- `.DirectoryCardData` (2m)
- `.DirectoryReward` (2m)
- `Gallop.WorkDirectoryData` (23m)
- `Gallop.DialogTrainerDirectory` (6m)