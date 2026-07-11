# 剧本 2: TeamRace (チームレース)

**WorkScenario类**: `WorkSingleModeScenarioTeamRace`
**ObscuredDataSet**: `ObscuredSingleModeTeamRaceDataSet`
---

## 相关类 (79个)

### `Gallop.MasterSingleModeTeamRaceSet` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithSuperTeamCharaIdOrderByIdAsc`
  - `_SelectWithSuperTeamCharaIdOrderByIdAsc`
  - `GetListWithSuperTeamCharaIdOrderByIdAsc`
  - `MaybeListWithSuperTeamCharaIdOrderByIdAsc`
  - `_ListSelectWithSuperTeamCharaIdOrderByIdAsc`
  - `_CreateOrmByQueryResultWithSuperTeamCharaIdOrderByIdAsc`
  - `Unload`

### `Gallop.MasterTeamBuildingTeamRaceSet` (6m)
  - `Get`
  - `_SelectOne`
  - `GetWithNpcGroupId`
  - `_SelectWithNpcGroupId`
  - `_CreateOrmByQueryResultWithNpcGroupId`
  - `Unload`

### `Gallop.SingleModeScenarioTeamRaceTrainingPartnerEntity` (3m)
  - `get_InterestState`
  - `get_SoulEventState`
  - `get_SoulThresholdId`

### `Gallop.SingleModeScenarioTeamRaceTrainingPartnerUniqueCharaEntity` (3m)
  - `get_InterestState`
  - `get_SoulEventState`
  - `get_SoulThresholdId`

### `Gallop.SingleModeScenarioTeamRaceTrainingPartnerScoutEntity` (3m)
  - `get_InterestState`
  - `get_SoulEventState`
  - `get_SoulThresholdId`

### `Gallop.ISingleModeScenarioTeamRaceTrainingPartnerEntity` (3m)
  - `get_InterestState`
  - `get_SoulEventState`
  - `get_SoulThresholdId`

### `Gallop.SingleModeScenarioTeamRaceTrainingPartnerRepository` (8m)
  - `Get`
  - `ConvertToTrainingCommandIdList`
  - `get_ScenarioId`
  - `get_WorkSingleModeHomeInfoData`
  - `Get`
  - `get_SingleModeCommandInfoDataArray`
  - `get_WorkSingleModeCharaData`
  - `get_EvaluationList`

### `Gallop.WorkSingleModeChangeParameterInfoScenarioTeamRace` (10m)
  - `get_RankingUp`
  - `set_RankingUp`
  - `get_RankingDown`
  - `set_RankingDown`
  - `get_TeamStatusUpDictionary`
  - `set_TeamStatusUpDictionary`
  - `get_TotalPower`
  - `set_TotalPower`
  - `Clear`
  - `Set`

### `Gallop.WorkSingleModeScenarioTeamRace` (77m)
  - `get_TeamName`
  - `set_TeamName`
  - `get_TeamNameId`
  - `set_TeamNameId`
  - `get_FinalWinType`
  - `set_FinalWinType`
  - `get_IsBossBattle`
  - `get_CanEnterRace`
  - `get_AddMusicId`
  - `ClearAddMusicId`
  - `get_TeamParameterRankSpeed`
  - `set_TeamParameterRankSpeed`
  - `get_TeamParameterRankStamina`
  - `set_TeamParameterRankStamina`
  - `get_TeamParameterRankPower`
  - `set_TeamParameterRankPower`
  - `get_TeamParameterRankGuts`
  - `set_TeamParameterRankGuts`
  - `get_TeamParameterRankWiz`
  - `set_TeamParameterRankWiz`

### `Gallop.RaceTitlePlayerTweenAnimationSingleModeTeamRace` (5m)
  - `SetupTweenAnimationInstance`
  - `Stop`
  - `Pause`
  - `Resume`
  - `GetTitleBGObject`

### `Gallop.MessagePlateTeamRace` (21m)
  - `get_BaseSkillObjName`
  - `get_SkillNameObjName`
  - `get_PlateEffectMotionName`
  - `get_PlateEffectBlinkMotionName`
  - `GetSkillNameObjName`
  - `InitStrideYStatus`
  - `InitStrideYSkill`
  - `PlaySeSkill`
  - `PlayOut`
  - `InitMotionSkill`
  - `InitMotionSkillSelf`
  - `InitMotionSkillEnemy`
  - `GetSkillSelfLabel`
  - `GetSkillEnemyLabel`
  - `GetChampionsTeamColor`
  - `InitMotionStatus`
  - `InitTextureSkill`
  - `InitCharaTextureUV`
  - `CacheCharaMesh`
  - `InitTextSkill`

