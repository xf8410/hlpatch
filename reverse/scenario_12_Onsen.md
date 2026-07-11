# 剧本 12: Onsen (温泉)

**WorkScenario类**: `WorkSingleModeScenarioOnsen`
**ObscuredDataSet**: `ObscuredSingleModeOnsenDataSet`
---

## 相关类 (216个)

### `Gallop.MasterOmakaseBasePtOnsen` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithCommandIdOrderByIdAsc`
  - `_SelectWithCommandIdOrderByIdAsc`
  - `GetListWithCommandIdOrderByIdAsc`
  - `MaybeListWithCommandIdOrderByIdAsc`
  - `_ListSelectWithCommandIdOrderByIdAsc`
  - `_CreateOrmByQueryResultWithCommandIdOrderByIdAsc`
  - `Unload`

### `Gallop.MasterOmakaseConditionSetOnsen` (6m)
  - `Get`
  - `_SelectOne`
  - `GetWithIdAndConditionType`
  - `_SelectWithIdAndConditionType`
  - `_CreateOrmByQueryResultWithIdAndConditionType`
  - `Unload`

### `Gallop.MasterSingleMode12AssistantEffect` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode12CheckDugResult` (6m)
  - `Get`
  - `_SelectOne`
  - `GetWithScheduleIdAndResultType`
  - `_SelectWithScheduleIdAndResultType`
  - `_CreateOrmByQueryResultWithScheduleIdAndResultType`
  - `Unload`

### `Gallop.MasterSingleMode12DigBonus` (8m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithStratumId`
  - `_SelectWithStratumId`
  - `_CreateOrmByQueryResultWithStratumId`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode12DigItem` (14m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithItemId`
  - `_SelectWithItemId`
  - `GetListWithItemId`
  - `MaybeListWithItemId`
  - `_ListSelectWithItemId`
  - `_CreateOrmByQueryResultWithItemId`
  - `GetWithItemIdAndItemLevel`
  - `_SelectWithItemIdAndItemLevel`
  - `_CreateOrmByQueryResultWithItemIdAndItemLevel`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode12DigPower` (11m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithStratumType`
  - `_SelectWithStratumType`
  - `GetListWithStratumType`
  - `MaybeListWithStratumType`
  - `_ListSelectWithStratumType`
  - `_CreateOrmByQueryResultWithStratumType`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode12DigPowerRank` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode12EffectCategory` (3m)
  - `Get`
  - `_SelectOne`
  - `Unload`

### `Gallop.MasterSingleMode12FactorBonus` (11m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithStratumType`
  - `_SelectWithStratumType`
  - `GetListWithStratumType`
  - `MaybeListWithStratumType`
  - `_ListSelectWithStratumType`
  - `_CreateOrmByQueryResultWithStratumType`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode12Onsen` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode12OnsenEffect` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithGroupId`
  - `_SelectWithGroupId`
  - `GetListWithGroupId`
  - `MaybeListWithGroupId`
  - `_ListSelectWithGroupId`
  - `_CreateOrmByQueryResultWithGroupId`
  - `Unload`

### `Gallop.MasterSingleMode12OutingEffect` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithCharaIdAndStoryStep`
  - `_SelectWithCharaIdAndStoryStep`
  - `GetListWithCharaIdAndStoryStep`
  - `MaybeListWithCharaIdAndStoryStep`
  - `_ListSelectWithCharaIdAndStoryStep`
  - `_CreateOrmByQueryResultWithCharaIdAndStoryStep`
  - `Unload`

### `Gallop.MasterSingleMode12RyokanRank` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode12Schedule` (7m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `get_OrderedSchedule`
  - `GetNextSchedule`

### `Gallop.MasterSingleMode12SpaTicket` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithTypeIdAndCharaId`
  - `_SelectWithTypeIdAndCharaId`
  - `GetListWithTypeIdAndCharaId`
  - `MaybeListWithTypeIdAndCharaId`
  - `_ListSelectWithTypeIdAndCharaId`
  - `_CreateOrmByQueryResultWithTypeIdAndCharaId`
  - `Unload`

### `Gallop.MasterSingleMode12Stratum` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithOnsenId`
  - `_SelectWithOnsenId`
  - `GetListWithOnsenId`
  - `MaybeListWithOnsenId`
  - `_ListSelectWithOnsenId`
  - `_CreateOrmByQueryResultWithOnsenId`
  - `Unload`

