# 剧本 7: Sport (スポーツ)

**WorkScenario类**: `WorkSingleModeScenarioSport`
**ObscuredDataSet**: `ObscuredSingleModeSportDataSet`
---

## 相关类 (90个)

### `Gallop.MasterSingleModeSportCompeEffect` (13m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithSportTypeOrderByIdAsc`
  - `_SelectWithSportTypeOrderByIdAsc`
  - `GetListWithSportTypeOrderByIdAsc`
  - `MaybeListWithSportTypeOrderByIdAsc`
  - `_ListSelectWithSportTypeOrderByIdAsc`
  - `_CreateOrmByQueryResultWithSportTypeOrderByIdAsc`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `GetListWithSportTypeAndWinNum`
  - `GetListWithSportTypeAndMinNumAndMaxNum`

### `Gallop.MasterSingleModeSportCompeSe` (6m)
  - `Get`
  - `_SelectOne`
  - `GetWithSheetId`
  - `_SelectWithSheetId`
  - `_CreateOrmByQueryResultWithSheetId`
  - `Unload`

### `Gallop.MasterSingleModeSportCompetition` (13m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithCompeType`
  - `_SelectWithCompeType`
  - `_CreateOrmByQueryResultWithCompeType`
  - `GetWithTurn`
  - `_SelectWithTurn`
  - `_CreateOrmByQueryResultWithTurn`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `GetSingleModeSportCompetition`
  - `GetScheduleList`

### `Gallop.MasterSingleModeSportItemEffect` (12m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithItemId`
  - `_SelectWithItemId`
  - `GetListWithItemId`
  - `MaybeListWithItemId`
  - `_ListSelectWithItemId`
  - `_CreateOrmByQueryResultWithItemId`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `GetWithItemIdAndTurn`

### `Gallop.MasterSingleModeSportLink` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithCommandId`
  - `_SelectWithCommandId`
  - `GetListWithCommandId`
  - `MaybeListWithCommandId`
  - `_ListSelectWithCommandId`
  - `_CreateOrmByQueryResultWithCommandId`
  - `Unload`

### `Gallop.MasterSingleModeSportMob` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleModeSportSportType` (14m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithSportType`
  - `_SelectWithSportType`
  - `GetListWithSportType`
  - `MaybeListWithSportType`
  - `_ListSelectWithSportType`
  - `_CreateOrmByQueryResultWithSportType`
  - `GetWithCommandId`
  - `_SelectWithCommandId`
  - `_CreateOrmByQueryResultWithCommandId`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleModeSportStance` (13m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithSportTypeOrderByTotalSportRankAsc`
  - `_SelectWithSportTypeOrderByTotalSportRankAsc`
  - `GetListWithSportTypeOrderByTotalSportRankAsc`
  - `MaybeListWithSportTypeOrderByTotalSportRankAsc`
  - `_ListSelectWithSportTypeOrderByTotalSportRankAsc`
  - `_CreateOrmByQueryResultWithSportTypeOrderByTotalSportRankAsc`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `GetActivatedStanceMasterListBySportRankChange`
  - `GetNextStanceMaster`

### `Gallop.MasterSingleModeSportStanceEffect` (4m)
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `GetByMasterSingleModeSportStance`

### `Gallop.MasterSingleModeSportTrainingCut` (6m)
  - `Get`
  - `_SelectOne`
  - `GetWithCommandIdAndTrainingLevel`
  - `_SelectWithCommandIdAndTrainingLevel`
  - `_CreateOrmByQueryResultWithCommandIdAndTrainingLevel`
  - `Unload`

### `Gallop.WorkSingleModeChangeParameterInfoScenarioSport` (6m)
  - `get_GainSportRankDic`
  - `get_UseItemId`
  - `Clear`
  - `Set`
  - `SetScenarioSportTrainingSportRankGain`
  - `SetScenarioSportUseItem`

