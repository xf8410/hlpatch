# IL2CPP 全量类转储 (按命名空间)

**总类数**: 27695  
**总方法数**: 160909
---

## 命名空间统计 (Top 50)

| 命名空间 | 类数 | 方法数 |
|---|---|---|
| `Gallop` | 8039 | 100282 |
| `` | 16977 | 45250 |
| `Gallop.CutIn.Cutt` | 309 | 2491 |
| `Gallop.SingleMode.ScenarioRamen` | 297 | 2158 |
| `Gallop.Live.Cutt` | 496 | 1633 |
| `Gallop.Live` | 76 | 1402 |
| `Gallop.SingleModeAutoPlay` | 415 | 1380 |
| `Gallop.Model.Component` | 98 | 1366 |
| `Gallop.TrainingChallenge` | 103 | 813 |
| `StandaloneSimulator` | 286 | 786 |
| `Gallop.RenderPipeline` | 93 | 700 |
| `Gallop.SingleMode.ScenarioPioneer` | 129 | 555 |
| `Gallop.SingleMode.ScenarioOnsen` | 77 | 398 |
| `Gallop.SingleMode.ScenarioBreeders` | 55 | 352 |
| `Gallop.ImageEffect` | 22 | 184 |
| `Gallop.CutIn` | 14 | 176 |
| `Gallop.Tutorial` | 13 | 133 |
| `Gallop.Mini` | 25 | 132 |
| `Gallop.SingleMode.ScenarioLegend` | 25 | 111 |
| `Gallop.StoryStill` | 17 | 107 |
| `Gallop.SingleModeRaceReserve` | 9 | 80 |
| `Gallop.Cyalume` | 8 | 60 |
| `Gallop.Model` | 17 | 53 |
| `UnityEngine.EventSystems` | 1 | 38 |
| `Gallop.StoryTimeline` | 9 | 37 |
| `Gallop.SupportCardDeckAutoBuild` | 14 | 37 |
| `Gallop.SingleModeTrainingCutHelperExtension` | 1 | 32 |
| `KeyframeCamera` | 31 | 25 |
| `Gallop.SingleMode` | 5 | 25 |
| `Gallop.StoryEvent` | 2 | 19 |
| `Gallop.Live.Cyalume` | 2 | 19 |
| `Gallop.AnimSequenceHelper` | 1 | 18 |
| `Gallop.Live.ShaderParam` | 10 | 16 |
| `Gallop.SingleModeAutoPlay.ScenarioBreeders` | 2 | 11 |
| `UnityEngine.UI` | 2 | 8 |
| `Gallop.MotionSe` | 2 | 6 |
| `Gallop.Model.Component.Expression` | 2 | 6 |
| `Gallop.Cutt.Util` | 1 | 4 |
| `AnimateToUnity` | 1 | 3 |
| `Gallop.LitJsonUtil` | 1 | 3 |
| `Microsoft.CodeAnalysis` | 1 | 0 |
| `System.Runtime.CompilerServices` | 3 | 0 |
| `Gallop.StoryExtensions` | 1 | 0 |
| `Gallop.Live.UVMoviePacker` | 3 | 0 |

## Gallop 命名空间详细 (8039 classes)

### `ApplicationSettingSaveLoader` (1432m)
  - `IsOpen`
  - `Dispose`
  - `Save`
  - `ForceSave`
  - `Load`

### `MasterDataManager` (1252m)
  - `get_masterAudioCuesheet`
  - `set_masterAudioCuesheet`
  - `get_masterAudioIgnoredCueOnHighspeed`
  - `set_masterAudioIgnoredCueOnHighspeed`
  - `get_masterBannerData`

### `MasterSingleModeDatabase` (1205m)
  - `get_masterIdleSingleModeTrainingCut`
  - `set_masterIdleSingleModeTrainingCut`
  - `get_masterSingleModeProgram`
  - `set_masterSingleModeProgram`
  - `get_masterSingleModeMessage`

### `ResourcePath` (932m)
  - `get_AssetBundleRoot`
  - `GetGallopResourcesFullPath`
  - `GetBundleResourcesFullPath`
  - `GetStreamingAssetFullPath`
  - `GetPreInTitleMoviePath`

### `StoryViewController` (485m)
  - `get_IsScenarioRamenSelectEvent`
  - `InitNoSendCheckEventForScenarioRamenSelect`
  - `StartScenarioRamenSelectSequence`
  - `get_IsNoSendCheckEvent`
  - `get_IsPrologue`

### `ModelController` (449m)
  - `GetFaceGroupArray`
  - `get_RenderQueueAlpha`
  - `OrderToBaseRenderQueue`
  - `get_OwnerObject`
  - `get_HeadObject`

### `AudioManager` (442m)
  - `get_IsAutoStopInterval`
  - `get_CriAudioManager`
  - `OnInitialize`
  - `OnFinalize`
  - `InitializeCuteCRI`

### `WorkSingleModeData` (409m)
  - `get_AutoPlayDefaultPlanArray`
  - `get_AutoPlayCustomPlanArray`
  - `get_AutoPlaySelectedPlanId`
  - `ApplyAutoPlayPlanData`
  - `ApplyAutoPlayDefaultPlanData`

### `StoryTimelineController` (310m)
  - `set_TimelineData`
  - `get_TimelineData`
  - `get_PlayedBlockIndexList`
  - `get_PrevBlockData`
  - `set_PrevBlockData`

### `HorseRaceInfo` (294m)
  - `get_RaceBaseSpeed`
  - `get_EpisodeRaceMode`
  - `set_EpisodeRaceMode`
  - `get_GroundType`
  - `set_GroundType`

### `EventTimelineModelController` (279m)
  - `get_TurnController`
  - `set_TurnController`
  - `get_HipTransformComponent`
  - `set_HipTransformComponent`
  - `get_PairMotionOffsetComponent`

### `RaceResultList` (277m)
  - `get_OnClickNext`
  - `set_OnClickNext`
  - `get_BgmOverScene`
  - `get_NextButton`
  - `get_LiveButton`

### `RaceModelController` (269m)
  - `get_IsExecuteCharaAudio`
  - `set_IsExecuteCharaAudio`
  - `get_IsFacialUpdate`
  - `set_IsFacialUpdate`
  - `get_LodType`

### `RaceCameraManager` (256m)
  - `get_IsPauseRace`
  - `get_IsRaceLandscape`
  - `get_IsDrawInMaskMultiCamera`
  - `ChangeDrawInMask`
  - `get_CaptureTexture`

### `MasterCardDatabase` (249m)
  - `get_masterCardData`
  - `set_masterCardData`
  - `get_masterCardRarityData`
  - `set_masterCardRarityData`
  - `get_masterCardTalentLevelUpgradeItem`

### `RaceUtil` (245m)
  - `GetRaceHorseMaxNum`
  - `IsRaceSceneLandscape`
  - `GetRaceLandScapeSettingData`
  - `IsEnableDynamicRaceType`
  - `GetRaceDynamicCameraSettingData`

### `RaceManagerReplayBase` (239m)
  - `get_FootDirtBaseEnergy`
  - `get_LowerDirtBaseEnergy`
  - `get_UpperDirtBaseEnergy`
  - `get_DirtDistance`
  - `get_DirtAngle`

### `RaceManager` (227m)
  - `get_RaceInfo`
  - `set_RaceInfo`
  - `DestroyRaceInfo`
  - `get_RaceBootMode`
  - `get_State`

### `HorseRaceInfoSimple` (218m)
  - `get_HorseData`
  - `set_HorseData`
  - `get_HorseIndex`
  - `get_CharaName`
  - `get_CardId`

### `TrainingParamChangeUI` (216m)
  - `get_ContentsRoot`
  - `get_MessageRoot`
  - `get_CheckScreenTapButton`
  - `get_MessageCanvas`
  - `get_SharedPartsLocator`

### `JikkyoTrigger` (212m)
  - `Initialize`
  - `Release`
  - `Update`
  - `GetHorse`
  - `IsJikkyoTriggerOK`

### `WorkSingleModeCharaData` (211m)
  - `get_Id`
  - `set_Id`
  - `get_CardId`
  - `set_CardId`
  - `get_CharaId`

### `RaceMainViewController` (211m)
  - `get_MainView`
  - `get_BootMode`
  - `set_BootMode`
  - `get_TitlePlayer`
  - `get_RaceLoader`

### `RaceInfo` (206m)
  - `get_RaceType`
  - `set_RaceType`
  - `get_IsExistPlayerRace`
  - `set_IsExistPlayerRace`
  - `get_IsExistGhostRace`

### `UIManager` (202m)
  - `get_DefaultResolution`
  - `get_DefaultResolutionOld`
  - `get_UICamera`
  - `get_BGCamera`
  - `get_BGCameraDefaultDepth`