### `Gallop.RaceResultSceneUIForTeamRace` (40m)
  - `get_IsPhotoCheckAwake`
  - `set_IsPhotoCheckAwake`
  - `Setup`
  - `InitSafeArea`
  - `PlayFinishOrder`
  - `CacheTeamScoreActiveInfo`
  - `OnOffTeamScore`
  - `PauseFinishOrder`
  - `IsPlayedFinishOrder`
  - `Hide`
  - `GetResultBonusData`
  - `CreateTeamScore`
  - `GetCharaResult`
  - `GetTargetHorses`
  - `CalcFlashScale`
  - `GetDispDesc`
  - `CalcFinishOrderDispTime`
  - `PlayHorseCoroutine`
  - `PlayFlash`
  - `PauseFlash`

### `Gallop.DialogSingleModeScenarioTeamRaceAnalyze` (6m)
  - `GetFormType`
  - `GetParentType`
  - `Setup`
  - `SetSortingOrder`
  - `PushDialog`
  - `PrecallTeamRaceAnalyzeApi`

### `Gallop.DialogSingleModeScenarioTeamRaceAoharuRaceList` (5m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `Setup`
  - `Open`

### `Gallop.DialogSingleModeScenarioTeamRaceAutoBuildOn` (3m)
  - `GetFormType`
  - `GetParentType`
  - `Open`

### `Gallop.DialogSingleModeScenarioTeamRaceContinue` (13m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `Open`
  - `Setup`
  - `SetupAllResult`
  - `SetupRank`
  - `OnClickContinue`
  - `GetContinueItemNum`
  - `OnClickCancel`
  - `SetOnClickSkillSelectButton`
  - `<Setup>b__18_1`
  - `<SetOnClickSkillSelectButton>b__24_0`

### `Gallop.DialogSingleModeScenarioTeamRaceNeedMember` (7m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `OnOpenDialog`
  - `PushDialogScenarioTeamRace`
  - `Setup`
  - `IsNeedOpenScenarioTeamRace`

### `Gallop.DialogSingleModeScenarioTeamRaceNextAoharuRaceList` (13m)
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
  - `<PlayContentsIn>b__15_0`
  - `<PlayContentsIn>b__15_1`

### `Gallop.DialogSingleModeScenarioTeamRaceRaceInfo` (6m)
  - `CreateDialogData`
  - `GetParentType`
  - `GetFormType`
  - `RegisterDownload`
  - `PushDialog`
  - `Setup`

### `Gallop.DialogSingleModeScenarioTeamRaceResultList` (7m)
  - `CreateDialogData`
  - `GetParentType`
  - `GetFormType`
  - `PushDialog`
  - `Setup`
  - `SetupRaceHorseList`
  - `OnClickClose`

### `Gallop.DialogSingleModeScenarioTeamRaceSupportCardDetail` (4m)
  - `PushDialog`
  - `PushDialog`
  - `SetupMemberStatus`
  - `SetupGuestSkillScrollView`

### `Gallop.DialogSingleModeScenarioTeamRaceTeamInfo` (14m)
  - `GetFormType`
  - `GetParentType`
  - `RegisterDownload`
  - `Setup`
  - `GetTeamMemberNum`
  - `ToggleScrollContent`
  - `ShowMemberList`
  - `ShowDeckInfo`
  - `SetupCharacter`
  - `SetupMember`
  - `SetupTeamDeckInfo`
  - `PushDialog`
  - `AsyncPushDialog`
  - `<ShowMemberList>b__43_0`

### `Gallop.DialogSingleModeScenarioTeamRaceTopAutoBuild` (3m)
  - `GetFormType`
  - `GetParentType`
  - `Open`

### `Gallop.PartsSingleModeScenarioTeamRaceAllRaceListItem` (17m)
  - `get_COLOR_ID`
  - `get_Button`
  - `get_RoundResult`
  - `SetUp`
  - `OnClick`
  - `ShowRaceResult`
  - `PlayResultSequence`
  - `OnDestroy`
  - `<ShowRaceResult>b__40_0`
  - `<PlayResultSequence>b__41_1`
  - `<PlayResultSequence>b__41_2`
  - `<PlayResultSequence>b__41_10`
  - `<PlayResultSequence>b__41_4`
  - `<PlayResultSequence>b__41_6`
  - `<PlayResultSequence>g__SetValue|41_7`
  - `<PlayResultSequence>g__SetScale|41_8`
  - `<PlayResultSequence>g__SetAddColor|41_9`

