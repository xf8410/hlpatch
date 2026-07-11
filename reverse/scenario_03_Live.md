# 剧本 3: Live (ライブ)

**WorkScenario类**: `WorkSingleModeScenarioLive`
**ObscuredDataSet**: `ObscuredSingleModeLiveDataSet`
---

## 相关类 (196个)

### `Gallop.LivePlayableAnimator` (12m)
  - `get_IsAlive`
  - `IsClip`
  - `GetClip`
  - `OnInitialize`
  - `Rebind`
  - `SetRaceMode`
  - `ResetRaceMode`
  - `SetPlayParameter`
  - `SetRaceClip`
  - `OnUpdate`
  - `UpdateRunAnimation`
  - `UpdateClipMixer`

### `Gallop.LiveImageEffect` (9m)
  - `set_MonitorTextureArray`
  - `Initialize`
  - `SetUpEdgeStyleController`
  - `SetUpEdgeStyleRenderList`
  - `CurrentEdgeStyle`
  - `CleanUpEdgeStyleController`
  - `OnRenderImagePrepareScreenMakeup`
  - `CopyMonitorTexture`
  - `Destroy`

### `Gallop.BgLiveWave` (9m)
  - `get_WaveMaterial`
  - `get_IsInitialized`
  - `get_IsUpdateRealTime`
  - `set_IsUpdateRealTime`
  - `Initialize`
  - `UpdateParam`
  - `GetTime`
  - `AlterUpdate`
  - `OnDestroy`

### `Gallop.BgLiveWaveDistortion` (1m)
  - `AlterUpdate`

### `Gallop.LiveCharacterAnimation` (31m)
  - `get_CurrentMode`
  - `get_Animation`
  - `set_Animation`
  - `get_IsUsingPlayableGraph`
  - `get_IsAlivePlayableGraph`
  - `get_IsPlayingTailMotion`
  - `HasEarSystemTextMotion`
  - `GetCurrentSystemTextEarType`
  - `HasTailSystemTextMotion`
  - `SetupPlayableGraph`
  - `Initialize`
  - `ReleasePlayableGraphAnimation`
  - `SetupLegacyAnimation`
  - `LoadBodySystemMotionClipList`
  - `LoadTailSystemMotionClip`
  - `PreloadSystemTextMotion`
  - `LoadAllTailRandomMotionClip`
  - `LoadTailRandomMotionClip`
  - `SetupSystemTextAnimation`
  - `PlaySystemTextAnimation`

### `Gallop.LiveModelController` (168m)
  - `get_HeadTransform`
  - `set_HeadTransform`
  - `get_NeckTransform`
  - `set_NeckTransform`
  - `get_ChestTransform`
  - `set_ChestTransform`
  - `get_WaistTransform`
  - `set_WaistTransform`
  - `get_HipTransform`
  - `get_LeftWristTransform`
  - `set_LeftWristTransform`
  - `get_LeftHandAttachTransform`
  - `set_LeftHandAttachTransform`
  - `get_RightWristTransform`
  - `set_RightWristTransform`
  - `get_RightHandAttachTransform`
  - `set_RightHandAttachTransform`
  - `get_LeftAnkleTransform`
  - `set_LeftAnkleTransform`
  - `get_RightAnkleTransform`

### `Gallop.MasterLiveData` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterLiveDressRestrictData` (11m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithMusicIdOrderByDressIdAsc`
  - `_SelectWithMusicIdOrderByDressIdAsc`
  - `GetListWithMusicIdOrderByDressIdAsc`
  - `MaybeListWithMusicIdOrderByDressIdAsc`
  - `_ListSelectWithMusicIdOrderByDressIdAsc`
  - `_CreateOrmByQueryResultWithMusicIdOrderByDressIdAsc`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterLiveExtraData` (11m)
  - `Get`
  - `_SelectOne`
  - `GetWithMusicIdOrderByIdAsc`
  - `_SelectWithMusicIdOrderByIdAsc`
  - `GetListWithMusicIdOrderByIdAsc`
  - `MaybeListWithMusicIdOrderByIdAsc`
  - `_ListSelectWithMusicIdOrderByIdAsc`
  - `_CreateOrmByQueryResultWithMusicIdOrderByIdAsc`
  - `Unload`
  - `GetListAllEntries`
  - `GetLiveExtraDataDictionary`

### `Gallop.MasterLiveFixMemberData` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithMusicId`
  - `_SelectWithMusicId`
  - `GetListWithMusicId`
  - `MaybeListWithMusicId`
  - `_ListSelectWithMusicId`
  - `_CreateOrmByQueryResultWithMusicId`
  - `Unload`