### `Gallop.SingleModeScenarioOnsenAssistantTrainingCommandEntity` (11m)
  - `get_DegreeType`
  - `set_DegreeType`
  - `get_CommandType`
  - `get_CommandId`
  - `get_IsEnable`
  - `get_BaseCommandId`
  - `get_TrainingLevel`
  - `get_TrainingFailureRate`
  - `get_TrainingGainParameterList`
  - `set_TrainingGainParameterList`
  - `get_TrainingGainBonusParameterList`

### `Gallop.SingleModeScenarioOnsenTrainingCommandListEntity` (4m)
  - `get_TrainingCommandList`
  - `set_TrainingCommandList`
  - `GetTrainingCommand`
  - `GetOnsenAssistantTraining`

### `Gallop.SingleModeScenarioOnsenTrainingCommandListRepository` (6m)
  - `Get`
  - `GetTrainingGainBonusParameterList`
  - `get_WorkSingleModeScenarioOnsen`
  - `get_OnsenAssistantCommandInfo`
  - `get_TrainingCommandInfoDataList`
  - `get_TrainingLevelInfoArray`

### `Gallop.SingleModeScenarioOnsenTrainingCommandService` (3m)
  - `ExecTraining`
  - `ExecOnsenAssistant`
  - `ExecTrainingBase`

### `Gallop.SingleModeScenarioOnsenTrainingBackGroundService` (2m)
  - `Get`
  - `GetDressIdList`

### `Gallop.WorkAutoPlayOnsenCustomPlanRecord` (4m)
  - `get_PriorityOnsenIdList`
  - `set_PriorityOnsenIdList`
  - `ApplyPlan`
  - `ApplyOnsenPriorityMapSelect`

### `Gallop.WorkAutoPlayOnsenDefaultPlanRecord` (4m)
  - `get_PriorityOnsenIdList`
  - `set_PriorityOnsenIdList`
  - `ApplyPlan`
  - `ApplyOnsenPriorityMapSelect`

### `Gallop.IWorkAutoPlayOnsenPlanRecord` (2m)
  - `get_PriorityOnsenIdList`
  - `ApplyOnsenPriorityMapSelect`

### `Gallop.ObscuredSingleModeOnsenAssistantCommandInfo` (12m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_IsEnable`
  - `set_IsEnable`
  - `get_AssistantPartnerIdArray`
  - `set_AssistantPartnerIdArray`
  - `get_ParamsIncDecInfoArray`
  - `set_ParamsIncDecInfoArray`
  - `get_BonusParamsIncDecInfoArray`
  - `set_BonusParamsIncDecInfoArray`
  - `get_DigInfoArray`
  - `set_DigInfoArray`

### `Gallop.ObscuredSingleModeOnsenAssistantCommandInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenBathingInfo` (6m)
  - `get_TicketNum`
  - `set_TicketNum`
  - `get_OnsenEffectRemainCount`
  - `set_OnsenEffectRemainCount`
  - `get_SuperiorState`
  - `set_SuperiorState`

### `Gallop.ObscuredSingleModeOnsenBathingInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenCheckDugResult` (4m)
  - `get_ScheduleId`
  - `set_ScheduleId`
  - `get_ResultType`
  - `set_ResultType`

### `Gallop.ObscuredSingleModeOnsenCheckDugResultExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenCommandInfo` (8m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_CommandId`
  - `set_CommandId`
  - `get_ParamsIncDecInfoArray`
  - `set_ParamsIncDecInfoArray`
  - `get_DigInfoArray`
  - `set_DigInfoArray`

### `Gallop.ObscuredSingleModeOnsenCommandInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenDataSet` (28m)
  - `get_CommandInfoArray`
  - `set_CommandInfoArray`
  - `get_BathingInfo`
  - `set_BathingInfo`
  - `get_OnsenInfoArray`
  - `set_OnsenInfoArray`
  - `get_DigEffectInfoArray`
  - `set_DigEffectInfoArray`
  - `get_DugOnsenIdArray`
  - `set_DugOnsenIdArray`
  - `get_EffectedOnsenIdArray`
  - `set_EffectedOnsenIdArray`
  - `get_LevelUpDigEffectInfoArray`
  - `set_LevelUpDigEffectInfoArray`
  - `get_EvaluationInfoArray`
  - `set_EvaluationInfoArray`
  - `get_AssistantCommandInfo`
  - `set_AssistantCommandInfo`
  - `get_RyokanRank`
  - `set_RyokanRank`