### `Gallop.WorkSingleModeScenarioSport` (17m)
  - `get_SportTrainingDataList`
  - `get_SportCommandInfoList`
  - `get_ItemIdList`
  - `get_EffectedItemIdList`
  - `get_SportCompetitionResultList`
  - `get_CompetitionEffectIdList`
  - `get_SportSportEffectedStanceList`
  - `get_GainSportRankList`
  - `Apply`
  - `ApplySportTrainingData`
  - `ApplySportCommandInfo`
  - `ApplySportItemId`
  - `ApplySportEffectedItemId`
  - `ApplySportCompetitionResult`
  - `ApplySportCompetitionEffect`
  - `ApplySportEffectedStance`
  - `ApplyGainSportRankList`

### `Gallop.DialogPhotoStudioSportTrainingType` (6m)
  - `GetFormType`
  - `GetParentType`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`
  - `OnRightButtonCallBack`

### `Gallop.PartsPhotoStudioToggleOptionSportCommand` (4m)
  - `Create`
  - `SetupOption`
  - `SetSportTrainingTypeIndex`
  - `<SetupOption>b__7_0`

### `Gallop.PhotoStudioViewModelSport` (12m)
  - `get_SportCommandOptionList`
  - `OnSelectCommandOption`
  - `get_ResultOptionList`
  - `OnSelectResultOption`
  - `get_SeasonOptionArray`
  - `get_IsActivePartnerOption`
  - `get_IsEnableNonePartner`
  - `ChangeCutSetting`
  - `SetupView`
  - `SetupTrainingPartnerCautionText`
  - `IsSoloTraining`
  - `CreateCuttPlayInfo`

### `Gallop.DialogSingleModeScenarioSportCompetitionResult` (18m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`
  - `SetupAnimationFrameCross`
  - `SetupTitleFlash`
  - `SetupPartsResultSummary`
  - `SetupPartsResultCommandList`
  - `OnClickNextButton`
  - `PlayIn`
  - `CreateWinEffect`
  - `PlayOut`
  - `<SetupPartsResultCommandList>b__26_0`
  - `<PlayIn>b__28_0`
  - `<PlayIn>b__28_1`
  - `<PlayIn>b__28_2`

### `Gallop.DialogSingleModeScenarioSportCompetitionResultDetail` (9m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`
  - `SetupLogoImage`
  - `SetupPartsResultSummary`
  - `SetupPartsResultCommandList`

### `Gallop.PartsSingleModeScenarioSportCompetitionResultCommandList` (3m)
  - `Setup`
  - `SetupScrollViewFade`
  - `GetActiveItemList`

### `Gallop.PartsSingleModeScenarioSportCompetitionResultCommandListItemModel` (2m)
  - `get_SportTrainingCommandModel`
  - `get_IsWin`

### `Gallop.PartsSingleModeScenarioSportCompetitionResultCommandListItem` (1m)
  - `OnItemUpdate`

### `Gallop.PartsSingleModeScenarioSportCompetitionResultSummaryItem` (1m)
  - `Setup`

### `Gallop.SingleModeScenarioSportCompetitionResultView` (14m)
  - `get_FlashOverlayCanvas`
  - `set_FlashOverlayCanvas`
  - `get_SportCompetitionTitleImage`
  - `set_SportCompetitionTitleImage`
  - `get_ResultFlashRoot`
  - `set_ResultFlashRoot`
  - `get_WinNumTextRoot`
  - `set_WinNumTextRoot`
  - `get_WinNumText`
  - `set_WinNumText`
  - `get_ResultDetailButton`
  - `set_ResultDetailButton`
  - `get_NextButton`
  - `set_NextButton`

### `Gallop.SingleModeScenarioSportCompetitionResultViewController` (30m)
  - `get_BgPath`
  - `get_CompetitionResult`
  - `get_SportCompetitionTitleImagePath`
  - `RegisterDownload`
  - `GetDynamicBgmId`
  - `GetDynamicBgmCueInfo`
  - `InitializeView`
  - `SetupSingleModeScene`
  - `SetCharacterBg`
  - `SetupBgEffect`
  - `SetupFlashOverlayCanvas`
  - `SetupSportCompetitionResult`
  - `SetupSportCompetitionResultFlash`
  - `PlayInCompetitionResult`
  - `SetNamePlate`
  - `SetupButton`
  - `PlayInView`
  - `PlayInCharacterVoice`
  - `get_MasterCharaMotionSet`
  - `GetBackButtonAnimationDelayTime`

