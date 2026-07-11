# 剧本 13: Breeders (ブリーダーズ/種田杯)

**WorkScenario类**: `WorkSingleModeScenarioBreeders`
**ObscuredDataSet**: `ObscuredSingleModeBreedersDataSet`
---

## 相关类 (228个)

### `Gallop.MasterSingleMode13AddDreamPoint` (8m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithDeckIdAndConditionTypeAndConditionValue`
  - `_SelectWithDeckIdAndConditionTypeAndConditionValue`
  - `_CreateOrmByQueryResultWithDeckIdAndConditionTypeAndConditionValue`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode13BcProgramFlag` (17m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithProgramId`
  - `_SelectWithProgramId`
  - `_CreateOrmByQueryResultWithProgramId`
  - `GetWithBcGroupId`
  - `_SelectWithBcGroupId`
  - `GetListWithBcGroupId`
  - `MaybeListWithBcGroupId`
  - `_ListSelectWithBcGroupId`
  - `_CreateOrmByQueryResultWithBcGroupId`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `GetWithBcGroupIdAndRaceTrackId`
  - `GetListWithRaceTrackIdOrderByIdAsc`
  - `GetRaceInstanceIdListByBcGroupId`

### `Gallop.MasterSingleMode13Member` (8m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithCharaId`
  - `_SelectWithCharaId`
  - `_CreateOrmByQueryResultWithCharaId`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode13Rank` (8m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithRankId`
  - `_SelectWithRankId`
  - `_CreateOrmByQueryResultWithRankId`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode13RankBonusEffectGroup` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithGroupIdOrderByIdAsc`
  - `_SelectWithGroupIdOrderByIdAsc`
  - `GetListWithGroupIdOrderByIdAsc`
  - `MaybeListWithGroupIdOrderByIdAsc`
  - `_ListSelectWithGroupIdOrderByIdAsc`
  - `_CreateOrmByQueryResultWithGroupIdOrderByIdAsc`
  - `Unload`

### `Gallop.MasterSingleMode13Schedule` (10m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithMeetingTurn`
  - `_SelectWithMeetingTurn`
  - `_CreateOrmByQueryResultWithMeetingTurn`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `get_OrderedReviewSchedule`
  - `GetNextReviewSchedule`

### `Gallop.MasterSingleMode13TeamRank` (12m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithRankGroupOrderByIdAsc`
  - `_SelectWithRankGroupOrderByIdAsc`
  - `GetListWithRankGroupOrderByIdAsc`
  - `MaybeListWithRankGroupOrderByIdAsc`
  - `_ListSelectWithRankGroupOrderByIdAsc`
  - `_CreateOrmByQueryResultWithRankGroupOrderByIdAsc`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `get_GroupedRecordListByRank`

### `Gallop.MasterSingleMode13TeamSpEffect` (20m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithGroupTypeOrderByIdAsc`
  - `_SelectWithGroupTypeOrderByIdAsc`
  - `GetListWithGroupTypeOrderByIdAsc`
  - `MaybeListWithGroupTypeOrderByIdAsc`
  - `_ListSelectWithGroupTypeOrderByIdAsc`
  - `_CreateOrmByQueryResultWithGroupTypeOrderByIdAsc`
  - `GetWithGroupTypeAndEffectGroupOrderByIdAsc`
  - `_SelectWithGroupTypeAndEffectGroupOrderByIdAsc`
  - `GetListWithGroupTypeAndEffectGroupOrderByIdAsc`
  - `MaybeListWithGroupTypeAndEffectGroupOrderByIdAsc`
  - `_ListSelectWithGroupTypeAndEffectGroupOrderByIdAsc`
  - `_CreateOrmByQueryResultWithGroupTypeAndEffectGroupOrderByIdAsc`
  - `GetWithEffectGroup`
  - `_SelectWithEffectGroup`
  - `_CreateOrmByQueryResultWithEffectGroup`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleMode13TeamSpLevel` (12m)
  - `Get`
  - `_SelectOne`
  - `GetWithGroupTypeAndLevel`
  - `_SelectWithGroupTypeAndLevel`
  - `_CreateOrmByQueryResultWithGroupTypeAndLevel`
  - `GetWithGroupTypeOrderByLevelAsc`
  - `_SelectWithGroupTypeOrderByLevelAsc`
  - `GetListWithGroupTypeOrderByLevelAsc`
  - `MaybeListWithGroupTypeOrderByLevelAsc`
  - `_ListSelectWithGroupTypeOrderByLevelAsc`
  - `_CreateOrmByQueryResultWithGroupTypeOrderByLevelAsc`
  - `Unload`

### `Gallop.MasterSingleMode13TopBgChara` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.WorkAutoPlayBreedersCustomPlanRecord` (4m)
  - `get_PriorityEnhanceGroupLevelIdList`
  - `set_PriorityEnhanceGroupLevelIdList`
  - `ApplyPlan`
  - `ApplyBreedersPriorityEnhanceGroupLevelIdSelect`

