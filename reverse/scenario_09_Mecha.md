# 剧本 9: Mecha (メカ)

**WorkScenario类**: `WorkSingleModeScenarioMecha`
**ObscuredDataSet**: `ObscuredSingleModeMechaDataSet`
---

## 相关类 (92个)

### `Gallop.WorkSingleModeChangeParameterInfoScenarioMecha` (22m)
  - `get_RivalData`
  - `get_PrevRivalData`
  - `set_PrevRivalData`
  - `get_NotUpStatusTypeArray`
  - `set_NotUpStatusTypeArray`
  - `get_SortedNotUpStatusTypeArray`
  - `get_PrevOverdriveInfo`
  - `set_PrevOverdriveInfo`
  - `get_OverdriveRemainNum`
  - `set_OverdriveRemainNum`
  - `get_OverdriveNumMaxFlag`
  - `set_OverdriveNumMaxFlag`
  - `get_OverdriveEnergyNum`
  - `set_OverdriveEnergyNum`
  - `get_TuningPoint`
  - `set_TuningPoint`
  - `Clear`
  - `Set`
  - `IsChangeRivalStatus`
  - `IsChangeRivalStatusAllSame`

### `Gallop.WorkSingleModeScenarioMecha` (28m)
  - `get_BoardInfoList`
  - `GetBoardInfo`
  - `GetChipInfo`
  - `GetChipInfoPoint`
  - `get_TuningPoint`
  - `set_TuningPoint`
  - `get_RivalData`
  - `set_RivalData`
  - `get_OverdriveInfo`
  - `set_OverdriveInfo`
  - `get_CommandInfoList`
  - `set_CommandInfoList`
  - `GetCommandInfo`
  - `GetCommandInfo`
  - `get_SubCommandCharaInfoList`
  - `set_SubCommandCharaInfoList`
  - `GetSubCommandCharaInfoList`
  - `get_HasSubCommandCharaInfo`
  - `get_UpgradeRaceResultList`
  - `set_UpgradeRaceResultList`

### `Gallop.SingleModeScenarioMechaGainMainViewChangeParameterInfo` (7m)
  - `IsNeed`
  - `get_Mecha`
  - `get_IsGroupPlay`
  - `get_MessageText`
  - `CreateA2UContext`
  - `<.ctor>g__AppendStatusLimitMessage|4_0`
  - `<.ctor>g__Append|4_1`

### `Gallop.SingleModeScenarioMechaGainMainViewA2U` (9m)
  - `get_FlashPath`
  - `get_PlayTime`
  - `PlaySE`
  - `LoadPlayer`
  - `Play`
  - `PlayOverdriveRcoveryAndProgress`
  - `GetTextController`
  - `PlayOut`
  - `<PlayOut>b__11_0`

### `Gallop.SingleModeScenarioMechaGainOverdriveRecoveryParameterInfo` (3m)
  - `get_IsGroupPlay`
  - `get_MessageText`
  - `CreateA2UContext`

### `Gallop.SingleModeScenarioMechaGainOverdriveRecoveryA2U` (6m)
  - `get_FlashPath`
  - `get_PlayTime`
  - `PlaySE`
  - `LoadPlayer`
  - `PlayInMotion`
  - `get_OutMotionName`

### `Gallop.SingleModeScenarioMechaGainRivalStatusLimitParameterInfo` (4m)
  - `get_Mecha`
  - `get_IsGroupPlay`
  - `get_MessageText`
  - `CreateA2UContext`

### `Gallop.SingleModeScenarioMechaGainRivalStatusLimitA2U` (14m)
  - `get_Mecha`
  - `get_FlashPath`
  - `get_PlayTime`
  - `get_OutMotionName`
  - `LoadPlayer`
  - `Play`
  - `PlayInMotion`
  - `PlayOutA2U`
  - `PlayOut`
  - `<Play>b__12_0`
  - `<Play>b__12_1`
  - `<Play>b__12_2`
  - `<Play>b__12_3`
  - `<PlayOut>b__15_0`