### `Gallop.MasterLivePermissionData` (14m)
  - `get_dictionary`
  - `GetKey`
  - `Get`
  - `Get`
  - `_SelectOne`
  - `GetWithMusicIdOrderByCharaIdAsc`
  - `_SelectWithMusicIdOrderByCharaIdAsc`
  - `GetListWithMusicIdOrderByCharaIdAsc`
  - `MaybeListWithMusicIdOrderByCharaIdAsc`
  - `_ListSelectWithMusicIdOrderByCharaIdAsc`
  - `_CreateOrmByQueryResultWithMusicIdOrderByCharaIdAsc`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `IsPlayChara`

### `Gallop.MasterLiveRecommendFormation` (7m)
  - `GetWithMusicId`
  - `_SelectWithMusicId`
  - `GetListWithMusicId`
  - `MaybeListWithMusicId`
  - `_ListSelectWithMusicId`
  - `_CreateOrmByQueryResultWithMusicId`
  - `Unload`

### `Gallop.MasterLiveDatabase` (31m)
  - `get_masterLiveData`
  - `set_masterLiveData`
  - `get_masterLivePermissionData`
  - `set_masterLivePermissionData`
  - `get_masterLiveExtraData`
  - `set_masterLiveExtraData`
  - `get_masterLiveDressRestrictData`
  - `set_masterLiveDressRestrictData`
  - `get_masterLiveFixMemberData`
  - `set_masterLiveFixMemberData`
  - `get_masterLiveRecommendFormation`
  - `set_masterLiveRecommendFormation`
  - `_GetOpenedConnection`
  - `Unload`
  - `Query`
  - `GetSelectQuery_LiveData`
  - `GetSelectAllQuery_LiveData`
  - `GetSelectQuery_LivePermissionData`
  - `GetSelectQueryWithIndex_LivePermissionData_MusicId`
  - `GetSelectAllQuery_LivePermissionData`

### `Gallop.MasterSingleModeLiveLiveData` (6m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `GetList`

### `Gallop.MasterSingleModeLiveMasterBonus` (3m)
  - `Get`
  - `_SelectOne`
  - `Unload`

### `Gallop.MasterSingleModeLiveSongList` (11m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithCommandId`
  - `_SelectWithCommandId`
  - `_CreateOrmByQueryResultWithCommandId`
  - `GetWithLiveId`
  - `_SelectWithLiveId`
  - `_CreateOrmByQueryResultWithLiveId`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleModeLiveSquare` (3m)
  - `Get`
  - `_SelectOne`
  - `Unload`

### `Gallop.MasterSingleModeRaceLive` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterStoryLivePosition` (19m)
  - `Get`
  - `_SelectOne`
  - `GetWithSetIdOrderByPositionIdAsc`
  - `_SelectWithSetIdOrderByPositionIdAsc`
  - `GetListWithSetIdOrderByPositionIdAsc`
  - `MaybeListWithSetIdOrderByPositionIdAsc`
  - `_ListSelectWithSetIdOrderByPositionIdAsc`
  - `_CreateOrmByQueryResultWithSetIdOrderByPositionIdAsc`
  - `GetWithMusicIdOrderByPositionIdAsc`
  - `_SelectWithMusicIdOrderByPositionIdAsc`
  - `GetListWithMusicIdOrderByPositionIdAsc`
  - `MaybeListWithMusicIdOrderByPositionIdAsc`
  - `_ListSelectWithMusicIdOrderByPositionIdAsc`
  - `_CreateOrmByQueryResultWithMusicIdOrderByPositionIdAsc`
  - `Unload`
  - `GetListAllEntries`
  - `IsExistData`
  - `GetModelDataArray`
  - `GetEmptyModelDataArray`

### `Gallop.SingleModeLiveMasterBonusExtensions` (1m)
  - `ToDefineParameterType`

### `Gallop.WorkSingleModeChangeParameterInfoScenarioLive` (13m)
  - `get_Performance`
  - `set_Performance`
  - `get_PerformanceMax`
  - `set_PerformanceMax`
  - `get_LimitPerformanceTypeList`
  - `set_LimitPerformanceTypeList`
  - `ExistLimitPerformanceType`
  - `get_LiveGetMusicId`
  - `set_LiveGetMusicId`
  - `GetLivePerformance`
  - `GetLivePerformanceMax`
  - `Clear`
  - `Set`

