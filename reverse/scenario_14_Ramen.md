# 剧本 14: Ramen (ラーメン/トゥインクル・ラーメン杯)

**WorkScenario类**: `WorkSingleModeScenarioRamen`
**ObscuredDataSet**: `ObscuredSingleModeRamenDataSet`
---

## 相关类 (112个)

### `Gallop.MasterSingleMode14BasicEffect` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithCheckPointTypeOrderByIdAsc`
  - `_SelectWithCheckPointTypeOrderByIdAsc`
  - `GetListWithCheckPointTypeOrderByIdAsc`
  - `MaybeListWithCheckPointTypeOrderByIdAsc`
  - `_ListSelectWithCheckPointTypeOrderByIdAsc`
  - `_CreateOrmByQueryResultWithCheckPointTypeOrderByIdAsc`
  - `Unload`

### `Gallop.MasterSingleMode14CheckPoint` (6m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `GetListWithOrderByTurn`

### `Gallop.MasterSingleMode14CheckPointEffect` (7m)
  - `GetWithCheckPointTypeAndResultStateOrderByIdAsc`
  - `_SelectWithCheckPointTypeAndResultStateOrderByIdAsc`
  - `GetListWithCheckPointTypeAndResultStateOrderByIdAsc`
  - `MaybeListWithCheckPointTypeAndResultStateOrderByIdAsc`
  - `_ListSelectWithCheckPointTypeAndResultStateOrderByIdAsc`
  - `_CreateOrmByQueryResultWithCheckPointTypeAndResultStateOrderByIdAsc`
  - `Unload`

### `Gallop.MasterSingleMode14CheckPointPt` (7m)
  - `GetWithCheckPointTypeOrderByIdAsc`
  - `_SelectWithCheckPointTypeOrderByIdAsc`
  - `GetListWithCheckPointTypeOrderByIdAsc`
  - `MaybeListWithCheckPointTypeOrderByIdAsc`
  - `_ListSelectWithCheckPointTypeOrderByIdAsc`
  - `_CreateOrmByQueryResultWithCheckPointTypeOrderByIdAsc`
  - `Unload`

### `Gallop.MasterSingleMode14CheckPointPtEffect` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode14DeckInfo` (1m)
  - `Unload`

### `Gallop.MasterSingleMode14FeelingBonus` (1m)
  - `Unload`

### `Gallop.MasterSingleMode14FinalsEffect` (11m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithSelectTypeOrderByIdAsc`
  - `_SelectWithSelectTypeOrderByIdAsc`
  - `GetListWithSelectTypeOrderByIdAsc`
  - `MaybeListWithSelectTypeOrderByIdAsc`
  - `_ListSelectWithSelectTypeOrderByIdAsc`
  - `_CreateOrmByQueryResultWithSelectTypeOrderByIdAsc`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode14FinalsGainSkill` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode14OutingEffect` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode14RegionEffect` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithRegionIdOrderByIdAsc`
  - `_SelectWithRegionIdOrderByIdAsc`
  - `GetListWithRegionIdOrderByIdAsc`
  - `MaybeListWithRegionIdOrderByIdAsc`
  - `_ListSelectWithRegionIdOrderByIdAsc`
  - `_CreateOrmByQueryResultWithRegionIdOrderByIdAsc`
  - `Unload`