### `CharacterButtonInfo` (188m)
  - `get_IdType`
  - `set_IdType`
  - `get_IconImageType`
  - `set_IconImageType`
  - `get_IsCharacterCardButton`

### `RaceEpisodeCameraEvent` (183m)
  - `get_FramePerSec`
  - `set_FramePerSec`
  - `get_IsFrameMode`
  - `get_CharacterParameterArray`
  - `set_CharacterParameterArray`

### `PaddockViewControllerBase` (181m)
  - `get_StoryTimelineControllerArray`
  - `get_RaceMaster`
  - `get_CutinParent`
  - `get_AutoPlayProxy`
  - `set_AutoPlayProxy`

### `MiniModelController` (176m)
  - `GetBuildInfo`
  - `get_HeadObject`
  - `get_BodyObject`
  - `get_TailObject`
  - `get_PositionNode`

### `SingleModeUtils` (172m)
  - `ClearPlayData`
  - `IsInSingleModeNextSceneView`
  - `GetTurnList`
  - `GetMasterTurn`
  - `GetMasterTurn`

### `MasterRaceDatabase` (171m)
  - `get_masterRace`
  - `set_masterRace`
  - `get_masterRaceMotivationRate`
  - `set_masterRaceMotivationRate`
  - `get_masterRaceProperDistanceRate`

### `LiveModelController` (168m)
  - `get_HeadTransform`
  - `set_HeadTransform`
  - `get_NeckTransform`
  - `set_NeckTransform`
  - `get_ChestTransform`

### `IHorseRaceInfo` (168m)
  - `get_HorseData`
  - `get_HorseIndex`
  - `get_CharaName`
  - `get_CardId`
  - `get_CharaId`

### `CutInHelper` (164m)
  - `MotionTypeToString`
  - `get_IsInitialized`
  - `set_IsInitialized`
  - `get_Status`
  - `set_Status`

### `GraphicSettings` (155m)
  - `get_RenderingManager`
  - `set_RenderingManager`
  - `get_IsMSAA`
  - `set_IsMSAA`
  - `get_ResolutionScale`

### `RaceUI` (145m)
  - `get_RaceIntroHorseUI`
  - `get_IsCaptureMode`
  - `Awake`
  - `OnDestroy`
  - `ReleasePauseDialog`

### `RaceLoaderManager` (140m)
  - `get_RaceResultPrefab`
  - `get_RaceResultBoardPrefab`
  - `get_RaceResultVariantPrefab`
  - `get_ResultSceneUIPrefab`
  - `get_MessagePlateUIFlashPrefab`

### `KeyframeCameraPlayer` (134m)
  - `get_TargetCamera`
  - `get_IsFlip`
  - `set_IsFlip`
  - `get_LookAtPosition`
  - `set_LookAtPosition`

### `DrivenKeyComponent` (130m)
  - `get_FaceOverrideController`
  - `get_FaceOverrideData`
  - `get_IsFaceOverrideEnabled`
  - `set_IsFaceOverrideEnabled`
  - `get_IsValidFaceOverride`

### `ServerDefine` (127m)
  - `get_ChangeDayHour`
  - `get_MaxTrainerPoint`
  - `get_TrainerPointRecoveryTime`
  - `get_TrainerPointRecoveryUnitValue`
  - `get_TrainerPointRecoveryCalcBase`

### `GallopUtil` (126m)
  - `get_DefaultUserName`
  - `GetUserName`
  - `ConvertCSV`
  - `SetSleepEnable`
  - `PushOpenUrlConfirmDialog`

### `HorseRaceAIBase` (124m)
  - `get_OwnerDistance`
  - `get_OwnerLane`
  - `get_OwnerRunningStyle`
  - `get_OwnerRunningStyleEx`
  - `get_OwnerPhase`

### `HeroesStage1RacingBaseViewController` (123m)
  - `get_IsPlayingHighlightRace`
  - `set_IsPlayingHighlightRace`
  - `get__workHeroesData`
  - `get__workCurrentRace`
  - `get__workSetInfo`

### `SingleModeRaceEntryViewController` (121m)
  - `get_SingleModeRaceEntryViewModel`
  - `get_IsLandscapeMode`
  - `get_PartsRivalEntry`
  - `get_AutoPlayProxy`
  - `set_AutoPlayProxy`

### `SimpleModelController` (120m)
  - `get_IdleMotionId`
  - `get_FadeType`
  - `set_FadeType`
  - `get_currentPlayMotionSetMaster`
  - `get_IdleMotionSetMaster`

### `MasterStoryDatabase` (120m)
  - `get_masterBackgroundData`
  - `set_masterBackgroundData`
  - `get_masterEventMotionData`
  - `set_masterEventMotionData`
  - `get_masterEventMotionPlusData`

### `WorkSingleModeChangeParameterInfo` (120m)
  - `get_Speed`
  - `set_Speed`
  - `get_Stamina`
  - `set_Stamina`
  - `get_Power`

### `StoryRaceUI` (120m)
  - `get_IsLandscape`
  - `SetRaceUIActive`
  - `SetOrientation`
  - `SetupUI`
  - `OnLoadEnd`

### `CharaPropController` (117m)
  - `IsMultiPropId`
  - `get_PropId`
  - `get_Animator`
  - `get_AnimatorOverride`
  - `get_PropAnimId`

### `SingleModeChangeViewManager` (117m)
  - `get_NowStateId`
  - `set_NowStateId`
  - `get_EventInfoAccesor`
  - `set_EventInfoAccesor`
  - `get_LastCheckRaceAlertDialog`

### `RaceViewBase` (116m)
  - `get_IsReady`
  - `get_IsModelInitialized`
  - `set_IsModelInitialized`
  - `get_UmaineMarkerInfoDict`
  - `get_PlayerHorseIndex`

### `SingleModeTrainingCutInHelper` (115m)
  - `Init`
  - `SetEndTimeRoundType`
  - `SkipRuntime`
  - `SnapShot`
  - `SetDrawSnapShot`

### `GameDefine` (114m)
  - `IsSpecialChara`
  - `IsScenarioUniqueChara`
  - `IsNotAvailableTargetChara`
  - `GetSpecialCharaDefaultDressId`
  - `GetCharaReplaceDressID`

### `StorySceneController` (114m)
  - `get_DisplayMode`
  - `get_FrameBuffer`
  - `set_FrameBuffer`
  - `get_CameraController`
  - `CopyCameraRenderParameters`

### `ModelLoader` (112m)
  - `get_PROPERTY_ID_TEXTURE_DIFFUSE`
  - `get_PROPERTY_ID_TEXTURE_MULTI_BASE`
  - `get_PROPERTY_ID_TEXTURE_MULTI_OPTION`
  - `get_PROPERTY_ID_TEXTURE_TOON`
  - `get_PROPERTY_ID_TEXTURE_AREA`

### `CySpringController` (112m)
  - `get_HasSpringRateArray`
  - `SetPartsSpringRate`
  - `get_SpringRate`
  - `set_SpringRate`
  - `get_PreviousPos`

### `SingleModeMainViewStablesPanel` (111m)
  - `SetupTrainingButtonBadgeArc`
  - `IsSelectionRaceBadge`
  - `SetupScenarioBreeders`
  - `SetupCommandExpGain`
  - `SetupScenarioCook`

### `SingleModeMainViewController` (110m)
  - `get_IsTapBackGroundBlack`
  - `set_IsTapBackGroundBlack`
  - `get_IsIgnoreBgm`
  - `set_IsIgnoreBgm`
  - `get_EnableBackKey`

### `HorseData` (107m)
  - `get_charaName`
  - `set_charaName`
  - `get_ResponseHorseData`
  - `get_Popularity`
  - `set_Popularity`

### `TeamStadiumRaceListViewController` (107m)
  - `RegisterDownload`
  - `IgnoreBgm`
  - `GetDynamicBgmId`
  - `GetDynamicBgmCueInfo`
  - `InitializeView`

### `MiniCharaObject` (106m)
  - `get_Model`
  - `get_Agent`
  - `get_TimelineActor`
  - `get_HeadLocator`
  - `get_BgData`

### `HomeDirector` (105m)
  - `get_CurrentStoryDict`
  - `get_CurrentCameraPos`
  - `get_IsEnableAsyncCreateTalkCharacter`
  - `IsAsyncCreateTalkCharacter`
  - `SetupOnPreRegisterDownload`

### `RaceViewReplay` (104m)
  - `get_EnvCameraDepth`
  - `get_UmaineMarkerInfoDict`
  - `Init`
  - `IsNeedMarker`
  - `InitGateCamera`

### `EditableCharacterBuildInfo` (103m)
  - `set_CardId`
  - `get_CardId`
  - `set_CharaId`
  - `get_CharaId`
  - `set_MobId`

