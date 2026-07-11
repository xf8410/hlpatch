# 剧本14 拉面杯(Twinkle Ramen)深度分析

**剧本ID**: 14  
**WorkScenario**: `WorkSingleModeScenarioRamen`
**DataSet**: `ObscuredSingleModeRamenDataSet`
---

## 拉面杯相关类 (97个)

### `DialogSingleModeScenarioRamenAlertLossFeeling` (5m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `PushDialog`
  - `Setup`

### `DialogSingleModeScenarioRamenAlertLossFeelingViewModel` (1m)
  - `GetLossFeelingIconSpriteList`

### `DialogSingleModeScenarioRamenEndLiveConfirm` (9m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `RegisterDownload`
  - `PushDialog`
  - `Initialize`
  - `Setup`
  - `OnClickDecide`
  - `OnClickCancel`

### `DialogSingleModeScenarioRamenEndLiveConfirmViewModel` (2m)
**Getters** (2个):
  - `get_LogoImage`
  - `get_TextImage`

### `DialogSingleModeScenarioRamenNextScheduleListModel` (7m)
**Getters** (5个):
  - `get_FrameType`
  - `get_DialogPrefabPath`
  - `get_LogoImagePath`
  - `get_ScheduleListItemModels`
  - `get_NextLogoImagePath`

### `DialogSingleModeScenarioRamenOutingPartnerSelectItem` (1m)
  - `Setup`

### `DialogSingleModeScenarioRamenOutingPartnerSelectItemViewModel` (6m)
**Getters** (6个):
  - `get_SupportPrefabPath`
  - `get_CharaPrefabPath`
  - `get_GainCountText`
  - `get_GainSimulateResult`
  - `get_HasScenarioUniqueEffect`
  - `get_OutingFriend`

### `IScenarioRamenUpdateReceiver` (2m)
  - `UpdateView`
  - `LateUpdateView`

### `IWorkAutoPlayRamenPlanRecord` (8m)
**Getters** (4个):
  - `get_ModifyPreferenceJuniorRegionIdList`
  - `get_ModifyPreferenceClassicRegionIdList`
  - `get_ModifyPreferenceSeniorRegionIdList`
  - `get_ModifyPreferenceSelectUrafEffectType`

### `MasterSingleMode14TwinkleRamen` (5m)
**Getters** (1个):
  - `get_dictionary`

### `ObscuredSingleModeRamenActiveEffectInfo` (6m)
**Getters** (3个):
  - `get_EffectCategory`
  - `get_EffectId`
  - `get_EffectValue`

### `ObscuredSingleModeRamenActiveEffectInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenAutoSelectInfo` (8m)
**Getters** (4个):
  - `get_IsAutoSelect`
  - `get_JuniorRegionSet`
  - `get_ClassicRegionSet`
  - `get_SeniorRegionSet`

### `ObscuredSingleModeRamenAutoSelectInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenAutoSelectSetInfo` (10m)
**Getters** (5个):
  - `get_SetId`
  - `get_SetName`
  - `get_JuniorRegionSet`
  - `get_ClassicRegionSet`
  - `get_SeniorRegionSet`

### `ObscuredSingleModeRamenAutoSelectSetInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenCheckPointInfo` (4m)
**Getters** (2个):
  - `get_CheckPointType`
  - `get_ResultState`

### `ObscuredSingleModeRamenCheckPointInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenCommandFeelingInfo` (6m)
**Getters** (3个):
  - `get_CommandType`
  - `get_CommandId`
  - `get_FeelingId`

### `ObscuredSingleModeRamenCommandFeelingInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenCommandInfo` (6m)
**Getters** (3个):
  - `get_CommandType`
  - `get_CommandId`
  - `get_ParamsIncDecInfoArray`

### `ObscuredSingleModeRamenCommandInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenDataSet` (26m)
**Getters** (13个):
  - `get_CommandInfoArray`
  - `get_EvaluationInfoArray`
  - `get_FeelingReduceTurnInfoArray`
  - `get_FeelingTurnInfoArray`
  - `get_FeelingInfoArray`
  - `get_SpecialFeelingNum`
  - `get_NotUpParameterInfo`
  - `get_ActiveEffectArray`
  - `get_UrafEffectInfo`
  - `get_CommandFeelingInfoArray`
  - `get_TrainingExecInfoArray`
  - `get_ReduceBaseTurnInfoArray`
  - `get_AllSelectedRegionIdArray`

### `ObscuredSingleModeRamenDataSetCheckEvent` (2m)
**Getters** (1个):
  - `get_IsGaugeGained`