### `Gallop.MasterSingleMode14RegionEffectBonus` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode14RegionFeeling` (20m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithRegionIdOrderByIdAsc`
  - `_SelectWithRegionIdOrderByIdAsc`
  - `GetListWithRegionIdOrderByIdAsc`
  - `MaybeListWithRegionIdOrderByIdAsc`
  - `_ListSelectWithRegionIdOrderByIdAsc`
  - `_CreateOrmByQueryResultWithRegionIdOrderByIdAsc`
  - `GetWithRegionSelectTypeOrderByIdAsc`
  - `_SelectWithRegionSelectTypeOrderByIdAsc`
  - `GetListWithRegionSelectTypeOrderByIdAsc`
  - `MaybeListWithRegionSelectTypeOrderByIdAsc`
  - `_ListSelectWithRegionSelectTypeOrderByIdAsc`
  - `_CreateOrmByQueryResultWithRegionSelectTypeOrderByIdAsc`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `GetListWithRegionIdAndRegionSelectTypeOrderByIdAsc`
  - `GetRegionIdListWithRegionSelectTypeOrderByIdAsc`
  - `GetAllRegionIdAndRegionSelectTypeListWithOrderByRegionSelectTypeThenByRegionIdAsc`

### `Gallop.MasterSingleMode14RegionSelect` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode14SpecialGainTurn` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode14TwinkleRamen` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.IWorkAutoPlayRamenPlanRecord` (8m)
  - `get_ModifyPreferenceJuniorRegionIdList`
  - `get_ModifyPreferenceClassicRegionIdList`
  - `get_ModifyPreferenceSeniorRegionIdList`
  - `get_ModifyPreferenceSelectUrafEffectType`
  - `ApplyModifyPreferenceJuniorRegionIdList`
  - `ApplyModifyPreferenceClassicRegionIdList`
  - `ApplyModifyPreferenceSeniorRegionIdList`
  - `ApplyModifyPreferenceSelectUrafEffectType`

### `Gallop.WorkAutoPlayRamenCustomPlanRecord` (9m)
  - `get_ModifyPreferenceJuniorRegionIdList`
  - `get_ModifyPreferenceClassicRegionIdList`
  - `get_ModifyPreferenceSeniorRegionIdList`
  - `get_ModifyPreferenceSelectUrafEffectType`
  - `ApplyPlan`
  - `ApplyModifyPreferenceJuniorRegionIdList`
  - `ApplyModifyPreferenceClassicRegionIdList`
  - `ApplyModifyPreferenceSeniorRegionIdList`
  - `ApplyModifyPreferenceSelectUrafEffectType`

### `Gallop.WorkAutoPlayRamenDefaultPlanRecord` (9m)
  - `get_ModifyPreferenceJuniorRegionIdList`
  - `get_ModifyPreferenceClassicRegionIdList`
  - `get_ModifyPreferenceSeniorRegionIdList`
  - `get_ModifyPreferenceSelectUrafEffectType`
  - `ApplyPlan`
  - `ApplyModifyPreferenceJuniorRegionIdList`
  - `ApplyModifyPreferenceClassicRegionIdList`
  - `ApplyModifyPreferenceSeniorRegionIdList`
  - `ApplyModifyPreferenceSelectUrafEffectType`

### `Gallop.ObscuredSingleModeRamenActiveEffectInfo` (6m)
  - `get_EffectCategory`
  - `set_EffectCategory`
  - `get_EffectId`
  - `set_EffectId`
  - `get_EffectValue`
  - `set_EffectValue`

### `Gallop.ObscuredSingleModeRamenActiveEffectInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenAutoSelectInfo` (8m)
  - `get_IsAutoSelect`
  - `set_IsAutoSelect`
  - `get_JuniorRegionSet`
  - `set_JuniorRegionSet`
  - `get_ClassicRegionSet`
  - `set_ClassicRegionSet`
  - `get_SeniorRegionSet`
  - `set_SeniorRegionSet`

### `Gallop.ObscuredSingleModeRamenAutoSelectInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenAutoSelectSetInfo` (10m)
  - `get_SetId`
  - `set_SetId`
  - `get_SetName`
  - `set_SetName`
  - `get_JuniorRegionSet`
  - `set_JuniorRegionSet`
  - `get_ClassicRegionSet`
  - `set_ClassicRegionSet`
  - `get_SeniorRegionSet`
  - `set_SeniorRegionSet`