### `MasterCampaignDatabase` (103m)
  - `get_masterCampaignData`
  - `set_masterCampaignData`
  - `get_masterCampaignCharaStorySchedule`
  - `set_masterCampaignCharaStorySchedule`
  - `get_masterCampaignSingleRaceAddData`

### `WorkHeroesData` (102m)
  - `get_HeroesId`
  - `get_Stage`
  - `get_StageStep`
  - `get_FirstAccessFlag`
  - `get_FirstAccessUmaineTipsFlag`

### `TextCommon` (101m)
  - `get_IsIgnoreParentColor`
  - `set_IsIgnoreParentColor`
  - `get_recieveColor`
  - `set_recieveColor`
  - `get__recieveColorOld`

### `LiveTheaterInfo` (101m)
  - `get_SortId`
  - `set_SortId`
  - `get_IsSkipStory`
  - `set_IsSkipStory`
  - `get_IsNew`

### `JikkyoControllerBase` (100m)
  - `get_SilentTimeMaxSec`
  - `get_ImmidiateSuspendTime`
  - `get_SceneType`
  - `get_IsExistPlayer`
  - `get_CurMode`

### `ObscuredRaceHorseData` (98m)
  - `get_ViewerId`
  - `set_ViewerId`
  - `get_OwnerViewerId`
  - `set_OwnerViewerId`
  - `get_TrainerName`

### `ButtonCommon` (98m)
  - `get_InvalidateMaterial`
  - `set_InvalidateMaterial`
  - `get_CanvasGroup`
  - `get_IsPlayScaleAnimeCurrent`
  - `set_IsPlayScaleAnimeCurrent`

### `HeroesStage1GrandResultViewController` (98m)
  - `get__workHeroesData`
  - `get__workGrandResultInfo`
  - `OverrideDynamicNowLoadingType`
  - `RegisterDownload`
  - `InitializeView`

### `PartsEpisodeList` (96m)
  - `get_CHARA_BG_PATH`
  - `get_EXTRA_BG_PATH`
  - `get_EXTRA_MOVIE_BG_PATH`
  - `get_CurrentScrollPosY`
  - `get_CurrentPartData`

### `SingleModeResultSequence` (95m)
  - `get_CurrentStep`
  - `Setup`
  - `Setup`
  - `SetSingleModeResultDataContainer`
  - `InitializeSystemText`

### `MyPageHomeTopUI` (94m)
  - `get_TouchArea`
  - `set_TouchArea`
  - `get_OverSafeAreaHeaderRoot`
  - `set_OverSafeAreaHeaderRoot`
  - `get_CharaMessage`

### `ModelControllerBehaviour` (93m)
  - `get_MeshHandle`
  - `get_UpdateOrder`
  - `get_ParentHeadConstraintData`
  - `get_ParentBodyConstraintData`
  - `get_HasParentHeadConstraint`

### `MasterHomeDatabase` (93m)
  - `get_masterHomeEnvSetting`
  - `set_masterHomeEnvSetting`
  - `get_masterHomeWalkGroup`
  - `set_masterHomeWalkGroup`
  - `get_masterHomeStoryTrigger`

### `RaceCameraEventBase` (93m)
  - `get_HasBusTarget`
  - `set_HasBusTarget`
  - `get_BusTargetPos`
  - `set_BusTargetPos`
  - `get_BusTargetForward`

### `CharacterBuildPathInfo` (91m)
  - `GetCharacterHeadModelFilePath`
  - `GetCharacterTailModelFilePath`
  - `GetCharacterAttachModelFilePath`
  - `GetCharacterHeadDirPath`
  - `GetBodyMeshPath`

### `SingleModeMainTrainingCuttController` (90m)
  - `TryGetPioneerOverrideTrainingLevel`
  - `PlayExecTrainingBeforePlayCutScenarioPioneer`
  - `PlayExecTrainingBeforePlayCutPioneerShimaTraining`
  - `CreateStartShimaTrainingCuttContext`
  - `PlayTagTrainingPioneerShimaTraining`

### `RaceSkillCutInHelper` (88m)
  - `get_NormalEffectLayer`
  - `get_AvoidEffectLayer`
  - `get_OrthoLayer`
  - `set_CharaColor`
  - `set_CharaSubColor`

### `JobsResultViewController` (87m)
  - `get__workJobsData`
  - `get__workJobsResultLimitInfo`
  - `RegisterDownload`
  - `InitializeView`
  - `InitializeEachPlayIn`

### `TeamStadiumGrandResultViewController` (87m)
  - `RegisterDownload`
  - `InitializeView`
  - `InitializeEachPlayIn`
  - `UpdateIsReplay`
  - `PlayInView`

### `StoryTimelineTextClipData` (87m)
  - `GetChoiceParam`
  - `GetLoopCountKey`
  - `GetLoopCountKeyDefault`
  - `GetLoopCountKeyByChoice`
  - `GetMaxLoopCount`

### `WorkTeamBuildingData` (86m)
  - `get_TeamBuildingEventId`
  - `get_MasterTeamBuildingData`
  - `get_MyTeamInfo`
  - `get_TotalRaceCount`
  - `get_TotalWinCount`

### `SingleModeScenarioSportCompetitionTopViewController` (86m)
  - `get_Model`
  - `RegisterDownload`
  - `GetDynamicBgmId`
  - `GetDynamicBgmCueInfo`
  - `InitializeView`

### `MenuShopViewController` (85m)
  - `RegisterDownload`
  - `InitializeEachPlayIn`
  - `PlayInView`
  - `CheckShowDialogOnReturning`
  - `SetCharacterBg`

### `MasterHeroesDatabase` (83m)
  - `get_masterHeroesData`
  - `set_masterHeroesData`
  - `get_masterHeroesStageSchedule`
  - `set_masterHeroesStageSchedule`
  - `get_masterHeroesLeagueRank`

### `ChampionsViewController` (83m)
  - `get_CurrentBgType`
  - `set_CurrentBgType`
  - `get_IsEditedEntryChara`
  - `set_IsEditedEntryChara`
  - `GetBackButtonAnimationDelayTime`

### `HeroesTopViewController` (83m)
  - `get__workHeroesData`
  - `PreRegisterDownload`
  - `RegisterDownload`
  - `InitializeView`
  - `InitializeEachPlayIn`

### `RaceUINull` (83m)
  - `SetOrientation`
  - `SetRaceUIActive`
  - `SetupUI`
  - `OnLoadEnd`
  - `OnFinalizeView`

### `SingleModeScenarioTeamRaceOpponentSelectViewController` (83m)
  - `get_HasSuperTeam`
  - `get_IsPlaySuperTeamAnim`
  - `RegisterDownload`
  - `InitializeView`
  - `UpdateView`

### `IRaceUI` (82m)
  - `SetOrientation`
  - `SetRaceUIActive`
  - `SetupUI`
  - `OnLoadEnd`
  - `OnFinalizeView`

### `StoryView` (82m)
  - `get_PortraitViewportRect`
  - `set_PortraitViewportRect`
  - `get_PortraitViewportCenterRect`
  - `set_PortraitViewportCenterRect`
  - `get_FlashRoot`

### `CharacterBuildInfo` (81m)
  - `CreateMiniMobData`
  - `SetMiniMobParam`
  - `get_CardId`
  - `get_CharaId`
  - `get_MobId`

### `ItemIcon` (81m)
  - `get_Button`
  - `get_CurrentSizeType`
  - `get_SeType`
  - `set_SeType`
  - `get_PopSe`

### `ChampionsLobbyViewController` (80m)
  - `IgnoreBgm`
  - `GetDynamicBgmId`
  - `GetDynamicBgmCueInfo`
  - `get_Work`
  - `get_RoundState`

### `CharacterCardHaveListViewController` (80m)
  - `get_SelectedPageType`
  - `get_SelectedPageState`
  - `get_BG_PATH`
  - `get_BG_PATH_VERTICAL`
  - `get_CardData`

### `RaceBGMController` (80m)
  - `get_IsNeedFirstBgmStop`
  - `get_IsNeedSecondBgmPlay`
  - `get_HasTimeAccessor`
  - `get_HasHorseAccessor`
  - `get_EntryTableBgmCueSheetName`

### `SingleModeScenarioOnsenMapSelectViewController` (80m)
  - `get_AutoPlayProxy`
  - `set_AutoPlayProxy`
  - `get_MapPinA2UList`
  - `get_OnsenItemList`
  - `GetMapPin`

### `MasterGachaDatabase` (79m)
  - `get_masterGachaData`
  - `set_masterGachaData`
  - `get_masterGachaFreeCampaign`
  - `set_masterGachaFreeCampaign`
  - `get_masterGachaAvailable`