### `Gallop.WorkAutoPlayBreedersDefaultPlanRecord` (4m)
  - `get_PriorityEnhanceGroupLevelIdList`
  - `set_PriorityEnhanceGroupLevelIdList`
  - `ApplyPlan`
  - `ApplyBreedersPriorityEnhanceGroupLevelIdSelect`

### `Gallop.IWorkAutoPlayBreedersPlanRecord` (2m)
  - `get_PriorityEnhanceGroupLevelIdList`
  - `ApplyBreedersPriorityEnhanceGroupLevelIdSelect`

### `Gallop.ObscuredSingleModeBreedersCommandGainExp` (4m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_GainExp`
  - `set_GainExp`

### `Gallop.ObscuredSingleModeBreedersCommandGainExpExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersCommandInfo` (10m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_CommandId`
  - `set_CommandId`
  - `get_ParamsIncDecInfoArray`
  - `set_ParamsIncDecInfoArray`
  - `get_TeamMemberInfoArray`
  - `set_TeamMemberInfoArray`
  - `get_RankUpPredict`
  - `set_RankUpPredict`

### `Gallop.ObscuredSingleModeBreedersCommandInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersCommandTeamMemberInfo` (4m)
  - `get_CharaId`
  - `set_CharaId`
  - `get_GainExp`
  - `set_GainExp`

### `Gallop.ObscuredSingleModeBreedersCommandTeamMemberInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersDataSet` (30m)
  - `get_CommandInfoArray`
  - `set_CommandInfoArray`
  - `get_TeamMemberInfoArray`
  - `set_TeamMemberInfoArray`
  - `get_TeamSpTrainingInfo`
  - `set_TeamSpTrainingInfo`
  - `get_NotUpParameterInfo`
  - `set_NotUpParameterInfo`
  - `get_BcRaceResultArray`
  - `set_BcRaceResultArray`
  - `get_TeamUnionProgress`
  - `set_TeamUnionProgress`
  - `get_BcRaceTrackId`
  - `set_BcRaceTrackId`
  - `get_TeamRank`
  - `set_TeamRank`
  - `get_HavingEnhancePoint`
  - `set_HavingEnhancePoint`
  - `get_PredictEnhancePoint`
  - `set_PredictEnhancePoint`

### `Gallop.ObscuredSingleModeBreedersDataSetExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersDataSetLoad` (8m)
  - `get_LastSelectBcGroupId`
  - `set_LastSelectBcGroupId`
  - `get_DeckId`
  - `set_DeckId`
  - `get_TeamReviewResultArray`
  - `set_TeamReviewResultArray`
  - `get_EnhanceGroupArray`
  - `set_EnhanceGroupArray`

### `Gallop.ObscuredSingleModeBreedersDataSetLoadExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersDataSetStart` (8m)
  - `get_LastSelectBcGroupId`
  - `set_LastSelectBcGroupId`
  - `get_DeckId`
  - `set_DeckId`
  - `get_TeamReviewResultArray`
  - `set_TeamReviewResultArray`
  - `get_EnhanceGroupArray`
  - `set_EnhanceGroupArray`

