# 剧本 8: Cook (クック)

**WorkScenario类**: `WorkSingleModeScenarioCook`
**ObscuredDataSet**: `ObscuredSingleModeCookDataSet`
---

## 相关类 (192个)

### `Gallop.CookingMiniCharaData` (13m)
  - `get_UniqueId`
  - `set_UniqueId`
  - `get_Name`
  - `set_Name`
  - `get_CharaId`
  - `set_CharaId`
  - `get_DressId`
  - `set_DressId`
  - `get_DressColorId`
  - `get_IsVisibleMessage`
  - `get_IsTrialCharacter`
  - `get_MemberIndex`
  - `set_MemberIndex`

### `Gallop.CookingResultMiniCharaData` (13m)
  - `get_UniqueId`
  - `set_UniqueId`
  - `get_Name`
  - `set_Name`
  - `get_CharaId`
  - `set_CharaId`
  - `get_DressId`
  - `set_DressId`
  - `get_DressColorId`
  - `get_IsVisibleMessage`
  - `get_IsTrialCharacter`
  - `get_Index`
  - `set_Index`

### `Gallop.MiniCharaStateAnimObjectCookGarden` (10m)
  - `Init`
  - `Update`
  - `UpdateClickCoolTime`
  - `Final`
  - `get_IsCurrentClickable`
  - `set_IsCurrentClickable`
  - `OnClick`
  - `OnStash`
  - `RestartAnimObject`
  - `OnPop`

### `Gallop.MiniCharaStateEmoteCookGarden` (10m)
  - `get_DefaultNextMotionName`
  - `Init`
  - `ApplyPositionMotionSpeed`
  - `InitAnimationNormalizedTime`
  - `Update`
  - `CheckPositionAnimationLoop`
  - `GetRandomEmote`
  - `InitRotate`
  - `OnMotionFinish`
  - `CreateOnTouchCommand`

### `Gallop.SingleModeCookMiniGardenBgParam` (2m)
  - `GetBgModelResorcePath`
  - `GetBgModelPartsResourcePath`

### `Gallop.SingleModeCookMiniCookingBgParam` (2m)
  - `GetBgModelPartsResourcePath`
  - `TryGetBgModelPartsNode`

### `Gallop.CookingMinidirectorCameraParam` (0m)

### `Gallop.MasterSingleModeCookCoinRate` (6m)
  - `Get`
  - `_SelectOne`
  - `GetWithMaterialCountAndPeriod`
  - `_SelectWithMaterialCountAndPeriod`
  - `_CreateOrmByQueryResultWithMaterialCountAndPeriod`
  - `Unload`

### `Gallop.MasterSingleModeCookCookingCutt` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithCookingNumOrderByIdAsc`
  - `_SelectWithCookingNumOrderByIdAsc`
  - `GetListWithCookingNumOrderByIdAsc`
  - `MaybeListWithCookingNumOrderByIdAsc`
  - `_ListSelectWithCookingNumOrderByIdAsc`
  - `_CreateOrmByQueryResultWithCookingNumOrderByIdAsc`
  - `Unload`

### `Gallop.MasterSingleModeCookCookingRate` (11m)
  - `Get`
  - `_SelectOne`
  - `GetWithDishTypeOrderByIdAsc`
  - `_SelectWithDishTypeOrderByIdAsc`
  - `GetListWithDishTypeOrderByIdAsc`
  - `MaybeListWithDishTypeOrderByIdAsc`
  - `_ListSelectWithDishTypeOrderByIdAsc`
  - `_CreateOrmByQueryResultWithDishTypeOrderByIdAsc`
  - `Unload`
  - `GetTotalRate`
  - `GetLotteryResultCuttCookingCuttId`

### `Gallop.MasterSingleModeCookCookingType` (11m)
  - `Get`
  - `_SelectOne`
  - `GetWithCookingMotionId`
  - `_SelectWithCookingMotionId`
  - `_CreateOrmByQueryResultWithCookingMotionId`
  - `GetWithCookingMotionIdAndSwapType`
  - `_SelectWithCookingMotionIdAndSwapType`
  - `_CreateOrmByQueryResultWithCookingMotionIdAndSwapType`
  - `Unload`
  - `GetTargetCookingTypeData`
  - `GetSwapMotionId`