### `Gallop.ObscuredSingleModeRamenAutoSelectSetInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenCheckPointInfo` (4m)
  - `get_CheckPointType`
  - `set_CheckPointType`
  - `get_ResultState`
  - `set_ResultState`

### `Gallop.ObscuredSingleModeRamenCheckPointInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenCommandFeelingInfo` (6m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_CommandId`
  - `set_CommandId`
  - `get_FeelingId`
  - `set_FeelingId`

### `Gallop.ObscuredSingleModeRamenCommandFeelingInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenCommandInfo` (6m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_CommandId`
  - `set_CommandId`
  - `get_ParamsIncDecInfoArray`
  - `set_ParamsIncDecInfoArray`

### `Gallop.ObscuredSingleModeRamenCommandInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenDataSet` (26m)
  - `get_CommandInfoArray`
  - `set_CommandInfoArray`
  - `get_EvaluationInfoArray`
  - `set_EvaluationInfoArray`
  - `get_FeelingReduceTurnInfoArray`
  - `set_FeelingReduceTurnInfoArray`
  - `get_FeelingTurnInfoArray`
  - `set_FeelingTurnInfoArray`
  - `get_FeelingInfoArray`
  - `set_FeelingInfoArray`
  - `get_SpecialFeelingNum`
  - `set_SpecialFeelingNum`
  - `get_NotUpParameterInfo`
  - `set_NotUpParameterInfo`
  - `get_ActiveEffectArray`
  - `set_ActiveEffectArray`
  - `get_UrafEffectInfo`
  - `set_UrafEffectInfo`
  - `get_CommandFeelingInfoArray`
  - `set_CommandFeelingInfoArray`

### `Gallop.ObscuredSingleModeRamenDataSetExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenDataSetCheckEvent` (2m)
  - `get_IsGaugeGained`
  - `set_IsGaugeGained`

### `Gallop.ObscuredSingleModeRamenDataSetCheckEventExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenDataSetLoad` (20m)
  - `get_AutoSelectInfo`
  - `set_AutoSelectInfo`
  - `get_AutoSelectSetInfo`
  - `set_AutoSelectSetInfo`
  - `get_SelectedRegionIdArray`
  - `set_SelectedRegionIdArray`
  - `get_ReduceBaseTurnInfoArray`
  - `set_ReduceBaseTurnInfoArray`
  - `get_CheckPointInfoArray`
  - `set_CheckPointInfoArray`
  - `get_LastTastingInfo`
  - `set_LastTastingInfo`
  - `get_CheckPointPt`
  - `set_CheckPointPt`
  - `get_ExpectedCheckPointPt`
  - `set_ExpectedCheckPointPt`
  - `get_UsedTwinkleTextIdArray`
  - `set_UsedTwinkleTextIdArray`
  - `get_IsCheckedUrafEvent`
  - `set_IsCheckedUrafEvent`

### `Gallop.ObscuredSingleModeRamenDataSetLoadExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenDataSetStart` (6m)
  - `get_AutoSelectInfo`
  - `set_AutoSelectInfo`
  - `get_AutoSelectSetInfo`
  - `set_AutoSelectSetInfo`
  - `get_IsCheckedUrafEvent`
  - `set_IsCheckedUrafEvent`

### `Gallop.ObscuredSingleModeRamenDataSetStartExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenEvaluationInfo` (4m)
  - `get_TargetId`
  - `set_TargetId`
  - `get_CharaId`
  - `set_CharaId`

### `Gallop.ObscuredSingleModeRamenEvaluationInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenFeeling` (4m)
  - `get_FeelingIndex`
  - `set_FeelingIndex`
  - `get_FeelingId`
  - `set_FeelingId`

### `Gallop.ObscuredSingleModeRamenFeelingExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenFeelingReduceTurnInfo` (6m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_CommandId`
  - `set_CommandId`
  - `get_FeelingTurnArray`
  - `set_FeelingTurnArray`