### `Gallop.ObscuredSingleModeBreedersDataSetStartExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersDataSetTeamMeeting` (2m)
  - `get_EnhanceGroupArray`
  - `set_EnhanceGroupArray`

### `Gallop.ObscuredSingleModeBreedersDataSetTeamMeetingExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersDataSetTeamReview` (2m)
  - `get_TeamReviewResultArray`
  - `set_TeamReviewResultArray`

### `Gallop.ObscuredSingleModeBreedersDataSetTeamReviewExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersEnhanceGroup` (4m)
  - `get_GroupType`
  - `set_GroupType`
  - `get_Level`
  - `set_Level`

### `Gallop.ObscuredSingleModeBreedersEnhanceGroupExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersMemberBCRaceResult` (4m)
  - `get_BcGroupId`
  - `set_BcGroupId`
  - `get_WinMemberId`
  - `set_WinMemberId`

### `Gallop.ObscuredSingleModeBreedersMemberBCRaceResultExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersNotUpParameterInfo` (4m)
  - `get_NotUpExpCharaIdArray`
  - `set_NotUpExpCharaIdArray`
  - `get_IsOverflowStock`
  - `set_IsOverflowStock`

### `Gallop.ObscuredSingleModeBreedersNotUpParameterInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersTeamMemberInfo` (8m)
  - `get_MemberId`
  - `set_MemberId`
  - `get_CharaId`
  - `set_CharaId`
  - `get_Rank`
  - `set_Rank`
  - `get_Exp`
  - `set_Exp`

### `Gallop.ObscuredSingleModeBreedersTeamMemberInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersTeamReviewResult` (4m)
  - `get_ScheduleId`
  - `set_ScheduleId`
  - `get_ResultType`
  - `set_ResultType`

### `Gallop.ObscuredSingleModeBreedersTeamReviewResultExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersTeamSpTrainingInfo` (6m)
  - `get_StockNum`
  - `set_StockNum`
  - `get_StockMax`
  - `set_StockMax`
  - `get_ActivatedState`
  - `set_ActivatedState`

### `Gallop.ObscuredSingleModeBreedersTeamSpTrainingInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.WorkSingleModeChangeParameterInfoScenarioBreeders` (15m)
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
  - `set_IsOverflowTeamSpTrainingStock`
  - `Clear`
  - `Set`

### `Gallop.WorkSingleModeScenarioBreeders` (7m)
  - `get_DataSet`
  - `set_DataSet`
  - `ApplyDataSetCommon`
  - `ApplyDataSetStart`
  - `ApplyDataSetLoad`
  - `ApplyDataSetTeamReview`
  - `ApplyDataSetTeamMeeting`

### `Gallop.WorkSingleModeScenarioBreedersDataSet` (45m)
  - `get_EqualityContract`
  - `get_CommandInfoArray`
  - `set_CommandInfoArray`
  - `get_TeamMemberInfoArray`
  - `set_TeamMemberInfoArray`
  - `get_TeamSpTrainingInfo`
  - `set_TeamSpTrainingInfo`
  - `get_TeamReviewResultArray`
  - `set_TeamReviewResultArray`
  - `get_NotUpParameterInfo`
  - `set_NotUpParameterInfo`
  - `get_BcRaceResultArray`
  - `set_BcRaceResultArray`
  - `get_TeamUnionProgress`
  - `set_TeamUnionProgress`
  - `get_BcRaceTrackId`
  - `set_BcRaceTrackId`
  - `get_EnhanceGroupArray`
  - `set_EnhanceGroupArray`
  - `get_TeamRank`

### `Gallop.DialogGenerateSuccessionCharaBreedersRaceSelect` (10m)
  - `GetFormType`
  - `GetParentType`
  - `PushDialog`
  - `Setup`
  - `GetProperGradeByGroundType`
  - `GetProperGradeByDistanceType`
  - `OnDecide`
  - `OnDecideBCRaceRouteInfo`
  - `OnCancelBCRaceRouteInfo`
  - `OnCancel`