### `Gallop.MasterSingleModeCookDish` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleModeCookDishCutt` (27m)
  - `Get`
  - `_SelectOne`
  - `GetWithCommandGroupId`
  - `_SelectWithCommandGroupId`
  - `_CreateOrmByQueryResultWithCommandGroupId`
  - `GetWithMaterialGroupIdOrderByIdAsc`
  - `_SelectWithMaterialGroupIdOrderByIdAsc`
  - `GetListWithMaterialGroupIdOrderByIdAsc`
  - `MaybeListWithMaterialGroupIdOrderByIdAsc`
  - `_ListSelectWithMaterialGroupIdOrderByIdAsc`
  - `_CreateOrmByQueryResultWithMaterialGroupIdOrderByIdAsc`
  - `GetWithCuttIdOrderByIdAsc`
  - `_SelectWithCuttIdOrderByIdAsc`
  - `GetListWithCuttIdOrderByIdAsc`
  - `MaybeListWithCuttIdOrderByIdAsc`
  - `_ListSelectWithCuttIdOrderByIdAsc`
  - `_CreateOrmByQueryResultWithCuttIdOrderByIdAsc`
  - `GetWithCommandGroupIdAndMaterialGroupIdOrderByIdAsc`
  - `_SelectWithCommandGroupIdAndMaterialGroupIdOrderByIdAsc`
  - `GetListWithCommandGroupIdAndMaterialGroupIdOrderByIdAsc`

### `Gallop.MasterSingleModeCookDishCuttSe` (12m)
  - `Get`
  - `_SelectOne`
  - `GetWithCommandGroupId`
  - `_SelectWithCommandGroupId`
  - `_CreateOrmByQueryResultWithCommandGroupId`
  - `GetWithCharaType`
  - `_SelectWithCharaType`
  - `_CreateOrmByQueryResultWithCharaType`
  - `GetWithCharaTypeAndCommandGroupId`
  - `_SelectWithCharaTypeAndCommandGroupId`
  - `_CreateOrmByQueryResultWithCharaTypeAndCommandGroupId`
  - `Unload`

### `Gallop.MasterSingleModeCookDishEffect` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithEffectGroupIdOrderByIdAsc`
  - `_SelectWithEffectGroupIdOrderByIdAsc`
  - `GetListWithEffectGroupIdOrderByIdAsc`
  - `MaybeListWithEffectGroupIdOrderByIdAsc`
  - `_ListSelectWithEffectGroupIdOrderByIdAsc`
  - `_CreateOrmByQueryResultWithEffectGroupIdOrderByIdAsc`
  - `Unload`

### `Gallop.MasterSingleModeCookDishIcon` (3m)
  - `Get`
  - `_SelectOne`
  - `Unload`

### `Gallop.MasterSingleModeCookDishMaterial` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithMaterialGroupIdOrderByIdAsc`
  - `_SelectWithMaterialGroupIdOrderByIdAsc`
  - `GetListWithMaterialGroupIdOrderByIdAsc`
  - `MaybeListWithMaterialGroupIdOrderByIdAsc`
  - `_ListSelectWithMaterialGroupIdOrderByIdAsc`
  - `_CreateOrmByQueryResultWithMaterialGroupIdOrderByIdAsc`
  - `Unload`