### `Gallop.SingleModeScenarioMechaGainTuningPointParameterInfo` (3m)
  - `get_IsGroupPlay`
  - `get_MessageText`
  - `CreateA2UContext`

### `Gallop.SingleModeScenarioMechaGainTuningPointA2U` (3m)
  - `get_FlashPath`
  - `get_PlayTime`
  - `Play`

### `Gallop.SingleModeScenarioMechaJoinMemberParameterInfo` (4m)
  - `get_IsGroupPlay`
  - `get_MessageText`
  - `set_MessageText`
  - `CreateA2UContext`

### `Gallop.SingleModeScenarioMechaJoinMemberA2U` (8m)
  - `get_FlashPath`
  - `get_PlayTime`
  - `get_OutMotionName`
  - `PlaySE`
  - `LoadPlayer`
  - `Play`
  - `GetSeLabel`
  - `PlayInMotion`

### `Gallop.SingleModeScenarioMechaResearchProgressUpParameterInfo` (6m)
  - `get_IsGroupPlay`
  - `get_MessageText`
  - `get_ProgressResultType`
  - `get_IsOverdriveRecovery`
  - `set_IsOverdriveRecovery`
  - `CreateA2UContext`

### `Gallop.SingleModeScenarioMechaResearchProgressUpA2U` (9m)
  - `get_FlashPath`
  - `get_PlayTime`
  - `PlaySE`
  - `LoadPlayer`
  - `PlayInMotion`
  - `get_OutMotionName`
  - `PlayOut`
  - `GetLabelId`
  - `<PlayInMotion>b__11_0`

### `Gallop.PartsSingleModeScenarioMechaImageNumberModel` (6m)
  - `get_EnableBonus`
  - `get_ImageNumberPrefabPath`
  - `get_BonusA2UPath`
  - `RegisterDownload`
  - `OnSetBonusValueExtend`
  - `SetOverdriveEffect`

### `Gallop.PartsSingleModeScenarioMechaMainStablesPanelButtonModel` (9m)
  - `get_ButtonSprite`
  - `get_LockButtonSprite`
  - `get_TextImageSprite`
  - `get_ButtonIconIdleSprite`
  - `get_ButtonIconEnterSprite`
  - `get_IsEnableUniqCommand`
  - `get_Interactable`
  - `get_NotificationMessage`
  - `GetSprite`

### `Gallop.PartsSingleModeScenarioMechaMainStablesPanelButton` (3m)
  - `get_Model`
  - `Create`
  - `Setup`

### `Gallop.PartsSingleModeScenarioMechaMainView` (61m)
  - `get_OverdriveButtonA2U`
  - `RegisterDownload`
  - `Create`
  - `Setup`
  - `SetupSimpleView`
  - `SetupCommon`
  - `Update`
  - `UpdateSyncMaxAnimation`
  - `SetupProgressGauge`
  - `SetupTotalLevelGauge`
  - `SetupTotalGaugeMiddleLine`
  - `SetupTotalLevelGaugeMaxEffect`
  - `LoadTotalLevelGaugeMaxEffect`
  - `UpdateTotalLevelGaugeRemainLevel`
  - `SetTrainingTotalLevelGaugeRemainLevel`
  - `PlayTotalLevelGaugeTopTAT`
  - `StopTotalLevelGaugeTopTAT`
  - `IsMaxTotalLevelGauge`
  - `SetRemainTotalLevelText`
  - `SetGaugeImage`

### `Gallop.PartsSingleModeScenarioMechaOverdriveButton` (16m)
  - `RegisterDownload`
  - `Setup`
  - `Initialize`
  - `SetStock`
  - `SetGauge`
  - `SetGaugeRange`
  - `SetGauge`
  - `SetGaugeUp`
  - `SetGaugeUpOff`
  - `SetGaugeUpStock`
  - `PlayGaugeUp`
  - `PlayStockUpOut`
  - `SetEffect`
  - `SetEffectOff`
  - `SetActionLamp`
  - `IsShowOverdriveBurstBadge`