### `Gallop.PhotoStudioPlayCutSettingsScenarioCutBreedersService` (16m)
  - `get_ScenarioCutCategory`
  - `Reset`
  - `CreateCuttPlayInfo`
  - `CreateVM`
  - `GetPhotoStudioCharId`
  - `ChangeCategory`
  - `ChangeCommand`
  - `ChangeTrainingPlace`
  - `ChangeTrainingSeason`
  - `ChangeReviewSeason`
  - `ChangeEverydaySeason`
  - `ChangeChara`
  - `ChangeEverydayDetail`
  - `ChangeRaceDetail`
  - `ChangeRaceCourse`
  - `ChangeRaceId`

### `Gallop.PhotoStudioScenarioCutBreedersController` (15m)
  - `Play`
  - `Download`
  - `GetCutPath`
  - `GetCutPathWhenEveryday`
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

### `Gallop.PhotoStudioScenarioCutBreedersCharaInfoListContext` (1m)
  - `GetCharaInfoList`

### `Gallop.PhotoStudioScenarioCutBreedersOverRunController` (21m)
  - `get_BreedersCutPlayExtraInfo`
  - `get_CutPlayExtraInfoTrackId`
  - `get_CutPlayExtraInfoRaceId`
  - `GetCutPlayExtraInfoGroundType`
  - `GetCutPlayExtraInfoRaceGoalGate`
  - `GetCutPlayExtraInfoRaceGoalFlower`
  - `GetCutPlayExtraInfoMasterRace`
  - `GetCutPlayExtraInfoTrackFlagType`
  - `Play`
  - `Download`
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

### `Gallop.PhotoStudioScenarioCutBreedersReviewController` (20m)
  - `get_TopCutPath`
  - `get_MainCutPath`
  - `Play`
  - `Download`
  - `PlayNextMainCut`
  - `FadeOut`
  - `FadeIn`
  - `IsPause`
  - `Pause`
  - `Resume`
  - `IsStatusPlaying`
  - `IsStatusEnd`
  - `SkipPause`
  - `get_HasAnyHelper`
  - `CleanUpAll`
  - `AlterUpdate`
  - `IsEndTopCut`
  - `AlterLateUpdate`
  - `DestroyHelper`
  - `DestroyHelper`

### `Gallop.PhotoStudioPlayCutSettingsScenarioCutBreedersProvider` (36m)
  - `get_UniqueCommand`
  - `set_UniqueCommand`
  - `get_TrainingPlace`
  - `set_TrainingPlace`
  - `get_TrainingSeasonData`
  - `get_ReviewSeasonData`
  - `get_EverydayDetail`
  - `set_EverydayDetail`
  - `get_EverydaySeasonData`
  - `get_RaceDetail`
  - `set_RaceDetail`
  - `get_RaceCourse`
  - `set_RaceCourse`
  - `get_RaceId`
  - `set_RaceId`
  - `get_CharaDressSetDataModelAccessor`
  - `GetPhotoStudioCharaDressId`
  - `GetMemberDressId`
  - `GetPhotoStudioCharaDressIdWhenTraining`
  - `GetDressIdWhenTraining`

### `Gallop.PartsPhotoStudioPlayCutSettingsScenarioCutBreeders` (24m)
  - `InitializeView`
  - `SetupTraining`
  - `SetupReview`
  - `SetupEveryday`
  - `SetupRace`
  - `SetupToggle`
  - `UpdateContents`
  - `UpdateRoot`
  - `UpdateUniqueCommand`
  - `UpdateTraining`
  - `UpdateReview`
  - `UpdateEveryday`
  - `UpdateRace`
  - `OpenSelectCommandDialog`
  - `OpenSelectRaceDetailDialog`
  - `OpenRaceSelectDialog`
  - `<SetupTraining>b__29_0`
  - `<SetupTraining>b__29_1`
  - `<SetupReview>b__30_0`
  - `<SetupReview>b__30_1`