### `Gallop.ObscuredSingleModeOnsenDataSetExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenDigEffectInfo` (10m)
  - `get_StratumType`
  - `set_StratumType`
  - `get_ItemId`
  - `set_ItemId`
  - `get_ItemLevel`
  - `set_ItemLevel`
  - `get_DigEffectValue`
  - `set_DigEffectValue`
  - `get_IsEnable`
  - `set_IsEnable`

### `Gallop.ObscuredSingleModeOnsenDigEffectInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenDigInfo` (4m)
  - `get_StratumId`
  - `set_StratumId`
  - `get_DigValue`
  - `set_DigValue`

### `Gallop.ObscuredSingleModeOnsenDigInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenEvaluationInfo` (4m)
  - `get_TargetId`
  - `set_TargetId`
  - `get_CharaId`
  - `set_CharaId`

### `Gallop.ObscuredSingleModeOnsenEvaluationInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenInfo` (6m)
  - `get_OnsenId`
  - `set_OnsenId`
  - `get_State`
  - `set_State`
  - `get_StratumInfoArray`
  - `set_StratumInfoArray`

### `Gallop.ObscuredSingleModeOnsenInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenNotUpParameterInfo` (2m)
  - `get_NotGainTicket`
  - `set_NotGainTicket`

### `Gallop.ObscuredSingleModeOnsenNotUpParameterInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenOutingEffect` (4m)
  - `get_CharaId`
  - `set_CharaId`
  - `get_DigInfoArray`
  - `set_DigInfoArray`

### `Gallop.ObscuredSingleModeOnsenOutingEffectExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenStratumInfo` (4m)
  - `get_StratumId`
  - `set_StratumId`
  - `get_RestVolume`
  - `set_RestVolume`

### `Gallop.ObscuredSingleModeOnsenStratumInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.WorkSingleModeChangeParameterInfoScenarioOnsen` (15m)
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
  - `set_BackupDataTurnPeriod`
  - `get_BackupDataDegreeType`
  - `set_BackupDataDegreeType`
  - `Clear`
  - `Set`

### `Gallop.WorkSingleModeScenarioOnsen` (3m)
  - `get_DataSet`
  - `set_DataSet`
  - `Apply`

### `Gallop.WorkSingleModeScenarioOnsenDataSet` (37m)
  - `get_EqualityContract`
  - `get_CommandInfoArray`
  - `set_CommandInfoArray`
  - `get_BathingInfo`
  - `set_BathingInfo`
  - `get_OnsenInfoArray`
  - `set_OnsenInfoArray`
  - `get_DigEffectInfoArray`
  - `set_DigEffectInfoArray`
  - `get_DugOnsenIdArray`
  - `set_DugOnsenIdArray`
  - `get_EffectedOnsenIdArray`
  - `set_EffectedOnsenIdArray`
  - `get_LevelUpDigEffectInfoArray`
  - `set_LevelUpDigEffectInfoArray`
  - `get_EvaluationInfoArray`
  - `set_EvaluationInfoArray`
  - `get_AssistantCommandInfo`
  - `set_AssistantCommandInfo`
  - `get_RyokanRank`

### `Gallop.LiveFlashOnsenController` (6m)
  - `Initialize`
  - `DestroySub`
  - `SetupFont`
  - `DestroyFont`
  - `SetCharacterName`
  - `GetCharacterNameByPosition`

### `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutOnsenEventDispatch` (6m)
  - `ChangeCommand`
  - `ChangeBathingDetail`
  - `ChangeBathingClassicChara`
  - `ChangePRDetail`
  - `ChangeYuamiDetail`
  - `ChangeYuamiFirstChara`

### `Gallop.IPhotoStudioPlayCutSettingsScenarioCutOnsenService` (2m)
  - `CreateVM`
  - `GetPhotoStudioCardId`