### `Gallop.WorkSingleModeScenarioLive` (36m)
  - `get_NextMusicNum`
  - `get_NextMusicIdArray`
  - `set_NextMusicIdArray`
  - `get_TotalMusicNum`
  - `get_TotalMusicIdArray`
  - `set_TotalMusicIdArray`
  - `get_NextLiveBonusEffect`
  - `set_NextLiveBonusEffect`
  - `get_CurrentLiveBonusEffect`
  - `set_CurrentLiveBonusEffect`
  - `get_CurrentLiveBonusMusicIdArray`
  - `set_CurrentLiveBonusMusicIdArray`
  - `get_TreeSquareInfoArray`
  - `set_TreeSquareInfoArray`
  - `get_ReservedTreeSquareId`
  - `set_ReservedTreeSquareId`
  - `get_TrainingBonusArray`
  - `set_TrainingBonusArray`
  - `get_EvaluationInfoList`
  - `set_EvaluationInfoList`

### `Gallop.DialogGetLiveMusic` (4m)
  - `GetFormType`
  - `GetParentType`
  - `Setup`
  - `Open`

### `Gallop.DialogGetLiveMusicMulti` (5m)
  - `GetFormType`
  - `GetParentType`
  - `Open`
  - `Setup`
  - `<Setup>b__5_0`

### `Gallop.DialogHeroesFinalRaceResultUmaineLive` (33m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `RegisterDownload`
  - `RegisterDownload3D`
  - `Open`
  - `Setup`
  - `SetupFlash`
  - `SetupNextButton`
  - `Setup3D`
  - `SetupMiniCharacter`
  - `SetupMiniShadow`
  - `UpdateShadowPos`
  - `SetupCamera`
  - `PlayIn`
  - `PlayInFrame`
  - `PlayInFlash`
  - `PlayUmaineBaloonAnim`
  - `FinishOrderToIndex`
  - `PrepareMiniCharacter`

### `Gallop.PartsGetLiveMusicListItem` (1m)
  - `UpdateItem`

### `Gallop.ChampionsLiveUI` (5m)
  - `SetLiveButtonPosition`
  - `Setup`
  - `SetupButtonCollision`
  - `PlayIn`
  - `PlayOut`

### `Gallop.PartsLiveTheaterButton` (4m)
  - `Setup`
  - `PlayIn`
  - `OnClick`
  - `NeedBadge`

### `Gallop.LiveViewController` (65m)
  - `LoadExtraResource`
  - `LoadOnsenFlashController`
  - `LoadRamenFlashController`
  - `get_IsScreenModeFullPortrait`
  - `get_LiveScreenCaptureController`
  - `get__isSwitchingToMainDirector`
  - `LoadForRegisterDownload`
  - `PreRegisterDownload`
  - `RegisterDownloadMain`
  - `RegisterDownload`
  - `GetChangeViewOrientation`
  - `OverrideDynamicNowLoadingType`
  - `InitializeView`
  - `InitializeOnlyMainDirector`
  - `CreateSubDirector`
  - `DestroySubDirector`
  - `SwitchMainDirector`
  - `BeginView`
  - `UpdateViewPre`
  - `UpdateView`

### `Gallop.LiveView` (40m)
  - `get_LandscapeRoot`
  - `get_LandscapePauseButton`
  - `get_LandscapeFadeImage`
  - `get_LandscapeTitle`
  - `get_PortraitRoot`
  - `get_PortraitPauseButton`
  - `get_PortraitFadeImage`
  - `get_PortraitTitle`
  - `get_JacketImage`
  - `get_TitleText`
  - `get_AuthorText`
  - `get_LyricsAllText`
  - `get_FullPortraitRoot`
  - `get_FullPortraitPauseButton`
  - `get_FullPortraitFadeImage`
  - `get_FullPortraitOptionButton`
  - `get_FullPortraitTitle`
  - `get_MenuRoot`
  - `get_ChangeOrientationButtonArray`
  - `get_SkipButtonArray`

### `Gallop.DialogLiveStartConfirm` (7m)
  - `GetFormType`
  - `GetParentType`
  - `Setup`
  - `SetupToggle`
  - `OnClickOK`
  - `SendApi`
  - `PushDialog`