### `SingleModeScenarioFreeUtils` (79m)
  - `GetItemName`
  - `GetItemDetailText`
  - `GetItemAboutText`
  - `IsSingleModeTurnUniqueCommandOn`
  - `IsSingleModeTurnUniqueCommandOnByCurrentTurn`

### `StoryTimelineCharaPropClipData` (79m)
  - `get_OgiginalClipLengthByMotionSpeed`
  - `get_PropCtrlDic`
  - `get_IsMotionEnabled`
  - `set_IsMotionEnabled`
  - `get_PlayedFrameCount`

### `GalleryViewController` (78m)
  - `get_GetGalleryEventManager`
  - `get_ScenarioDataList`
  - `RegisterDownload`
  - `InitializeView`
  - `InitializeEachPlayIn`

### `TimeUtil` (78m)
  - `get_DAY_SECOND`
  - `get_DAY_HOUR`
  - `get_HOUR_SECOND`
  - `get_MINUTE_SECOND`
  - `GetServerTimeStamp`

### `WorkSingleModeScenarioTeamRace` (77m)
  - `get_TeamName`
  - `set_TeamName`
  - `get_TeamNameId`
  - `set_TeamNameId`
  - `get_FinalWinType`

### `WorkDataManager` (77m)
  - `get_UserData`
  - `get_FriendData`
  - `get_CardData`
  - `get_SupportCardData`
  - `get_CharaData`

### `HeroesUtil` (77m)
  - `GoToHeroesTopView`
  - `GoToStage1RacingBaseView`
  - `GoToStage1Paddock`
  - `GoToFinalPaddockCoroutine`
  - `GoToFinalPaddockCoroutineOnAchievementReplay`

### `PartsCardListVertical` (77m)
  - `get_CharacterListUI`
  - `get_SortButton`
  - `get_Scroll`
  - `get_ViewPort`
  - `get_CurrentButtonInfoList`

### `HomeViewController` (76m)
  - `get_IsLandscapeMode`
  - `get_ViewInfo`
  - `GetDynamicBgmId`
  - `GetDynamicBgmCueInfo`
  - `get_SceneController`

### `RaceManagerStoryReplay` (76m)
  - `get_AudioCueSheetName`
  - `set_AudioCueSheetName`
  - `GetCourseDistance`
  - `get_FinishTime`
  - `get_CurrentFrame`

### `RouletteDirector2D` (76m)
  - `get_Rendered3dImage`
  - `RegisterDownload`
  - `Initialize`
  - `OnSendExecAPI`
  - `OnDestroy`

### `TeamStadiumGrandResultView` (76m)
  - `get_ScoreDetailButton`
  - `set_ScoreDetailButton`
  - `get_RaceResultButton`
  - `set_RaceResultButton`
  - `get_RetryButton`

### `TeamStadiumRaceListView` (76m)
  - `get_UseItem`
  - `set_UseItem`
  - `get_ClassText`
  - `set_ClassText`
  - `get_RaceScoreBase`

### `UIUtil` (76m)
  - `SetAnchorWithKeepingPosition`
  - `SetPivotWithKeepingPosition`
  - `GetAnchoredPosition`
  - `SetAnchoredPosition`
  - `FitChild`

### `SingleModeResultDataContainer` (75m)
  - `get_CardId`
  - `set_CardId`
  - `get_CharacterId`
  - `set_CharacterId`
  - `get_TrainedCharacterId`

### `TextUtil` (75m)
  - `ToCommaSeparatedString`
  - `ToCommaSeparatedString`
  - `ToCommaSeparatedString`
  - `ToStringViewerId`
  - `Format`

### `DialogCommon` (74m)
  - `get_DialogRootCanvas`
  - `get_CurrentDialogObj`
  - `get_IsOpen`
  - `get_IsDispClosing`
  - `get_IsDispClosed`

### `PartsSupportCardDeckListItem` (74m)
  - `get_CharacterButtonArray`
  - `set_CharacterButtonArray`
  - `get_IsSingleMode`
  - `get_IsDirty`
  - `set_IsDirty`

### `CollectRaidTopViewController` (74m)
  - `PreRegisterDownload`
  - `RegisterDownload`
  - `RegisterPartsDownload`
  - `GetPartsCreateData`
  - `InitializeView`

### `GachaHomeTopUI` (74m)
  - `get_OverrideBgmId`
  - `CreateSetupParameterWithCardId`
  - `PreRegisterDownload`
  - `RegisterDownload`
  - `InitializeView`

### `RaceCourseCameraEvent` (74m)
  - `get_CameraIndex`
  - `get_IsCameraUpdate`
  - `get_IsCameraFrameSkip`
  - `get_CameraParamNum`
  - `get_CameraParam`

### `SingleModeLogGroupBase` (74m)
  - `get_BackTopImage`
  - `set_BackTopImage`
  - `get_BackTopImageNoContent`
  - `set_BackTopImageNoContent`
  - `get_BackBaseImage`

### `StoryCameraController` (74m)
  - `get_CharaCamera`
  - `set_CharaCamera`
  - `get_AnimationNode`
  - `set_AnimationNode`
  - `get_CameraOffset`

### `StoryViewTextControllerBase` (74m)
  - `GetCalcTextLabel`
  - `OnDestroy`
  - `IsAvailable`
  - `Initialize`
  - `Initialize`

### `SocialServiceUtility` (74m)
  - `Awake`
  - `WaitPluginInit`
  - `initAppleLogin`
  - `LoginWithApple`
  - `OnLogin`

### `MasterTeamBuildingDatabase` (73m)
  - `get_masterTeamBuildingCharaCount`
  - `set_masterTeamBuildingCharaCount`
  - `get_masterTeamBuildingCharaGroup`
  - `set_masterTeamBuildingCharaGroup`
  - `get_masterTeamBuildingCollectionChara`

### `GachaBGController` (73m)
  - `RegisterDownload`
  - `RegisterDownloadForChara`
  - `RegisterDownloadForSupport`
  - `Initialize`
  - `Delete`

### `HeroesFinalRaceListViewController` (73m)
  - `GetCurrentRound`
  - `get__workHeroesData`
  - `RegisterDownload`
  - `InitializeView`
  - `InitializeEachPlayIn`

### `PartsSingleModeScenarioCookSpecialEventViewUIHelper` (73m)
  - `get_Root`
  - `set_Root`
  - `get_Image3D`
  - `set_Image3D`
  - `get_CookTastingButton`

### `PartsItemExchangeList` (72m)
  - `get_Model`
  - `get_VerticalNormalizedPositionDictionary`
  - `GetFocusFilteredShopItemIndex`
  - `SetVerticalNormalizedPositionDictionary`
  - `SetVerticalNormalizedPositionDictionary`

### `StoryManager` (72m)
  - `get_StoryId`
  - `set_StoryId`
  - `SetPrevViewId`
  - `get_TimelineController`
  - `get_HasTimelineController`

### `PaymentUtility` (72m)
  - `get_LastCheckedTime`
  - `set_LastCheckedTime`
  - `GetProductItemParam`
  - `GetAllProductItemParam`
  - `GetProductItemPrice`

### `MasterChampionsDatabase` (69m)
  - `get_masterChampionsSchedule`
  - `set_masterChampionsSchedule`
  - `get_masterChampionsRoundSchedule`
  - `set_masterChampionsRoundSchedule`
  - `get_masterChampionsRoundDetail`

### `ChampionsUtils` (69m)
  - `IsReleasedContent`
  - `IsShowNotifyBadge`
  - `IsShowHolding`
  - `IsShowByRaceEvent`
  - `GetStartDate`

### `SingleModeMainView` (69m)
  - `get_BottomRect`
  - `set_BottomRect`
  - `get_SkillPointPanel`
  - `set_SkillPointPanel`
  - `get_CharaGradeRoot`

### `SceneManager` (69m)
  - `get__backableStateStack`
  - `get_IsRunChangeView`
  - `get_IsRunChangeInHubView`
  - `get_IsRunChangeScene`
  - `set_IsRunChangeScene`

### `CircleUtil` (68m)
  - `GetChatInstance`
  - `GetRelativeTimeText`
  - `GetRelativeTimeText`
  - `GetDigit`
  - `GetDigit`

### `DialogSupportDeckCardSelectBase` (67m)
  - `GetFormType`
  - `GetParentType`
  - `get__workCampaignRentalSupportCardData`
  - `get_EquippedSupportSerialIdArray`
  - `get_RentalEquippedSupportCardIdArray`

### `DialogCollectRaidReceiveGeneralReward` (67m)
  - `get_RewardNumUseSmallFrame`
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `RegisterPartsDownload`

### `TutorialSingleMode` (67m)
  - `GetVirtualCharaData`
  - `get_Instance`
  - `get_IsTutorial`
  - `get_IsComplete`
  - `get_IsTutorialSingleModeStart`