### `Gallop.PartsPhotoStudioPlayCutSettingsScenarioCutBreedersVM` (29m)
  - `get_CategoryName`
  - `get_UniqueCommand`
  - `get_UniqueCommandDisplayName`
  - `get_SeasonHeaderText`
  - `get_TrainingPlaceSelectIndex`
  - `get_EverydayDetailSelectIndex`
  - `get_RaceDetail`
  - `get_RaceDetailName`
  - `get_IsActiveRaceConquestSetting`
  - `get_RaceCourse`
  - `get_RaceThumbnailId`
  - `get_RaceName`
  - `CreateRaceSelectDialogModel`
  - `get_TrainingSeasonArray`
  - `get_ReviewSeasonArray`
  - `get_EverydaySeasonArray`
  - `get_TrainingSeasonSelectedIndex`
  - `get_ReviewSeasonSelectedIndex`
  - `get_EverydaySeasonSelectedIndex`
  - `get_TrainingSeasonNotificationMessage`

### `Gallop.RaceResultBoardAssetLoaderForSingleModeBreeders` (3m)
  - `RegisterDownload`
  - `Load`
  - `Unload`

### `Gallop.DialogIdleSingleModeBreedersRaceSelect` (10m)
  - `GetFormType`
  - `GetParentType`
  - `PushDialog`
  - `Setup`
  - `GetProperGradeByGroundType`
  - `GetProperGradeByDistanceType`
  - `OnDecide`
  - `OnDecideBCRaceRouteInfo`
  - `OnCancelBCRaceRouteInfo`
  - `OnCancel`

### `Gallop.PartsSingleModeScenarioBreedersMainViewTrainingFooter` (2m)
  - `SetupTeamSpTrainingEffectActivationAnimation`
  - `PlayTeamSpTrainingEffectActivationAnimation`

### `Gallop.PartsSingleModeScenarioBreedersMemberBCRaceA2UUtils` (1m)
  - `InitializeFlashActionPlayer`

### `Gallop.PartsSingleModeScenarioBreedersMemberBCRaceResultBackA2U` (6m)
  - `RegisterDownload`
  - `Create`
  - `Setup`
  - `PlayHide`
  - `PlayIn`
  - `PlayOut`

### `Gallop.PartsSingleModeScenarioBreedersMemberBCRaceResultFrontA2U` (13m)
  - `RegisterDownload`
  - `Create`
  - `Setup`
  - `PlayHide`
  - `PlayIn00`
  - `PlayIn01`
  - `PlayIn00`
  - `PlayIn01`
  - `PlayRank`
  - `SetRaceName`
  - `PlaySe`
  - `PlayOut00`
  - `PlayOut01`

### `Gallop.PartsSingleModeScenarioBreedersMemberBCRaceStartBackA2U` (6m)
  - `RegisterDownload`
  - `Create`
  - `Setup`
  - `PlayHide`
  - `PlayIn`
  - `Hide`


## Obscured加密数据类 (28个)

### `Gallop.ObscuredSingleModeBreedersCommandGainExp` (4m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_GainExp`
  - `set_GainExp`

### `Gallop.ObscuredSingleModeBreedersCommandGainExpExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersCommandInfo` (10m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_CommandId`
  - `set_CommandId`
  - `get_ParamsIncDecInfoArray`
  - `set_ParamsIncDecInfoArray`
  - `get_TeamMemberInfoArray`
  - `set_TeamMemberInfoArray`
  - `get_RankUpPredict`
  - `set_RankUpPredict`

### `Gallop.ObscuredSingleModeBreedersCommandInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersCommandTeamMemberInfo` (4m)
  - `get_CharaId`
  - `set_CharaId`
  - `get_GainExp`
  - `set_GainExp`

### `Gallop.ObscuredSingleModeBreedersCommandTeamMemberInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersDataSet` (30m)
  - `get_CommandInfoArray`
  - `set_CommandInfoArray`
  - `get_TeamMemberInfoArray`
  - `set_TeamMemberInfoArray`
  - `get_TeamSpTrainingInfo`
  - `set_TeamSpTrainingInfo`
  - `get_NotUpParameterInfo`
  - `set_NotUpParameterInfo`
  - `get_BcRaceResultArray`
  - `set_BcRaceResultArray`
  - `get_TeamUnionProgress`
  - `set_TeamUnionProgress`
  - `get_BcRaceTrackId`
  - `set_BcRaceTrackId`
  - `get_TeamRank`