### `ObscuredSingleModeRamenDataSetCheckEventExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenDataSetExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenDataSetLoad` (20m)
**Getters** (10个):
  - `get_AutoSelectInfo`
  - `get_AutoSelectSetInfo`
  - `get_SelectedRegionIdArray`
  - `get_ReduceBaseTurnInfoArray`
  - `get_CheckPointInfoArray`
  - `get_LastTastingInfo`
  - `get_CheckPointPt`
  - `get_ExpectedCheckPointPt`
  - `get_UsedTwinkleTextIdArray`
  - `get_IsCheckedUrafEvent`

### `ObscuredSingleModeRamenDataSetLoadExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenDataSetStart` (6m)
**Getters** (3个):
  - `get_AutoSelectInfo`
  - `get_AutoSelectSetInfo`
  - `get_IsCheckedUrafEvent`

### `ObscuredSingleModeRamenDataSetStartExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenEvaluationInfo` (4m)
**Getters** (2个):
  - `get_TargetId`
  - `get_CharaId`

### `ObscuredSingleModeRamenEvaluationInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenFeeling` (4m)
**Getters** (2个):
  - `get_FeelingIndex`
  - `get_FeelingId`

### `ObscuredSingleModeRamenFeelingExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenFeelingReduceTurnInfo` (6m)
**Getters** (3个):
  - `get_CommandType`
  - `get_CommandId`
  - `get_FeelingTurnArray`

### `ObscuredSingleModeRamenFeelingReduceTurnInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenFeelingTurnInfo` (4m)
**Getters** (2个):
  - `get_FeelingId`
  - `get_RemainTurn`

### `ObscuredSingleModeRamenFeelingTurnInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenLastTastingInfo` (10m)
**Getters** (5个):
  - `get_FeelingId1Num`
  - `get_FeelingId2Num`
  - `get_FeelingId3Num`
  - `get_SpecialFeelingNum`
  - `get_RegionId`

### `ObscuredSingleModeRamenLastTastingInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenNotUpParameterInfo` (2m)
**Getters** (1个):
  - `get_NotGainSpecialFeeling`

### `ObscuredSingleModeRamenNotUpParameterInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenReduceBaseTurnInfo` (4m)
**Getters** (2个):
  - `get_FeelingId`
  - `get_ReduceBaseTurn`

### `ObscuredSingleModeRamenReduceBaseTurnInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenReduceFeelingTurn` (4m)
**Getters** (2个):
  - `get_FeelingId`
  - `get_Turn`

### `ObscuredSingleModeRamenReduceFeelingTurnExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenTrainingExecInfo` (4m)
**Getters** (2个):
  - `get_BaseCommandId`
  - `get_ExecCount`

### `ObscuredSingleModeRamenTrainingExecInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenUrafEffectInfo` (4m)
**Getters** (2个):
  - `get_UrafEffectType`
  - `get_UrafEffectState`

### `ObscuredSingleModeRamenUrafEffectInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `PartsSingleModeScenarioRamenImageNumberModel` (4m)
**Getters** (4个):
  - `get_EnableBonus`
  - `get_ImageNumberPrefabPath`
  - `get_BonusA2UPath`
  - `get_IsShowZeroGainParamTextByHasBonusValue`

### `PartsSingleModeScenarioRamenMainStablesPanelButton` (3m)
  - `Create`
  - `Setup`
  - `SetupCustom`

### `PartsSingleModeScenarioRamenMainStablesPanelButtonViewModel` (10m)
**Getters** (9个):
  - `get_ButtonSprite`
  - `get_LockButtonSprite`
  - `get_ButtonSizeTypeSprite`
  - `get_TextImageSprite`
  - `get_ButtonIconIdleSprite`
  - `get_ButtonIconEnterSprite`
  - `get_IsEnableUniqCommand`
  - `get_Interactable`
  - `get_NotificationMessage`

### `PartsSingleModeScenarioRamenPlayServingPracticeCutIn` (2m)
  - `Create`
  - `Initialize`

### `PartsSingleModeScenarioRamenRaceEntry` (3m)
  - `Create`
  - `Setup`
  - `OnClickEffectListButton`

### `PartsSingleModeScenarioRamenRegionSelectNowLoadingWipe` (6m)
  - `CreatePlayInfo`
  - `CreateWipe`
  - `RegisterDownloadCustom`
  - `ShowRegionSelectStart`
  - `ShowWipePlayerActionCustom`
  - `ShowRegionSelectEnd`