### `Gallop.ObscuredSingleModeRamenFeelingReduceTurnInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenFeelingTurnInfo` (4m)
  - `get_FeelingId`
  - `set_FeelingId`
  - `get_RemainTurn`
  - `set_RemainTurn`

### `Gallop.ObscuredSingleModeRamenFeelingTurnInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenLastTastingInfo` (10m)
  - `get_FeelingId1Num`
  - `set_FeelingId1Num`
  - `get_FeelingId2Num`
  - `set_FeelingId2Num`
  - `get_FeelingId3Num`
  - `set_FeelingId3Num`
  - `get_SpecialFeelingNum`
  - `set_SpecialFeelingNum`
  - `get_RegionId`
  - `set_RegionId`

### `Gallop.ObscuredSingleModeRamenLastTastingInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenNotUpParameterInfo` (2m)
  - `get_NotGainSpecialFeeling`
  - `set_NotGainSpecialFeeling`

### `Gallop.ObscuredSingleModeRamenNotUpParameterInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenReduceBaseTurnInfo` (4m)
  - `get_FeelingId`
  - `set_FeelingId`
  - `get_ReduceBaseTurn`
  - `set_ReduceBaseTurn`

### `Gallop.ObscuredSingleModeRamenReduceBaseTurnInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenReduceFeelingTurn` (4m)
  - `get_FeelingId`
  - `set_FeelingId`
  - `get_Turn`
  - `set_Turn`

### `Gallop.ObscuredSingleModeRamenReduceFeelingTurnExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenTrainingExecInfo` (4m)
  - `get_BaseCommandId`
  - `set_BaseCommandId`
  - `get_ExecCount`
  - `set_ExecCount`

### `Gallop.ObscuredSingleModeRamenTrainingExecInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenUrafEffectInfo` (4m)
  - `get_UrafEffectType`
  - `set_UrafEffectType`
  - `get_UrafEffectState`
  - `set_UrafEffectState`

### `Gallop.ObscuredSingleModeRamenUrafEffectInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.WorkSingleModeChangeParameterInfoScenarioRamen` (9m)
  - `get_EvaluationInfoArray`
  - `get_CommandInfoArray`
  - `get_FeelingTurnInfoArray`
  - `get_FeelingInfoArray`
  - `get_SpecialFeelingNum`
  - `get_CheckPointPt`
  - `get_UrafEffectInfo`
  - `Clear`
  - `Set`


## Obscured加密数据类 (40个)

### `Gallop.ObscuredSingleModeRamenActiveEffectInfo` (6m)
  - `get_EffectCategory`
  - `set_EffectCategory`
  - `get_EffectId`
  - `set_EffectId`
  - `get_EffectValue`
  - `set_EffectValue`

### `Gallop.ObscuredSingleModeRamenActiveEffectInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenAutoSelectInfo` (8m)
  - `get_IsAutoSelect`
  - `set_IsAutoSelect`
  - `get_JuniorRegionSet`
  - `set_JuniorRegionSet`
  - `get_ClassicRegionSet`
  - `set_ClassicRegionSet`
  - `get_SeniorRegionSet`
  - `set_SeniorRegionSet`

### `Gallop.ObscuredSingleModeRamenAutoSelectInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenAutoSelectSetInfo` (10m)
  - `get_SetId`
  - `set_SetId`
  - `get_SetName`
  - `set_SetName`
  - `get_JuniorRegionSet`
  - `set_JuniorRegionSet`
  - `get_ClassicRegionSet`
  - `set_ClassicRegionSet`
  - `get_SeniorRegionSet`
  - `set_SeniorRegionSet`

### `Gallop.ObscuredSingleModeRamenAutoSelectSetInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenCheckPointInfo` (4m)
  - `get_CheckPointType`
  - `set_CheckPointType`
  - `get_ResultState`
  - `set_ResultState`

### `Gallop.ObscuredSingleModeRamenCheckPointInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenCommandFeelingInfo` (6m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_CommandId`
  - `set_CommandId`
  - `get_FeelingId`
  - `set_FeelingId`