### `Gallop.DialogLiveStartConfirmFullPortrait` (8m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `PushDialog`
  - `Initialize`
  - `SendApi`
  - `OnClickOK`
  - `<Initialize>b__10_0`

### `Gallop.DialogOptionLiveFullPortrait` (7m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `PushDialog`
  - `Initialize`
  - `OnSaveSetting`
  - `OnInitValueSetting`

### `Gallop.LiveFlashControllerUtil` (2m)
  - `CreateFlashCamera`
  - `CreateFlashCanvas`

### `Gallop.LiveFlashOnsenController` (6m)
  - `Initialize`
  - `DestroySub`
  - `SetupFont`
  - `DestroyFont`
  - `SetCharacterName`
  - `GetCharacterNameByPosition`

### `Gallop.LiveDefine` (5m)
  - `IsValidLiveVariationId`
  - `IsValidBgVariationId`
  - `IsValidOkeVariationId`
  - `TrySetMaterialBlendMode`
  - `TrySetLightBlendModeMaterialProperty`

### `Gallop.LiveQualitySettings` (9m)
  - `get_IsUseMonitor`
  - `get_IsUseRealShadow`
  - `get_IsUseProjector`
  - `get_IsUseReflection`
  - `get_IsDefaultMonitorCameraResolution`
  - `get_IsUseLensflare`
  - `get_IsUseMirrorScan`
  - `get_IsUseCameraShake`
  - `get_IsUseMob2D`

### `Gallop.LiveScene` (0m)

### `Gallop.LiveSceneController` (1m)
  - `FinalizeScene`

### `Gallop.LiveUtil` (13m)
  - `GetVirtualResolution3D`
  - `ResizeArray`
  - `GetComponentArrayFromGameObjectArray`
  - `GetRendererArrayFromMaterial`
  - `GetGrayScaleColor`
  - `InitializeLoadSettings`
  - `ChangeLiveSimple`
  - `ChangeLive`
  - `SendLiveStartApi`
  - `GetLiveSettingForLog`
  - `GetSingCharaIdList`
  - `TransformAspectRatio`
  - `GetLiveDressRestrictionList`

### `Gallop.DialogLiveTheaterAutoSetting` (9m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `PushDialog`
  - `IndexToPosAndTarget`
  - `IndexToAutoSetType`
  - `Initialize`
  - `Apply`
  - `<Initialize>g__DisableRecommend|15_0`

### `Gallop.DialogLiveTheaterAutoSettingOld` (8m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `PushDialog`
  - `IndexToPosAndTarget`
  - `IndexToAutoSetType`
  - `Initialize`
  - `Apply`

### `Gallop.DialogLiveTheaterCharaSelect` (14m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `get_SelectCharaId`
  - `get_ParentDialog`
  - `set_ParentDialog`
  - `PushDialog`
  - `Initialize`
  - `OnClickRightButton`
  - `AddCallback`
  - `RemoveCallback`
  - `OnUpdateCharaButton`
  - `OnTapCharaButton`
  - `InitButtonCollision`

### `Gallop.DialogLiveTheaterDressSelect` (21m)
  - `CreateDialogData`
  - `GetFormType`
  - `GetParentType`
  - `get_DressId`
  - `set_DressId`
  - `get_ColorId`
  - `set_ColorId`
  - `get_ChangeType`
  - `set_ChangeType`
  - `get_AllDressChangePattern`
  - `set_AllDressChangePattern`
  - `PushDialog`
  - `Initialize`
  - `OnClick`
  - `GetAllDressChangePattern`
  - `SetCursor`
  - `GetIcon`
  - `OnClickChangeDressAll`
  - `OnClickRightButton`
  - `<Initialize>b__43_0`

### `Gallop.DialogLiveTheaterMusicDetail` (5m)
  - `GetParentType`
  - `GetFormType`
  - `CreateDialogData`
  - `PushDialog`
  - `Initialize`

### `Gallop.LiveTheaterCharaSelect` (15m)
  - `get_CurrentMemberType`
  - `GetNextButtonText`
  - `Initialize`
  - `Show`
  - `Hide`
  - `CheckSwapChara`
  - `ChangeToggle`
  - `UnloadFormation`
  - `LoadFormation`
  - `OnClickSetRecommend`
  - `OnClickSetFormationSave`
  - `SetRecommend`
  - `OnAllDressChange`
  - `SetCharaDressIdList`
  - `<Initialize>b__19_0`