### `Gallop.PartsSingleModeScenarioMechaOverdriveRecoveryAndProgressUp` (11m)
  - `get_SortingLayerName`
  - `RegisterDownload`
  - `CreateAndPlay`
  - `Create`
  - `Setup`
  - `Play`
  - `PlayOut`
  - `GetBluerDuration`
  - `PlayMessageWindow`
  - `DestroyMessageWindow`
  - `<PlayOut>b__11_0`

### `Gallop.PartsSingleModeScenarioMechaProgressGaugeA2U` (5m)
  - `RegisterDownload`
  - `Setup`
  - `Initialize`
  - `SetResult`
  - `PlaySuperSuccess`

### `Gallop.PartsSingleModeScenarioMechaRivalIncDec` (1m)
  - `SetValue`

### `Gallop.PartsSingleModeScenarioMechaStatusValue` (20m)
  - `get_MaxTweenAnimation`
  - `Initialize`
  - `Setup`
  - `Setup`
  - `SetLevelText`
  - `SetLevelLimitText`
  - `SetupSprite`
  - `OnDestroy`
  - `SetupMax`
  - `PlayLoopMaxTweenAnimation`
  - `SetupAddEffect`
  - `GetLevel`
  - `GetLevelLimit`
  - `SetIncDec`
  - `SetActivateIncDec`
  - `IsActivateIncDec`
  - `IsActiveMaxIcon`
  - `PlayRivalStatusUp`
  - `PlayInMaxTweenAnimation`
  - `PlayLevelLimitUp`

### `Gallop.SingleModeMainCharaScenarioMechaController` (11m)
  - `get_UseTrainingViewFocusCameraScenarioPreset`
  - `get_UseRealTimeShadow`
  - `SetupTrainingBGCharaModel`
  - `AfterSetupImageEffect`
  - `OnClearBgModel`
  - `OnSetVisibleTrainingModel`
  - `SetupMechaModel`
  - `PlayMechaCharaCutt`
  - `TryGetRemoveCuttData`
  - `SetupFocusCameraCustomRender`
  - `CreateCuttRenderTexture`

### `Gallop.TrainingBackupDataScenarioMecha` (0m)

### `Gallop.SingleModeMainTrainingDecideConfirmScenarioMecha` (2m)
  - `CreateExecCommandAlertConfirmActionQueue`
  - `OverdriveWarning`

### `Gallop.SingleModeMainViewHeaderScenarioMechaModel` (7m)
  - `get_RemainTurnA2UPath`
  - `GetBaseFrameSprite`
  - `get_TitleOutlineColorType`
  - `get_NeedMultiTurn`
  - `GetNextScenarioScheduleTurnNum`
  - `IsDisable`
  - `OnClickScenarioScheduleButton`

### `Gallop.SingleModeMainViewHpGaugeA2UScenarioMecha` (6m)
  - `get_A2UPath`
  - `RegisterDownload`
  - `PlayScenarioGimmick`
  - `PlayHpRecoverAnimation`
  - `SetEffect`
  - `<PlayHpRecoverAnimation>b__12_1`

### `Gallop.SingleModeMainViewMotivationButtonA2UScenarioMecha` (7m)
  - `RegisterDownloadScenario`
  - `OnCreateA2UScenario`
  - `OnActionOutMotivationUpScenario`
  - `CreateOverdriveEffect`
  - `PlayOverdriveEffect`
  - `SetOneShotOverdrive`
  - `<PlayOverdriveEffect>b__8_1`

### `Gallop.SingleModeMainViewScenarioMechaController` (20m)
  - `RegisterDownload`
  - `SetupCore`
  - `UpdateCommonUIPosition`
  - `OnClickTuning`
  - `UpdateView`
  - `LateUpdateView`
  - `GetShowTurnStartNotice`
  - `IsNeedPlayProgressGaugeSuperSuccess`
  - `SaveMainViewProgressResultType`
  - `PlayProgressGaugeSuperSuccess`
  - `IsNeedShowOverdrivePowerUp`
  - `ShowOverdrivePowerUp`
  - `IsNeedShowOverdriveBurstContinue`
  - `IsShowNextTargetLevel`
  - `ShowOverdriveBurstContinue`
  - `TutorialCommandSelectStart`
  - `PlayCutIn`
  - `FinalizeCutIn`
  - `ShowNextTargetLevelNotice`
  - `ShowNextTargetLevelSetup`