### `Gallop.DialogSingleModeScenarioSportCompetitionShowDownEndLiveConfirm` (9m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`
  - `SetupLogoImage`
  - `OnRightButtonCallBack`
  - `OnLeftButtonCallBack`

### `Gallop.NowLoadingWipeSingleModeScenarioSportCompetitionTop` (5m)
  - `RegisterDownload`
  - `Show`
  - `CreateWipePlayerAction`
  - `ShowWipePlayerAction`
  - `HideWipePlayerAction`

### `Gallop.SingleModeScenarioSportCompetitionShowDownCutInController` (17m)
  - `RegisterDownload`
  - `get_CutInHelper`
  - `get_RenderTexture`
  - `get_ModelController`
  - `PlayCutIn`
  - `PrepareCutIn`
  - `CreateCutInHelper`
  - `CreateRenderTexture`
  - `CreateCharacterModel`
  - `GetUserModelControllerAction`
  - `OnStartCutIn`
  - `Update`
  - `LateUpdate`
  - `Destroy`
  - `DestroyCutInHelper`
  - `DesotryRenderTexture`
  - `DesotryCharacterModel`

### `Gallop.SingleModeScenarioSportCompetitionTopViewModel` (3m)
  - `get_CompetitionModel`
  - `get_IsLastCompetition`
  - `ExecuteCompetition`

### `Gallop.SingleModeScenarioSportCompetitionTopView` (40m)
  - `get_BlackPanel`
  - `set_BlackPanel`
  - `get_SportCompetitionButton`
  - `set_SportCompetitionButton`
  - `get_SportCompetitionShowDownButton`
  - `set_SportCompetitionShowDownButton`
  - `get_SportCompetitionLogoFlashRoot`
  - `set_SportCompetitionLogoFlashRoot`
  - `get_SportCompetitionExpectationRoot`
  - `set_SportCompetitionExpectationRoot`
  - `get_SportCompetitionExpectationBaseImage`
  - `set_SportCompetitionExpectationBaseImage`
  - `get_SportCompetitionExpectationImage`
  - `set_SportCompetitionExpectationImage`
  - `get_SportCompetitionExpectationGauge`
  - `set_SportCompetitionExpectationGauge`
  - `get_SportCompetitionExpectationInfoButton`
  - `set_SportCompetitionExpectationInfoButton`
  - `get_CompetitoionHighlightTelopFlashParent`
  - `set_CompetitoionHighlightTelopFlashParent`

### `Gallop.SingleModeScenarioSportCompetitionTopViewController` (86m)
  - `get_Model`
  - `RegisterDownload`
  - `GetDynamicBgmId`
  - `GetDynamicBgmCueInfo`
  - `InitializeView`
  - `SetLastCompetitionBg`
  - `SetupBgEffect`
  - `SetupSingleModeHeader`
  - `SetupSingleModeScene`
  - `SetupSportCompetitionLogo`
  - `SetupSportCompetitionExpectation`
  - `SetupSportCompetitionExpectationGauge`
  - `SetupStartSportCompetitionButton`
  - `SetupCompetitionButtonEffect`
  - `SetupSportCompetitoionHighlight`
  - `get_CharacterCatchCopyText`
  - `GetCatchCopyLevel`
  - `SetupCutInShowDownTargetImage`
  - `InitializeEachPlayIn`
  - `PlayInView`

### `Gallop.SingleModeScenarioSportCompetitionCutInInfoModel` (6m)
  - `get_CommandId`
  - `get_CutInPattern`
  - `get_SportColorPattern`
  - `get_TrainingName`
  - `CommandIdToCutInPattern`
  - `SportTypeToColorPattern`