### `Gallop.LiveTheaterCharaSelectCharaDressScroll` (4m)
  - `Initialize`
  - `InitializeForFormationSave`
  - `Refresh`
  - `SetCharaId`

### `Gallop.LiveTheaterCharaSelectCharaDressSetUI` (18m)
  - `get_Index`
  - `Initialize`
  - `Initialize`
  - `OnClickCharaButton`
  - `OnClickDressButton`
  - `GetDressList`
  - `SetCharaId`
  - `SetDressId`
  - `UpdateCharaTex`
  - `UpdateDressTex`
  - `AllButtonEnable`
  - `SetDisplayNumber`
  - `GetDisplayNumber`
  - `UseSecondDress`
  - `<Initialize>b__24_0`
  - `<Initialize>b__24_1`
  - `<Initialize>b__24_2`
  - `<UpdateDressTex>g__GetDressTexture|31_0`

### `Gallop.PartsLiveTheaterVoiceIcon` (3m)
  - `get_RectTransform`
  - `FindOrCreate`
  - `SetVisible`

### `Gallop.LiveTheaterFormation` (9m)
  - `get_HightScaleRate`
  - `get_CameraDistance`
  - `get_CharaPlaneRotation`
  - `get_CharaRenderTarget`
  - `get_PositionArray`
  - `Initialize`
  - `OnSceneUpdate`
  - `SetRenderTex`
  - `GetRenderOrder`

### `Gallop.LiveTheaterFormationPosition` (15m)
  - `get_OffsetZ`
  - `get_Scale`
  - `get_ShadowImage`
  - `get_RenderOrder`
  - `set_RenderOrder`
  - `get_ModelIndex`
  - `set_ModelIndex`
  - `Initialize`
  - `UpdateShadowScale`
  - `UpdateIndexRect`
  - `FixHightScaleRate`
  - `GetNormalizedPos`
  - `SetGrayColor`
  - `SetBaseSing`
  - `GeBaseSprite`

### `Gallop.DialogLiveTheaterFormationSave` (10m)
  - `GetFormType`
  - `GetParentType`
  - `Open`
  - `Setup`
  - `SetupDataDisplay`
  - `OnClickSaveButton`
  - `OnClickLoadButton`
  - `OnClickInfoButton`
  - `<Open>g__onLoadSuccess|5_0`
  - `<Setup>b__6_0`

### `Gallop.DialogLiveTheaterFormationSaveConfirm` (6m)
  - `GetFormType`
  - `GetParentType`
  - `Open`
  - `Setup`
  - `GetSetupParameterForCopyPaste`
  - `<Setup>g__OnClickInfoButton|22_10`

### `Gallop.DialogLiveTheaterFormationSaveDetail` (10m)
  - `GetParentType`
  - `GetFormType`
  - `Open`
  - `Init`
  - `Setup`
  - `OnDecide`
  - `OnCopied`
  - `<Init>b__10_0`
  - `<Setup>b__11_0`
  - `<Setup>b__11_1`

### `Gallop.DialogLiveTheaterFormationSaveEdit` (9m)
  - `CreateDialogData`
  - `GetFormType`
  - `GetParentType`
  - `Open`
  - `SetUp`
  - `OnChangeChara`
  - `OnChangeDress`
  - `OnAllDressChange`
  - `OnSaveSuccess`

### `Gallop.DialogLiveTheaterFormationSaveName` (9m)
  - `GetFormType`
  - `GetParentType`
  - `Open`
  - `SetDialogData`
  - `Initialize`
  - `SetButton`
  - `OnEndEdit`
  - `CheckText`
  - `OnClickDecide`

### `Gallop.LiveTheaterFormationSaveUtil` (12m)
  - `Save`
  - `Load`
  - `GetParameterList`
  - `SetupCharaList`
  - `AlignLength`
  - `GetSetupParameterForCurrent`
  - `GetValidCount`
  - `SetParentScreenInfo`
  - `GetLiveTheaterInfo`
  - `GetLiveTheaterCharaSelect`
  - `Clean`
  - `<SetupCharaList>g__Load|10_1`

### `Gallop.LiveTheaterInfo` (101m)
  - `get_SortId`
  - `set_SortId`
  - `get_IsSkipStory`
  - `set_IsSkipStory`
  - `get_IsNew`
  - `set_IsNew`
  - `get_Id`
  - `set_Id`
  - `get_SongCueSheetName`
  - `set_SongCueSheetName`
  - `get_SongCueName`
  - `set_SongCueName`
  - `get_SongCueSheetArray`
  - `set_SongCueSheetArray`
  - `get_MoviePath`
  - `set_MoviePath`
  - `get_FormationPrefabPathMain`
  - `set_FormationPrefabPathMain`
  - `get_FormationPrefabPathAll`
  - `set_FormationPrefabPathAll`