### `PartsSingleModeScenarioRamenScheduleListItemModel` (12m)
**Getters** (11个):
  - `get_PrefabPath`
  - `get_ScheduleTurn`
  - `get_IsNextSchedule`
  - `get_HasResultData`
  - `get_HeaderImageSprite`
  - `get_TitleText`
  - `get_ScheduleOpenText`
  - `get_ResultImageSprite`
  - `get_ResultAudioId`
  - `get_IsNeedPlayWinEffectOnResultAnimation`
  - `get_IsSuccess`

### `PartsSingleModeScenarioRamenServingPracticePrepareMiniCharaAnimation` (9m)
  - `RegisterDownload`
  - `CreateMiniCharaBgPlayer`
  - `CreateMiniCharaController`
  - `PlayInit`
  - `Play`
  - `SetRegionWithoutAnimation`
  - `SetBgTex`
  - `SetFlagTex`
  - `Dispose`

### `PartsSingleModeScenarioRamenUrafRamenSelectController` (5m)
  - `RegisterDownload`
  - `ShowUrafRamenSelect`
  - `ChangeFooterExclusionUIImageEffect`
  - `SetFooterSortingOrder`
  - `ResetFooterSortingOrder`

### `PartsSingleModeScenarioRamenUseFeelingCountList` (1m)
  - `Setup`

### `PartsSingleModeScenarioRamenUseFeelingCountListViewModel` (8m)
**Getters** (2个):
  - `get_IsEnoughTotal`
  - `get_UseFeelingCountDic`

### `SingleModeMainTrainingDecideConfirmScenarioRamen` (4m)
  - `CreateExecCommandAlertConfirmActionQueue`
  - `AlertLossRamenFeeling`
  - `SkipShowAlertLossFeeling`
  - `GetInitializedAcquireFeelingContext`

### `SingleModeMainViewHeaderScenarioRamenModel` (15m)
**Getters** (10个):
  - `get_TitleOutlineColorType`
  - `get_NeedMultiTurn`
  - `get_ScenarioTargetText`
  - `get_ScenarioTargetHoldText`
  - `get_AtlasType`
  - `get_TurnBaseSpriteName`
  - `get_MultiTurnBaseSpriteName`
  - `get_FontColorType`
  - `get_OutlineColorType`
  - `get_VerticalGradientColorType`

### `SingleModeMainViewScenarioRamenController` (31m)
  - `RegisterDownload`
  - `SetupCore`
  - `OnClickAddonScenarioButton`
  - `UpdateCommonUIPosition`
  - `UpdateTrainingFooter`
  - `GetShowTurnStartNotice`
  - `ShowStartUrafIfNeed`
  - `ShowContinueUrafIfNeed`
  - `ShowScenarioNoticeIfNeed`
  - `ShowUseSpecialFeelingNoticeIfNeed`

### `SingleModeMainViewTrainingFooterItemA2UScenarioRamen` (11m)
**Getters** (2个):
  - `get_PlayerA2UPath`
  - `get_TipsBadgeFlashPlayerPath`

### `SingleModeMainViewTrainingFooterItemScenarioPowerUpEffectRamen` (6m)
  - `Setup`
  - `SetupTrainingName`
  - `SetTextTrainingNameAndColor`
  - `SetTextShadowTrainingNameAndColor`
  - `SetupScenarioPowerUpEffect`
  - `PlayIn`

### `SingleModeMainViewTrainingHorseIconA2UScenarioRamen` (6m)
  - `RegisterDownloadScenario`
  - `SetBadgeSortOffsetScenario`
  - `SetBadgeBalloon`
  - `CreateHintBadgeEffect`
  - `IsHintBadgeEffectActive`
  - `SetupHintBadgeEffect`

### `SingleModeRamenAPI` (38m)
  - `IsSingleModeRaceEndRequest`
  - `IsSingleModeFinishRequest`
  - `IsSingleModeContinueRequest`
  - `IsDateChangeCheckAPI`
  - `SendStart`
  - `SendLoad`
  - `SendExecCommand`
  - `SendCheckEvent`
  - `SendFinish`
  - `SendFactorSelect`

### `SingleModeScenarioRamenCheckPointProgressUpA2U` (9m)
**Getters** (4个):
  - `get_FlashPath`
  - `get_PlayTime`
  - `get_InMotionName`
  - `get_OutMotionName`

### `SingleModeScenarioRamenCheckPointProgressUpParameterInfo` (5m)
**Getters** (4个):
  - `get_IsSuccess`
  - `get_IsGreatSuccess`
  - `get_IsGroupPlay`
  - `get_MessageText`