### `Gallop.SingleModeScenarioSportCompetitionExpectationModel` (11m)
  - `get_CompeType`
  - `get_CompetitionMaster`
  - `get_SportTrainingDataList`
  - `get_Turn`
  - `get_AchievedCommandNum`
  - `get_CommandNumToWin`
  - `get_SportRankToWin`
  - `get_ResultExpectation`
  - `get_IsAllCommandAchieved`
  - `<get_AchievedCommandNum>b__11_0`
  - `<get_IsAllCommandAchieved>b__19_0`

### `Gallop.SingleModeScenarioSportCompetitionModel` (9m)
  - `get_WorkSport`
  - `get_CompetitionResultList`
  - `get_NextCompetitionExpectation`
  - `get_IsAllCompetitionEnd`
  - `GetCompetitionResultWithCompeType`
  - `GetCompetitionResultWithTurn`
  - `GetCompetitionWinCommandIdCountWithSportType`
  - `get_CompetitionCutInTrainingCommandModelList`
  - `GetCurrentEffectAmountWithEffectType`

### `Gallop.SingleModeScenarioSportItemModel` (9m)
  - `get_WorkSport`
  - `get_EffectMaster`
  - `get_EffectMasterId`
  - `get_ItemId`
  - `get_ItemNum`
  - `get_HasItem`
  - `get_EffectType`
  - `get_CharacterName`
  - `<get_ItemNum>b__11_0`

### `Gallop.SingleModeScenarioSportStanceModel` (13m)
  - `get_SportType`
  - `get_SportTrainingCommandModelList`
  - `get_MasterSingleModeSportStanceList`
  - `get_MasterSingleModeSportStanceEffectTurn`
  - `get_ActiveMasterSingleModeSportStance`
  - `get_IsActiveEffect`
  - `get_ActiveEffectRemainCount`
  - `get_NextMasterSingleModeSportStance`
  - `get_SumSportTrainingSportRank`
  - `get_NextStanceActivationSportRank`
  - `get_NextStanceActivationRemainSportRank`
  - `get_IsMaxLevelAllSportTraining`
  - `<get_ActiveEffectRemainCount>b__16_0`

### `Gallop.SingleModeScenarioSportTrainingCommandModel` (14m)
  - `get_CommandId`
  - `get_SportType`
  - `get_MasterSingleModeTraining`
  - `get_BaseCommandId`
  - `get_TrainingName`
  - `get_WorkSportTrainingData`
  - `get_SportRank`
  - `get_IsMaxSportRank`
  - `get_GainSportRank`
  - `get_IsOverCompetitionWinSportRank`
  - `GetSingleModeScenarioSportTrainingCommandModelList`
  - `<get_WorkSportTrainingData>b__14_0`
  - `<get_GainSportRank>b__20_0`
  - `<get_GainSportRank>b__20_1`

### `Gallop.DialogSingleModeScenarioSportCompetitionEffectDetail` (10m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`
  - `SetupToggleGroup`
  - `SetupSportTypeInfo`
  - `SetupCompetitionEffectDetailList`
  - `OnSelectToggle`

### `Gallop.DialogSingleModeScenarioSportCompetitionInfo` (6m)
  - `GetFormType`
  - `GetParentType`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`
  - `SetupCompetitionExpectation`

### `Gallop.DialogSingleModeScenarioSportDataListModel` (2m)
  - `get_CompetitionModel`
  - `get_LogoImagePath`

### `Gallop.DialogSingleModeScenarioSportDataList` (10m)
  - `GetFormType`
  - `GetParentType`
  - `get_Model`
  - `RegisterDownload`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`
  - `SetupLogoImage`
  - `SetupCompetitionExpectation`
  - `SetupScheduleList`

### `Gallop.DialogSingleModeScenarioSportNextScheduleListModel` (6m)
  - `get_CompetitionModel`
  - `RegisterDownload`
  - `get_FrameType`
  - `get_DialogPrefabPath`
  - `get_LogoImagePath`
  - `get_ScheduleListItemModels`

### `Gallop.DialogSingleModeScenarioSportSportRankDetail` (10m)
  - `GetFormType`
  - `GetParentType`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`
  - `InitializeToggleGroup`
  - `SetupSportRankList`
  - `OnSelectToggle`
  - `GetToggleIndex`
  - `GetToggleSportType`

### `Gallop.DialogSingleModeScenarioSportStanceInfo` (6m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`