### `Gallop.SingleModeScenarioMechaMainViewStablesPanelModel` (3m)
  - `CreateScenarioStablesPanelModel`
  - `CreateScenarioCommandButton`
  - `get_UseScenarioButtonOnlyRaceEntry`

### `Gallop.SingleModeMainViewTrainingFooterItemA2UScenarioMecha` (9m)
  - `get_PlayerA2UPath`
  - `get_TipsBadgeFlashPlayerPath`
  - `RegisterDownloadScenario`
  - `PlayIn`
  - `SetupTrainingButtonBadgeScenario`
  - `SetupOverdrive`
  - `CreateOverdriveEffect`
  - `SetupTrainingFailureRateScenario`
  - `SetupTrainingFailureRateScenarioMecha`

### `Gallop.SingleModeMainViewTrainingHeaderA2UScenarioMecha` (3m)
  - `get_FlashPlayerPath`
  - `ApplyTrainingInfo`
  - `SetupStatusLevelBonusIcon`

### `Gallop.DialogSingleModeScenarioMechaBoardDetail` (16m)
  - `GetFormType`
  - `GetParentType`
  - `PushDialog`
  - `Setup`
  - `SetupBoard`
  - `SetupHeader`
  - `SetupHeaderBoardList`
  - `GetBoardIndex`
  - `OnChangeNextBoard`
  - `OnChangePrevBoard`
  - `OnChangeBoard`
  - `SetupTab`
  - `OnChangeTabToggle`
  - `SetupOverdriveList`
  - `SetupChipList`
  - `<GetBoardIndex>b__26_0`

### `Gallop.DialogSingleModeScenarioMechaChipDetail` (3m)
  - `GetFormType`
  - `GetParentType`
  - `PushDialog`

### `Gallop.DialogSingleModeScenarioMechaChipUnlock` (6m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `PushDialog`
  - `Setup`
  - `CreateEffect`

### `Gallop.DialogSingleModeScenarioMechaDataList` (7m)
  - `GetFormType`
  - `GetParentType`
  - `Setup`
  - `SetupRemainLevel`
  - `SetupScheduleList`
  - `RegisterDownload`
  - `PushDialog`

### `Gallop.DialogSingleModeScenarioMechaEndLiveConfirm` (6m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`

### `Gallop.DialogSingleModeScenarioMechaNextScheduleListModel` (5m)
  - `RegisterDownload`
  - `get_FrameType`
  - `get_DialogPrefabPath`
  - `get_LogoImagePath`
  - `get_ScheduleListItemModels`

### `Gallop.DialogSingleModeScenarioMechaOverdriveComfirm` (4m)
  - `GetFormType`
  - `GetParentType`
  - `Setup`
  - `PushDialog`

### `Gallop.DialogSingleModeScenarioMechaOverdriveContinue` (7m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `PushDialog`
  - `Setup`
  - `OnChangeDialogSortingOrder`
  - `OnEndAnimation`

### `Gallop.DialogSingleModeScenarioMechaOverdriveDetail` (4m)
  - `GetFormType`
  - `GetParentType`
  - `Setup`
  - `PushDialog`

### `Gallop.DialogSingleModeScenarioMechaOverdrivePowerUp` (7m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `PushDialog`
  - `Setup`
  - `CreateA2U`
  - `OnChangeDialogSortingOrder`

### `Gallop.DialogSingleModeScenarioMechaOverdriveWarning` (5m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `PushDialog`
  - `Setup`

### `Gallop.DialogSingleModeScenarioMechaRivalDetail` (13m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `PushDialog`
  - `Setup`
  - `SetHeaderInfo`
  - `SetupTotalLevelTargetLevel`
  - `SetupTab`
  - `OnChangeTabToggle`
  - `SetupEffectChipList`
  - `SetupStatus`
  - `SetupTrainingBonus`
  - `SetupCharaDetailButton`