### `Gallop.PartsSingleModeScenarioTeamRaceAllRaceListItemCharaIcon` (4m)
  - `Setup`
  - `SetFinishOrder`
  - `ShowFinishOrderTweenAnimation`
  - `SetButtonColor`

### `Gallop.PartsSingleModeScenarioTeamRaceAoharuRaceListItem` (7m)
  - `get_Schedule`
  - `Setup`
  - `Setup`
  - `SetBgColor`
  - `ShowRaceResultAndFadeOut`
  - `PlayResultSequence`
  - `ShowNextIconAndTurnInfo`

### `Gallop.PartsSingleModeScenarioTeamRaceImageNumberModel` (5m)
  - `get_EnableBonus`
  - `get_ImageNumberPrefabPath`
  - `get_BonusA2UPath`
  - `get_IsFlashAction`
  - `get_BonusPlateMotionObjectName`

### `Gallop.PartsSingleModeScenarioTeamRaceInfoHeader` (27m)
  - `get_RootImage`
  - `set_RootImage`
  - `get_InnerAnimationRoot`
  - `set_InnerAnimationRoot`
  - `get_TeamNameText`
  - `set_TeamNameText`
  - `get_HonorIcon`
  - `set_HonorIcon`
  - `get_HonorText`
  - `set_HonorText`
  - `get_HonorTextOutline`
  - `set_HonorTextOutline`
  - `get_RankIcon`
  - `set_RankIcon`
  - `get_TitleHeaderInfoButton`
  - `set_TitleHeaderInfoButton`
  - `get_TeamRankText`
  - `set_TeamRankText`
  - `get_RankText`
  - `set_RankText`

### `Gallop.PartsSingleModeScenarioTeamRaceMainView` (22m)
  - `Create`
  - `Setup`
  - `SetTeamName`
  - `PlayIn`
  - `PlayOut`
  - `PlayGoTrainingSelect`
  - `PlayReturnTrainingSelect`
  - `PlayInCommon`
  - `PlayExecTrainingCut`
  - `PlayInBackTrainingFromAdditiveView`
  - `SetActive`
  - `GetCurrentYearMonthAndHalf`
  - `IsNeedShowScenarioNotice`
  - `ShowScenarioNotice`
  - `OnTrainingItemSelected`
  - `SetHonorId`
  - `SetHonorId`
  - `SetPower`
  - `OnClickButton`
  - `<PlayOut>b__15_0`

### `Gallop.PartsSingleModeScenarioTeamRaceOpponent` (16m)
  - `RegisterDownload`
  - `Setup`
  - `SetupDialog`
  - `SetSortingOrder`
  - `SetupInformation`
  - `SetupSuperTeamText`
  - `ShowCharaMotion`
  - `SetupCharaImage`
  - `UpdateFadeMask`
  - `PlayIn`
  - `PlayInStrongTeam`
  - `PlayLoopStrongTeam`
  - `PlayOut`
  - `LateUpdate`
  - `<SetupCharaImage>g__SetTexture|24_5`
  - `<SetupCharaImage>g__SetTextureWithFade|24_6`

### `Gallop.PartsSingleModeScenarioTeamRaceTeamInfo` (1m)
  - `Setup`

### `Gallop.SingleModeMainCharaScenarioTeamRaceController` (3m)
  - `get_UseMainViewFocusCameraScenarioPreset`
  - `SetupBGCharaModel`
  - `SetupTeamRaceBGCharacter`

### `Gallop.SingleModeMainScenarioTeamRaceController` (5m)
  - `RegisterDownload`
  - `SetupCore`
  - `UpdateCommonUIPosition`
  - `GetShowTurnStartNotice`
  - `TutorialCommandSelectStart`

### `Gallop.SingleModeMainViewHeaderScenarioTeamRaceModel` (7m)
  - `get_RemainTurnA2UPath`
  - `GetBaseFrameSprite`
  - `get_TitleOutlineColorType`
  - `get_NeedMultiTurn`
  - `GetNextScenarioScheduleTurnNum`
  - `IsDisable`
  - `OnClickScenarioScheduleButton`

### `Gallop.SingleModeMainViewTrainingFooterItemA2UScenarioTeamRace` (3m)
  - `get_PlayerA2UPath`
  - `RegisterDownloadScenario`
  - `SetupTrainingButtonBadgeScenario`