### `Gallop.MasterSingleModeCookDishName` (11m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithDishGroupIdOrderByIdAsc`
  - `_SelectWithDishGroupIdOrderByIdAsc`
  - `GetListWithDishGroupIdOrderByIdAsc`
  - `MaybeListWithDishGroupIdOrderByIdAsc`
  - `_ListSelectWithDishGroupIdOrderByIdAsc`
  - `_CreateOrmByQueryResultWithDishGroupIdOrderByIdAsc`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleModeCookGarden` (3m)
  - `Get`
  - `_SelectOne`
  - `Unload`

### `Gallop.MasterSingleModeCookGardenBg` (15m)
  - `Get`
  - `_SelectOne`
  - `GetWithMonthAndHalfTypeOrderByIdAsc`
  - `_SelectWithMonthAndHalfTypeOrderByIdAsc`
  - `GetListWithMonthAndHalfTypeOrderByIdAsc`
  - `MaybeListWithMonthAndHalfTypeOrderByIdAsc`
  - `_ListSelectWithMonthAndHalfTypeOrderByIdAsc`
  - `_CreateOrmByQueryResultWithMonthAndHalfTypeOrderByIdAsc`
  - `GetWithBgIdOrderByIdAsc`
  - `_SelectWithBgIdOrderByIdAsc`
  - `GetListWithBgIdOrderByIdAsc`
  - `MaybeListWithBgIdOrderByIdAsc`
  - `_ListSelectWithBgIdOrderByIdAsc`
  - `_CreateOrmByQueryResultWithBgIdOrderByIdAsc`
  - `Unload`

### `Gallop.MasterSingleModeCookGardenEffect` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithEffectGroupIdOrderByIdAsc`
  - `_SelectWithEffectGroupIdOrderByIdAsc`
  - `GetListWithEffectGroupIdOrderByIdAsc`
  - `MaybeListWithEffectGroupIdOrderByIdAsc`
  - `_ListSelectWithEffectGroupIdOrderByIdAsc`
  - `_CreateOrmByQueryResultWithEffectGroupIdOrderByIdAsc`
  - `Unload`

### `Gallop.MasterSingleModeCookGardenLevel` (20m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `GetWithFacilityIdOrderByFacilityLvAsc`
  - `_SelectWithFacilityIdOrderByFacilityLvAsc`
  - `GetListWithFacilityIdOrderByFacilityLvAsc`
  - `MaybeListWithFacilityIdOrderByFacilityLvAsc`
  - `_ListSelectWithFacilityIdOrderByFacilityLvAsc`
  - `_CreateOrmByQueryResultWithFacilityIdOrderByFacilityLvAsc`
  - `GetWithGardenLv`
  - `_SelectWithGardenLv`
  - `_CreateOrmByQueryResultWithGardenLv`
  - `GetWithFacilityIdAndGardenLv`
  - `_SelectWithFacilityIdAndGardenLv`
  - `GetListWithFacilityIdAndGardenLv`
  - `MaybeListWithFacilityIdAndGardenLv`
  - `_ListSelectWithFacilityIdAndGardenLv`
  - `_CreateOrmByQueryResultWithFacilityIdAndGardenLv`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleModeCookListener` (6m)
  - `Get`
  - `_SelectOne`
  - `GetWithCamera`
  - `_SelectWithCamera`
  - `_CreateOrmByQueryResultWithCamera`
  - `Unload`

### `Gallop.MasterSingleModeCookMaterialRate` (6m)
  - `Get`
  - `_SelectOne`
  - `GetWithMaterialCountAndBoostType`
  - `_SelectWithMaterialCountAndBoostType`
  - `_CreateOrmByQueryResultWithMaterialCountAndBoostType`
  - `Unload`

### `Gallop.MasterSingleModeCookMessage` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithViewTypeOrderByIdAsc`
  - `_SelectWithViewTypeOrderByIdAsc`
  - `GetListWithViewTypeOrderByIdAsc`
  - `MaybeListWithViewTypeOrderByIdAsc`
  - `_ListSelectWithViewTypeOrderByIdAsc`
  - `_CreateOrmByQueryResultWithViewTypeOrderByIdAsc`
  - `Unload`

### `Gallop.MasterSingleModeCookMotionGroup` (3m)
  - `Get`
  - `_SelectOne`
  - `Unload`