### `Gallop.DialogSingleModeScenarioMechaTuningConfirm` (3m)
  - `GetFormType`
  - `GetParentType`
  - `PushDialog`

### `Gallop.DialogSingleModeScenarioMechaTuningDetail` (4m)
  - `GetFormType`
  - `GetParentType`
  - `PushDialog`
  - `Setup`

### `Gallop.DialogSingleModeScenarioMechaUpgradeRaceResult` (19m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `PushDialog`
  - `Setup`
  - `SetupAnimationFrameCross`
  - `OnChangeSortingOrder`
  - `SetupTitleA2U`
  - `SetupMiniChara`
  - `OnButtonClicked`
  - `PlayIn`
  - `PlayInTitle`
  - `PlayResultTypeSe`
  - `PlayInMiniChara`
  - `PlayInNextButton`
  - `OnDestroy`
  - `<OnChangeSortingOrder>g__UpdateFlashPlayerOrder|23_0`
  - `<SetupTitleA2U>g__CreateActionPlayer|24_0`
  - `<SetupMiniChara>g__GetMotionCharaType|25_0`

### `Gallop.PartsSingleModeScenarioMechaBoardListItem` (1m)
  - `Setup`

### `Gallop.PartsSingleModeScenarioMechaChipEffectList` (21m)
  - `Setup`
  - `Setup`
  - `Setup`
  - `SetupOverdrive`
  - `SetupChipListItem`
  - `SetupPlayAnimation`
  - `ForceRebuildLayout`
  - `CheckScrollFocus`
  - `CheckItemList`
  - `RemoveDestroyItemList`
  - `SetScrollTargetPosition`
  - `PlayNewEffectScroll`
  - `PlayFocusScroll`
  - `ClearFocusScroll`
  - `SetEnableScrollMovement`
  - `SetupMain`
  - `SetupRivalStatusBonus`
  - `<ForceRebuildLayout>b__25_0`
  - `<PlayFocusScroll>b__31_0`
  - `<PlayFocusScroll>b__31_1`

### `Gallop.PartsSingleModeScenarioMechaChipListItem` (3m)
  - `Setup`
  - `SetupEffectList`
  - `<SetupEffectList>b__5_2`

### `Gallop.PartsSingleModeScenarioMechaChipPointList` (2m)
  - `Setup`
  - `SetupLock`

### `Gallop.PartsSingleModeScenarioMechaChipPointListItem` (1m)
  - `Setup`

### `Gallop.PartsSingleModeScenarioMechaChipUnlock` (1m)
  - `Setup`

### `Gallop.PartsSingleModeScenarioMechaDataListItemModel` (13m)
  - `get_PrefabPath`
  - `get_ScheduleTurn`
  - `get_IsNextSchedule`
  - `get_HasResultData`
  - `GetResultData`
  - `get_HeaderImageSprite`
  - `get_TitleText`
  - `get_ScheduleOpenText`
  - `get_ResultImageSprite`
  - `GetRemainTurnText`
  - `get_ResultAudioId`
  - `IsGreat`
  - `get_IsNeedPlayWinEffectOnResultAnimation`

### `Gallop.PartsSingleModeScenarioMechaEffectedChipEffectListItem` (15m)
  - `get_Index`
  - `set_Index`
  - `Setup`
  - `SetupBadge`
  - `PlayChangeTextIfChenged`
  - `PlayInsertAnimation`
  - `PlayInsertAnimationAfter`
  - `OnCompletePlayInsertAnimation`
  - `PlayInsertAnimationCompleteEffect`
  - `get_IsNew`
  - `PlayUpDownAnimation`
  - `<PlayChangeTextIfChenged>b__18_0`
  - `<PlayChangeTextIfChenged>b__18_1`
  - `<PlayInsertAnimationCompleteEffect>b__22_0`
  - `<PlayUpDownAnimation>g__OnComplete|25_0`