### `TimelineWipeController` (67m)
  - `get_WipeCircleMaterial`
  - `set_WipeCircleMaterial`
  - `get_UpdateWipeCirclePosition`
  - `set_UpdateWipeCirclePosition`
  - `get_GetFovFactor`

### `MasterStoryEventDatabase` (66m)
  - `get_masterStoryEventData`
  - `set_masterStoryEventData`
  - `get_masterStoryEventPointReward`
  - `set_masterStoryEventPointReward`
  - `get_masterStoryEventBonusCard`

### `GateCamera` (66m)
  - `get_GateCamObjRootTransform`
  - `get_GateRenderCamera`
  - `Awake`
  - `getGateCameraPosition`
  - `CalcGateOffsetX`

### `MasterCollectRaidDatabase` (65m)
  - `get_masterCollectRaidMaster`
  - `set_masterCollectRaidMaster`
  - `get_masterCollectRaidIndividualReward`
  - `set_masterCollectRaidIndividualReward`
  - `get_masterCollectRaidAllReward`

### `WorkChampionsData` (65m)
  - `get_ChampionsId`
  - `set_ChampionsId`
  - `get_EntryTimes`
  - `set_EntryTimes`
  - `get_FreeEntryTimes`

### `MultiCharacterBg` (65m)
  - `get_BGRenderer`
  - `get_ViewerCamera`
  - `get_GetCameraController`
  - `get_ResultTexture`
  - `get_ModelCameraSetList`

### `NowLoading` (65m)
  - `get_Instance`
  - `get_CurrentType`
  - `Show`
  - `ShowCustomWipeFlash`
  - `SetSkipHide`

### `PartsRaceEntryCharacterSelect` (65m)
  - `get_NoticeText`
  - `get_IsPartnerTab`
  - `get_IsTrialCharacterTab`
  - `RegisterDownload`
  - `OverrideBgPath`

### `RaceHomeTopUI` (65m)
  - `get_TeamStadiumButton`
  - `get_IsLockContentRoomMatch`
  - `get_IsLockContentPracticeRace`
  - `get_IsUseTwoLines`
  - `RegisterDownloadStatic`

### `LiveViewController` (65m)
  - `LoadExtraResource`
  - `LoadOnsenFlashController`
  - `LoadRamenFlashController`
  - `get_IsScreenModeFullPortrait`
  - `get_LiveScreenCaptureController`

### `EventCamera` (65m)
  - `get_CameraNearClipPlane`
  - `get_EventCameraPlayInterval`
  - `set_EventCameraPlayInterval`
  - `get_Controller`
  - `get_AudioListenerPosition`

### `FilterMenuEx` (64m)
  - `GetGroup`
  - `GetText`
  - `IsFactorCommon`
  - `IsFactorCommon`
  - `GetFactorCommonIndex`

### `DialogGenerateSuccessionCharaSelectConfirmPresetView` (64m)
  - `get_OnClickRightButton`
  - `set_OnClickRightButton`
  - `get_OnClickLeftButton`
  - `set_OnClickLeftButton`
  - `get_OnUpdateValue`

### `DialogSupportDeckCardSelectMulti` (64m)
  - `GetFormType`
  - `get_EquippedSupportSerialIdArray`
  - `get_RentalEquippedSupportCardIdArray`
  - `RegisterDownload`
  - `Open`

### `AnnounceViewController` (64m)
  - `get_StateUpdater`
  - `set_StateUpdater`
  - `get_AnnounceList`
  - `set_AnnounceList`
  - `get_CurrentAnnounceIdx`

### `CourseManagerReplay` (64m)
  - `get_CourseBg`
  - `get_MonitorTexture`
  - `get_BaseObj`
  - `get_Bg3dCamera`
  - `Awake`

### `SingleModeScenarioTeamRaceRaceListViewController` (64m)
  - `get_SelectedTeamPower`
  - `RegisterDownload`
  - `InitializeView`
  - `OnClickRaceButton`
  - `PlayInView`

### `SingleModeMainViewHeaderBase` (64m)
  - `get_TargetRaceFrameRoot`
  - `get_DifficultyTitle`
  - `get_TrainingChallengeTitle`
  - `set_TrainingChallengeTitle`
  - `CreateModel`

### `HubViewControllerBase` (64m)
  - `get_IsSetChildViewInstances`
  - `set_IsSetChildViewInstances`
  - `get_ChildViewControllerList`
  - `set_ChildViewControllerList`
  - `get_ChildCurrentController`

### `PartsMissionList` (63m)
  - `get_ListNum`
  - `set_ListNum`
  - `get_CanRewardGetIdList`
  - `set_CanRewardGetIdList`
  - `get_ScrollViewRectTransform`

### `JikkyoVoice` (63m)
  - `Init`
  - `Clear`
  - `Cancel`
  - `Pause`
  - `Update`

### `SingleModeStartViewController` (63m)
  - `get_SingleModeStartModel`
  - `get_IsBegin`
  - `set_IsBegin`
  - `get_Entry`
  - `set_Entry`

### `TempData` (62m)
  - `GetChampionsMissionClearNum`
  - `SetChampionsMissionClearData`
  - `AddChampionsMissionClearData`
  - `SubChampionsMissionClearData`
  - `get_ChampionsData`

### `CharacterHomeTopUI` (62m)
  - `RegisterDownloadStatic`
  - `InitializeView`
  - `InitializeEachPlayIn`
  - `PlayInView`
  - `PlayOutView`

### `JobsTopViewController` (62m)
  - `get_IsMiniCharaEmpty`
  - `get_IsPageDotEmpty`
  - `get_IsPageDotEmptyOrSingle`
  - `get__workJobsData`
  - `get__saveLoader`

### `SingleModeScenarioPioneerPlanningIslandViewController` (62m)
  - `get_AutoPlayProxy`
  - `set_AutoPlayProxy`
  - `RegisterDownload`
  - `RegisterDownloadIslandFacilityIconBg`
  - `GetDynamicBgmId`

### `TrainingParamChangeA2U` (62m)
  - `get_IsPlaying`
  - `set_IsPlaying`
  - `get_IsEnd`
  - `set_IsEnd`
  - `get_IsValid`

### `CySpring` (62m)
  - `get_SpringData`
  - `set_SpringData`
  - `get_MainWindParamData`
  - `get_SpringAddData`
  - `set_SpringAddData`

### `ResourceManager` (62m)
  - `GetViewLoadHash`
  - `GetViewRefCountLoadHash`
  - `GetViewLoadHashImpl`
  - `GetSceneLoadHash`
  - `GetModelLoaderResourceHash`

### `StoryTimelineCharaTrackData` (62m)
  - `get_Type`
  - `get_KeyType`
  - `get_UpdatePriority`
  - `get_CharacterTrackIndex`
  - `get_ModelObject`

### `CharacterButton` (61m)
  - `get_CurrentSizeType`
  - `set_CurrentSizeType`
  - `get_MyImage`
  - `get_MyButton`
  - `get_CanvasGroup`

### `MainStoryUtil` (61m)
  - `GetMainStoryPartByStoryId`
  - `GetMainStoryPart`
  - `IsLock`
  - `GetFirstEpisode`
  - `GetLastEpisode`

### `TrainedCharaListViewController` (61m)
  - `get_CharaExtendSaveNum`
  - `get_CharaSaveExtendIncreaseNum`
  - `get_CharaSaveExtendCountLimit`
  - `InitializeView`
  - `InitializeEachPlayIn`

### `RaceEntryTablePanel` (61m)
  - `get_MobCaptureRenderTextureDic`
  - `get_PageMaxNum`
  - `get_IsAutoCheckMotivation`
  - `set_IsAutoCheckMotivation`
  - `Setup`

### `PartsSingleModeScenarioMechaMainView` (61m)
  - `get_OverdriveButtonA2U`
  - `RegisterDownload`
  - `Create`
  - `Setup`
  - `SetupSimpleView`

### `ViewControllerBase`1` (61m)
  - `GetViewId`
  - `GetViewInfo`
  - `SetViewInfo`
  - `SetChildViewInfo`
  - `GetViewInfo`

### `TouchManager` (61m)
  - `get_IsEnable`
  - `set_IsEnable`
  - `get_IsEnableOnDialog`
  - `set_IsEnableOnDialog`
  - `get_TargetCamera`

### `WorkRoomMatchData` (60m)
  - `get_GuestEntryRoomList`
  - `get_CurrentRoomData`
  - `get_CurrentRoomUserList`
  - `get_MyEntryRoomList`
  - `get_JoinWatchRoomNum`