### `Gallop.MasterSingleModeCookMotionStatus` (9m)
  - `Get`
  - `_SelectOne`
  - `GetWithTurnProgressAndGardenProgressOrderByIdAsc`
  - `_SelectWithTurnProgressAndGardenProgressOrderByIdAsc`
  - `GetListWithTurnProgressAndGardenProgressOrderByIdAsc`
  - `MaybeListWithTurnProgressAndGardenProgressOrderByIdAsc`
  - `_ListSelectWithTurnProgressAndGardenProgressOrderByIdAsc`
  - `_CreateOrmByQueryResultWithTurnProgressAndGardenProgressOrderByIdAsc`
  - `Unload`

### `Gallop.MasterSingleModeCookPowerData` (6m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`
  - `get_OrderedList`

### `Gallop.MasterSingleModeCookStandMotion` (18m)
  - `Get`
  - `_SelectOne`
  - `GetWithTypeOrderByIdAsc`
  - `_SelectWithTypeOrderByIdAsc`
  - `GetListWithTypeOrderByIdAsc`
  - `MaybeListWithTypeOrderByIdAsc`
  - `_ListSelectWithTypeOrderByIdAsc`
  - `_CreateOrmByQueryResultWithTypeOrderByIdAsc`
  - `GetWithTypeAndCharaId`
  - `_SelectWithTypeAndCharaId`
  - `_CreateOrmByQueryResultWithTypeAndCharaId`
  - `GetWithTypeAndCharaIdAndConditionType`
  - `_SelectWithTypeAndCharaIdAndConditionType`
  - `_CreateOrmByQueryResultWithTypeAndCharaIdAndConditionType`
  - `GetWithTypeAndConditionType`
  - `_SelectWithTypeAndConditionType`
  - `_CreateOrmByQueryResultWithTypeAndConditionType`
  - `Unload`

### `Gallop.MasterSingleModeCookSuccessEffect` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleModeCookSuccessOdds` (5m)
  - `get_dictionary`
  - `Get`
  - `_SelectOne`
  - `Unload`
  - `_ForcePreloadAllEntries`

### `Gallop.MasterSingleModeCookSuccessType` (6m)
  - `Get`
  - `_SelectOne`
  - `GetWithSuccessMotionId`
  - `_SelectWithSuccessMotionId`
  - `_CreateOrmByQueryResultWithSuccessMotionId`
  - `Unload`

### `Gallop.MasterSingleModeCookCookingCuttUtil` (9m)
  - `GetSwapMotionId`
  - `GetMasterDataMotionId`
  - `GetMotionId`
  - `GetCameraMotionPathArray`
  - `GetCookingPartCameraPosition`
  - `GetMasterDataSuccessMotionId`
  - `GetAllSuccessEffectPath`
  - `GetTargetSuccessEffectName`
  - `GetTargetSuccessEffectPosition`

### `Gallop.MasterSingleModeCookCookingRateUtil` (1m)
  - `GetRate`

### `Gallop.MasterSingleModeCookCookingTypeUtil` (5m)
  - `GetCookMotionEffectPathArray`
  - `GetCookMotionEffect1`
  - `GetCookMotionEffect2`
  - `GetCookMotionEffectPosition1`
  - `GetCookMotionEffectPosition2`

### `Gallop.MasterSingleModeCookDishCuttUtil` (3m)
  - `GetDishCutAssetPath`
  - `IsCommonCutt`
  - `GetPropPrefab`

### `Gallop.MasterSingleModeCookDishCuttSeUtils` (1m)
  - `RegisterDownloadAllCharaTypeSe`

### `Gallop.MasterSingleModeCookGardenBgUtil` (2m)
  - `GetWithMonthAndHalfTypeAndProgress`
  - `GetGardenBgSubIdArray`

### `Gallop.MatserSingleModeCookStandMotionUtils` (1m)
  - `GetCookSpecialEventCookCuttResultData`