### `Gallop.DialogSingleModeScenarioSportItemUpdateNotice` (13m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`
  - `SetupTitleEffect`
  - `SetupIconEffect`
  - `SetupDescriptionText`
  - `CreateEffect`
  - `PlayIn`
  - `<CreateDialogData>b__9_0`
  - `<PlayIn>b__15_0`

### `Gallop.DialogSingleModeScenarioSportUseItemChangeSportTypeAllModel` (14m)
  - `get_ItemModel`
  - `get_CurrentTrainingCommandTurnInfoList`
  - `IsExistSportTypeTraining`
  - `GetGainSportRank`
  - `get_IsAllCompetitionEnd`
  - `get_CurrentSelectedConversion`
  - `set_CurrentSelectedConversion`
  - `get_PrevSelectedConversion`
  - `set_PrevSelectedConversion`
  - `get_DefaultConversion`
  - `get_IsConversionSelected`
  - `UpdateSelectedConversion`
  - `GetDstCommandId`
  - `ExecuteUseItem`

### `Gallop.DialogSingleModeScenarioSportUseItemChangeSportTypeAll` (23m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `PushDialog`
  - `CreateDialogData`
  - `OnClickRightButton`
  - `Setup`
  - `SetupHeader`
  - `SetupSportTypeDetailButton`
  - `SetupCurrentCommandList`
  - `SetupPredictCommandList`
  - `SetupPredictCommandListTAT`
  - `PlayInPredictCommandList`
  - `SetupConversionSelectPanel`
  - `SetupConversionSelectPanelCursor`
  - `SetupItemNum`
  - `SetupNoItemText`
  - `SetupDialogRightButtonEnable`
  - `OnChangeSortingOrder`
  - `OnSelectConversionPanelItem`

### `Gallop.DialogSingleModeScenarioSportUseItemChangeSportTypeAllConfirm` (10m)
  - `GetFormType`
  - `GetParentType`
  - `PushDialog`
  - `CreateDialogData`
  - `OnClickRightButton`
  - `Setup`
  - `SetupCurrentCommandList`
  - `SetupPredictCommandList`
  - `SetupItemNum`
  - `SetupItemConfirmText`

### `Gallop.PartsSingleModeScenarioSportUseItemChangeSportTypeAllSelectPanel` (7m)
  - `get_SportType`
  - `RegisterDownload`
  - `Setup`
  - `SetOnSelect`
  - `SetupBaseImage`
  - `GetButtonObject`
  - `OnClickSelectButton`

### `Gallop.PartsSingleModeScenarioSportUseItemTrainingCommandButtonForGUI` (6m)
  - `Setup`
  - `SetupTrainingIcon`
  - `SetupSportTypeIcon`
  - `SetupTrainingTypeIcon`
  - `SetupSportRankText`
  - `SetupInteractableColor`

### `Gallop.PartsSingleModeScenarioSportUseItemTrainingCommandButtonForGUIAccessory` (7m)
  - `RegisterDownload`
  - `SetActiveTagTrainingEffect`
  - `SetTagTrainingBackEffectParent`
  - `SetupTrainingPartnerNum`
  - `SetupGainSportRank`
  - `HideGainSportRank`
  - `SetActiveChangedBadge`

### `Gallop.PartsSingleModeScenarioSportUserItemButtonModel` (2m)
  - `get_UserItemNum`
  - `get_HasUserItem`

### `Gallop.PartsSingleModeScenarioSportUserItemButton` (5m)
  - `get_Model`
  - `Setup`
  - `SetupBadge`
  - `SetInteractable`
  - `SetButtonEnable`

### `Gallop.PartsSingleModeScenarioSportCompetitionEffectDetailList` (3m)
  - `Setup`
  - `InstantiateListItemObjects`
  - `UpdateListItems`

### `Gallop.PartsSingleModeScenarioSportCompetitionEffectDetailListItem` (5m)
  - `Setup`
  - `SetupEffectActivationStatus`
  - `SetupWinNumRangeText`
  - `SetupEffectValueText`
  - `PrepareEffectValueTextItemObjects`