### `SingleModeScenarioRamenCheckPointTopView` (0m)

### `SingleModeScenarioRamenCheckPointTopViewController` (27m)
**Getters** (5个):
  - `get_AutoPlayProxy`
  - `get_ViewModel`
  - `get_PartsViewModel`
  - `get_Top3DModel`
  - `get_ExtraEditionImageController`

### `SingleModeScenarioRamenCheckPointTopViewModel` (5m)
**Getters** (3个):
  - `get_EnableBackKey`
  - `get_SingleModeTrainingCharaId`
  - `get_TutorialId`

### `SingleModeScenarioRamenCutInHelper` (33m)
**Getters** (2个):
  - `get_CutInContext`
  - `get_ScreenShotCameraParam`

### `SingleModeScenarioRamenDefine` (0m)

### `SingleModeScenarioRamenExtraEditionImageController` (14m)
**Getters** (1个):
  - `get_CaptureRenderTextureList`

### `SingleModeScenarioRamenFinalCheckPointTopView` (0m)

### `SingleModeScenarioRamenFinalCheckPointTopViewController` (25m)
**Getters** (4个):
  - `get_AutoPlayProxy`
  - `get_ViewModel`
  - `get_PartsViewModel`
  - `get_ExtraEditionImageController`

### `SingleModeScenarioRamenFinalCheckPointTopViewViewModel` (4m)
**Getters** (1个):
  - `get_EnableBackKey`

### `SingleModeScenarioRamenMainTrainingMassageService` (2m)
  - `GetTrainingResultMassage`
  - `GetMessageList`

### `SingleModeScenarioRamenMainViewStablesPanelModel` (2m)
  - `CreateScenarioStablesPanelModel`
  - `CreateScenarioCommandButton`

### `SingleModeScenarioRamenMiniCharaController` (11m)
**Getters** (1个):
  - `get_RenderTexture`

### `SingleModeScenarioRamenMiniCharaFinalTopParam` (9m)
**Getters** (6个):
  - `get_FinalTopMainCharaMotionName`
  - `get_FinalTop1022MotionName`
  - `get_FinalTop1058MotionName`
  - `get_FinalTop1060MotionName`
  - `get_FinalTop1077MotionName`
  - `get_FinalTop1120MotionName`

### `SingleModeScenarioRamenMiniCharaParam` (2m)
  - `RegisterDownload`
  - `LoadSettingData`

### `SingleModeScenarioRamenRegionMapView` (20m)
**Getters** (10个):
  - `get_RegionSelectBg`
  - `get_RegionSelectedContent`
  - `get_ProgressingCheckPointObject`
  - `get_AllClearCheckPointObject`
  - `get_RegionEffectButton`
  - `get_CharaDetailButton`
  - `get_AutoSelectRegionButton`
  - `get_AutoSelectRegionOnLabel`
  - `get_ParticleParent`
  - `get_MessageBalloon`

### `SingleModeScenarioRamenRegionMapViewController` (27m)
  - `CreateViewModel`
  - `CreateViewModelByUraf`
  - `RegisterDownload`
  - `GetDynamicBgmId`
  - `GetDynamicBgmCueInfo`
  - `InitializeView`
  - `InitializeBg`
  - `PlayInView`
  - `PlayInAnimation`
  - `PlayOutView`

### `SingleModeScenarioRamenRegionMapViewViewModel` (5m)
**Getters** (3个):
  - `get_SelectedRegionList`
  - `get_ShouldShowAllClearCheckPointObject`
  - `get_TutorialId`

### `SingleModeScenarioRamenRegionSelectView` (14m)
**Getters** (7个):
  - `get_RegionSelectBg`
  - `get_RegionSelectingContent`
  - `get_CharaDetailButton`
  - `get_AutoSelectRegionButton`
  - `get_AutoSelectRegionOnLabel`
  - `get_MessageTAT`
  - `get_ParticleParent`

### `SingleModeScenarioRamenRegionSelectViewController` (38m)
**Getters** (1个):
  - `get_AutoPlayProxy`

### `SingleModeScenarioRamenRegionSelectViewModel` (21m)
**Getters** (13个):
  - `get_SelectableRegionFeelingList`
  - `get_RegionEffectList`
  - `get_BasicEffect`
  - `get_WritableSelectRegionContext`
  - `get_ReadOnlySelectRegionContext`
  - `get_ReadOnlySelectRegionFeelingContext`
  - `get_EquipSupportCardList`
  - `get_SelectedRegionList`
  - `get_DecideButtonNotificationMessage`
  - `get_SelectRegionNotificationMessage`
  - `get_ShouldShowInvalidEffectWarningDialog`
  - `get_TutorialId`
  - `get_FeelingGaugeGainCountDict`