### `JukeboxHomeTopUI` (60m)
  - `get_JukeboxBgmSelector`
  - `set_JukeboxBgmSelector`
  - `get_TempSetListPlayingData`
  - `InitializeView`
  - `OpenRequestDialog`

### `CourseBaseObject` (60m)
  - `get_BaseObject`
  - `set_BaseObject`
  - `get_BaseObjectAssetHolder`
  - `get_GrassFurController`
  - `set_GrassFurController`

### `SingleModeMainViewTrainingFooterItemA2UBase` (60m)
  - `RegisterDownloadScenario`
  - `get_PlayerA2UPath`
  - `get_TipsBadgeFlashPlayerPath`
  - `get_TipsBadgeSortOffset`
  - `get_AnMotionIcoTrainingMenuName`

### `StoryMenuLandscape` (60m)
  - `get_LogButton`
  - `set_LogButton`
  - `get_FullDispButtonRoot`
  - `set_FullDispButtonRoot`
  - `get_FullDispButton`

### `TrainingBg` (60m)
  - `get_BgObjects`
  - `get_BgObjectMainIndex`
  - `get_BgObjectMain`
  - `get_SkyDome`
  - `get_EffectCamera`

### `MirrorReflection` (59m)
  - `get_MirrorClipPlaneOffset`
  - `set_MirrorClipPlaneOffset`
  - `get_BaseCamera`
  - `get_MirrorReflectionRate`
  - `set_MirrorReflectionRate`

### `WorkTrainingChallengeData` (59m)
  - `get_TrainingChallengeMasterId`
  - `set_TrainingChallengeMasterId`
  - `get_UserInfo`
  - `set_UserInfo`
  - `get_RankingState`

### `PartsTrainerInfo` (59m)
  - `get_IsMine`
  - `get_IsModifyFriend`
  - `get_IsScouted`
  - `set_IsScouted`
  - `get_FriendState`

### `ScheduleBookTopViewController` (59m)
  - `get_TempScheduleBookData`
  - `get_WorkSettingInfo`
  - `RegisterDownload`
  - `InitializeView`
  - `InitializeEachPlayIn`

### `RaceHorseManagerBase` (59m)
  - `get_LastOrder`
  - `set_LastOrder`
  - `Init`
  - `Release`
  - `CreatePaseMakerCalculator`

### `WorkUserData` (58m)
  - `get_ViewerId`
  - `set_ViewerId`
  - `get_ViewerIdString`
  - `get_UserName`
  - `set_UserName`

### `HeroesTeamEditViewController` (58m)
  - `get__workHeroes`
  - `RegisterDownload`
  - `InitializeView`
  - `InitializeEachPlayIn`
  - `InitializeButtonFunction`

### `PartsGachaResultStampSheetGachaRoulette` (58m)
  - `RegisterDownload`
  - `Create`
  - `Setup`
  - `LoadRouletteFlash`
  - `SetupBg`

### `JobsResultView` (58m)
  - `get_RootRectTransform`
  - `get_FirstPageCanvasGroup`
  - `get_StoryEventPagePrefab`
  - `get_StoryEventPageRoot`
  - `get_TitleFlashRoot`

### `RaceViewNull` (58m)
  - `get_RaceCameraManager`
  - `get_IsReady`
  - `Release`
  - `ReleaseModel`
  - `get_CourseManager`

### `RaceViewStoryReplay` (58m)
  - `get_EnvCameraDepth`
  - `get_UmaineMarkerInfoDict`
  - `get_EnvToonBrightColor`
  - `set_EnvToonBrightColor`
  - `get_EnvToonDarkColor`

### `RouletteFlashAnimation` (58m)
  - `get_IsSupportCardOverLimitBreak`
  - `get_NumRunCount`
  - `RegisterDownload`
  - `Initialize`
  - `SetResultLimitBreakInfo`

### `StoryEventTopView` (58m)
  - `get_BonusButton`
  - `set_BonusButton`
  - `get_HelpButton`
  - `set_HelpButton`
  - `get_LogoImageRoot`

### `LoopScroll` (57m)
  - `get_ItemBase`
  - `set_ItemBase`
  - `get_ScrollRect`
  - `get_Margin`
  - `get_Spacing`

### `CampaignRaffleViewController` (57m)
  - `get_BackGroundPath`
  - `get_CampaignRaffleViewModel`
  - `RegisterDownload`
  - `RegisterDownloadBGM`
  - `RegisterDownloadStory`

### `PhotoStudioCuttController` (57m)
  - `get_BreedersCutController`
  - `get_BreedersReviewCutController`
  - `get_BreedersOverRunCutController`
  - `RegisterDownloadTraining`
  - `RegisterCuttResource`

### `StoryLogItem` (57m)
  - `get_CharaIcon`
  - `set_CharaIcon`
  - `get_OtherIcon`
  - `set_OtherIcon`
  - `get_NameText`

### `WebViewManager` (57m)
  - `GetSafeResourceVer`
  - `GetGachaURLProperty`
  - `IsSelectPickupGacha`
  - `GetSelectPickupGachaURLProperty`
  - `GetGachaUrl`

### `TimelineLiveStreamingController` (57m)
  - `get_IsEnable`
  - `get_IsRichFilter`
  - `get_IsSimpleFilter`
  - `get_CommentScrollDeltaY`
  - `get_CurrentModuleSettingData`

### `StoryTimelineTrackData` (57m)
  - `get_Type`
  - `get_KeyType`
  - `get_TimelineController`
  - `HasPrevClip`
  - `get_PreviousBlockIndex`

### `CutInModelController` (56m)
  - `get_NeckTransform`
  - `set_NeckTransform`
  - `get_ChestTransform`
  - `set_ChestTransform`
  - `get_TransformCanceler`

### `EpisodeExtraCommercialViewController` (56m)
  - `get_WorkExtraCommercialData`
  - `get_IsEmpty`
  - `RegisterDownload`
  - `InitializeEachPlayIn`
  - `SetupUpperUI`

### `MapEventView` (56m)
  - `get_LogoContentsRectTransform`
  - `set_LogoContentsRectTransform`
  - `get_LogoImage`
  - `set_LogoImage`
  - `get_TermText`

### `CourseBg` (56m)
  - `get_FogColor`
  - `set_FogColor`
  - `get_FogMinDistance`
  - `set_FogMinDistance`
  - `get_FogMaxDistance`

### `RaceSoundReplay` (56m)
  - `Update`
  - `Initialize`
  - `Release`
  - `CreateResidentCrowdController`
  - `CreateEventCrowdController`

### `RoomMatchLobbyViewController` (56m)
  - `get_CanStartRaceTime`
  - `get_CanForceRaceStartTime`
  - `RegisterDownload`
  - `InitializeView`
  - `InitializeEachPlayIn`

### `GardenFacilityData` (56m)
  - `get_FacilityInfo`
  - `get_VegetableType`
  - `get_LevelMasterList`
  - `get_CurrentLevel`
  - `get_TargetLevel`

### `SingleModeMainViewScenarioPioneerController` (56m)
  - `RegisterDownload`
  - `SetupCore`
  - `UpdateCommonUIPosition`
  - `GetShowTurnStartNotice`
  - `IsNeedPlayShimaTrainingWarning`

### `ShortStoryPlayer` (56m)
  - `Play`
  - `RegisterDownload`
  - `InitTimelineController`
  - `SetupDotText`
  - `PreventCySpringAccident`

### `DialogManager` (56m)
  - `OnInitialize`
  - `get_IsShowErrorDialog`
  - `get_PrevClickedButtonPosition`
  - `set_PrevClickedButtonPosition`
  - `get_DispDialogCount`

### `MaterialPropertyBlock` (56m)
  - `op_Implicit`
  - `SetTexture`
  - `SetTexture`
  - `SetTexture`
  - `get_isEmpty`

### `WorkCampaignWalkingData` (55m)
  - `get_TodayWalkingNum`
  - `get_WalkingLimitOneDay`
  - `get_TodayWalkingLeft`
  - `get_WalkingGauge`
  - `get_WalkingGaugeMax`

### `WorkMapEventData` (55m)
  - `get_EventId`
  - `get_GaugeValue`
  - `get_CurrentMapPointId`
  - `get_CurrentAreaId`
  - `get_IsAreaOpen`

### `MapEventViewController` (55m)
  - `RegisterDownload`
  - `InitializeView`
  - `InitializeEachPlayIn`
  - `PlayInView`
  - `PlayMoveStartAnimation`

### `ProfileCardEdit` (55m)
  - `SetServerRecievedHash`
  - `Init`
  - `SetFirstPage`
  - `BackEditpage`
  - `PlayOutContentAnimation`

### `SingleModeScenarioTeamRaceUtils` (55m)
  - `GetSimpleTeamRaceResultByRound`
  - `GetMainRaceResult`
  - `IsEnableTeamRaceContinue`
  - `IsEnableTeamRaceContinue`
  - `GetRunRaceDeckDataByCharaId`

