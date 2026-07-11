# 剧本 5: Venus (ヴィーナス)

**WorkScenario类**: `WorkSingleModeScenarioVenus`
**ObscuredDataSet**: `ObscuredSingleModeVenusDataSet`
---

## 相关类 (87个)

### `Gallop.MasterRaceJikkyoBaseVenus` (8m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithMode`
  - `_SelectWithMode`
  - `_CreateOrmByQueryResultWithMode`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleModeVenusCrystalGroup` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithCharaId`
  - `_SelectWithCharaId`
  - `GetListWithCharaId`
  - `MaybeListWithCharaId`
  - `_ListSelectWithCharaId`
  - `_CreateOrmByQueryResultWithCharaId`
  - `Unload`

### `Gallop.MasterSingleModeVenusSpiritEffect` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithSpiritEffectGroupId`
  - `_SelectWithSpiritEffectGroupId`
  - `GetListWithSpiritEffectGroupId`
  - `MaybeListWithSpiritEffectGroupId`
  - `_ListSelectWithSpiritEffectGroupId`
  - `_CreateOrmByQueryResultWithSpiritEffectGroupId`
  - `Unload`

### `Gallop.MasterSingleModeVenusSpiritGroup` (3m)
  - `Get`
  - `_SelectOne`
  - `Unload`

### `Gallop.MasterSingleModeVenusSpraceData` (9m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithRaceGroupId`
  - `_SelectWithRaceGroupId`
  - `_CreateOrmByQueryResultWithRaceGroupId`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `GetSpraceList`

### `Gallop.WorkSingleModeChangeParameterInfoScenarioVenus` (10m)
  - `get_SpiritInfoList`
  - `get_AppearVenus`
  - `get_UsedSpirit`
  - `get_InfoLevelUp`
  - `Clear`
  - `Set`
  - `SetSpitirInfo`
  - `SetAppearVenus`
  - `SetUseSpirit`
  - `SetLevelUpSpirit`

### `Gallop.WorkSingleModeScenarioVenus` (14m)
  - `get_SpiritInfoArray`
  - `get_VenusSpiritActiveEffectInfoArray`
  - `get_VenusCharaInfoArray`
  - `get_VenusCharaCommandInfoArray`
  - `get_VenusRaceHistryArray`
  - `get_RaceStartInfo`
  - `get_RaceScenario`
  - `get_VenusRaceCondition`
  - `get_LiveItemId`
  - `get_RaceRewardInfo`
  - `Apply`
  - `Apply`
  - `Apply`
  - `GetHighLevelVenusId`

### `Gallop.DialogSingleModeScenarioVenusActiveSpiritListModel` (13m)
  - `get_CurrentActiveSpiritEffect`
  - `get_VenusCharaInfo`
  - `get_MasterSingleModeVenusCrystalGroup`
  - `get_HasCurrentActiveSpiritEffect`
  - `get_ActiveMasterSingleModeVenusCrystalGroupList`
  - `get_ActiveVenusPassiveEffectGroupIdList`
  - `get_ActiveVenusPassiveMasterSingleModeVenusSpiritEffectList`
  - `get_HasActiveVenusPassiveEffect`
  - `get_ActiveSpiritEffectGroupIdList`
  - `get_ActiveMasterSingleModeVenusSpiritEffectList`
  - `get_HasActiveSpiritEffect`
  - `<get_VenusCharaInfo>b__3_0`
  - `<get_MasterSingleModeVenusCrystalGroup>b__5_0`