### `Gallop.LiveTheaterScene` (1m)
  - `get_FitCamera`

### `Gallop.LiveTheaterSceneController` (17m)
  - `GetCamera`
  - `InitializeScene`
  - `FinalizeScene`
  - `UpdateScene`
  - `UpdateCharaModels`
  - `InitCamera`
  - `InitializeFormation`
  - `SetTheaterInfo`
  - `ApplyFormation`
  - `ApplyStencil`
  - `CharaPositionComparer`
  - `SetDispMemberType`
  - `ApplyCharaVisible`
  - `UnSetTheaterInfo`
  - `BuildModel`
  - `GetFormation`
  - `<UpdateCharaModels>g__IsChanged|20_0`


## Master数据库表 (13个)

| 表名 | 方法数 |
|---|---|
| `MasterLiveData` | 5 |
| `MasterLiveDressRestrictData` | 11 |
| `MasterLiveExtraData` | 11 |
| `MasterLiveFixMemberData` | 9 |
| `MasterLivePermissionData` | 14 |
| `MasterLiveRecommendFormation` | 7 |
| `MasterLiveDatabase` | 31 |
| `MasterSingleModeLiveLiveData` | 6 |
| `MasterSingleModeLiveMasterBonus` | 3 |
| `MasterSingleModeLiveSongList` | 11 |
| `MasterSingleModeLiveSquare` | 3 |
| `MasterSingleModeRaceLive` | 5 |
| `MasterStoryLivePosition` | 19 |

## WorkSingleModeScenarioLive

方法数: 36

  - `get_NextMusicNum`
  - `get_NextMusicIdArray`
  - `set_NextMusicIdArray`
  - `get_TotalMusicNum`
  - `get_TotalMusicIdArray`
  - `set_TotalMusicIdArray`
  - `get_NextLiveBonusEffect`
  - `set_NextLiveBonusEffect`
  - `get_CurrentLiveBonusEffect`
  - `set_CurrentLiveBonusEffect`
  - `get_CurrentLiveBonusMusicIdArray`
  - `set_CurrentLiveBonusMusicIdArray`
  - `get_TreeSquareInfoArray`
  - `set_TreeSquareInfoArray`
  - `get_ReservedTreeSquareId`
  - `set_ReservedTreeSquareId`
  - `get_TrainingBonusArray`
  - `set_TrainingBonusArray`
  - `get_EvaluationInfoList`
  - `set_EvaluationInfoList`
  - `get_LiveResultList`
  - `set_LiveResultList`
  - `GetLiveResult`
  - `GetPerformance`
  - `GetPerformanceMax`
  - `CanGetTreeSquare`
  - `CanGetTreeSquare`
  - `GetExpectationGauge`
  - `GetNextExpectationGauge`
  - `GetChangeExpectationGauge`
  - `GetExpectationGauge`
  - `IsMaxExpectationGauge`
  - `GetTrainingBonus`
  - `HasTrueGrandLiveMusic`
  - `Apply`
  - `<CanGetTreeSquare>b__54_0`

## 剧本独立属性变化 (13m, 4 getters)

  - `get_Performance`
  - `get_PerformanceMax`
  - `get_LimitPerformanceTypeList`
  - `get_LiveGetMusicId`

## lib.rs相关引用

```
let mot_s = match mot { 5=>"Best", 4=>"Good", 3=>"Normal", 2=>"Bad", 1=>"Worst", _=>"?" };
1=>"URA", 2=>"TeamRace", 3=>"Live", 4=>"Free", 5=>"Venus",
11=>"Pioneer", 12=>"Onsen", 13=>"Breeders", 14=>"Ramen", _=>"Unknown"
101=>"Speed", 102=>"Stamina", 103=>"Guts",
1=>"Speed", 2=>"Stamina", 3=>"Guts",
3=>"WorkSingleModeScenarioLive", 4=>"WorkSingleModeScenarioFree",
13=>"WorkSingleModeScenarioBreeders", 14=>"WorkSingleModeScenarioRamen",
3 => "WorkSingleModeScenarioLive",
```