### `Gallop.SingleModeMainViewTrainingHorseIconA2UScenarioTeamRace` (16m)
  - `get_PartnerA2UPath`
  - `get_KiwamiAnMotion`
  - `get_AoharuSoulAnMotion`
  - `get_CoachingAnMotion`
  - `RegisterDownloadScenario`
  - `SetBadgeSortOffsetScenario`
  - `SetBadgeBalloon`
  - `SetAoharuSoulBadge`
  - `SetAoharuSoulExploded`
  - `SetSpAoharuSoulExploded`
  - `PlayScenarioContentGaugeUp`
  - `PlayAoharuSoulGaugeUp`
  - `PlaySoulExplode`
  - `PlaySpSoulExplode`
  - `<SetAoharuSoulBadge>b__18_0`
  - `<PlayAoharuSoulGaugeUp>b__22_0`

### `Gallop.SingleModeMainViewTrainingHorseIconModelScenarioTeamRace` (7m)
  - `get_WorkTeamMember`
  - `get_SupportCardIdTeamRace`
  - `get_SupportCardId`
  - `get_IsGuide`
  - `get_IsSoulExplode`
  - `get_IsSpSoulExplode`
  - `OnClick`

### `Gallop.PartsSingleModeScenarioTeamRaceCharacterSelect` (34m)
  - `get_SetupComplete`
  - `set_SetupComplete`
  - `RegisterDownload`
  - `SetBg`
  - `Awake`
  - `Setup`
  - `SetupToggle`
  - `SetupRaceHeader`
  - `SetupChengeBaseCharaInfoHeader`
  - `SetRaceInstance`
  - `OnToggleSelect`
  - `SetupCardList`
  - `UpdateEntryNum`
  - `UpdateSelectButtonInteractable`
  - `UpdateNameWithStatus`
  - `OnTapCharacterButton`
  - `OnLongTapCharacterButton`
  - `OnTapUnselectButton`
  - `OnUpdateCharacterButton`
  - `OnSelectCharacter`

### `Gallop.PartsSingleModeScenarioTeamRaceCharaInfoHeader` (5m)
  - `Setup`
  - `SetupCharacterButton`
  - `SetupEmpty`
  - `SetupCurrnetSortSetting`
  - `SetupProperInfo`

### `Gallop.PartsSingleModeTeamRaceCharacterSelectArea` (11m)
  - `get_CurrentButtonInfoList`
  - `get__currentSupCharacterInfoList`
  - `Init`
  - `UpdateAllCharacterButton`
  - `ResetFilter`
  - `GetTargetCharacterButton`
  - `GetFirstSupCharaButtonInfo`
  - `SetUpSortButton`
  - `UpdateContents`
  - `SortAndFilter`
  - `UpdateCharacterListOnSort`

### `Gallop.PartsSingleModeTeamRaceCharacterSelectButtonContainer` (8m)
  - `Setup`
  - `ResetData`
  - `GetCharacterButtonInfoList`
  - `GetTargetCharacterButton`
  - `SetTitleActive`
  - `UpdateAllButton`
  - `GetTargetButton`
  - `<Setup>g__OneFrameDelayAction|8_0`

### `Gallop.SingleModeScenarioTeamRaceCharaSelectView` (2m)
  - `get_EntryCharacterSelect`
  - `set_EntryCharacterSelect`

### `Gallop.SingleModeScenarioTeamRaceCharaSelectViewInfo` (0m)

### `Gallop.SingleModeScenarioTeamRaceCharaSelectViewController` (12m)
  - `RegisterDownload`
  - `InitializeView`
  - `PlayInView`
  - `GetDynamicBgmCueInfo`
  - `PlayOutView`
  - `OnClickBackButton`
  - `OnClickOsBackKey`
  - `FinalizeView`
  - `OnDecide`
  - `OpenRunStyleDialog`
  - `<InitializeView>b__3_0`
  - `<>n__0`

### `Gallop.PartsSingleModeScenarioTeamRaceNamePlate` (5m)
  - `RegisterDownload`
  - `Setup`
  - `UpdateName`
  - `UpdateRank`
  - `PlayIn`

### `Gallop.DialogScenarioTeamRaceDeckEditConfirm` (4m)
  - `GetFormType`
  - `GetParentType`
  - `PushDialog`
  - `Initialize`