### `CySpringBoneBase` (55m)
  - `get_BoneName`
  - `get_ParentBone`
  - `get_Transform`
  - `get_GameObject`
  - `get_StiffnessForce`

### `FlashPlayer` (55m)
  - `set_SortOffset`
  - `get_SortOffset`
  - `set_SortLayer`
  - `get_SortLayer`
  - `set_IsOneShot`

### `PartsCharacterCardFrame` (54m)
  - `get_LockIcon`
  - `Initialize`
  - `Setup`
  - `SetupFavoriteIcon`
  - `SetActiveFavoriteIcon`

### `RaceFitAssistanceUtil` (54m)
  - `GetEventLogo`
  - `GetEventLogoDefault`
  - `GetEventLogoChampions`
  - `GetEventLogoBaseChampions`
  - `GetChampionsLogoResourceId`

### `HeroesStage1RacingBaseView` (54m)
  - `get_UseItem`
  - `set_UseItem`
  - `get_TotalScoreRoot`
  - `set_TotalScoreRoot`
  - `get_ScoreEffectTarget`

### `StoryChoiceButton` (54m)
  - `get_ChoiceParam`
  - `get_IsVisible`
  - `set_IsVisible`
  - `get_EanbleButton`
  - `get_MaxCount`

### `DialogScheduleBookResult` (53m)
  - `get_PageDailyRaceResume`
  - `get_PageDailyRace`
  - `get_PageDailyLegendRaceResume`
  - `get_PageDailyLegendRace`
  - `get_PageTeamStadiumResume`

### `CourseManager` (53m)
  - `get_IsReady`
  - `set_IsReady`
  - `Awake`
  - `OnDestroy`
  - `ReleaseMonitorTexture`

### `TeamStadiumUtil` (53m)
  - `GetRankText`
  - `GetPointText`
  - `GetBonusPointText`
  - `GetFunCountText`
  - `GetRpTimeText`

### `MiniDirector` (52m)
  - `get_BgParam`
  - `set_BgParam`
  - `get_CharaParam`
  - `set_CharaParam`
  - `get_CameraParam`

### `PhotoStudioViewController` (52m)
  - `get_BG_PATH`
  - `get_Model`
  - `RegisterDownload`
  - `InitializeView`
  - `InitializeCharaViewer`

### `AbstractSingleModeMainScenarioController` (52m)
  - `get_ScenarioParts`
  - `set_ScenarioParts`
  - `RegisterDownload`
  - `Initialize`
  - `Setup`

### `TeamStadium3DController` (52m)
  - `RegisterDownload`
  - `Initialize`
  - `CreateBgTexture`
  - `CreateBgTextureForSquare`
  - `CreateBgTextureFromVerticalBg`

### `ChallengeMatchTopViewController` (51m)
  - `GetDynamicBgId`
  - `PreRegisterDownload`
  - `RegisterDownload`
  - `InitializeView`
  - `InitializeEachPlayIn`

### `Jikkyo` (51m)
  - `get_SceneType`
  - `get_NearGoalDistance`
  - `get_NearGoalInterruptDisableDistance`
  - `get_CommentDisableDistance`
  - `GetFirstHorseDistance`

### `SingleModeRaceEntryView` (51m)
  - `get_PlayerInfoFrame`
  - `get_FanCountRootObj`
  - `get_FanCountNumText`
  - `get_CharacterDetailButton`
  - `set_CharacterDetailButton`

### `PartsSingleModeResultFactorLottery` (51m)
  - `RegisterDownload`
  - `Open`
  - `UpdateFactorSelectButton`
  - `SetupButton`
  - `SetupNextButton`

### `DialogUpdateTeamEvaluationPoint` (51m)
  - `get_CloseButton`
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `Open`

### `ObscuredSuccessionGainInfo` (50m)
  - `get_LotteryId`
  - `set_LotteryId`
  - `get_Speed`
  - `set_Speed`
  - `get_Stamina`

### `WorkPracticeRaceData` (50m)
  - `get_State`
  - `get_ResumePracticeRaceId`
  - `get_CurrentRaceData`
  - `get_SavedRaceList`
  - `get_CurrentRaceEntryCharaList`

### `HorseRaceInfoReplay` (50m)
  - `get_RunMotionRate`
  - `get_RaceMotion`
  - `get_RunMotionSpeed`
  - `get_IsTemptation`
  - `get_TemptationCount`

### `RaceUIStatusPlate` (50m)
  - `get_IsUsing`
  - `set_IsUsing`
  - `get_IsStartPlaying`
  - `set_IsStartPlaying`
  - `get_IsHidePlaying`

### `SingleModeScenarioMechaTuningViewController` (50m)
  - `get_BgEffectPath`
  - `GetDynamicBgId`
  - `GetDynamicBgmId`
  - `GetDynamicBgmCueInfo`
  - `RegisterDownload`

### `SingleModeLogItem` (50m)
  - `get_CharaIcon`
  - `set_CharaIcon`
  - `get_OtherIcon`
  - `set_OtherIcon`
  - `get_NameText`

### `RaceResultCutInHelper` (50m)
  - `get_PlayIndex`
  - `InitWithBackgroundRT`
  - `SetPlayNum`
  - `Setup`
  - `RegisterCallback`

### `SimplePlayableAnimator` (49m)
  - `get_IsUseMainState`
  - `set_IsUseMainState`
  - `OnInitialize`
  - `OnDestroy`
  - `GetPlayTime`

### `HomeWalkModelController` (49m)
  - `get_CharaIdList`
  - `set_CharaIdList`
  - `get_IsFinish`
  - `get_Speed`
  - `set_Speed`

### `MasterTeamStadiumDatabase` (49m)
  - `get_masterTeamStadiumRawScore`
  - `set_masterTeamStadiumRawScore`
  - `get_masterTeamStadiumScoreBonus`
  - `set_masterTeamStadiumScoreBonus`
  - `get_masterTeamStadium`

### `EpisodeStoryData` (49m)
  - `get_Id`
  - `get_EpisodeIndex`
  - `get_Title`
  - `set_Title`
  - `get_Description`

### `PartsSingleModeResultSupportCardExpAndItem` (49m)
  - `RegisterDownload`
  - `Initialize`
  - `ShowContents`
  - `InitializeCampaign`
  - `InitializeRewardLayout`

### `StoryCharacter3D` (49m)
  - `get_CharaId`
  - `set_CharaId`
  - `get_CardId`
  - `set_CardId`
  - `get_DressId`

### `TeamBuildingTopViewController` (49m)
  - `RegisterDownload`
  - `InitializeView`
  - `InitializeEachPlayIn`
  - `PlayInView`
  - `PlayOutView`

### `MasterSingleModeStoryData` (48m)
  - `Get`
  - `_SelectOne`
  - `GetWithStoryId`
  - `_SelectWithStoryId`
  - `_CreateOrmByQueryResultWithStoryId`

### `StoryUtil` (48m)
  - `UpdateIsMark`
  - `IsMark`
  - `GetWorkStoryData`
  - `SendChangeStoryMarkInStory`
  - `ShowStoryMarkNotification`

### `HomeBgController` (48m)
  - `set_BeginMirrorColor`
  - `get_BeginMirrorColor`
  - `set_EndMirrorColor`
  - `get_EndMirrorColor`
  - `set_BeginMirrorBlendRate`

### `PhotoStudioPlayCutViewController` (48m)
  - `get_Model`
  - `get_ScreenShotFlashTat`
  - `RegisterDownload`
  - `RegisterDownloadSingleModeScenarioLegend`
  - `RegisterDownloadSingleModeScenarioOnsenIfNeeded`

### `RaceGateSetCutController` (48m)
  - `RegisterDownloadPath`
  - `Initialize`
  - `PlayCutIn`
  - `EndCutIn`
  - `AlterUpdate`

### `PartsSingleModeSuccessionFactorSelect` (48m)
  - `get_RawScrollIndex`
  - `get_IsPlayingScrollSequence`
  - `get_CurrentLotteryInfo`
  - `get_CurrentLotteryId`
  - `get_WrappedScrollIndex`

### `SingleModeMainViewScenarioArcController` (48m)
  - `RegisterDownload`
  - `SetupCore`
  - `InitializeEachPlayInFromAdditiveView`
  - `UpdateCommonUIPosition`
  - `FinalizeView`

### `PartsSingleModeScenarioCookCookingCutRunner` (48m)
  - `PlayCookingCut`
  - `OnUpdateCookingCuttSpeed`
  - `OnStartCookingCutt`
  - `OnEndCookingCutt`
  - `CreateCookImageObjects`