### `Gallop.WorkSingleModeChangeParameterInfoScenarioCook` (32m)
  - `get_CookedDishModel`
  - `get_GainMaterialIconInfo`
  - `set_GainMaterialIconInfo`
  - `get_GainMaterialDictionary`
  - `get_GainCarePoint`
  - `set_GainCarePoint`
  - `get_GainFriendsPowerPoint`
  - `set_GainFriendsPowerPoint`
  - `get_EventGainMaterialNumDictionary`
  - `set_EventGainMaterialNumDictionary`
  - `get_SubCommandCharaInfo`
  - `set_SubCommandCharaInfo`
  - `Clear`
  - `Set`
  - `SetCooked`
  - `SetGainMaterialId`
  - `GetAddDishInfoList`
  - `SetUpdateDishInfoList`
  - `GetUpdateDishInfoList`
  - `SetUpdateDishEffectInfoList`

### `Gallop.WorkSingleModeScenarioCook` (37m)
  - `get_MaterialInfoDataArray`
  - `get_CommandMaterialCareInfoArray`
  - `set_CommandMaterialCareInfoArray`
  - `get_CookInfo`
  - `get_ResultInfoArray`
  - `get_PowerEffectInfoArray`
  - `get_DishInfo`
  - `get_MaterialHarvestInfoArray`
  - `get_MaterialCareHistoryInfoArray`
  - `get_CookingSuccessRate`
  - `get_GainMaterialInfo`
  - `set_GainMaterialInfo`
  - `get_CookEvaluationInfoList`
  - `set_CookEvaluationInfoList`
  - `get_DishSkillInfo`
  - `set_DishSkillInfo`
  - `get_DishSuccessEffectIdArray`
  - `set_DishSuccessEffectIdArray`
  - `get_CarePointGainNum`
  - `get_CareSpecialHomeId`

### `Gallop.PhotoStudioPlayCutSettingsScenarioCutCookService` (12m)
  - `get_ScenarioCutCategory`
  - `Reset`
  - `CreateCuttPlayInfo`
  - `Gallop.IPhotoStudioPlayCutSettingsScenarioCutCookService.CreateVM`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCategoryEventDispatch.ChangeCategory`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutDishEventDispatch.ChangeDish`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCharaDressSetEventDispatch.ChangeCharacter`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCharaDressSetEventDispatch.ChangeDress`
  - `DecideCharacterSelect`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutOptionSeasonEventDispatch.ChangeSeason`
  - `Gallop.IPartsPhotoStudioToggleOptionPlaceEventDispatch.SelectPlace`
  - `<Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutDishEventDispatch.ChangeDish>b__8_0`

### `Gallop.PhotoStudioScenarioCutCookController` (13m)
  - `StartCutt`
  - `OnInitFinish`
  - `OnStartCutt`
  - `Gallop.IPhotoStudioPlayer.IsPause`
  - `Gallop.IPhotoStudioPlayer.SkipPause`
  - `Gallop.IPhotoStudioPlayer.Pause`
  - `Gallop.IPhotoStudioPlayer.Resume`
  - `Gallop.IPhotoStudioPlayer.IsStatusPlaying`
  - `Gallop.IPhotoStudioPlayer.IsStatusEnd`
  - `Gallop.IPhotoStudioPlayer.get_HasAnyHelper`
  - `Gallop.IPhotoStudioPlayer.AlterUpdate`
  - `Gallop.IPhotoStudioPlayer.AlterLateUpdate`
  - `CleanUpAll`

### `Gallop.PhotoStudioPlayCutSettingsScenarioCutCookProvider` (11m)
  - `get_DishDataModel`
  - `get_CharaDressSetProvider`
  - `get_OptionSeasonDataModel`
  - `get_CharaDressSetDataModelAccessor`
  - `get_OptionPlaceDataModel`
  - `SetDish`
  - `SetCharaId`
  - `SetBgSeason`
  - `SetPlace`
  - `Reset`
  - `CreateCharaDressSetArray`

### `Gallop.IPhotoStudioPlayCutSettingsScenarioCutCookService` (1m)
  - `CreateVM`

### `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCookEventDispatch` (0m)

### `Gallop.PartsPhotoStudioPlayCutSettingsScenarioCutCook` (3m)
  - `InitializeView`
  - `UpdateContents`
  - `CreateAndSetupCharaDressSetArray`

### `Gallop.PartsPhotoStudioPlayCutSettingsScenarioCutCookCharaDressSetVM` (8m)
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCharaDressSetVM.get_Id`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCharaDressSetVM.get_CharaId`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCharaDressSetVM.get_DressTexturePath`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCharaDressSetVM.get_IsCharacterChangeable`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCharaDressSetVM.get_IsDressChangeable`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCharaDressSetVM.get_DressUnChangeableNotificationMessage`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCharaDressSetVM.get_LabelText`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCharaDressSetVM.get_ShouldShowRandomCharaIcon`

### `Gallop.PartsPhotoStudioPlayCutSettingsScenarioCutCookVM` (5m)
  - `get_CategoryName`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCookVM.get_ScenarioCutDishVM`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCookVM.get_ScenarioCutCharaDressSetVMArray`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCookVM.get_ScenarioCutOptionSeasonVM`
  - `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCookVM.get_ScenarioCutOptionPlaceVM`