### `Gallop.ObscuredSingleModeRamenCommandFeelingInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenCommandInfo` (6m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_CommandId`
  - `set_CommandId`
  - `get_ParamsIncDecInfoArray`
  - `set_ParamsIncDecInfoArray`

### `Gallop.ObscuredSingleModeRamenCommandInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenDataSet` (26m)
  - `get_CommandInfoArray`
  - `set_CommandInfoArray`
  - `get_EvaluationInfoArray`
  - `set_EvaluationInfoArray`
  - `get_FeelingReduceTurnInfoArray`
  - `set_FeelingReduceTurnInfoArray`
  - `get_FeelingTurnInfoArray`
  - `set_FeelingTurnInfoArray`
  - `get_FeelingInfoArray`
  - `set_FeelingInfoArray`
  - `get_SpecialFeelingNum`
  - `set_SpecialFeelingNum`
  - `get_NotUpParameterInfo`
  - `set_NotUpParameterInfo`
  - `get_ActiveEffectArray`

### `Gallop.ObscuredSingleModeRamenDataSetExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenDataSetCheckEvent` (2m)
  - `get_IsGaugeGained`
  - `set_IsGaugeGained`

### `Gallop.ObscuredSingleModeRamenDataSetCheckEventExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenDataSetLoad` (20m)
  - `get_AutoSelectInfo`
  - `set_AutoSelectInfo`
  - `get_AutoSelectSetInfo`
  - `set_AutoSelectSetInfo`
  - `get_SelectedRegionIdArray`
  - `set_SelectedRegionIdArray`
  - `get_ReduceBaseTurnInfoArray`
  - `set_ReduceBaseTurnInfoArray`
  - `get_CheckPointInfoArray`
  - `set_CheckPointInfoArray`
  - `get_LastTastingInfo`
  - `set_LastTastingInfo`
  - `get_CheckPointPt`
  - `set_CheckPointPt`
  - `get_ExpectedCheckPointPt`

### `Gallop.ObscuredSingleModeRamenDataSetLoadExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenDataSetStart` (6m)
  - `get_AutoSelectInfo`
  - `set_AutoSelectInfo`
  - `get_AutoSelectSetInfo`
  - `set_AutoSelectSetInfo`
  - `get_IsCheckedUrafEvent`
  - `set_IsCheckedUrafEvent`

### `Gallop.ObscuredSingleModeRamenDataSetStartExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenEvaluationInfo` (4m)
  - `get_TargetId`
  - `set_TargetId`
  - `get_CharaId`
  - `set_CharaId`

### `Gallop.ObscuredSingleModeRamenEvaluationInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenFeeling` (4m)
  - `get_FeelingIndex`
  - `set_FeelingIndex`
  - `get_FeelingId`
  - `set_FeelingId`

### `Gallop.ObscuredSingleModeRamenFeelingExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenFeelingReduceTurnInfo` (6m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_CommandId`
  - `set_CommandId`
  - `get_FeelingTurnArray`
  - `set_FeelingTurnArray`

### `Gallop.ObscuredSingleModeRamenFeelingReduceTurnInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenFeelingTurnInfo` (4m)
  - `get_FeelingId`
  - `set_FeelingId`
  - `get_RemainTurn`
  - `set_RemainTurn`

### `Gallop.ObscuredSingleModeRamenFeelingTurnInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenLastTastingInfo` (10m)
  - `get_FeelingId1Num`
  - `set_FeelingId1Num`
  - `get_FeelingId2Num`
  - `set_FeelingId2Num`
  - `get_FeelingId3Num`
  - `set_FeelingId3Num`
  - `get_SpecialFeelingNum`
  - `set_SpecialFeelingNum`
  - `get_RegionId`
  - `set_RegionId`