### `SingleModeScenarioPioneerPlanningIslandView` (48m)
  - `get_IslandRoot`
  - `set_IslandRoot`
  - `get_ShimaView`
  - `set_ShimaView`
  - `get_CharaDetailButton`

### `SingleModeStartStepRouteSelect` (48m)
  - `RegisterPath`
  - `InitializeEachPlayIn`
  - `EndView`
  - `SetupHorizontalFade`
  - `OnFlick`

### `StoryEventUtil` (48m)
  - `GetNextRewardItem`
  - `GetLastRewardItem`
  - `GetPrevRewardItem`
  - `SetupRewardItemIcon`
  - `GetEventName`

### `TeamBuildingScoutViewController` (48m)
  - `get__workTeamBuilding`
  - `get_CharacterButtonNoticeModelFactory`
  - `RegisterDownload`
  - `InitializeView`
  - `InitializeEachPlayIn`

### `UltimateRaceViewController` (48m)
  - `PreRegisterDownload`
  - `RegisterDownload`
  - `InitializeEachPlayIn`
  - `PlayInView`
  - `PlayCutIn`

### `IViewController` (48m)
  - `GetViewId`
  - `SetFromSceneManager`
  - `SetViewBase`
  - `GetViewBase`
  - `GetViewInfo`

### `GallopImageEffect` (47m)
  - `get_CameraData`
  - `get_TargetCamera`
  - `set_TargetCamera`
  - `get_IsInitialized`
  - `get_IsUseImageEffect`

### `MasterCampaignData` (47m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithTargetTypeOrderByCampaignIdAsc`
  - `_SelectWithTargetTypeOrderByCampaignIdAsc`

### `WorkCollectRaidData` (47m)
  - `get_EventId`
  - `get_MasterCollectRaidMaster`
  - `get_IndividualCollectItemNum`
  - `get_AllCollectItemNum`
  - `get_EventTopPollingTime`

### `WorkJobsData` (47m)
  - `get_PreStartInfo`
  - `get_GoingJobInfoList`
  - `get_ResultJobInfoList`
  - `get_ResultLimitInfo`
  - `get_ResultStoryEventInfoData`

### `GenerateSuccessionCharaStartViewController` (47m)
  - `get_IsBegin`
  - `set_IsBegin`
  - `get_OpenDialogAction`
  - `set_OpenDialogAction`
  - `get_GenerateInfoData`

### `HeroesStage1GrandResultView` (47m)
  - `get_DarkPanel`
  - `set_DarkPanel`
  - `get_FlashRoot`
  - `set_FlashRoot`
  - `get_TicketRoot`

### `CharacterNoteDressChangeUI` (47m)
  - `get_DressUIRoot`
  - `get_CharaFadeImage`
  - `set_CharaFadeImage`
  - `get_IsMini`
  - `get_CurrentDressId`

### `SupportCardWaitingRoomViewController` (47m)
  - `get_BG_PATH`
  - `GetBgPath`
  - `RegisterDownload`
  - `InitializeView`
  - `InitializeEachPlayIn`

### `IRaceView` (47m)
  - `get_IsReady`
  - `Init`
  - `InitModels`
  - `ReleaseModel`
  - `Release`

### `SingleModeScenarioLegendReputationPhotoCutInController` (47m)
  - `get_CutInController`
  - `RegisterDownload`
  - `RegisterDownloadPage00`
  - `RegisterDownloadPage01`
  - `Initialize`

### `SingleModeMainCharaController` (47m)
  - `get_UseMainViewFocusCameraScenarioPreset`
  - `get_UseTrainingViewFocusCameraScenarioPreset`
  - `get_UseRealTimeShadow`
  - `SetupBGCharaModel`
  - `SetupTrainingBGCharaModel`

### `ObscuredIdleSingleModeEndInfo` (46m)
  - `get_CharaInfo`
  - `set_CharaInfo`
  - `get_RaceConditionArray`
  - `set_RaceConditionArray`
  - `get_UncheckedEventArray`

### `WorkRouletteDerbyData` (46m)
  - `get_CoinNum`
  - `get_BingoSheetNum`
  - `get_NextBingoSheetNum`
  - `get_ExecCount`
  - `get_DirectorOrder`

### `WorkTeamStadiumData` (46m)
  - `get_TeamStadiumInfo`
  - `set_TeamStadiumInfo`
  - `get_TeamStadiumDeckInfo`
  - `set_TeamStadiumDeckInfo`
  - `get_TeamStadiumStatus`

### `JukeboxBgmSelector` (46m)
  - `get_CurrentJukeboxMode`
  - `get_JukeboxBgmDic`
  - `set_JukeboxBgmDic`
  - `get_CurrentBgmMusicId`
  - `set_CurrentBgmMusicId`

### `HomeSceneController` (46m)
  - `get_ChangeCameraSubject`
  - `get_ChangeTopSubject`
  - `get_PlayOutSubject`
  - `get_TapScreenSubject`
  - `get_TouchCharacterSubject`

### `SingleModeScenarioMechaUtils` (46m)
  - `RegisterDownloadCommon`
  - `GetMechaCharaDressIdSet`
  - `GetMechaCharaDressIdSet`
  - `GetMechaCharaDressIdSet`
  - `DressId`

### `PartsSingleModeScenarioPioneerMainView` (46m)
  - `Create`
  - `RegisterDownload`
  - `Initialize`
  - `Setup`
  - `Setup`

### `PartsSingleModeCommonHeader` (46m)
  - `get_ContentsRoot`
  - `get_Header`
  - `set_Header`
  - `get_HpGauge`
  - `set_HpGauge`

### `SingleModeSkillLearningViewController` (46m)
  - `get_IsEnableScenarioUpgradeSkill`
  - `RegisterDownload`
  - `InitializeView`
  - `SetupScenarioUpgradeSkillButton`
  - `BeginView`

### `DrivenKeyPlayableAnimator` (46m)
  - `get_IsEnable`
  - `set_IsEnable`
  - `get_CullingMode`
  - `set_CullingMode`
  - `get_ActiveSelf`

### `RacePlayableAnimation` (45m)
  - `get_AddBlendWeight`
  - `set_AddBlendWeight`
  - `get_Speed`
  - `set_Speed`
  - `get_CurrentState`

### `HomeStandModelController` (45m)
  - `get_NeedSpecialMotion`
  - `get_StandPos`
  - `set_StandPos`
  - `get_MotionPersonality`
  - `get_IsIgnoreResetCySpringOnVisible`

### `WorkSingleModeScenarioBreedersDataSet` (45m)
  - `get_EqualityContract`
  - `get_CommandInfoArray`
  - `set_CommandInfoArray`
  - `get_TeamMemberInfoArray`
  - `set_TeamMemberInfoArray`

### `WorkFactorResearchData` (45m)
  - `get_FactorResearchEventId`
  - `set_FactorResearchEventId`
  - `get_BoxId`
  - `set_BoxId`
  - `get_GetBoxItemNum`

### `DialogHeroesLeagueRankUpCutin` (45m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `RegisterDownload`
  - `Open`

### `CircleChat` (45m)
  - `get_IsLandscapeMode`
  - `AddChatResourcesToList`
  - `Initialize`
  - `SettingPartnerShareButton`
  - `UpdateItemRequestStatus`

### `MapEventDirector` (45m)
  - `get_EndignMotionVoiceID`
  - `get_EndignMotionLipSyncDuration`
  - `get_LockedBGColor`
  - `get_UnLockedBGColor`
  - `get_ClearBGColor`

### `CampaignsValentineSpStoryViewController` (45m)
  - `RegisterDownload`
  - `InitializeView`
  - `PlayInView`
  - `BeginView`
  - `PlayOutView`

### `TeamStadiumDeckViewController` (45m)
  - `get_IsReleaseContentDailyRace`
  - `get_IsReleaseContentCircle`
  - `get_IsReleaseContentChampions`
  - `get_IsReleaseContentHeroes`
  - `get_IsReleaseContentRoomMatch`

### `CampaignTrainingCutInHelper` (45m)
  - `get_TargetRT`
  - `set_TargetRT`
  - `get_CuttPath`
  - `set_CuttPath`
  - `get_CuttContext`

### `CharaPropRendererAccessor` (44m)
  - `CheckTranslucentName`
  - `FindMaterial`
  - `GetMaterial`
  - `ApplyShader`
  - `SetMaterialColor`

### `ObscuredIdleSingleModeGainInfo` (44m)
  - `get_Speed`
  - `set_Speed`
  - `get_Stamina`
  - `set_Stamina`
  - `get_Power`

### `CollectRaidUtil` (44m)
  - `RegisterDownloadStoryRace`
  - `RegisterDownloadStoryRaceForAllEvent`
  - `GetItemNumText`
  - `GetItemNumText`
  - `GetRoundItemNumText`