### `Gallop.PartsSingleModeScenarioTeamRaceDeckEntryItem` (11m)
  - `Gallop.MasterRaceCourseSet.IRaceCourseInfo.get_GroundType`
  - `Gallop.MasterRaceCourseSet.IRaceCourseInfo.get_DistanceType`
  - `Setup`
  - `SetupAce`
  - `SetupDefault`
  - `SetupEmpty`
  - `SetupProperInfo`
  - `OnTapIcon`
  - `UpdateRunningStyle`
  - `GoCharacterSelect`
  - `<SetupEmpty>b__25_0`

### `Gallop.PartsSingleModeScenarioTeamRaceDeckEntryListItem` (1m)
  - `Setup`

### `Gallop.SingleModeScenarioTeamRaceDeckBuilder` (25m)
  - `get_IsDeckChanged`
  - `set_IsDeckChanged`
  - `get_IsManualChanged`
  - `set_IsManualChanged`
  - `get_DeckInfo`
  - `set_DeckInfo`
  - `get_TempDeckInfo`
  - `set_TempDeckInfo`
  - `Setup`
  - `Reset`
  - `AutoBuild`
  - `OnDeckChange`
  - `ChangeMemberFromCharacterSelect`
  - `Save`
  - `OnRespondDeckEditRequest`
  - `ClearCache`
  - `HasAceEmptyRace`
  - `IsSingleModeCharaAnyAce`
  - `GetDeckTeamMemberCount`
  - `HasError`

### `Gallop.SingleModeScenarioTeamRaceDeckInfo` (8m)
  - `Initialize`
  - `Clone`
  - `Copy`
  - `GetMemberList`
  - `GetUnlockMemberList`
  - `GetMemberList`
  - `GetMember`
  - `UpdateMemberData`

### `Gallop.SingleModeScenarioTeamRaceDeckView` (20m)
  - `get_DecideButton`
  - `set_DecideButton`
  - `get_AutoButton`
  - `set_AutoButton`
  - `get_AutoButtonText`
  - `set_AutoButtonText`
  - `get_AutoBuildOn`
  - `set_AutoBuildOn`
  - `get_AutoBuildOff`
  - `set_AutoBuildOff`
  - `get_Area3D`
  - `set_Area3D`
  - `get_Image3D`
  - `set_Image3D`
  - `get_WarningText`
  - `set_WarningText`
  - `get_InfoHeader`
  - `set_InfoHeader`
  - `get_DeckEntryListItemArray`
  - `set_DeckEntryListItemArray`

### `Gallop.SingleModeScenarioTeamRaceDeckViewInfo` (0m)

### `Gallop.SingleModeScenarioTeamRaceDeckViewController` (31m)
  - `RegisterDownload`
  - `InitializeView`
  - `FinalizeView`
  - `PlayInView`
  - `GetDynamicBgmCueInfo`
  - `OnClickBackButton`
  - `OnClickOsBackKey`
  - `OnClickDecide`
  - `OnClickAuto`
  - `OnClickAutoBuildSwitchButton`
  - `UpdateAutoButtonState`
  - `OnEditConfirmed`
  - `SetupTeamInfoHeader`
  - `UpdateDeck`
  - `UpdateWarningText`
  - `BackConfirm`
  - `ShowChangeAbortConfirmDialog`
  - `Back`
  - `RegisterDownload3D`
  - `Initialize3D`

### `Gallop.SingleModeScenarioTeamRaceDefine` (0m)

### `Gallop.DialogSingleModeScenarioTeamRaceAllRaceList` (8m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `Open`
  - `OnOpen`
  - `Setup`
  - `OnRoundClicked`
  - `<Setup>b__9_0`

### `Gallop.DialogSingleModeScenarioTeamRaceAllRaceResult` (18m)
  - `GetFormType`
  - `GetParentType`
  - `CreateDialogData`
  - `PushDialog`
  - `PreLoad`
  - `RegisterDownload`
  - `ShowFadeImageFadeOut`
  - `Initialize`
  - `CreateListItem`
  - `SetUpRaceResultItem`
  - `LoadTitleFlashPlayer`
  - `CloseDialog`
  - `OnClickNextButton`
  - `FlashIn00`
  - `FlashIn01`
  - `ShowItems`
  - `PlayCloseAnimation`
  - `<Initialize>b__27_0`

### `Gallop.PartsSingleModeScenarioTeamRaceRankCharaIcon` (1m)
  - `SetUp`