### `Gallop.PhotoStudioPlayCutSettingsScenarioCutOnsenService` (14m)
  - `get_ScenarioCutCategory`
  - `get__provider`
  - `CreateCuttPlayInfo`
  - `Reset`
  - `Gallop.IPhotoStudioPlayCutSettingsScenarioCutOnsenService.CreateVM`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCategoryEventDispatch.ChangeCategory`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutOnsenEventDispatch.ChangeCommand`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutOnsenEventDispatch.ChangeBathingDetail`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutOnsenEventDispatch.ChangeBathingClassicChara`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutOnsenEventDispatch.ChangePRDetail`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutOnsenEventDispatch.ChangeYuamiDetail`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutOnsenEventDispatch.ChangeYuamiFirstChara`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutOptionSeasonEventDispatch.ChangeSeason`
  - `GetPhotoStudioCardId`

### `Gallop.PhotoStudioScenarioCutOnsenBathingController` (13m)
  - `RegisterDownload`
  - `Play`
  - `IsPause`
  - `Pause`
  - `Resume`
  - `IsStatusPlaying`
  - `IsStatusEnd`
  - `SkipPause`
  - `get_HasAnyHelper`
  - `CleanUpAll`
  - `AlterUpdate`
  - `AlterLateUpdate`
  - `DestroyHelper`

### `Gallop.PhotoStudioScenarioCutOnsenYuamiController` (13m)
  - `RegisterDownload`
  - `Play`
  - `IsPause`
  - `Pause`
  - `Resume`
  - `IsStatusPlaying`
  - `IsStatusEnd`
  - `SkipPause`
  - `get_HasAnyHelper`
  - `CleanUpAll`
  - `AlterUpdate`
  - `AlterLateUpdate`
  - `DestroyHelper`

### `Gallop.PhotoStudioPlayCutSettingsScenarioCutOnsenPRSeasonProvider` (4m)
  - `get_BgSeason`
  - `set_BgSeason`
  - `get_SelectableSeason`
  - `get_UnSelectableReason`

### `Gallop.PhotoStudioPlayCutSettingsScenarioCutOnsenProvider` (14m)
  - `get_UniqueCommand`
  - `set_UniqueCommand`
  - `get_BathingDetail`
  - `set_BathingDetail`
  - `get_BathingClassicCharaDressSetDataModelAccessor`
  - `get_PRDetail`
  - `set_PRDetail`
  - `get_PRSeasonData`
  - `get_YuamiDetail`
  - `set_YuamiDetail`
  - `get_YuamiFirstCharaDressSetDataModelAccessor`
  - `Reset`
  - `<.ctor>b__34_0`
  - `<.ctor>g__CreateCharaDressSetDataModelAccessor|34_1`


## Obscured加密数据类 (24个)

### `Gallop.ObscuredSingleModeOnsenAssistantCommandInfo` (12m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_IsEnable`
  - `set_IsEnable`
  - `get_AssistantPartnerIdArray`
  - `set_AssistantPartnerIdArray`
  - `get_ParamsIncDecInfoArray`
  - `set_ParamsIncDecInfoArray`
  - `get_BonusParamsIncDecInfoArray`
  - `set_BonusParamsIncDecInfoArray`
  - `get_DigInfoArray`
  - `set_DigInfoArray`

### `Gallop.ObscuredSingleModeOnsenAssistantCommandInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenBathingInfo` (6m)
  - `get_TicketNum`
  - `set_TicketNum`
  - `get_OnsenEffectRemainCount`
  - `set_OnsenEffectRemainCount`
  - `get_SuperiorState`
  - `set_SuperiorState`

### `Gallop.ObscuredSingleModeOnsenBathingInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenCheckDugResult` (4m)
  - `get_ScheduleId`
  - `set_ScheduleId`
  - `get_ResultType`
  - `set_ResultType`

### `Gallop.ObscuredSingleModeOnsenCheckDugResultExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenCommandInfo` (8m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_CommandId`
  - `set_CommandId`
  - `get_ParamsIncDecInfoArray`
  - `set_ParamsIncDecInfoArray`
  - `get_DigInfoArray`
  - `set_DigInfoArray`

### `Gallop.ObscuredSingleModeOnsenCommandInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenDataSet` (28m)
  - `get_CommandInfoArray`
  - `set_CommandInfoArray`
  - `get_BathingInfo`
  - `set_BathingInfo`
  - `get_OnsenInfoArray`
  - `set_OnsenInfoArray`
  - `get_DigEffectInfoArray`
  - `set_DigEffectInfoArray`
  - `get_DugOnsenIdArray`
  - `set_DugOnsenIdArray`
  - `get_EffectedOnsenIdArray`
  - `set_EffectedOnsenIdArray`
  - `get_LevelUpDigEffectInfoArray`
  - `set_LevelUpDigEffectInfoArray`
  - `get_EvaluationInfoArray`