### `Gallop.DialogSingleModeScenarioVenusActiveSpiritList` (16m)
  - `GetFormType`
  - `GetParentType`
  - `get_Model`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`
  - `SetupActiveVenusSpiritEffect`
  - `SetupVenusName`
  - `SetupVenusSpiritIcon`
  - `SetupVenusActiveEffectText`
  - `SetupActiveVenusPassiveEffectList`
  - `CreateActiveVenusPassiveEffectList`
  - `SetupSpiritEffectList`
  - `CreateActiveSpiritEffectList`
  - `CreateSpiritEffectListItem`
  - `CreateParameterUpSpiritEffectListItem`

### `Gallop.DialogSingleModeScenarioVenusScenarioRaceContinue` (11m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `Open`
  - `Setup`
  - `OnClickContinue`
  - `GetContinueItemNum`
  - `OnClickCancel`
  - `SetOnClickSkillSelectButton`
  - `<OnClickCancel>b__18_0`
  - `<SetOnClickSkillSelectButton>b__19_0`

### `Gallop.DialogSingleModeScenarioVenusScenarioRaceList` (13m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `RegisterDownload`
  - `Setup`
  - `PlayContentsIn`
  - `PlayWinAnimation`
  - `PlayNextScheduleAnimation`
  - `ShowNextButton`
  - `TriggerClose`
  - `Open`
  - `<PlayContentsIn>b__21_0`
  - `<PlayContentsIn>b__21_1`

### `Gallop.DialogSingleModeScenarioVenusScenarioRaceSchedule` (6m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `Open`
  - `GetDisplayDegree`
  - `Setup`

### `Gallop.DialogSingleModeScenarioVenusSpiritAcquired` (10m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `Open`
  - `CreateDialogData`
  - `Setup`
  - `SetupImage`
  - `SetupTitleImage`
  - `SetupTitleFlash`
  - `OnRightButtonCallBack`

### `Gallop.DialogSingleModeScenarioVenusSpiritActivateConfirmModel` (7m)
  - `get_VenusSpiritInfo`
  - `get_VenusCharaInfo`
  - `get_NextLevelMasterSingleModeVenusCrystalGroup`
  - `get_NextVenusLevel`
  - `get_IsLevelMax`
  - `<get_VenusCharaInfo>b__3_0`
  - `<get_NextLevelMasterSingleModeVenusCrystalGroup>b__5_0`

### `Gallop.DialogSingleModeScenarioVenusSpiritActivateConfirm` (14m)
  - `GetFormType`
  - `GetParentType`
  - `get_Model`
  - `RegisterDownload`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`
  - `SetupVenusName`
  - `SetupVenusSpiritIcon`
  - `SetupVenusActiveEffectText`
  - `SetupVenusLevel`
  - `SetupButtonEffect`
  - `OnRightButtonCallBack`
  - `<SetupVenusLevel>b__22_0`

### `Gallop.DialogSingleModeScenarioVenusSpiritEffectDetailModel` (8m)
  - `get_EnableAcquireSpirit`
  - `get_EnableTatSpiritIcon`
  - `get_SpiritInfo`
  - `get_VenusCharaInfo`
  - `get_NextVenusLevel`
  - `get_NextLevelMasterSingleModeVenusCrystalGroup`
  - `<get_VenusCharaInfo>b__11_0`
  - `<get_NextLevelMasterSingleModeVenusCrystalGroup>b__15_0`

### `Gallop.DialogSingleModeScenarioVenusSpiritEffectDetail` (17m)
  - `GetFormType`
  - `GetParentType`
  - `PushDialog`
  - `CreateDialogData`
  - `CreateDialogDataSpiritFragment`
  - `CreateDialogDataSpirit`
  - `CreateDialogDataVenusSpirit`
  - `Setup`
  - `SetupWithHash`
  - `SetupSpiritRoot`
  - `SetupSpiritIcon`
  - `SetupSpiritName`
  - `SetupSpiritEffectText`
  - `SetupHeaderTAT`
  - `CreateSpiritEnableAcquireTAT`
  - `OnRightButtonCallBack`
  - `OnChangeSortingOrder`

### `Gallop.DialogSingleModeScenarioVenusSpiritHistoryModel` (6m)
  - `get_SpiritHistoryInfoList`
  - `get_CanScroll`
  - `get_CurrentScrollIndex`
  - `set_CurrentScrollIndex`
  - `get_CurrentSpiritHistoryInfo`
  - `SendSingleModeVenusVenusSpiritHistoryRequest`