### `SingleModeScenarioRamenScreenShotParam` (3m)
  - `RegisterDownload`
  - `Load`
  - `GetScreenShotParamList`

### `SingleModeScenarioRamenSettingParam` (9m)
  - `RegisterDownload`
  - `LoadSettingData`
  - `GetPath`
  - `RegisterDownloadReplaceEffectPrefabName`
  - `TryGetReplacePropId`
  - `TryGetBackgroundReplaceTexture`
  - `IsEffectDisabled`
  - `TryGetReplaceEffectPrefabPath`
  - `<TryGetBackgroundReplaceTexture>g__Equals|13_0`

### `SingleModeScenarioRamenStoryOriginalRaceA2U` (4m)
  - `RegisterDownload`
  - `CreateStoryOriginalRaceStartA2U`
  - `GetA2UPath`
  - `GetA2UAudioId`

### `WorkAutoPlayRamenCustomPlanRecord` (9m)
**Getters** (4个):
  - `get_ModifyPreferenceJuniorRegionIdList`
  - `get_ModifyPreferenceClassicRegionIdList`
  - `get_ModifyPreferenceSeniorRegionIdList`
  - `get_ModifyPreferenceSelectUrafEffectType`

### `WorkAutoPlayRamenDefaultPlanRecord` (9m)
**Getters** (4个):
  - `get_ModifyPreferenceJuniorRegionIdList`
  - `get_ModifyPreferenceClassicRegionIdList`
  - `get_ModifyPreferenceSeniorRegionIdList`
  - `get_ModifyPreferenceSelectUrafEffectType`

### `WorkSingleModeChangeParameterInfoScenarioRamen` (9m)
**Getters** (7个):
  - `get_EvaluationInfoArray`
  - `get_CommandInfoArray`
  - `get_FeelingTurnInfoArray`
  - `get_FeelingInfoArray`
  - `get_SpecialFeelingNum`
  - `get_CheckPointPt`
  - `get_UrafEffectInfo`

### `WorkSingleModeScenarioRamen` (19m)
**Getters** (1个):
  - `get_DataSet`


## ObscuredSingleModeRamenDataSet (26m)

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
  - `get_TrainingExecInfoArray`
  - `set_TrainingExecInfoArray`
  - `get_ReduceBaseTurnInfoArray`
  - `set_ReduceBaseTurnInfoArray`
  - `get_AllSelectedRegionIdArray`
  - `set_AllSelectedRegionIdArray`

## 拉面杯指令系统

### `ObscuredSingleModeRamenCommandFeelingInfo` (6m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_CommandId`
  - `set_CommandId`
  - `get_FeelingId`
  - `set_FeelingId`

### `ObscuredSingleModeRamenCommandFeelingInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenCommandInfo` (6m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_CommandId`
  - `set_CommandId`
  - `get_ParamsIncDecInfoArray`
  - `set_ParamsIncDecInfoArray`

### `ObscuredSingleModeRamenCommandInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `RamenCommandSelector` (5m)
  - `SelectAction`
  - `SelectNonConsumptionAction`
  - `GenerateCommandAndTrainingAction`
  - `SelectTraining`
  - `GenerateAutoPlayOutingActionInfo`


## 拉面杯Feeling(心情)系统

### `ObscuredSingleModeRamenFeeling` (4m)
  - `get_FeelingIndex`
  - `set_FeelingIndex`
  - `get_FeelingId`
  - `set_FeelingId`

### `ObscuredSingleModeRamenFeelingExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenFeelingReduceTurnInfo` (6m)
  - `get_CommandType`
  - `set_CommandType`
  - `get_CommandId`
  - `set_CommandId`
  - `get_FeelingTurnArray`
  - `set_FeelingTurnArray`

### `ObscuredSingleModeRamenFeelingReduceTurnInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenFeelingTurnInfo` (4m)
  - `get_FeelingId`
  - `set_FeelingId`
  - `get_RemainTurn`
  - `set_RemainTurn`

### `ObscuredSingleModeRamenFeelingTurnInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`


## 拉面杯检查点系统

### `ObscuredSingleModeRamenCheckPointInfo` (4m)
  - `get_CheckPointType`
  - `set_CheckPointType`
  - `get_ResultState`
  - `set_ResultState`

### `ObscuredSingleModeRamenCheckPointInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `SingleModeScenarioRamenCheckPointProgressUpParameterInfo` (5m)
  - `get_IsSuccess`
  - `get_IsGreatSuccess`
  - `get_IsGroupPlay`
  - `get_MessageText`
  - `CreateA2UContext`

### `SingleModeScenarioRamenCheckPointProgressUpA2U` (9m)
  - `get_FlashPath`
  - `get_PlayTime`
  - `PlaySE`
  - `LoadPlayer`
  - `PlayInMotion`
  - `PlayOut`
  - `PlayOutInternal`
  - `get_InMotionName`
  - `get_OutMotionName`

### `SingleModeScenarioRamenCheckPointTopViewModel` (5m)
  - `get_EnableBackKey`
  - `set_EnableBackKey`
  - `get_SingleModeTrainingCharaId`
  - `get_TutorialId`
  - `SetEnableBackKey`

### `SingleModeScenarioRamenCheckPointTopView` (0m)

### `SingleModeScenarioRamenCheckPointTopViewControllerProxy` (1m)
  - `OnClickStart`

### `SingleModeScenarioRamenCheckPointTopViewController` (27m)
  - `get_AutoPlayProxy`
  - `set_AutoPlayProxy`
  - `GetByRandomRegion`
  - `GetDynamicBgmId`
  - `GetDynamicBgmCueInfo`
  - `get_ViewModel`
  - `CreateViewModel`
  - `get_PartsViewModel`
  - `CreatePartsViewModel`
  - `get_Top3DModel`
  - `Create3dModel`
  - `get_ExtraEditionImageController`
  - `CreateTwinkleExtraEdition`
  - `RegisterDownload`
  - `OnClickOsBackKey`

### `RamenCheckPointDecideAction` (1m)
  - `ExecuteAction`

### `RamenCheckPointResultAction` (1m)
  - `ExecuteAction`

### `RamenCheckPointResultWaitAction` (1m)
  - `ExecuteAction`

### `RamenCheckPointTwinkleDecideAction` (1m)
  - `ExecuteAction`

### `RamenCheckPointResultState` (1m)
  - `OnStateEnter`

### `RamenCheckPointResultWaitTapState` (2m)
  - `OnStateEnter`
  - `<OnStateEnter>b__1_0`

### `RamenCheckPointTwinkleState` (1m)
  - `OnStateEnter`

### `RamenCheckPointViewState` (1m)
  - `OnStateEnter`

### `DialogSingleModeScenarioRamenCheckPointResultViewModel` (10m)
  - `get_MiniCharaId`
  - `get_IsGreatSuccess`
  - `get_IsSuccess`
  - `get_AnimationFrameType`
  - `get_ResultTypeSeAudioId`
  - `get_InLabel`
  - `get_InLastCheckPointTextLabel`
  - `get_CheckPointTextMotName`
  - `get_MiniCharaImageAnimationLabel`
  - `get_MiniCharaProductType`

### `DialogSingleModeScenarioRamenCheckPointResultProxy` (2m)
  - `get_IsActiveNextButton`
  - `OnClickNext`

### `DialogSingleModeScenarioRamenCheckPointResult` (24m)
  - `get_TitleFlashRoot`
  - `get_CharaImage`
  - `get_CharaImageAnimation`
  - `get_AutoPlayProxy`
  - `set_AutoPlayProxy`
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `RegisterDownload`
  - `PushDialog`
  - `OnChangeSortingOrder`
  - `Initialize`
  - `Setup`
  - `SetupAnimationFrameCross`
  - `SetupTitleA2U`

### `DialogSingleModeScenarioRamenCheckPointTwinkleViewModel` (20m)
  - `RegisterDownloadAssetPathList`
  - `get_TitleTextList`
  - `get_ReportText`
  - `get_ImpressionTextList`
  - `get_ServeNameText`
  - `get_IntroductionText`
  - `Replace`
  - `get_LeftPhotoTexture`
  - `get_RightPhotoTexture`
  - `get_TitleBgTexture`
  - `get_FooterBgTexture`
  - `get_PopularityTexture`
  - `get_ServeIconTexture`
  - `get_ServeNameTexture`
  - `get_ImpressionTitleTexture`

### `DialogSingleModeScenarioRamenCheckPointTwinkleProxy` (2m)
  - `get_CanClose`
  - `OnClickNext`

### `DialogSingleModeScenarioRamenCheckPointTwinkle` (18m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `RegisterDownload`
  - `PushDialog`
  - `get_AutoPlayProxy`
  - `set_AutoPlayProxy`
  - `Initialize`
  - `Setup`
  - `SetupPhoto`
  - `SetupTitle`
  - `SetupReport`
  - `SetupFooter`
  - `SetupImpression`
  - `SetupIntroduction`