### `Gallop.PartsSingleModeScenarioSportCompetitionEffectDetailListValueTextItem` (1m)
  - `Setup`

### `Gallop.PartsSingleModeScenarioSportCompetitionEffectSummaryList` (2m)
  - `Setup`
  - `CreateCompetitionEffectListItem`

### `Gallop.PartsSingleModeScenarioSportCompetitionEffectSummaryListItem` (3m)
  - `Create`
  - `Setup`
  - `SetupValueText`

### `Gallop.PartsSingleModeScenarioSportCompetitionExpectation` (11m)
  - `RegisterDownload`
  - `Setup`
  - `SetupCompetitionEnd`
  - `PlayGaugeAnimation`
  - `PlayGaugeMaxStateAnimation`
  - `SetupFlashActionPlayer`
  - `SetupGauge`
  - `PrepareGaugeUpAnimationCallback`
  - `PlayGaugeUpAnimation`
  - `PlayMaxDisplayStateAnimation`
  - `IsGaugeMaxValue`

### `Gallop.PartsSingleModeScenarioSportCompetitionExpectationGauge` (1m)
  - `SetupGauge`

### `Gallop.PartsSingleModeScenarioSportCompetitionShowDownButton` (8m)
  - `RegisterDownload`
  - `Setup`
  - `SetupFlashActionPlayer`
  - `PlayIn`
  - `PlayOut`
  - `<SetupFlashActionPlayer>b__6_0`
  - `<SetupFlashActionPlayer>b__6_1`
  - `<SetupFlashActionPlayer>b__6_2`

### `Gallop.PartsSingleModeScenarioSportImageNumberModel` (12m)
  - `get_EnableBonus`
  - `get_ImageNumberPrefabPath`
  - `get_BonusA2UPath`
  - `get_IsActiveBallGameStance`
  - `get_IsActiveMartialArtsStance`
  - `RegisterDownload`
  - `get_IsShowZeroGainParamTextByHasBonusValue`
  - `get_IsBadConditionOverWeight`
  - `get_IsTrainingGainTargetParameter`
  - `SetValue`
  - `OnSetBonusValueExtend`
  - `SetHeatUpEffect`


## Master数据库表 (10个)

| 表名 | 方法数 |
|---|---|
| `MasterSingleModeSportCompeEffect` | 13 |
| `MasterSingleModeSportCompeSe` | 6 |
| `MasterSingleModeSportCompetition` | 13 |
| `MasterSingleModeSportItemEffect` | 12 |
| `MasterSingleModeSportLink` | 9 |
| `MasterSingleModeSportMob` | 5 |
| `MasterSingleModeSportSportType` | 14 |
| `MasterSingleModeSportStance` | 13 |
| `MasterSingleModeSportStanceEffect` | 4 |
| `MasterSingleModeSportTrainingCut` | 6 |

## WorkSingleModeScenarioSport

方法数: 17

  - `get_SportTrainingDataList`
  - `get_SportCommandInfoList`
  - `get_ItemIdList`
  - `get_EffectedItemIdList`
  - `get_SportCompetitionResultList`
  - `get_CompetitionEffectIdList`
  - `get_SportSportEffectedStanceList`
  - `get_GainSportRankList`
  - `Apply`
  - `ApplySportTrainingData`
  - `ApplySportCommandInfo`
  - `ApplySportItemId`
  - `ApplySportEffectedItemId`
  - `ApplySportCompetitionResult`
  - `ApplySportCompetitionEffect`
  - `ApplySportEffectedStance`
  - `ApplyGainSportRankList`

## 剧本独立属性变化 (6m, 2 getters)

  - `get_GainSportRankDic`
  - `get_UseItemId`

## lib.rs相关引用

```
6=>"Arc", 7=>"Sport", 8=>"Cook", 9=>"Mecha", 10=>"Legend",
7=>"WorkSingleModeScenarioSport", 8=>"WorkSingleModeScenarioCook",
7 => "WorkSingleModeScenarioSport",
```