### `Gallop.IPartsPhotoStudioPlayCutSettingsScenarioCutCookVM` (4m)
  - `get_ScenarioCutDishVM`
  - `get_ScenarioCutCharaDressSetVMArray`
  - `get_ScenarioCutOptionSeasonVM`
  - `get_ScenarioCutOptionPlaceVM`

### `Gallop.DialogSingleModeScenarioCookNextScheduleListModel` (5m)
  - `RegisterDownload`
  - `get_FrameType`
  - `get_DialogPrefabPath`
  - `get_LogoImagePath`
  - `get_ScheduleListItemModels`

### `Gallop.PartsSingleModeScenarioCookScheduleListItemModel` (12m)
  - `get_PrefabPath`
  - `get_ScheduleTurn`
  - `get_IsNextSchedule`
  - `get_HasResultData`
  - `get_HeaderImageSprite`
  - `get_TitleText`
  - `get_ScheduleOpenText`
  - `get_ResultImageSprite`
  - `GetRemainTurnText`
  - `get_IsSuccess`
  - `get_ResultAudioId`
  - `get_IsNeedPlayWinEffectOnResultAnimation`

### `Gallop.SingleModeScenarioCookCookedDishModel` (12m)
  - `get_DishModel`
  - `set_DishModel`
  - `get_DishName`
  - `get_FlavourText`
  - `get_ResultState`
  - `get_IsGreatSuccess`
  - `get_AlreadyCooked`
  - `get_IsSeniorGreatSuccessDish`
  - `CreateChangeParameterInfoAtCooked`
  - `CreateChangeParameterInfoAtEating`
  - `CreateChangeParameterInfoList`
  - `<.ctor>b__23_0`

### `Gallop.SingleModeScenarioCookCookingSuccessPointModel` (7m)
  - `get_WorkScenarioCook`
  - `get_CookingSuccessRate`
  - `get_IsMaxCookingGreatSuccessRate`
  - `get_CookingSuccessPoint`
  - `get_CookingSuccessBasePoint`
  - `get_NextGreatSuccessPoint`
  - `get_CookingCuttCharaIdArray`

### `Gallop.SingleModeScenarioCookDishDirectoryItemModel` (17m)
  - `get_NameMaster`
  - `get_SpecialNameMaster`
  - `get_DefaultFlavourTextModel`
  - `set_DefaultFlavourTextModel`
  - `get_SpecialFlavourTextModel`
  - `set_SpecialFlavourTextModel`
  - `get_IsActive`
  - `get_IsNew`
  - `get_IsLocked`
  - `get_IsNotCooked`
  - `get_IconSprite`
  - `get_IconColor`
  - `get_Name`
  - `get_IsSeniorDegreeItem`
  - `GetModelsByDegreeIndex`
  - `SetupFlavourTextMode`
  - `GetUnreadFlavourTextIdArray`