### `IPartsSingleModeScenarioRamenCheckPointTopViewInternal` (9m)
  - `Setup`
  - `Release`
  - `PrePlayIn`
  - `PlayIn`
  - `UpdateView`
  - `LateUpdateView`
  - `OnClickStartButton`
  - `OnClickSkipButton`
  - `OnClickCheckPointResultButton`

### `PartsSingleModeScenarioRamenCheckPointCutDirector` (14m)
  - `get_IsSkipped`
  - `set_IsSkipped`
  - `get_IsInitialized`
  - `GetPlayCutType`
  - `GetShuffledScenarioLinkCharaIdArray`
  - `RegisterDownload`
  - `CreateSequenceCutInPlayer`
  - `DownloadResource`
  - `Play`
  - `Skip`
  - `Pause`
  - `CleanUpAll`
  - `UpdateDisplay`
  - `LateUpdateDisplay`

### `IPartsSingleModeScenarioRamenCheckPointInternalHelper` (1m)
  - `SetFocusCameraEnable`

### `PartsSingleModeScenarioRamenCheckPointInternalHelper` (1m)
  - `RegisterDownload`

### `PartsSingleModeScenarioRamenCheckPointTopViewModel` (17m)
  - `get_TrainingCharacter`
  - `get_ShuffledScenarioLinkCharaIdArray`
  - `get_RegionBgTexturePath`
  - `get_TitleLogoTexturePath`
  - `get_TitleLogoTexture`
  - `get_CheckPointResultImage`
  - `get_CheckPointType`
  - `get_TargetRegion`
  - `get_CheckPointTwinkleContext`
  - `get_TwinklePhotoLeftImage`
  - `set_TwinklePhotoLeftImage`
  - `get_TwinklePhotoRightImage`
  - `set_TwinklePhotoRightImage`
  - `AddCheckPointResultLog`
  - `GetLogTitle`

### `PartsSingleModeScenarioRamenCheckPointTopView` (21m)
  - `get_TitleLogoImage`
  - `set_TitleLogoImage`
  - `get_CheckPointResultRoot`
  - `set_CheckPointResultRoot`
  - `get_CheckPointResultImage`
  - `set_CheckPointResultImage`
  - `get_CheckPointResultButton`
  - `set_CheckPointResultButton`
  - `get_StartButton`
  - `set_StartButton`
  - `get_CutInSkipButton`
  - `set_CutInSkipButton`
  - `Create`
  - `Initialize`
  - `Setup`

### `PartsSingleModeScenarioRamenCheckPointTopViewInternal` (31m)
  - `get_CheckPointInternalHelper`
  - `CreateAndResolve`
  - `RegisterDownload`
  - `Setup`
  - `SetupTitleLogo`
  - `SetupCheckPointResult`
  - `SetupStartButtonEffect`
  - `SetupCutInSkipButton`
  - `Release`
  - `PrePlayIn`
  - `PlayIn`
  - `UpdateView`
  - `LateUpdateView`
  - `OnClickStartButton`
  - `StartCheckPointEvent`

### `SingleModeScenarioRamenCheckPointTop3DCamera` (8m)
  - `SetupCameraAndRenderTexture`
  - `ApplyImageEffectParam`
  - `SetUpCamera`
  - `OnCreateTexture`
  - `OnReleaseTexture`
  - `Release`
  - `Setup`
  - `SetVisible`

### `SingleModeScenarioRamenCheckPointTop3DModel` (7m)
  - `get_EnvParamPath`
  - `get_CharacterBuildInfo`
  - `get__dressId`
  - `get_IsDownloadCharacterVoice`
  - `get_IsSingleModePlaying`
  - `get_DownloadVoiceCharaIdList`
  - `get_DownloadVoiceTriggerList`

### `SingleModeScenarioRamenCheckPointTop3DController` (6m)
  - `CreateModel`
  - `Release`
  - `PlayVoiceAndMotion`
  - `SetVisible3D`
  - `SetupBgCamera`
  - `CreateModelInternal`

### `DialogSingleModeScenarioRamenCheckPointProgressViewModel` (10m)
  - `get_CurrentPointText`
  - `get_NeedPointText`
  - `get_RemainPointTitleText`
  - `get_RemainPointText`
  - `get_ShouldShowRemainPoint`
  - `get_ExpectationTextSprite`
  - `get_HeaderBgTexture`
  - `get_CheckPointLogoTexture`
  - `get_CurrentPointFontType`
  - `GetCurrentPassiveViewModel`