### `Gallop.DialogSingleModeScenarioVenusSpiritHistory` (19m)
  - `GetFormType`
  - `GetParentType`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`
  - `SetupArrowButton`
  - `SetupCarouselScroll`
  - `OnClickSpiritSlot`
  - `SetupDot`
  - `UpdateDot`
  - `OnUpdateIndex`
  - `OnFlick`
  - `ScrollNextItem`
  - `ScrollPrevItem`
  - `ScrollLockGameCanvas`
  - `ScrollLockGameCanvasCoroutine`
  - `SetupVenusSpirit`
  - `<SetupCarouselScroll>b__17_0`
  - `<ScrollLockGameCanvasCoroutine>b__26_0`

### `Gallop.DialogSingleModeScenarioVenusSpiritTreeModel` (3m)
  - `get_VenusSpiritInfo`
  - `get_HasVenusSpiritInfo`
  - `get_IsSpiritActive`

### `Gallop.DialogSingleModeScenarioVenusSpiritTree` (28m)
  - `GetFormType`
  - `GetParentType`
  - `get_Model`
  - `RegisterDownload`
  - `PushDialog`
  - `CreateDialogData`
  - `Setup`
  - `OnCloseDialog`
  - `SetupVenusCharaInfoArray`
  - `SetupSpiritTree`
  - `SetupBackGroundImage`
  - `SetupSpiritTat`
  - `CreateSpiritEnableAcquireTAT`
  - `CreateSpiritActiveTAT`
  - `SetupRightButtonDisable`
  - `OnLeftButtonCallBack`
  - `OnRightButtonCallBack`
  - `OnClickDialogSingleModeScenarioVenusSpiritActivateConfirm`
  - `OnCallbackSingleModeVenusSpiritUseRequest`
  - `PlayParamUpSequenece`

### `Gallop.DialogSingleModeScenarioVenusVenusDetail` (9m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `RegisterDownload`
  - `PushDialog`
  - `Setup`
  - `SetupContent`
  - `SetupToggle`
  - `OnSelectTab`

### `Gallop.DialogSingleModeScenarioVenusVenusLevelDetailModel` (8m)
  - `get_CharaId`
  - `set_CharaId`
  - `get_Level`
  - `set_Level`
  - `get_IsLevelZero`
  - `get_IsLevelMax`
  - `get_ItemModelList`
  - `<get_ItemModelList>b__15_1`

### `Gallop.DialogSingleModeScenarioVenusVenusLevelDetail` (8m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `RegisterDownload`
  - `PushDialog`
  - `Setup`
  - `SetupHeader`
  - `SetupLevelList`

### `Gallop.PartsSingleModeScenarioVenusActiveSpiritEffectListItemModel` (7m)
  - `get_SpiritEffectType`
  - `get_SpiritEffectText`
  - `get_SpiritEffectParameterUpText`
  - `get_SpitieEffectValue`
  - `get_IsVenusPassiveSpiritEffect`
  - `get_VenusPassiveSpiritEffectVenusCharaInfoList`
  - `HasVenusPassiveSpiritEffect`

### `Gallop.PartsSingleModeScenarioVenusActiveSpiritEffectListItem` (5m)
  - `Create`
  - `Setup`
  - `SetupSpiritEffectText`
  - `SetupSpiritEffectValue`
  - `SetupVenusIcon`

### `Gallop.PartsSingleModeScenarioVenusCharaInfo` (7m)
  - `get_VenusCharaId`
  - `RegisterDownload`
  - `Setup`
  - `SetupIcon`
  - `SetupButton`
  - `PlayVenusLevelUp`
  - `<SetupButton>b__10_0`

### `Gallop.PartsSingleModeScenarioVenusCharaLevelText` (12m)
  - `RegisterDownload`
  - `Setup`
  - `SetActiveMaxIcon`
  - `PlayVenusLevelUp`
  - `PlayLevelUpEffect`
  - `WaitCallback`
  - `get_PlayLevelUpLavel`
  - `get_VenusCharaInfo`
  - `get_IsLevelMax`
  - `get_LevelUpEffectPath`
  - `<WaitCallback>b__13_0`
  - `<get_VenusCharaInfo>b__17_0`

### `Gallop.PartsSingleModeScenarioVenusFragmentList` (2m)
  - `Setup`
  - `SetupFragmentIcon`

### `Gallop.PartsSingleModeScenarioVenusFragmentListItemModel` (3m)
  - `get_FragmentID`
  - `set_FragmentID`
  - `GetFragmentSprite`

### `Gallop.PartsSingleModeScenarioVenusFragmentListItem` (3m)
  - `Setup`
  - `Hide`
  - `SetupIcon`

### `Gallop.PartsSingleModeScenarioVenusGetSpiritFlashPlayer` (12m)
  - `RegisterDownload`
  - `Create`
  - `Initialize`
  - `CreateGetSpiritFlash`
  - `PlayGetSpiritFromFragment`
  - `PlayOnDialog`
  - `PlayGetSpiritFromTree`
  - `SetupSpiritIcon`
  - `PlayOut`
  - `PlayMessageWindow`
  - `CallbackParamChangeUI`
  - `<SetupSpiritIcon>b__15_0`

### `Gallop.PartsSingleModeScenarioVenusGetVenusSpiritFlashPlayer` (14m)
  - `get_InLabel`
  - `get_InTextLabel`
  - `get_EffInLabel`
  - `get_OutLabel`
  - `get_OutEndLabel`
  - `RegisterDownload`
  - `Create`
  - `Initialize`
  - `CreateFlash`
  - `PlayIn`
  - `Setup`
  - `PlayMessageWindow`
  - `CallbackParamChangeUI`
  - `PlayOut`

### `Gallop.PartsSingleModeScenarioVenusIconFragment` (5m)
  - `RegisterDownload`
  - `Setup`
  - `CreateTat`
  - `SetupIcon`
  - `SetupEffect`

### `Gallop.PartsSingleModeScenarioVenusIconSpirit` (5m)
  - `RegisterDownload`
  - `Setup`
  - `CreateTat`
  - `SetupIcon`
  - `SetupEffect`

### `Gallop.PartsSingleModeScenarioVenusIconVenusSpirit` (5m)
  - `RegisterDownload`
  - `Setup`
  - `CreateTat`
  - `GetPrefabPath`
  - `<Setup>b__3_0`

### `Gallop.PartsSingleModeScenarioVenusImageNumberModel` (3m)
  - `get_EnableBonus`
  - `get_ImageNumberPrefabPath`
  - `get_BonusA2UPath`

### `Gallop.PartsSingleModeScenarioVenusMainStablesPanelButtonModel` (11m)
  - `get_ButtonSprite`
  - `get_ButtonSpriteSize`
  - `get_LockButtonSprite`
  - `get_TextImageSprite`
  - `get_ButtonIconIdleSprite`
  - `get_ButtonIconEnterSprite`
  - `get_Interactable`
  - `get_IsHidden`
  - `get_NotificationMessage`
  - `GetSprite`
  - `get_EnableGetSpirit`

### `Gallop.PartsSingleModeScenarioVenusMainStablesPanelButton` (5m)
  - `get_Model`
  - `RegisterDownload`
  - `Create`
  - `Setup`
  - `SetupEnableGetSpirit`

### `Gallop.PartsSingleModeScenarioVenusMainViewModel` (3m)
  - `get_EnableAcquireVenusSpirit`
  - `get_CurrentVenusActiveSpiritEffectList`
  - `get_IsGM`

### `Gallop.PartsSingleModeScenarioVenusMainView` (26m)
  - `get_Model`
  - `get_PartsSpiritFragmentList`
  - `Create`
  - `RegisterDownlaod`
  - `Initialize`
  - `Setup`
  - `SetupSpiritTreeButton`
  - `SetupActiveSpiritEffectButton`
  - `SetupActiveSpiritEffect`
  - `SetActive`
  - `PlayIn`
  - `PlayOut`
  - `PlayGoTrainingSelect`
  - `PlayReturnTrainingSelect`
  - `PlayExecTrainingCut`
  - `PlayInBackTrainingFromAdditiveView`
  - `IsNeedShowScenarioNotice`
  - `ShowScenarioNotice`
  - `OnTrainingItemSelected`
  - `ShowPredictSpirit`

### `Gallop.PartsSingleModeScenarioVenusScenarioRaceListItem` (7m)
  - `get_Schedule`
  - `Setup`
  - `Setup`
  - `SetBgColor`
  - `ShowRaceResultAndFadeOut`
  - `PlayResultSequence`
  - `ShowNextIconAndTurnInfo`

### `Gallop.PartsSingleModeScenarioVenusSpiritEffectList` (2m)
  - `Setup`
  - `SetupSpiritEffectIcon`

### `Gallop.PartsSingleModeScenarioVenusSpiritEffectListItemModel` (2m)
  - `get_SpiritInfo`
  - `GetSpiritSprite`

### `Gallop.PartsSingleModeScenarioVenusSpiritEffectListItem` (3m)
  - `Setup`
  - `Hide`
  - `SetupIcon`

### `Gallop.PartsSingleModeScenarioVenusSpiritEffectListTitle` (2m)
  - `Create`
  - `Setup`

### `Gallop.PartsSingleModeScenarioVenusSpiritFragmentList` (35m)
  - `get_SpiritFragmentListFlashPlayer`
  - `get_SortLayer`
  - `get_SortOffset`
  - `get_MotMcFragmentListRoot00`
  - `RegisterDownlaod`
  - `Create`
  - `Initialize`
  - `CreateFragmentListFlash`
  - `CreateSpiritTreeFlashPlayer`
  - `CreateGetSpiritFlashPlayer`
  - `Initialize`
  - `Setup`
  - `SetupTitleText`
  - `SetupSpiritIcon`
  - `SetSpiritIconToIndex`
  - `SetupPredictSpirit`
  - `ClearPredictSpirit`
  - `GetPredictIndex`
  - `GetSpriteChildName`
  - `PlayIn`

### `Gallop.PartsSingleModeScenarioVenusSpiritHistoryCarouselScroll` (2m)
  - `GetScrollItem`
  - `SetScrollerEnable`

### `Gallop.PartsSingleModeScenarioVenusSpiritHistoryCarouselScrollItem` (3m)
  - `get_ItemData`
  - `UpdateItem`
  - `UpdateSpiritTree`

### `Gallop.PartsSingleModeScenarioVenusSpiritTree` (3m)
  - `Setup`
  - `SetupTatSpiritIcon`
  - `OnClickSpiritSlot`

### `Gallop.PartsSingleModeScenarioVenusSpiritTreeFlashPlayer` (27m)
  - `get_MotionSpiritTree`
  - `GetBlurDuration`
  - `RegisterDownlaod`
  - `Create`
  - `Initialize`
  - `CreateFlash`
  - `CreateGetSpiritFlashPlayer`
  - `CreateGetVenusSpiritFlashPlayer`
  - `PlayGetSpirit`
  - `PlayGetVenusSpirit`
  - `PlayGetSpirit_12_14`
  - `CallbackPlayGetSpiritFromTree`
  - `PlayGetSpirit_15`
  - `PlayInGetVenusSpirit`
  - `SetupSpiritEffect`
  - `SetupGetSpiritParticle`
  - `SetupGetSpiritFlash`
  - `SetupSpiritTree`
  - `SetupSpiritSlotBaseImage`
  - `SetupSpiritSlotDashLineColor`

### `Gallop.PartsSingleModeScenarioVenusSpiritTreeSlot` (10m)
  - `get_SpiritInfo`
  - `Setup`
  - `SetBaseImage`
  - `SetSpiritIcon`
  - `SetTatSpiritIcon`
  - `SetupButton`
  - `SetupLineImage`
  - `SetupDashedLineImage`
  - `OnClickButton`
  - `<get_SpiritInfo>b__9_0`

### `Gallop.PartsSingleModeScenarioVenusUseVenusSpiritPerformance` (20m)
  - `PlayUseItemPerformance`
  - `Create`
  - `InitWipeAnimation`
  - `RegisterDownload`
  - `PlayUseItemPerformanceCoroutine`
  - `StartMessageSequence`
  - `PlayInFrameAnimation`
  - `PlayModelAnimationAndParamUp`
  - `InitializeParamUp`
  - `PlayOutWipeAnimation`
  - `Release`
  - `PlayMiniModelInspirationMotion`
  - `CallbackPlayMotion`
  - `SetupMiniModel`
  - `SetupMiniModelController`
  - `SetupMiniModelRenderTexture`
  - `DestroyMiniModel`
  - `InitializeParamChange`
  - `CallbackParamChangeUI`
  - `DestoryParamChange`

### `Gallop.PartsSingleModeScenarioVenusVenusEffectIcon` (4m)
  - `RegisterDownload`
  - `Setup`
  - `SetupEffect`
  - `OnClick`

### `Gallop.PartsSingleModeScenarioVenusVenusInfoModel` (13m)
  - `get_CharaId`
  - `get_VenusLevel`
  - `get_IsLevelZero`
  - `get_IsLevelMax`
  - `set_IsLevelMax`
  - `get_PassiveEffectTextList`
  - `get_SpiritTitleText`
  - `get_SpiritEffectTurnText`
  - `get_SpiritEffectDescText`
  - `get_EnableGetSpirit`
  - `get_IsSpiritActive`
  - `<get_PassiveEffectTextList>b__15_0`
  - `<get_EnableGetSpirit>b__23_0`

### `Gallop.PartsSingleModeScenarioVenusVenusInfo` (11m)
  - `RegisterDownload`
  - `Setup`
  - `SetupLevel`
  - `SetupPassiveTextList`
  - `SetupSpirit`
  - `CreateSpiritActiveEffect`
  - `CreateSpiritActiveTAT`
  - `CreateSpiritEnableAcquireTAT`
  - `PlayGlowFade`
  - `OnClickInfoButton`
  - `OnPartsActive`

### `Gallop.PartsSingleModeScenarioVenusVenusLevelListItemModel` (6m)
  - `get_IsActive`
  - `set_IsActive`
  - `get_IsMax`
  - `set_IsMax`
  - `get_Level`
  - `get_PassiveEffectTextList`

### `Gallop.PartsSingleModeScenarioVenusVenusLevelListItem` (4m)
  - `Setup`
  - `SetupActive`
  - `SetupLevel`
  - `SetupPassiveTextList`

### `Gallop.SingleModeScenarioVenusSpiritCutin` (8m)
  - `Initialize`
  - `InitializeCutinUI`
  - `ShowSpiritUseTriggerFlash`
  - `PlaySkillCutin`
  - `OnPlayCutin`
  - `ShowSpiritTextFlash`
  - `GetPerformanceIndexByVenusId`
  - `OnEndCutin`

### `Gallop.SingleModeScenarioVenusVenusSpiritUseUI` (15m)
  - `get_Image3D`
  - `get_ContentsRoot`
  - `Initialize`
  - `SetupBG`
  - `GetBGPath`
  - `GetParticlePath`
  - `GetGlitterPath`
  - `GetAcquireBgParticlePath`
  - `SetupFlash`
  - `ShowSpiritTextFlash`
  - `FadeIn`
  - `FadeOut`
  - `Update`
  - `LateUpdate`
  - `<ShowSpiritTextFlash>b__24_0`


## Master数据库表 (5个)

| 表名 | 方法数 |
|---|---|
| `MasterRaceJikkyoBaseVenus` | 8 |
| `MasterSingleModeVenusCrystalGroup` | 9 |
| `MasterSingleModeVenusSpiritEffect` | 9 |
| `MasterSingleModeVenusSpiritGroup` | 3 |
| `MasterSingleModeVenusSpraceData` | 9 |

## WorkSingleModeScenarioVenus

方法数: 14

  - `get_SpiritInfoArray`
  - `get_VenusSpiritActiveEffectInfoArray`
  - `get_VenusCharaInfoArray`
  - `get_VenusCharaCommandInfoArray`
  - `get_VenusRaceHistryArray`
  - `get_RaceStartInfo`
  - `get_RaceScenario`
  - `get_VenusRaceCondition`
  - `get_LiveItemId`
  - `get_RaceRewardInfo`
  - `Apply`
  - `Apply`
  - `Apply`
  - `GetHighLevelVenusId`

## 剧本独立属性变化 (10m, 4 getters)

  - `get_SpiritInfoList`
  - `get_AppearVenus`
  - `get_UsedSpirit`
  - `get_InfoLevelUp`

## lib.rs相关引用

```
let mot_s = match mot { 5=>"Best", 4=>"Good", 3=>"Normal", 2=>"Bad", 1=>"Worst", _=>"?" };
1=>"URA", 2=>"TeamRace", 3=>"Live", 4=>"Free", 5=>"Venus",
105=>"Power", 106=>"Wiz", _=>"Unknown"
4=>"Power", 5=>"Wiz", 10=>"HP",
5=>"WorkSingleModeScenarioVenus", 6=>"WorkSingleModeScenarioArc",
5 => "WorkSingleModeScenarioVenus",
```