### `Gallop.SingleModeScenarioCookDishModel` (26m)
  - `get_WorkScenarioCook`
  - `get_Master`
  - `get_Id`
  - `get_Name`
  - `get_GainFriendsPower`
  - `get_DishGroupId`
  - `get_EffectList`
  - `get_ActiveEffectArray`
  - `get_MaterialList`
  - `get_TargetTrainingTypeArray`
  - `get_Unlocked`
  - `get_Cookable`
  - `get_GetDegreeIndex`
  - `TargetTrainingEffectValue`
  - `get_ExistCookableDish`
  - `ExistCookableRecommendDish`
  - `GetDishModelList`
  - `GetUnlockedDishModelListOrderByCookable`
  - `GetLowCostDishModelListOrderByCookable`
  - `GetRecommendedDishModelList`

### `Gallop.SingleModeScenarioCookFlavourTextModel` (13m)
  - `get_NameMaster`
  - `get_BaseNameMaster`
  - `get_Status`
  - `get_DishName`
  - `get_BaseDishName`
  - `get_CharaName`
  - `get_IsActive`
  - `get_IsNew`
  - `get_IsShowUnlockCondition`
  - `get_IsNotCooked`
  - `get_IsLocked`
  - `get_FlavourText`
  - `get_UnlockConditionText`

### `Gallop.SingleModeScenarioCookFriendsPowerModel` (26m)
  - `get_OrderedFriendsPowerMissionMasterList`
  - `get_NextFriendsPowerMissionMaster`
  - `GetNextFriendsPowerMissionMasterByTurn`
  - `get_IsLastFriendsPowerMission`
  - `get_IsGreatSuccess`
  - `get_CookSpecialEventId`
  - `get_CurrentGreatSuccessRate`
  - `get_TrainingEffectUpRate`
  - `get_BestTrainingEffectUpRate`
  - `get_SkillPointUpRate`
  - `get_GetFanUpRate`
  - `get_CookingFriendsPower`
  - `get_ExistNextFriendsPowerMissionGoal`
  - `get_NextFriendsPowerMissionPoint`
  - `get_IsCompleteNextFriendsPowerMission`
  - `get_OpenGardenLevel`
  - `get_PowerDataMaster`
  - `get_IsComplete`
  - `get_ResultInfoArray`
  - `get_HasResultInfo`

### `Gallop.SingleModeScenarioCookGardenUpgradeModel` (15m)
  - `get_WorkScenarioCook`
  - `get_SelectedFacilityData`
  - `set_SelectedFacilityData`
  - `get_ExistLevelUp`
  - `get_ExistOverLimitLevel`
  - `get_CurrentCarePointNum`
  - `get_RemainCarePointNum`
  - `get_EnoughTotalCarePointToLvUp`
  - `get_LevelUpInfoArray`
  - `InitializeFacilityDataList`
  - `GetFacilityDataByFacilityType`
  - `SelectFacilityDataByFacilityType`
  - `ResetAllFacilityData`
  - `GetCarePointConsume`
  - `IsShowGardenLevelUpDialog`

### `Gallop.SingleModeScenarioCookMaterialCareInfoModel` (14m)
  - `get_CareInfoNum`
  - `get_RemainNextHarvestTurnNum`
  - `get_GainCarePoint`
  - `get_IsShowSingleModeChara`
  - `get_DefaultBoostNum`
  - `get_BoostMaterialQuantityRate`
  - `get_CareInfo`
  - `get_Exist`
  - `get_MaterialId`
  - `get_BoostType`
  - `get_IsInstantHarvestEnabled`
  - `get_InstantHarvestInfoText`
  - `GetBoostValueByCommandId`
  - `<get_CareInfo>b__18_0`


## Master数据库表 (31个)