### `Gallop.ObscuredSingleModeOnsenDataSetExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenDigEffectInfo` (10m)
  - `get_StratumType`
  - `set_StratumType`
  - `get_ItemId`
  - `set_ItemId`
  - `get_ItemLevel`
  - `set_ItemLevel`
  - `get_DigEffectValue`
  - `set_DigEffectValue`
  - `get_IsEnable`
  - `set_IsEnable`

### `Gallop.ObscuredSingleModeOnsenDigEffectInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenDigInfo` (4m)
  - `get_StratumId`
  - `set_StratumId`
  - `get_DigValue`
  - `set_DigValue`

### `Gallop.ObscuredSingleModeOnsenDigInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenEvaluationInfo` (4m)
  - `get_TargetId`
  - `set_TargetId`
  - `get_CharaId`
  - `set_CharaId`

### `Gallop.ObscuredSingleModeOnsenEvaluationInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenInfo` (6m)
  - `get_OnsenId`
  - `set_OnsenId`
  - `get_State`
  - `set_State`
  - `get_StratumInfoArray`
  - `set_StratumInfoArray`

### `Gallop.ObscuredSingleModeOnsenInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenNotUpParameterInfo` (2m)
  - `get_NotGainTicket`
  - `set_NotGainTicket`

### `Gallop.ObscuredSingleModeOnsenNotUpParameterInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenOutingEffect` (4m)
  - `get_CharaId`
  - `set_CharaId`
  - `get_DigInfoArray`
  - `set_DigInfoArray`

### `Gallop.ObscuredSingleModeOnsenOutingEffectExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeOnsenStratumInfo` (4m)
  - `get_StratumId`
  - `set_StratumId`
  - `get_RestVolume`
  - `set_RestVolume`

### `Gallop.ObscuredSingleModeOnsenStratumInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`


## Master数据库表 (17个)

| 表名 | 方法数 |
|---|---|
| `MasterOmakaseBasePtOnsen` | 9 |
| `MasterOmakaseConditionSetOnsen` | 6 |
| `MasterSingleMode12AssistantEffect` | 5 |
| `MasterSingleMode12CheckDugResult` | 6 |
| `MasterSingleMode12DigBonus` | 8 |
| `MasterSingleMode12DigItem` | 14 |
| `MasterSingleMode12DigPower` | 11 |
| `MasterSingleMode12DigPowerRank` | 5 |
| `MasterSingleMode12EffectCategory` | 3 |
| `MasterSingleMode12FactorBonus` | 11 |
| `MasterSingleMode12Onsen` | 5 |
| `MasterSingleMode12OnsenEffect` | 9 |
| `MasterSingleMode12OutingEffect` | 9 |
| `MasterSingleMode12RyokanRank` | 5 |
| `MasterSingleMode12Schedule` | 7 |
| `MasterSingleMode12SpaTicket` | 9 |
| `MasterSingleMode12Stratum` | 9 |

## WorkSingleModeScenarioOnsen

方法数: 3

  - `get_DataSet`
  - `set_DataSet`
  - `Apply`

## WorkSingleModeScenarioOnsenDataSet

方法数: 37, 15 getters

  - `get_EqualityContract`
  - `get_CommandInfoArray`
  - `get_BathingInfo`
  - `get_OnsenInfoArray`
  - `get_DigEffectInfoArray`
  - `get_DugOnsenIdArray`
  - `get_EffectedOnsenIdArray`
  - `get_LevelUpDigEffectInfoArray`
  - `get_EvaluationInfoArray`
  - `get_AssistantCommandInfo`
  - `get_RyokanRank`
  - `get_RyokanRankClearState`
  - `get_CheckDugResultArray`
  - `get_NotUpParameterInfo`
  - `get_OutingEffectArray`

## 剧本独立属性变化 (15m, 11 getters)

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

## lib.rs相关引用

```
11=>"Pioneer", 12=>"Onsen", 13=>"Breeders", 14=>"Ramen", _=>"Unknown"
11=>"WorkSingleModeScenarioPioneer", 12=>"WorkSingleModeScenarioOnsen",
12 => "WorkSingleModeScenarioOnsen",
```