### `Gallop.ObscuredSingleModeBreedersDataSetExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersDataSetLoad` (8m)
  - `get_LastSelectBcGroupId`
  - `set_LastSelectBcGroupId`
  - `get_DeckId`
  - `set_DeckId`
  - `get_TeamReviewResultArray`
  - `set_TeamReviewResultArray`
  - `get_EnhanceGroupArray`
  - `set_EnhanceGroupArray`

### `Gallop.ObscuredSingleModeBreedersDataSetLoadExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersDataSetStart` (8m)
  - `get_LastSelectBcGroupId`
  - `set_LastSelectBcGroupId`
  - `get_DeckId`
  - `set_DeckId`
  - `get_TeamReviewResultArray`
  - `set_TeamReviewResultArray`
  - `get_EnhanceGroupArray`
  - `set_EnhanceGroupArray`

### `Gallop.ObscuredSingleModeBreedersDataSetStartExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersDataSetTeamMeeting` (2m)
  - `get_EnhanceGroupArray`
  - `set_EnhanceGroupArray`

### `Gallop.ObscuredSingleModeBreedersDataSetTeamMeetingExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersDataSetTeamReview` (2m)
  - `get_TeamReviewResultArray`
  - `set_TeamReviewResultArray`

### `Gallop.ObscuredSingleModeBreedersDataSetTeamReviewExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersEnhanceGroup` (4m)
  - `get_GroupType`
  - `set_GroupType`
  - `get_Level`
  - `set_Level`

### `Gallop.ObscuredSingleModeBreedersEnhanceGroupExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersMemberBCRaceResult` (4m)
  - `get_BcGroupId`
  - `set_BcGroupId`
  - `get_WinMemberId`
  - `set_WinMemberId`

### `Gallop.ObscuredSingleModeBreedersMemberBCRaceResultExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersNotUpParameterInfo` (4m)
  - `get_NotUpExpCharaIdArray`
  - `set_NotUpExpCharaIdArray`
  - `get_IsOverflowStock`
  - `set_IsOverflowStock`

### `Gallop.ObscuredSingleModeBreedersNotUpParameterInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersTeamMemberInfo` (8m)
  - `get_MemberId`
  - `set_MemberId`
  - `get_CharaId`
  - `set_CharaId`
  - `get_Rank`
  - `set_Rank`
  - `get_Exp`
  - `set_Exp`

### `Gallop.ObscuredSingleModeBreedersTeamMemberInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersTeamReviewResult` (4m)
  - `get_ScheduleId`
  - `set_ScheduleId`
  - `get_ResultType`
  - `set_ResultType`

### `Gallop.ObscuredSingleModeBreedersTeamReviewResultExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `Gallop.ObscuredSingleModeBreedersTeamSpTrainingInfo` (6m)
  - `get_StockNum`
  - `set_StockNum`
  - `get_StockMax`
  - `set_StockMax`
  - `get_ActivatedState`
  - `set_ActivatedState`

### `Gallop.ObscuredSingleModeBreedersTeamSpTrainingInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`


## Master数据库表 (10个)

| 表名 | 方法数 |
|---|---|
| `MasterSingleMode13AddDreamPoint` | 8 |
| `MasterSingleMode13BcProgramFlag` | 17 |
| `MasterSingleMode13Member` | 8 |
| `MasterSingleMode13Rank` | 8 |
| `MasterSingleMode13RankBonusEffectGroup` | 9 |
| `MasterSingleMode13Schedule` | 10 |
| `MasterSingleMode13TeamRank` | 12 |
| `MasterSingleMode13TeamSpEffect` | 20 |
| `MasterSingleMode13TeamSpLevel` | 12 |
| `MasterSingleMode13TopBgChara` | 5 |

## WorkSingleModeScenarioBreeders

方法数: 7

  - `get_DataSet`
  - `set_DataSet`
  - `ApplyDataSetCommon`
  - `ApplyDataSetStart`
  - `ApplyDataSetLoad`
  - `ApplyDataSetTeamReview`
  - `ApplyDataSetTeamMeeting`

## WorkSingleModeScenarioBreedersDataSet