### `DialogSingleModeScenarioRamenCheckPointProgress` (9m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `RegisterDownload`
  - `PushDialog`
  - `Initialize`
  - `Setup`
  - `SetupCheckPointPtText`
  - `OnClickExpectationDetailButton`

### `DialogSingleModeScenarioRamenCheckPointProgressByUrafViewModel` (3m)
  - `get_HeaderBgTexture`
  - `get_CheckPointLogoTexture`
  - `GetCurrentPassiveViewModel`

### `DialogSingleModeScenarioRamenCheckPointProgressByUraf` (8m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `RegisterDownload`
  - `PushDialog`
  - `Initialize`
  - `Setup`
  - `OnClickExpectationDetailButton`

### `DialogSingleModeScenarioRamenCheckPointResultPassiveListViewModel` (3m)
  - `GetListItemViewModelList`
  - `NeedDefaultScrollPosBottom`
  - `<GetListItemViewModelList>b__4_0`

### `DialogSingleModeScenarioRamenCheckPointResultPassiveList` (7m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `PushDialog`
  - `Initialize`
  - `Setup`
  - `CreateList`

### `PartsSingleModeScenarioRamenCheckPointCurrentPassiveEffectViewModel` (8m)
  - `get_TrainingText`
  - `get_TagTrainingText`
  - `get_AffinityText`
  - `get_HintText`
  - `GetOrDefaultTrainingUpEffectValue`
  - `GetOrDefaultTagTrainingUpEffect`
  - `GetOrDefaultAffinityRateUpEffect`
  - `GetOrDefaultHintUpEffect`

### `PartsSingleModeScenarioRamenCheckPointCurrentPassiveEffect` (1m)
  - `Setup`

### `PartsSingleModeScenarioRamenCheckPointResultPassiveListItemViewModel` (17m)
  - `get_IsActiveNormalEffect`
  - `get_IsActiveSuccessEffect`
  - `get_IsActiveGreatSuccessEffect`
  - `get_TitleText`
  - `get_ShouldShowGreatSuccessEffect`
  - `get_ResultContentNormalEffectColor`
  - `get_ResultContentSuccessEffectColor`
  - `get_ResultContentGreatSuccessEffectColor`
  - `get_SuccessNeedCheckPointPt`
  - `get_GreatSuccessNeedCheckPointPt`
  - `get_ResultContentSuccessNeedPtText`
  - `get_ResultContentGreatSuccessNeedPtText`
  - `GetResultContentColor`
  - `get_NormalEffectTextList`
  - `get_SuccessEffectTextList`

### `PartsSingleModeScenarioRamenCheckPointResultPassiveListItem` (5m)
  - `Setup`
  - `SetupNormalResultContent`
  - `SetupSuccessResultContent`
  - `SetupGreatSuccessResultContent`
  - `SetResultContent`


## 拉面杯羁绊/评价系统

### `ObscuredSingleModeRamenEvaluationInfo` (4m)
  - `get_TargetId`
  - `set_TargetId`
  - `get_CharaId`
  - `set_CharaId`

### `ObscuredSingleModeRamenEvaluationInfoExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`


## 拉面杯初始化系统

### `ObscuredSingleModeRamenDataSetLoad` (20m)
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

### `ObscuredSingleModeRamenDataSetLoadExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`

### `ObscuredSingleModeRamenDataSetStart` (6m)
  - `get_AutoSelectInfo`
  - `set_AutoSelectInfo`
  - `get_AutoSelectSetInfo`
  - `set_AutoSelectSetInfo`
  - `get_IsCheckedUrafEvent`
  - `set_IsCheckedUrafEvent`

### `ObscuredSingleModeRamenDataSetStartExtensions` (3m)
  - `AsObscured`
  - `AsObscuredArray`
  - `AsObscuredArrayOrEmpty`


## Master数据库拉面杯表 (1个)

| 表名 | 方法数 |
|---|---|
| `MasterSingleMode14TwinkleRamen` | 5 |

## libil2cpp.so 中的拉面杯字符串

## "RamenFeeling" — 无匹配
## "RamenCheckPoint" — 无匹配
## "RamenSozai" — 无匹配
## "RamenRecipe" — 无匹配
## "RamenNoodle" — 无匹配
## "RamenTopping" — 无匹配
## "RamenShop" — 无匹配
## "RamenGuest" — 无匹配
## "RamenUraf" — 无匹配
## "RamenTasting" — 无匹配
## "RamenCommand" — 无匹配
## "RamenSelect" — 无匹配
## "RamenDataSet" — 无匹配