### `Gallop.PartsSingleModeScenarioMechaEffectedChipListItem` (27m)
  - `get_ContentTop`
  - `get_ContentBottom`
  - `get_MasterChipEffectList`
  - `set_MasterChipEffectList`
  - `get_ChipModel`
  - `set_ChipModel`
  - `get_ChipId`
  - `Setup`
  - `Update`
  - `SetupPointDot`
  - `SetupEffectItemList`
  - `IsChangeEffected`
  - `IsChangeRemoveEffected`
  - `PlayInsertAnimationIfNew`
  - `PlayInsertAnimation`
  - `CoroutinePlayInsertAnimation`
  - `OnCompletePlayInsertAnimation`
  - `StopInsertAnimationCompleteEffect`
  - `PlayInsertAnimationCompleteEffect`
  - `PlayDestroyAnimation`

### `Gallop.PartsSingleModeScenarioMechaOverdriveCutIn` (17m)
  - `GetBgEffectPath`
  - `GetBurstBgEffectPath`
  - `RegisterDownload`
  - `CreateAndPlay`
  - `CreateAndPlay`
  - `Setup`
  - `Play`
  - `CreateCutInBgEffect`
  - `CreateA2UText`
  - `get_TargetCanvas`
  - `get_SortingLayerName`
  - `get_SortingOrder`
  - `OnEndCutIn`
  - `UpdateView`
  - `LateUpdateView`
  - `DestroyCutIn`
  - `<Play>b__14_0`

### `Gallop.PartsSingleModeScenarioMechaOverdriveSequence` (24m)
  - `RegisterDownload`
  - `CreateAndPlay`
  - `SetHeaderAndFooterSortingOrder`
  - `ResetHeaderAndFooterSortingOrder`
  - `GetMessagePlayTime`
  - `CreateChangeParameterInfoList`
  - `CreateChangeParameterInfoListByChip`
  - `CreateChangeParameterInfoByChipEffect`
  - `IsMigraine`
  - `OnParamChangeStart`
  - `PlayA2UText`
  - `GetA2ULabelNum`
  - `GetA2UInLabel`
  - `GetA2UOutLabel`
  - `SetupA2UBoard`
  - `SetupA2UBurst`
  - `OnPlayOutFrameAnimation`
  - `GetHp`
  - `GetPrevHp`
  - `GetDeltaHp`

### `Gallop.PartsSingleModeScenarioMechaProgressGauge` (1m)
  - `Setup`


## WorkSingleModeScenarioMecha

方法数: 28

  - `get_BoardInfoList`
  - `GetBoardInfo`
  - `GetChipInfo`
  - `GetChipInfoPoint`
  - `get_TuningPoint`
  - `set_TuningPoint`
  - `get_RivalData`
  - `set_RivalData`
  - `get_OverdriveInfo`
  - `set_OverdriveInfo`
  - `get_CommandInfoList`
  - `set_CommandInfoList`
  - `GetCommandInfo`
  - `GetCommandInfo`
  - `get_SubCommandCharaInfoList`
  - `set_SubCommandCharaInfoList`
  - `GetSubCommandCharaInfoList`
  - `get_HasSubCommandCharaInfo`
  - `get_UpgradeRaceResultList`
  - `set_UpgradeRaceResultList`
  - `GetUpgradeRaceResult`
  - `Apply`
  - `IsUsingOverdrive`
  - `GetTotalTuningPoint`
  - `GetBoardTuningPoint`
  - `GetTrendBoardId`
  - `<GetTotalTuningPoint>b__46_0`
  - `<GetTrendBoardId>b__48_0`

## 剧本独立属性变化 (22m, 9 getters)

  - `get_RivalData`
  - `get_PrevRivalData`
  - `get_NotUpStatusTypeArray`
  - `get_SortedNotUpStatusTypeArray`
  - `get_PrevOverdriveInfo`
  - `get_OverdriveRemainNum`
  - `get_OverdriveNumMaxFlag`
  - `get_OverdriveEnergyNum`
  - `get_TuningPoint`

## lib.rs相关引用

```
6=>"Arc", 7=>"Sport", 8=>"Cook", 9=>"Mecha", 10=>"Legend",
9=>"WorkSingleModeScenarioMecha", 10=>"WorkSingleModeScenarioLegend",
9 => "WorkSingleModeScenarioMecha",
```