### `Gallop.ObscuredSingleModeRamenLastTastingInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenNotUpParameterInfo` (2m)
  - `get_NotGainSpecialFeeling`
  - `set_NotGainSpecialFeeling`

### `Gallop.ObscuredSingleModeRamenNotUpParameterInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenReduceBaseTurnInfo` (4m)
  - `get_FeelingId`
  - `set_FeelingId`
  - `get_ReduceBaseTurn`
  - `set_ReduceBaseTurn`

### `Gallop.ObscuredSingleModeRamenReduceBaseTurnInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenReduceFeelingTurn` (4m)
  - `get_FeelingId`
  - `set_FeelingId`
  - `get_Turn`
  - `set_Turn`

### `Gallop.ObscuredSingleModeRamenReduceFeelingTurnExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenTrainingExecInfo` (4m)
  - `get_BaseCommandId`
  - `set_BaseCommandId`
  - `get_ExecCount`
  - `set_ExecCount`

### `Gallop.ObscuredSingleModeRamenTrainingExecInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeRamenUrafEffectInfo` (4m)
  - `get_UrafEffectType`
  - `set_UrafEffectType`
  - `get_UrafEffectState`
  - `set_UrafEffectState`

### `Gallop.ObscuredSingleModeRamenUrafEffectInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`


## Master数据库表 (16个)

| 表名 | 方法数 |
|---|---|
| `MasterSingleMode14BasicEffect` | 9 |
| `MasterSingleMode14CheckPoint` | 6 |
| `MasterSingleMode14CheckPointEffect` | 7 |
| `MasterSingleMode14CheckPointPt` | 7 |
| `MasterSingleMode14CheckPointPtEffect` | 5 |
| `MasterSingleMode14DeckInfo` | 1 |
| `MasterSingleMode14FeelingBonus` | 1 |
| `MasterSingleMode14FinalsEffect` | 11 |
| `MasterSingleMode14FinalsGainSkill` | 5 |
| `MasterSingleMode14OutingEffect` | 5 |
| `MasterSingleMode14RegionEffect` | 9 |
| `MasterSingleMode14RegionEffectBonus` | 5 |
| `MasterSingleMode14RegionFeeling` | 20 |
| `MasterSingleMode14RegionSelect` | 5 |
| `MasterSingleMode14SpecialGainTurn` | 5 |
| `MasterSingleMode14TwinkleRamen` | 5 |

## WorkSingleModeScenarioRamen

方法数: 19

  - `get_DataSet`
  - `set_DataSet`
  - `GetOrCreate`
  - `ApplyDataSetCommon`
  - `ApplyRamenDataSetStart`
  - `ApplyRamenDataSetLoad`
  - `ApplyRamenDataSetCheckEvent`
  - `ApplyIsGaugeGained`
  - `ApplyRamenTasting`
  - `ApplySelectRegion`
  - `ApplySelectedRegionIdArray`
  - `ApplyCheckPoint`
  - `ApplyUsedTwinkleTextIdArray`
  - `ApplyAutoRegionSelectInfo`
  - `ApplyAutoRegionSelectSetInfo`
  - `ApplyAutoRegionSelectSet`
  - `ApplyAutoRegionSelectSetName`
  - `ApplyUrafEffectInfoBySelf`
  - `ApplyUrafEffectSelectEventChecked`

## 剧本独立属性变化 (9m, 7 getters)

  - `get_EvaluationInfoArray`
  - `get_CommandInfoArray`
  - `get_FeelingTurnInfoArray`
  - `get_FeelingInfoArray`
  - `get_SpecialFeelingNum`
  - `get_CheckPointPt`
  - `get_UrafEffectInfo`

## lib.rs相关引用

```
11=>"Pioneer", 12=>"Onsen", 13=>"Breeders", 14=>"Ramen", _=>"Unknown"
13=>"WorkSingleModeScenarioBreeders", 14=>"WorkSingleModeScenarioRamen",
14 => "WorkSingleModeScenarioRamen",
```