### `Gallop.SingleModeScenarioTeamRaceGrandResultView` (20m)
  - `get_NextButton`
  - `set_NextButton`
  - `get_ContinueButton`
  - `set_ContinueButton`
  - `get_BottomButtonRoot`
  - `set_BottomButtonRoot`
  - `get_LiveButton`
  - `set_LiveButton`
  - `get_RaceResultFlashRoot`
  - `set_RaceResultFlashRoot`
  - `get_StretchContent`
  - `set_StretchContent`
  - `get_Area3D`
  - `set_Area3D`
  - `get_Image3D`
  - `set_Image3D`
  - `get_RaceResultItemPrefab`
  - `set_RaceResultItemPrefab`
  - `get_GridLayout`
  - `set_GridLayout`

### `Gallop.SingleModeScenarioTeamRaceGrandResultViewController` (33m)
  - `RegisterDownload`
  - `InitializeView`
  - `IgnoreBgm`
  - `PlayInView`
  - `FinalizeView`
  - `OnClickOsBackKey`
  - `OnClickShowLive`
  - `OnClickToRaceOutEvent`
  - `SendRaceOutApi`
  - `OnClickContinue`
  - `SetupRaceResultItems`
  - `FadeIn`
  - `FlashRaceResult`
  - `ShowGetLiveMusicDialog`
  - `ButtonsIn`
  - `OnNext`
  - `ChangeViewSingleCoroutine`
  - `RegisterDownload3D`
  - `Setup3D`
  - `Initialize3D`

### `Gallop.SingleModeScenarioTeamRaceLiveCoordinator` (4m)
  - `UpdateSingleModeLiveSetting`
  - `GetSingleModeTeamRaceLiveHorseData`
  - `SingleModeTeamRaceMemberToLiveMemberList`
  - `<UpdateSingleModeLiveSetting>g__SetupLoadSettingData|0_0`


## Master数据库表 (2个)

| 表名 | 方法数 |
|---|---|
| `MasterSingleModeTeamRaceSet` | 9 |
| `MasterTeamBuildingTeamRaceSet` | 6 |

## WorkSingleModeScenarioTeamRace

方法数: 77

  - `get_TeamName`
  - `set_TeamName`
  - `get_TeamNameId`
  - `set_TeamNameId`
  - `get_FinalWinType`
  - `set_FinalWinType`
  - `get_IsBossBattle`
  - `get_CanEnterRace`
  - `get_AddMusicId`
  - `ClearAddMusicId`
  - `get_TeamParameterRankSpeed`
  - `set_TeamParameterRankSpeed`
  - `get_TeamParameterRankStamina`
  - `set_TeamParameterRankStamina`
  - `get_TeamParameterRankPower`
  - `set_TeamParameterRankPower`
  - `get_TeamParameterRankGuts`
  - `set_TeamParameterRankGuts`
  - `get_TeamParameterRankWiz`
  - `set_TeamParameterRankWiz`
  - `get_GuidePartnerCount`
  - `set_GuidePartnerCount`
  - `get_IsScoutEnable`
  - `set_IsScoutEnable`
  - `get_TeamTotalPower`
  - `get_TeamRanking`
  - `set_TeamRanking`
  - `get_TeamHonorName`
  - `set_TeamHonorName`
  - `get_TeamHonorId`
  - `set_TeamHonorId`
  - `get_TeamMemberList`
  - `get_TeamRaceDeckInfo`
  - `get_SelectedTeamRaceSetId`
  - `set_SelectedTeamRaceSetId`
  - `get_TeamFrameOrderArray`
  - `set_TeamFrameOrderArray`
  - `get_OpponentListArray`
  - `set_OpponentListArray`
  - `get_SuperTeam`

## 剧本独立属性变化 (10m, 4 getters)

  - `get_RankingUp`
  - `get_RankingDown`
  - `get_TeamStatusUpDictionary`
  - `get_TotalPower`

## lib.rs相关引用

```
let mot_s = match mot { 5=>"Best", 4=>"Good", 3=>"Normal", 2=>"Bad", 1=>"Worst", _=>"?" };
1=>"URA", 2=>"TeamRace", 3=>"Live", 4=>"Free", 5=>"Venus",
11=>"Pioneer", 12=>"Onsen", 13=>"Breeders", 14=>"Ramen", _=>"Unknown"
101=>"Speed", 102=>"Stamina", 103=>"Guts",
1=>"Speed", 2=>"Stamina", 3=>"Guts",
1=>"WorkSingleModeScenarioURA", 2=>"WorkSingleModeScenarioTeamRace",
11=>"WorkSingleModeScenarioPioneer", 12=>"WorkSingleModeScenarioOnsen",
2 => "WorkSingleModeScenarioTeamRace",
```