| 表名 | 方法数 |
|---|---|
| `MasterSingleModeCookCoinRate` | 6 |
| `MasterSingleModeCookCookingCutt` | 9 |
| `MasterSingleModeCookCookingRate` | 11 |
| `MasterSingleModeCookCookingType` | 11 |
| `MasterSingleModeCookDish` | 5 |
| `MasterSingleModeCookDishCutt` | 27 |
| `MasterSingleModeCookDishCuttSe` | 12 |
| `MasterSingleModeCookDishEffect` | 9 |
| `MasterSingleModeCookDishIcon` | 3 |
| `MasterSingleModeCookDishMaterial` | 9 |
| `MasterSingleModeCookDishName` | 11 |
| `MasterSingleModeCookGarden` | 3 |
| `MasterSingleModeCookGardenBg` | 15 |
| `MasterSingleModeCookGardenEffect` | 9 |
| `MasterSingleModeCookGardenLevel` | 20 |
| `MasterSingleModeCookListener` | 6 |
| `MasterSingleModeCookMaterialRate` | 6 |
| `MasterSingleModeCookMessage` | 9 |
| `MasterSingleModeCookMotionGroup` | 3 |
| `MasterSingleModeCookMotionStatus` | 9 |
| `MasterSingleModeCookPowerData` | 6 |
| `MasterSingleModeCookStandMotion` | 18 |
| `MasterSingleModeCookSuccessEffect` | 5 |
| `MasterSingleModeCookSuccessOdds` | 5 |
| `MasterSingleModeCookSuccessType` | 6 |
| `MasterSingleModeCookCookingCuttUtil` | 9 |
| `MasterSingleModeCookCookingRateUtil` | 1 |
| `MasterSingleModeCookCookingTypeUtil` | 5 |
| `MasterSingleModeCookDishCuttUtil` | 3 |
| `MasterSingleModeCookDishCuttSeUtils` | 1 |
| `MasterSingleModeCookGardenBgUtil` | 2 |

## WorkSingleModeScenarioCook

方法数: 37

  - `get_MaterialInfoDataArray`
  - `get_CommandMaterialCareInfoArray`
  - `set_CommandMaterialCareInfoArray`
  - `get_CookInfo`
  - `get_ResultInfoArray`
  - `get_PowerEffectInfoArray`
  - `get_DishInfo`
  - `get_MaterialHarvestInfoArray`
  - `get_MaterialCareHistoryInfoArray`
  - `get_CookingSuccessRate`
  - `get_GainMaterialInfo`
  - `set_GainMaterialInfo`
  - `get_CookEvaluationInfoList`
  - `set_CookEvaluationInfoList`
  - `get_DishSkillInfo`
  - `set_DishSkillInfo`
  - `get_DishSuccessEffectIdArray`
  - `set_DishSuccessEffectIdArray`
  - `get_CarePointGainNum`
  - `get_CareSpecialHomeId`
  - `get_TastingResultState`
  - `get_AvailableDishInfoArray`
  - `set_AvailableDishInfoArray`
  - `get_LastCommandInfo`
  - `set_LastCommandInfo`
  - `get_EventGainMaterialInfoList`
  - `set_EventGainMaterialInfoList`
  - `get_UnlockDegreeType`
  - `set_UnlockDegreeType`
  - `get_UnlockFlavourTextArray`
  - `set_UnlockFlavourTextArray`
  - `get_UnReadFlavourTextArray`
  - `set_UnReadFlavourTextArray`
  - `get_DisplayFlavourTextId`
  - `set_DisplayFlavourTextId`
  - `GetFacilityInfoByType`
  - `Apply`

## 剧本独立属性变化 (32m, 7 getters)

  - `get_CookedDishModel`
  - `get_GainMaterialIconInfo`
  - `get_GainMaterialDictionary`
  - `get_GainCarePoint`
  - `get_GainFriendsPowerPoint`
  - `get_EventGainMaterialNumDictionary`
  - `get_SubCommandCharaInfo`

## lib.rs相关引用

```
6=>"Arc", 7=>"Sport", 8=>"Cook", 9=>"Mecha", 10=>"Legend",
7=>"WorkSingleModeScenarioSport", 8=>"WorkSingleModeScenarioCook",
8 => "WorkSingleModeScenarioCook",
```