方法数: 45, 19 getters

  - `get_EqualityContract`
  - `get_CommandInfoArray`
  - `get_TeamMemberInfoArray`
  - `get_TeamSpTrainingInfo`
  - `get_TeamReviewResultArray`
  - `get_NotUpParameterInfo`
  - `get_BcRaceResultArray`
  - `get_TeamUnionProgress`
  - `get_BcRaceTrackId`
  - `get_EnhanceGroupArray`
  - `get_TeamRank`
  - `get_HavingEnhancePoint`
  - `get_PredictEnhancePoint`
  - `get_TeamUnionEvent`
  - `get_CommandGainExpArray`
  - `get_ScenarioDressSetting`
  - `get_TeamSpLevelLimit`
  - `get_LinkFriendOutingMemberInfoArray`
  - `get_DeckId`

## 剧本独立属性变化 (15m, 12 getters)

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

## lib.rs相关引用

```
11=>"Pioneer", 12=>"Onsen", 13=>"Breeders", 14=>"Ramen", _=>"Unknown"
// --- Buffs: chara_effect_ids → readable names (ALL scenarios) + EnhanceGroup (Breeders) ---
13=>"WorkSingleModeScenarioBreeders", 14=>"WorkSingleModeScenarioRamen",
// ★ EnhanceGroups (Breeders buff data) → override chara_effect_ids buffs
// Only for Breeders scenario; enhances have proper levels 1-8
13 => "ObscuredSingleModeBreedersEnhanceGroup",
let (gtn, desc) = breeders_buff_desc(gt, lv);
buffs.push(format!(r#"{{"name":"{}","level":{},"desc":"{}","type":"Breeders"}}"#, gtn, lv, desc));
// ★ Breeders team member data (v3.15.4)
let team_json = if sid == 13 {
let team_result = read_breeders_team();
r#"{"status":"ok","version":"3.15.3","endpoints":["/summary","/data","/scenario","/debug/params","/debug/breeders","/log","/status","/health"]}"#.to_string()
} else if path == "/debug/breeders" {
unsafe { debug_breeders_team() }
13 => "WorkSingleModeScenarioBreeders",
let cmd_elem_class = find_class_by_short_name(image, "ObscuredSingleModeBreedersCommandInfo");
// ★ Debug: Breeders scenario team member exploration (v3.15.4)
// Explores the Breeders DataSet to find team member fields
/// Read team member data for the Breeders (Dreams) scenario
unsafe fn read_breeders_team() -> String {
if sid != 13 { return format!(r#"{{"error":"not_breeders","scenario_id":{}}}"#, sid); }
let sc_class = find_class_by_short_name(image, "WorkSingleModeScenarioBreeders");
let ds_class = find_class_by_short_name(image, "WorkSingleModeScenarioBreedersDataSet");
// Common field name patterns for team members in Breeders scenario
"get_BreedersMemberArray", "get_UnitArray", "get_UnitInfoArray"];
"ObscuredSingleModeBreedersMemberInfo",
"SingleModeBreedersMemberInfo",
"ObscuredSingleModeBreedersUnitInfo",
"SingleModeBreedersUnitInfo",
/// Debug endpoint for Breeders team exploration
/// Lists all getter methods on the Breeders DataSet to find team member field names
unsafe fn debug_breeders_team() -> String {
if sid != 13 { return format!(r#"{{"error":"not_breeders","scenario_id":{}}}"#, sid); }
// First, try the read_breeders_team function
let team_data = read_breeders_team();
// Also enumerate classes to find Breeders-related class names
let breeders_classes = search_classes("Breeders");
team_data, breeders_classes
// Use /classes/search to find Breeders-related classes
"WorkSingleModeScenarioBreeders",
"WorkSingleModeScenarioBreedersDataSet",
"ObscuredSingleModeBreedersMemberInfo",
"SingleModeBreedersMemberInfo",
"ObscuredSingleModeBreedersUnitInfo",
"SingleModeBreedersUnitInfo",
"ObscuredSingleModeBreedersEnhanceGroup",
"ObscuredSingleModeBreedersCommandInfo",
```