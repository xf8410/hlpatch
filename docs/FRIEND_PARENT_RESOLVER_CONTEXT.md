# Complete friend parent resolver context

record_count=32

## Gallop.ApplicationSettingSaveLoader

```json
{
  "class": "ApplicationSettingSaveLoader",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 1432,
  "methods": [
    {
      "name": "IsOpen",
      "addr": "0x7339d56724",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "Dispose",
      "addr": "0x7339d56778",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Save",
      "addr": "0x7339d567f8",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "ForceSave",
      "addr": "0x7339d569b4",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "Load",
      "addr": "0x7339d56b70",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_ResourceVersion",
      "addr": "0x7339d56bf4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ResourceVersion",
      "addr": "0x7339d56ca0",
      "params": 0,
      "return_type": "string",
      "static": false
    },
    {
      "name": "set_LastUpdateVersionManifestGroup",
      "addr": "0x7339d56d08",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastUpdateVersionManifestGroup",
      "addr": "0x7339d56db4",
      "params": 0,
      "return_type": "string",
      "static": false
    },
    {
      "name": "set_IsTutorialFinished",
      "addr": "0x7339d56e1c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsTutorialFinished",
      "addr": "0x7339d56ed0",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_MasterHash",
      "addr": "0x7339d56f40",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_MasterHash",
      "addr": "0x7339d56fec",
      "params": 0,
      "return_type": "string",
      "static": false
    },
    {
      "name": "set_ViewerID",
      "addr": "0x7339d57054",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ViewerID",
      "addr": "0x7339d57130",
      "params": 0,
      "return_type": "i8",
      "static": false
    },
    {
      "name": "set_DmmViewerID",
      "addr": "0x7339d571c4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DmmViewerID",
      "addr": "0x7339d57270",
      "params": 0,
      "return_type": "string",
      "static": false
    },
    {
      "name": "set_SteamID",
      "addr": "0x7339d572d8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SteamID",
      "addr": "0x7339d57384",
      "params": 0,
      "return_type": "string",
      "static": false
    },
    {
      "name": "set_Udid",
      "addr": "0x7339d573ec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_Udid",
      "addr": "0x7339d57498",
      "params": 0,
      "return_type": "string",
      "static": false
    },
    {
      "name": "set_AuthKey",
      "addr": "0x7339d57500",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_AuthKey",
      "addr": "0x7339d575ac",
      "params": 0,
      "return_type": "string",
      "static": false
    },
    {
      "name": "set_RecheckDmmJewel",
      "addr": "0x7339d57614",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RecheckDmmJewel",
      "addr": "0x7339d576c0",
      "params": 0,
      "return_type": "string",
      "static": false
    },
    {
      "name": "set_NewOpProgress",
      "addr": "0x7339d57728",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_NewOpProgress",
      "addr": "0x7339d57804",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_IsConfirmDeleteUser",
      "addr": "0x7339d57894",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsConfirmDeleteUser",
      "addr": "0x7339d57948",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsConfirmTutorialSkip",
      "addr": "0x7339d579b8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsConfirmTutorialSkip",
      "addr": "0x7339d57a6c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsSkippableTutorial",
      "addr": "0x7339d57adc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsSkippableTutorial",
      "addr": "0x7339d57b90",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_DataLinkState",
      "addr": "0x7339d57c00",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DataLinkState",
      "addr": "0x7339d57cdc",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_IsNotifiediCloudDisable",
      "addr": "0x7339d57d6c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsNotifiediCloudDisable",
      "addr": "0x7339d57e20",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsConfirmiCloudBackupUserOverwrite",
      "addr": "0x7339d57e90",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsConfirmiCloudBackupUserOverwrite",
      "addr": "0x7339d57f44",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsNotifiedAlarmAndReminderSettings",
      "addr": "0x7339d57fb4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsNotifiedAlarmAndReminderSettings",
      "addr": "0x7339d58068",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_GameQuality",
      "addr": "0x7339d580dc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GameQuality",
      "addr": "0x7339d581b8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsLastSelectRaceRichQuality",
      "addr": "0x7339d5821c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsLastSelectRaceRichQuality",
      "addr": "0x7339d582d0",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_GameFrameRate",
      "addr": "0x7339d58344",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GameFrameRate",
      "addr": "0x7339d58420",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_InformationLastOpenTime",
      "addr": "0x7339d58484",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_InformationLastOpenTime",
      "addr": "0x7339d58560",
      "params": 0,
      "return_type": "i8",
      "static": false
    },
    {
      "name": "set_MasterVolume",
      "addr": "0x7339d585f0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_MasterVolume",
      "addr": "0x7339d586cc",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_IsUseMaster",
      "addr": "0x7339d58760",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsUseMaster",
      "addr": "0x7339d58814",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_BgmVolume",
      "addr": "0x7339d58888",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_BgmVolume",
      "addr": "0x7339d58964",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_IsUseBgm",
      "addr": "0x7339d589f8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsUseBgm",
      "addr": "0x7339d58aac",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_SeVolume",
      "addr": "0x7339d58b20",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SeVolume",
      "addr": "0x7339d58bfc",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_IsUseSe",
      "addr": "0x7339d58c90",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsUseSe",
      "addr": "0x7339d58d44",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_VoiceVolume",
      "addr": "0x7339d58db8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_VoiceVolume",
      "addr": "0x7339d58e94",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_IsUseVoice",
      "addr": "0x7339d58f28",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsUseVoice",
      "addr": "0x7339d58fdc",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_JikkyoVolume",
      "addr": "0x7339d59050",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_JikkyoVolume",
      "addr": "0x7339d5912c",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_IsUseJikkyo",
      "addr": "0x7339d591c0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsUseJikkyo",
      "addr": "0x7339d59274",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_LiveVolume",
      "addr": "0x7339d592e8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LiveVolume",
      "addr": "0x7339d593c4",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_IsUseLive",
      "addr": "0x7339d59458",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsUseLive",
      "addr": "0x7339d5950c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnablePhotoButton",
      "addr": "0x7339d59580",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnablePhotoButton",
      "addr": "0x7339d59634",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnablePhotoPopUpButton",
      "addr": "0x7339d596a8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnablePhotoPopUpButton",
      "addr": "0x7339d5975c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsPhotoHashRefreshed",
      "addr": "0x7339d597cc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsPhotoHashRefreshed",
      "addr": "0x7339d59880",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsPhotoHashRefreshedAdd",
      "addr": "0x7339d598f4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsPhotoHashRefreshedAdd",
      "addr": "0x7339d599a8",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsPaymentAlert",
      "addr": "0x7339d59a18",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsPaymentAlert",
      "addr": "0x7339d59acc",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsAccountHoldNoticeDialogChecked",
      "addr": "0x7339d59b3c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsAccountHoldNoticeDialogChecked",
      "addr": "0x7339d59bf0",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_CanBuySubscriptionNoticeCheckTimeArray",
      "addr": "0x7339d59c64",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CanBuySubscriptionNoticeCheckTimeArray",
      "addr": "0x7339d59cd4",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_IsEnableNotificationTP",
      "addr": "0x7339d59cf0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableNotificationTP",
      "addr": "0x7339d59da4",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableNotificationRP",
      "addr": "0x7339d59e14",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableNotificationRP",
      "addr": "0x7339d59ec8",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableNotificationJOB",
      "addr": "0x7339d59f3c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableNotificationJOB",
      "addr": "0x7339d59ff0",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableNotificationIdleSingleMode",
      "addr": "0x7339d5a060",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableNotificationIdleSingleMode",
      "addr": "0x7339d5a114",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsAvoidNotifigationMidnight",
      "addr": "0x7339d5a188",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsAvoidNotifigationMidnight",
      "addr": "0x7339d5a23c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_RaceQuality",
      "addr": "0x7339d5a2ac",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RaceQuality",
      "addr": "0x7339d5a388",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_JikkyoVoice",
      "addr": "0x7339d5a41c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_JikkyoVoice",
      "addr": "0x7339d5a4f8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsFanfareGameOriginalSound",
      "addr": "0x7339d5a58c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsFanfareGameOriginalSound",
      "addr": "0x7339d5a60c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsDownloadingRealFanfare",
      "addr": "0x7339d5a630",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsDownloadingRealFanfare",
      "addr": "0x7339d5a6b0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RaceSkillCutInPlayMode",
      "addr": "0x7339d5a6d0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RaceSkillCutInPlayMode",
      "addr": "0x7339d5a7ac",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RaceSkillCutInLastCheckUnixTime",
      "addr": "0x7339d5a840",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RaceSkillCutInLastCheckUnixTime",
      "addr": "0x7339d5a944",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RaceSkillCutInTodayPlayedSkillIdArray",
      "addr": "0x7339d5a96c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RaceSkillCutInTodayPlayedSkillIdArray",
      "addr": "0x7339d5a9dc",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_IsRaceLandscape",
      "addr": "0x7339d5a9f8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsRaceLandscape",
      "addr": "0x7339d5aaac",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsTryRaceDynamicCamera",
      "addr": "0x7339d5ab1c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsTryRaceDynamicCamera",
      "addr": "0x7339d5abd0",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsStoryRaceNormal",
      "addr": "0x7339d5ac44",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsStoryRaceNormal",
      "addr": "0x7339d5acf8",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsSpecialUnlockRaceLandscape",
      "addr": "0x7339d5ad68",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsSpecialUnlockRaceLandscape",
      "addr": "0x7339d5ae1c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableRaceOrientationPopup",
      "addr": "0x7339d5ae90",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableRaceOrientationPopup",
      "addr": "0x7339d5af44",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableRaceOrientationPopupVer2",
      "addr": "0x7339d5afb4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableRaceOrientationPopupVer2",
      "addr": "0x7339d5b068",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableAutoOpenDialogForRaceEntry",
      "addr": "0x7339d5b0dc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableAutoOpenDialogForRaceEntry",
      "addr": "0x7339d5b190",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_UseLiveLyric",
      "addr": "0x7339d5b204",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_UseLiveLyric",
      "addr": "0x7339d5b2b8",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsLiveCall",
      "addr": "0x7339d5b32c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsLiveCall",
      "addr": "0x7339d5b3e0",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsLivePortrait",
      "addr": "0x7339d5b450",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsLivePortrait",
      "addr": "0x7339d5b504",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableLiveSkipPopup",
      "addr": "0x7339d5b578",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableLiveSkipPopup",
      "addr": "0x7339d5b62c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_LiveStartSettingIsOrientationLandscape",
      "addr": "0x7339d5b69c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LiveStartSettingIsOrientationLandscape",
      "addr": "0x7339d5b750",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_LiveTrainerCameraOperation",
      "addr": "0x7339d5b7c4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LiveTrainerCameraOperation",
      "addr": "0x7339d5b860",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LiveTrainerCameraYawDirection",
      "addr": "0x7339d5b888",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LiveTrainerCameraYawDirection",
      "addr": "0x7339d5b924",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LiveTrainerCameraPitchDirection",
      "addr": "0x7339d5b950",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LiveTrainerCameraPitchDirection",
      "addr": "0x7339d5b9ec",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LiveTrainerCameraSwipeSensitivity",
      "addr": "0x7339d5ba18",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LiveTrainerCameraSwipeSensitivity",
      "addr": "0x7339d5bab4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LiveTrainerCameraGyroSensitivity",
      "addr": "0x7339d5bae0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LiveTrainerCameraGyroSensitivity",
      "addr": "0x7339d5bb7c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LiveTrainerCameraDisplayCameraReset",
      "addr": "0x7339d5bba4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LiveTrainerCameraDisplayCameraReset",
      "addr": "0x7339d5bc24",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsEnableStorySkipPopup",
      "addr": "0x7339d5bc48",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableStorySkipPopup",
      "addr": "0x7339d5bcfc",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableNewStoryAuto",
      "addr": "0x7339d5bd6c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableNewStoryAuto",
      "addr": "0x7339d5be20",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsCancelSkipMainStory",
      "addr": "0x7339d5be94",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsCancelSkipMainStory",
      "addr": "0x7339d5bf48",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsCancelSkipCharaStory",
      "addr": "0x7339d5bfb8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsCancelSkipCharaStory",
      "addr": "0x7339d5c06c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsCancelSkipOnsenEvent",
      "addr": "0x7339d5c0e0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsCancelSkipOnsenEvent",
      "addr": "0x7339d5c194",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableSuperHighSpeedSkip",
      "addr": "0x7339d5c204",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableSuperHighSpeedSkip",
      "addr": "0x7339d5c2b8",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_StoryHighSpeedType",
      "addr": "0x7339d5c32c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_StoryHighSpeedType",
      "addr": "0x7339d5c408",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_TrainingHighSpeedType",
      "addr": "0x7339d5c49c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainingHighSpeedType",
      "addr": "0x7339d5c578",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_EpisodeStartSettingsIsOrientationLandscape",
      "addr": "0x7339d5c60c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_EpisodeStartSettingsIsOrientationLandscape",
      "addr": "0x7339d5c6c0",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_EpisodeStartSettingIsNeedVoiceDownload",
      "addr": "0x7339d5c730",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_EpisodeStartSettingIsNeedVoiceDownload",
      "addr": "0x7339d5c7e4",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsDeleteOnLoginSettingStory",
      "addr": "0x7339d5c858",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsDeleteOnLoginSettingStory",
      "addr": "0x7339d5c90c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsDeleteOnLoginSettingEvent",
      "addr": "0x7339d5c97c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsDeleteOnLoginSettingEvent",
      "addr": "0x7339d5ca30",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsDeleteOnLoginSettingLive",
      "addr": "0x7339d5caa4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsDeleteOnLoginSettingLive",
      "addr": "0x7339d5cb58",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsDeleteOnLoginSettingRace",
      "addr": "0x7339d5cbc8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsDeleteOnLoginSettingRace",
      "addr": "0x7339d5cc7c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsDeleteOnLoginSettingOthers",
      "addr": "0x7339d5ccf0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsDeleteOnLoginSettingOthers",
      "addr": "0x7339d5cda4",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_StoryCraneGameResultId",
      "addr": "0x7339d5ce14",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_StoryCraneGameResultId",
      "addr": "0x7339d5cef0",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_StoryCraneGameResultInfo",
      "addr": "0x7339d5cf84",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_StoryCraneGameResultInfo",
      "addr": "0x7339d5d030",
      "params": 0,
      "return_type": "string",
      "static": false
    },
    {
      "name": "set_CharacterTitleVoiceArray",
      "addr": "0x7339d5d098",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CharacterTitleVoiceArray",
      "addr": "0x7339d5d108",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeLastStartTime",
      "addr": "0x7339d5d124",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeLastStartTime",
      "addr": "0x7339d5d1c0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsEnableSingleStartRaceRecommendPopup",
      "addr": "0x7339d5d1ec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableSingleStartRaceRecommendPopup",
      "addr": "0x7339d5d2a0",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_SingleRaceRecommendSelected",
      "addr": "0x7339d5d310",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleRaceRecommendSelected",
      "addr": "0x7339d5d3c4",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_SingleRaceRecommendedCourse",
      "addr": "0x7339d5d438",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleRaceRecommendedCourse",
      "addr": "0x7339d5d514",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsEnableRaceDressTrackSuit",
      "addr": "0x7339d5d5a8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableRaceDressTrackSuit",
      "addr": "0x7339d5d65c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableRaceDisplayEnemySkill",
      "addr": "0x7339d5d6d0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableRaceDisplayEnemySkill",
      "addr": "0x7339d5d784",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableResultNicknamePopup",
      "addr": "0x7339d5d7f4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableResultNicknamePopup",
      "addr": "0x7339d5d8a8",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableSingleFriendSupportPopup",
      "addr": "0x7339d5d91c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableSingleFriendSupportPopup",
      "addr": "0x7339d5d9d0",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableSingleHolidayPopup",
      "addr": "0x7339d5da40",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableSingleHolidayPopup",
      "addr": "0x7339d5daf4",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableSingleOutingPopup",
      "addr": "0x7339d5db68",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableSingleOutingPopup",
      "addr": "0x7339d5dc1c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableSingleHospitalPopup",
      "addr": "0x7339d5dc8c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableSingleHospitalPopup",
      "addr": "0x7339d5dd40",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableSingleFreeShopExchangePop",
      "addr": "0x7339d5ddb4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableSingleFreeShopExchangePop",
      "addr": "0x7339d5de68",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableSingleModeScenarioArcPotentialLevelupPopup",
      "addr": "0x7339d5ded8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableSingleModeScenarioArcPotentialLevelupPopup",
      "addr": "0x7339d5df8c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableSingleLegendPopularityCuttAutoSkip",
      "addr": "0x7339d5e000",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableSingleLegendPopularityCuttAutoSkip",
      "addr": "0x7339d5e0b4",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableSingleFreeShopExchangeAutoUsePopup",
      "addr": "0x7339d5e124",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableSingleFreeShopExchangeAutoUsePopup",
      "addr": "0x7339d5e1d8",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_SingleModeHintShow",
      "addr": "0x7339d5e24c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeHintShow",
      "addr": "0x7339d5e2cc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsEnableScenarioLiveSkipGrandLive",
      "addr": "0x7339d5e2ec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableScenarioLiveSkipGrandLive",
      "addr": "0x7339d5e36c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeOverrideBGMId",
      "addr": "0x7339d5e390",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeOverrideBGMId",
      "addr": "0x7339d5e46c",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_IsEnableScenarioSkillUpgradePopup",
      "addr": "0x7339d5e500",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableScenarioSkillUpgradePopup",
      "addr": "0x7339d5e5b4",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_SingleModeSkillLearningSkillDescriptionOmitSaveKey",
      "addr": "0x7339d5e624",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeSkillLearningSkillDescriptionOmitSaveKey",
      "addr": "0x7339d5e6c0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsEnableSingleModeSkillLearningSkillDescriptionOmit",
      "addr": "0x7339d5e6ec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableSingleModeSkillLearningSkillDescriptionOmit",
      "addr": "0x7339d5e76c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "get_IsShowSpecialEffect",
      "addr": "0x7339d5e78c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsShowSpecialEffect",
      "addr": "0x7339d5e7fc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableShowSpecialEffectSetting",
      "addr": "0x7339d5e8b0",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableShowSpecialEffectSetting",
      "addr": "0x7339d5e924",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_AlreadyShowSpecialEffectSettingAtStill",
      "addr": "0x7339d5e9d8",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_AlreadyShowSpecialEffectSettingAtStill",
      "addr": "0x7339d5ea48",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableShowSpecialHomeStateRaceConfirm",
      "addr": "0x7339d5eafc",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableShowSpecialHomeStateRaceConfirm",
      "addr": "0x7339d5eb70",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "set_IsSkipSingleCookDishPerformance",
      "addr": "0x7339d5ec24",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsSkipSingleCookDishPerformance",
      "addr": "0x7339d5eca4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsRecommendLowCostDish",
      "addr": "0x7339d5ecc8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsRecommendLowCostDish",
      "addr": "0x7339d5ed48",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeScenarioLegendGoalCaptureHash",
      "addr": "0x7339d5ed68",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeScenarioLegendGoalCaptureHash",
      "addr": "0x7339d5edd8",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_IsSkipBathingSingleModeScenarioOnsen",
      "addr": "0x7339d5edf4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsSkipBathingSingleModeScenarioOnsen",
      "addr": "0x7339d5ee74",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsBulkReacquireFactor",
      "addr": "0x7339d5ee94",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsBulkReacquireFactor",
      "addr": "0x7339d5ef14",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "get_IsEnableShowBreedersScenarioDressChangeSetting",
      "addr": "0x7339d5ef38",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableShowBreedersScenarioDressChangeSetting",
      "addr": "0x7339d5efa8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableBreedersScenarioRouteAlertDialog",
      "addr": "0x7339d5f05c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableBreedersScenarioRouteAlertDialog",
      "addr": "0x7339d5f0d0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsSkipSingleModeScenarioBreedersTeamSpTrainingCutIn",
      "addr": "0x7339d5f184",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsSkipSingleModeScenarioBreedersTeamSpTrainingCutIn",
      "addr": "0x7339d5f1f4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "set_IsEnableSingleBreedersTeamUnionProgressCutAutoSkip",
      "addr": "0x7339d5f2a8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableSingleBreedersTeamUnionProgressCutAutoSkip",
      "addr": "0x7339d5f35c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsSuccessionFactorListFilterSetting",
      "addr": "0x7339d5f3d0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsSuccessionFactorListFilterSetting",
      "addr": "0x7339d5f450",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsSkipServingPracticeForScenarioRamen",
      "addr": "0x7339d5f470",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsSkipServingPracticeForScenarioRamen",
      "addr": "0x7339d5f4f0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsEnableStartIdleSingleModeConfirm",
      "addr": "0x7339d5f514",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableStartIdleSingleModeConfirm",
      "addr": "0x7339d5f594",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsShowFirstIdleSingleModeConfirmEntry",
      "addr": "0x7339d5f5b4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsShowFirstIdleSingleModeConfirmEntry",
      "addr": "0x7339d5f634",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CardLevelUpSortMenu",
      "addr": "0x7339d5f658",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CardLevelUpSortMenu",
      "addr": "0x7339d5f6f4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CardLevelUpSortAsc",
      "addr": "0x7339d5f720",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CardLevelUpSortAsc",
      "addr": "0x7339d5f7a0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CardLevelUpFilterMenuArray",
      "addr": "0x7339d5f7c4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CardLevelUpFilterMenuArray",
      "addr": "0x7339d5f834",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_CardRarityUpSortMenu",
      "addr": "0x7339d5f850",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CardRarityUpSortMenu",
      "addr": "0x7339d5f8ec",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CardRarityUpSortAsc",
      "addr": "0x7339d5f914",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CardRarityUpSortAsc",
      "addr": "0x7339d5f994",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CardRarityUpFilterMenuArray",
      "addr": "0x7339d5f9b8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CardRarityUpFilterMenuArray",
      "addr": "0x7339d5fa28",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_CardHintLevelUpSortMenu",
      "addr": "0x7339d5fa44",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CardHintLevelUpSortMenu",
      "addr": "0x7339d5fae0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CardHintLevelUpSortAsc",
      "addr": "0x7339d5fb0c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CardHintLevelUpSortAsc",
      "addr": "0x7339d5fb8c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CardHintLevelUpFilterMenuArray",
      "addr": "0x7339d5fbb0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CardHintLevelUpFilterMenuArray",
      "addr": "0x7339d5fc20",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_CardCatalogSortMenu",
      "addr": "0x7339d5fc3c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CardCatalogSortMenu",
      "addr": "0x7339d5fcd8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CardCatalogSortAsc",
      "addr": "0x7339d5fd00",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CardCatalogSortAsc",
      "addr": "0x7339d5fd80",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CardCatalogFilterMenuArray",
      "addr": "0x7339d5fda4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CardCatalogFilterMenuArray",
      "addr": "0x7339d5fe14",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SupportCardDeckEditSortMenu",
      "addr": "0x7339d5fe30",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardDeckEditSortMenu",
      "addr": "0x7339d5fecc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SupportCardDeckEditSortAsc",
      "addr": "0x7339d5fef8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardDeckEditSortAsc",
      "addr": "0x7339d5ff78",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SupportCardDeckEditFilterMenuArray",
      "addr": "0x7339d5ff9c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardDeckEditFilterMenuArray",
      "addr": "0x7339d6000c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SupportCardConvertSortMenu",
      "addr": "0x7339d60028",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardConvertSortMenu",
      "addr": "0x7339d600c4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SupportCardConvertSortAsc",
      "addr": "0x7339d600ec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardConvertSortAsc",
      "addr": "0x7339d6016c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SupportCardConvertFilterMenuArray",
      "addr": "0x7339d60190",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardConvertFilterMenuArray",
      "addr": "0x7339d60200",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SupportCardListSortMenu",
      "addr": "0x7339d6021c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardListSortMenu",
      "addr": "0x7339d602b8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SupportCardListSortAsc",
      "addr": "0x7339d602e4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardListSortAsc",
      "addr": "0x7339d60364",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SupportCardListFilterMenuArray",
      "addr": "0x7339d60388",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardListFilterMenuArray",
      "addr": "0x7339d603f8",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_TrainedCharaSortMenu",
      "addr": "0x7339d60414",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainedCharaSortMenu",
      "addr": "0x7339d604b0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrainedCharaSortAsc",
      "addr": "0x7339d604d8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainedCharaSortAsc",
      "addr": "0x7339d60558",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrainedCharaFilterMenuArray",
      "addr": "0x7339d6057c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainedCharaFilterMenuArray",
      "addr": "0x7339d605ec",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_TrainedCharaFactorRarityMinStatus",
      "addr": "0x7339d60608",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainedCharaFactorRarityMinStatus",
      "addr": "0x7339d606a4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrainedCharaFactorRarityMinProper",
      "addr": "0x7339d606d0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainedCharaFactorRarityMinProper",
      "addr": "0x7339d6076c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrainedCharaFactorRarityMinUnique",
      "addr": "0x7339d60798",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainedCharaFactorRarityMinUnique",
      "addr": "0x7339d60834",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrainedCharaNeedSuccessionFactorStatus",
      "addr": "0x7339d6085c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainedCharaNeedSuccessionFactorStatus",
      "addr": "0x7339d608dc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrainedCharaNeedSuccessionFactorProper",
      "addr": "0x7339d60900",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainedCharaNeedSuccessionFactorProper",
      "addr": "0x7339d60980",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrainedCharaNeedSuccessionFactorUnique",
      "addr": "0x7339d609a0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainedCharaNeedSuccessionFactorUnique",
      "addr": "0x7339d60a20",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrainedCharaFactorIdUnique",
      "addr": "0x7339d60a44",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainedCharaFactorIdUnique",
      "addr": "0x7339d60ae0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrainedCharaCommonFactorIdArray",
      "addr": "0x7339d60b0c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainedCharaCommonFactorIdArray",
      "addr": "0x7339d60b7c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_TrainedCharaCommonFactorRarityMinArray",
      "addr": "0x7339d60b98",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainedCharaCommonFactorRarityMinArray",
      "addr": "0x7339d60c08",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_TrainedCharaNeedSuccessionCommonFactorArray",
      "addr": "0x7339d60c24",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainedCharaNeedSuccessionCommonFactorArray",
      "addr": "0x7339d60c94",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_TeamEditEntrySortMenu",
      "addr": "0x7339d60cb0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamEditEntrySortMenu",
      "addr": "0x7339d60d4c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TeamEditEntrySortAsc",
      "addr": "0x7339d60d78",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamEditEntrySortAsc",
      "addr": "0x7339d60df8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TeamEditEntryFilterMenuArray",
      "addr": "0x7339d60e1c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamEditEntryFilterMenuArray",
      "addr": "0x7339d60e8c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ChampionsEntrySortMenu",
      "addr": "0x7339d60ea8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChampionsEntrySortMenu",
      "addr": "0x7339d60f44",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChampionsEntrySortAsc",
      "addr": "0x7339d60f6c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChampionsEntrySortAsc",
      "addr": "0x7339d60fec",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChampionsEntryFilterMenuArray",
      "addr": "0x7339d61010",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChampionsEntryFilterMenuArray",
      "addr": "0x7339d61080",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_HeroesEntrySortMenu",
      "addr": "0x7339d6109c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeroesEntrySortMenu",
      "addr": "0x7339d61138",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_HeroesEntrySortAsc",
      "addr": "0x7339d61164",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeroesEntrySortAsc",
      "addr": "0x7339d611e4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_HeroesEntryFilterMenuArray",
      "addr": "0x7339d61208",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeroesEntryFilterMenuArray",
      "addr": "0x7339d61278",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_HeroesEntryRecordSortMenu",
      "addr": "0x7339d61294",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeroesEntryRecordSortMenu",
      "addr": "0x7339d61330",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_HeroesEntryRecordSortAsc",
      "addr": "0x7339d61358",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeroesEntryRecordSortAsc",
      "addr": "0x7339d613d8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_HeroesEntryRecordFilterMenuArray",
      "addr": "0x7339d613fc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeroesEntryRecordFilterMenuArray",
      "addr": "0x7339d6146c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_HeroesLeagueScoreSortMenu",
      "addr": "0x7339d61488",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeroesLeagueScoreSortMenu",
      "addr": "0x7339d61524",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_HeroesLeagueScoreSortAsc",
      "addr": "0x7339d61550",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeroesLeagueScoreSortAsc",
      "addr": "0x7339d615d0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_HeroesLeagueScoreFilterMenuArray",
      "addr": "0x7339d615f4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeroesLeagueScoreFilterMenuArray",
      "addr": "0x7339d61664",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_HeroesIsAutoUseItem",
      "addr": "0x7339d61680",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeroesIsAutoUseItem",
      "addr": "0x7339d61700",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_HeroesIsShowConfirmPopup",
      "addr": "0x7339d61720",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeroesIsShowConfirmPopup",
      "addr": "0x7339d617a0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_HeroesIsEnableAllRoundSkip",
      "addr": "0x7339d617c4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeroesIsEnableAllRoundSkip",
      "addr": "0x7339d61844",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_HeroesRankingRealRewardLastRemindTime",
      "addr": "0x7339d61864",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeroesRankingRealRewardLastRemindTime",
      "addr": "0x7339d61900",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_UltimateEntrySortMenu",
      "addr": "0x7339d6192c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_UltimateEntrySortMenu",
      "addr": "0x7339d619c8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_UltimateEntrySortAsc",
      "addr": "0x7339d619f0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_UltimateEntrySortAsc",
      "addr": "0x7339d61a70",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_UltimateEntryFilterMenuArray",
      "addr": "0x7339d61a94",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_UltimateEntryFilterMenuArray",
      "addr": "0x7339d61b04",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_UltimateRaceLogGroupIdArray",
      "addr": "0x7339d61b20",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_UltimateRaceLogGroupIdArray",
      "addr": "0x7339d61b90",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_UltimateRaceLogTrainedCharaIdArray",
      "addr": "0x7339d61bac",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_UltimateRaceLogTrainedCharaIdArray",
      "addr": "0x7339d61c1c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_UltimateClearHistorySortMenu",
      "addr": "0x7339d61c38",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_UltimateClearHistorySortMenu",
      "addr": "0x7339d61cd4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_UltimateClearHistorySortAsc",
      "addr": "0x7339d61d00",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_UltimateClearHistorySortAsc",
      "addr": "0x7339d61d80",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_UltimateClearHistoryFilterMenuArray",
      "addr": "0x7339d61da4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_UltimateClearHistoryFilterMenuArray",
      "addr": "0x7339d61e14",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_UltimateClearHistorySelectedContentsId",
      "addr": "0x7339d61e30",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_UltimateClearHistorySelectedContentsId",
      "addr": "0x7339d61ecc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_UltimateTopToggleIndex",
      "addr": "0x7339d61ef4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_UltimateTopToggleIndex",
      "addr": "0x7339d61f90",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_UltimateTopLastCheckLatestEventId",
      "addr": "0x7339d61fbc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_UltimateTopLastCheckLatestEventId",
      "addr": "0x7339d62058",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RoomMatchEntrySortMenu",
      "addr": "0x7339d62084",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchEntrySortMenu",
      "addr": "0x7339d62120",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RoomMatchEntrySortAsc",
      "addr": "0x7339d6214c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchEntrySortAsc",
      "addr": "0x7339d621cc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RoomMatchEntryFilterMenuArray",
      "addr": "0x7339d621ec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchEntryFilterMenuArray",
      "addr": "0x7339d6225c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_RoomMatchEntryTrialSortMenu",
      "addr": "0x7339d62278",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchEntryTrialSortMenu",
      "addr": "0x7339d62314",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RoomMatchEntryTrialSortAsc",
      "addr": "0x7339d62340",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchEntryTrialSortAsc",
      "addr": "0x7339d623c0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RoomMatchEntryTrialFilterMenuArray",
      "addr": "0x7339d623e4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchEntryTrialFilterMenuArray",
      "addr": "0x7339d62454",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_RoomMatchRoomListSortMenu",
      "addr": "0x7339d62470",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchRoomListSortMenu",
      "addr": "0x7339d6250c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RoomMatchRoomListSortAsc",
      "addr": "0x7339d62534",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchRoomListSortAsc",
      "addr": "0x7339d625b4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RoomMatchRoomListFilterMenuArray",
      "addr": "0x7339d625d8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchRoomListFilterMenuArray",
      "addr": "0x7339d62648",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_RoomMatchPresetSortMenu",
      "addr": "0x7339d62888",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchPresetSortMenu",
      "addr": "0x7339d62924",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RoomMatchPresetSortAsc",
      "addr": "0x7339d62950",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchPresetSortAsc",
      "addr": "0x7339d629d0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RoomMatchPresetFilterArray",
      "addr": "0x7339d629f4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchPresetFilterArray",
      "addr": "0x7339d62a64",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_RoomMatchPresetTrialSortMenu",
      "addr": "0x7339d62a80",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchPresetTrialSortMenu",
      "addr": "0x7339d62b1c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RoomMatchPresetTrialSortAsc",
      "addr": "0x7339d62b48",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchPresetTrialSortAsc",
      "addr": "0x7339d62bc8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RoomMatchPresetTrialFilterArray",
      "addr": "0x7339d62bec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RoomMatchPresetTrialFilterArray",
      "addr": "0x7339d62c5c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_StoryUnlockEntrySortMenu",
      "addr": "0x7339d62c78",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_StoryUnlockEntrySortMenu",
      "addr": "0x7339d62d14",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_StoryUnlockEntrySortAsc",
      "addr": "0x7339d62d3c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_StoryUnlockEntrySortAsc",
      "addr": "0x7339d62dbc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_StoryUnlockEntryFilterMenuArray",
      "addr": "0x7339d62de0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_StoryUnlockEntryFilterMenuArray",
      "addr": "0x7339d62e50",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_EpisodeCharacterSortMenu",
      "addr": "0x7339d62e6c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_EpisodeCharacterSortMenu",
      "addr": "0x7339d62f08",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_EpisodeCharacterSortAsc",
      "addr": "0x7339d62f34",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_EpisodeCharacterSortAsc",
      "addr": "0x7339d62fb4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_EpisodeExtraCommercialSortMenu",
      "addr": "0x7339d62fd8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_EpisodeExtraCommercialSortMenu",
      "addr": "0x7339d63074",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_EpisodeExtraCommercialSortAsc",
      "addr": "0x7339d630a0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_EpisodeExtraCommercialSortAsc",
      "addr": "0x7339d63120",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_EpisodeExtraCommercialFilterMenuArray",
      "addr": "0x7339d63144",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_EpisodeExtraCommercialFilterMenuArray",
      "addr": "0x7339d631b4",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_LegendEntrySortMenu",
      "addr": "0x7339d631d0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LegendEntrySortMenu",
      "addr": "0x7339d6326c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LegendEntrySortAsc",
      "addr": "0x7339d63294",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LegendEntrySortAsc",
      "addr": "0x7339d63314",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LegendEntryFilterMenuArray",
      "addr": "0x7339d63338",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LegendEntryFilterMenuArray",
      "addr": "0x7339d633a8",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_DailyLegendEntrySortMenu",
      "addr": "0x7339d633c4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyLegendEntrySortMenu",
      "addr": "0x7339d63460",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_DailyLegendEntrySortAsc",
      "addr": "0x7339d6348c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyLegendEntrySortAsc",
      "addr": "0x7339d6350c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_DailyLegendEntryFilterMenuArray",
      "addr": "0x7339d63530",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyLegendEntryFilterMenuArray",
      "addr": "0x7339d635a0",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_DailyLegendListSortMenu",
      "addr": "0x7339d635bc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyLegendListSortMenu",
      "addr": "0x7339d63658",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_DailyLegendListSortAsc",
      "addr": "0x7339d63680",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyLegendListSortAsc",
      "addr": "0x7339d63700",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_DailyLegendListFilterMenuArray",
      "addr": "0x7339d63724",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyLegendListFilterMenuArray",
      "addr": "0x7339d63794",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_DailyEntrySortMenu",
      "addr": "0x7339d637b0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyEntrySortMenu",
      "addr": "0x7339d6384c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_DailyEntrySortAsc",
      "addr": "0x7339d63878",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyEntrySortAsc",
      "addr": "0x7339d638f8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_DailyEntryFilterMenuArray",
      "addr": "0x7339d6391c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyEntryFilterMenuArray",
      "addr": "0x7339d6398c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_DailyIsEnableSkipMultiRace",
      "addr": "0x7339d639a8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyIsEnableSkipMultiRace",
      "addr": "0x7339d63a28",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PresentSortAsc",
      "addr": "0x7339d63a48",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PresentSortAsc",
      "addr": "0x7339d63ac8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PresentFilterMenuArray",
      "addr": "0x7339d63aec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PresentFilterMenuArray",
      "addr": "0x7339d63b5c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_PresentHistorySortAsc",
      "addr": "0x7339d63b78",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PresentHistorySortAsc",
      "addr": "0x7339d63bf8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PresentHistoryFilterMenuArray",
      "addr": "0x7339d63c18",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PresentHistoryFilterMenuArray",
      "addr": "0x7339d63c88",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_FriendFollowSortMenu",
      "addr": "0x7339d63ca4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendFollowSortMenu",
      "addr": "0x7339d63d40",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FriendFollowSortAsc",
      "addr": "0x7339d63d6c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendFollowSortAsc",
      "addr": "0x7339d63dec",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FriendFollowerSortMenu",
      "addr": "0x7339d63e10",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendFollowerSortMenu",
      "addr": "0x7339d63eac",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FriendFollowerSortAsc",
      "addr": "0x7339d63ed8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendFollowerSortAsc",
      "addr": "0x7339d63f58",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FriendRecommendSortMenu",
      "addr": "0x7339d63f7c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendRecommendSortMenu",
      "addr": "0x7339d64018",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FriendRecommendSortAsc",
      "addr": "0x7339d64044",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendRecommendSortAsc",
      "addr": "0x7339d640c4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FriendDirectorySortMenu",
      "addr": "0x7339d640e8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendDirectorySortMenu",
      "addr": "0x7339d64184",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FriendDirectorySortAsc",
      "addr": "0x7339d641b0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendDirectorySortAsc",
      "addr": "0x7339d64230",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CharacterDirectorySortMenu",
      "addr": "0x7339d64254",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CharacterDirectorySortMenu",
      "addr": "0x7339d642f0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CharacterDirectorySortAsc",
      "addr": "0x7339d6431c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CharacterDirectorySortAsc",
      "addr": "0x7339d6439c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CharacterNoteSortMenu",
      "addr": "0x7339d643c0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CharacterNoteSortMenu",
      "addr": "0x7339d6445c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CharacterNoteSortAsc",
      "addr": "0x7339d64488",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CharacterNoteSortAsc",
      "addr": "0x7339d64508",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TipsComicSortMenu",
      "addr": "0x7339d6452c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TipsComicSortMenu",
      "addr": "0x7339d645c8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TipsComicSortAsc",
      "addr": "0x7339d645f4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TipsComicSortAsc",
      "addr": "0x7339d64674",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TipsCharaSortMenu",
      "addr": "0x7339d64698",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TipsCharaSortMenu",
      "addr": "0x7339d64734",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TipsCharaSortAsc",
      "addr": "0x7339d64760",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TipsCharaSortAsc",
      "addr": "0x7339d647e0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrophyRoomSortMenu",
      "addr": "0x7339d64804",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrophyRoomSortMenu",
      "addr": "0x7339d648a0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrophyRoomSortAsc",
      "addr": "0x7339d648cc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrophyRoomSortAsc",
      "addr": "0x7339d6494c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CircleMemberSortMenu",
      "addr": "0x7339d64970",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleMemberSortMenu",
      "addr": "0x7339d64a0c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CircleMemberSortAsc",
      "addr": "0x7339d64a38",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleMemberSortAsc",
      "addr": "0x7339d64ab8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CircleMemberLightSortMenu",
      "addr": "0x7339d64adc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleMemberLightSortMenu",
      "addr": "0x7339d64b78",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CircleMemberLightSortAsc",
      "addr": "0x7339d64ba4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleMemberLightSortAsc",
      "addr": "0x7339d64c24",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CircleChatFilter",
      "addr": "0x7339d64c48",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleChatFilter",
      "addr": "0x7339d64ce4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CircleRequestItemShoesSortMenu",
      "addr": "0x7339d64d10",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleRequestItemShoesSortMenu",
      "addr": "0x7339d64dac",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CircleRequestItemShoesSortAsc",
      "addr": "0x7339d64dd8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleRequestItemShoesSortAsc",
      "addr": "0x7339d64e58",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CircleRequestItemBlanketSortMenu",
      "addr": "0x7339d64e78",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleRequestItemBlanketSortMenu",
      "addr": "0x7339d64f14",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CircleRequestItemBlanketSortAsc",
      "addr": "0x7339d64f40",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleRequestItemBlanketSortAsc",
      "addr": "0x7339d64fc0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CircleMultiDonateSettingTypeIndex",
      "addr": "0x7339d64fe0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleMultiDonateSettingTypeIndex",
      "addr": "0x7339d6507c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_HonorSortAsc",
      "addr": "0x7339d650a8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HonorSortAsc",
      "addr": "0x7339d65128",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_HonorFilterMenuArray",
      "addr": "0x7339d65148",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HonorFilterMenuArray",
      "addr": "0x7339d651b8",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_NickNameSortAsc",
      "addr": "0x7339d651d4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_NickNameSortAsc",
      "addr": "0x7339d65254",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_NickNameListSortMenu",
      "addr": "0x7339d65274",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_NickNameListSortMenu",
      "addr": "0x7339d65310",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_NickNameListFilterMenuArray",
      "addr": "0x7339d6533c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_NickNameListFilterMenuArray",
      "addr": "0x7339d653ac",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ValentineCharaSelectSortAsc",
      "addr": "0x7339d653c8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ValentineCharaSelectSortAsc",
      "addr": "0x7339d65448",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ValentineCharaSelectSortMenu",
      "addr": "0x7339d65468",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ValentineCharaSelectSortMenu",
      "addr": "0x7339d65504",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ValentineCharaSelectSpecialSortAsc",
      "addr": "0x7339d65530",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ValentineCharaSelectSpecialSortAsc",
      "addr": "0x7339d655b0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ValentineCharaSelectSpecialSortMenu",
      "addr": "0x7339d655d0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ValentineCharaSelectSpecialSortMenu",
      "addr": "0x7339d6566c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_MissionRaceRecommendSortAsc",
      "addr": "0x7339d65698",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_MissionRaceRecommendSortAsc",
      "addr": "0x7339d65718",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_MissionRaceRecommendSortMenu",
      "addr": "0x7339d65738",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_MissionRaceRecommendSortMenu",
      "addr": "0x7339d657d4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_MissionRaceRecommendFilterMenuArray",
      "addr": "0x7339d65800",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_MissionRaceRecommendFilterMenuArray",
      "addr": "0x7339d65870",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ProfileFavoriteSortMenu",
      "addr": "0x7339d6588c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileFavoriteSortMenu",
      "addr": "0x7339d65928",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileFavoriteSortAsc",
      "addr": "0x7339d65954",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileFavoriteSortAsc",
      "addr": "0x7339d659d4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileFavoriteFilterMenuArray",
      "addr": "0x7339d659f8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileFavoriteFilterMenuArray",
      "addr": "0x7339d65a68",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ProfileSupportSortMenu",
      "addr": "0x7339d65a84",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileSupportSortMenu",
      "addr": "0x7339d65b20",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileSupportSortAsc",
      "addr": "0x7339d65b48",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileSupportSortAsc",
      "addr": "0x7339d65bc8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileSupportFilterMenuArray",
      "addr": "0x7339d65bf0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileSupportFilterMenuArray",
      "addr": "0x7339d65c60",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ProfileRepCharaSortMenu",
      "addr": "0x7339d65c7c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileRepCharaSortMenu",
      "addr": "0x7339d65d18",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileRepCharaSortAsc",
      "addr": "0x7339d65d48",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileRepCharaSortAsc",
      "addr": "0x7339d65dc8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileRepCharaFilterMenuArray",
      "addr": "0x7339d65df0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileRepCharaFilterMenuArray",
      "addr": "0x7339d65e60",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ProfileRepCharaFactorRarityMinStatus",
      "addr": "0x7339d65e7c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileRepCharaFactorRarityMinStatus",
      "addr": "0x7339d65f18",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileRepCharaFactorRarityMinProper",
      "addr": "0x7339d65f40",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileRepCharaFactorRarityMinProper",
      "addr": "0x7339d65fdc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileRepCharaFactorRarityMinUnique",
      "addr": "0x7339d6600c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileRepCharaFactorRarityMinUnique",
      "addr": "0x7339d660a8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileRepCharaNeedSuccessionFactorStatus",
      "addr": "0x7339d660d8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileRepCharaNeedSuccessionFactorStatus",
      "addr": "0x7339d66158",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileRepCharaNeedSuccessionFactorProper",
      "addr": "0x7339d66180",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileRepCharaNeedSuccessionFactorProper",
      "addr": "0x7339d66200",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileRepCharaNeedSuccessionFactorUnique",
      "addr": "0x7339d66220",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileRepCharaNeedSuccessionFactorUnique",
      "addr": "0x7339d662a0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileRepCharaFactorIdUnique",
      "addr": "0x7339d662c8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileRepCharaFactorIdUnique",
      "addr": "0x7339d66364",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileRepCharaCommonFactorIdArray",
      "addr": "0x7339d6638c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileRepCharaCommonFactorIdArray",
      "addr": "0x7339d663fc",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ProfileRepCharaCommonFactorRarityMinArray",
      "addr": "0x7339d66418",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileRepCharaCommonFactorRarityMinArray",
      "addr": "0x7339d66488",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ProfileRepCharaNeedSuccessionCommonFactorArray",
      "addr": "0x7339d664a4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileRepCharaNeedSuccessionCommonFactorArray",
      "addr": "0x7339d66514",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeStartCardSelectSortMenu",
      "addr": "0x7339d66530",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartCardSelectSortMenu",
      "addr": "0x7339d665cc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeStartCardSelectSortAsc",
      "addr": "0x7339d665fc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartCardSelectSortAsc",
      "addr": "0x7339d6667c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeStartCardSelectFilterMenuArray",
      "addr": "0x7339d6669c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartCardSelectFilterMenuArray",
      "addr": "0x7339d6670c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaSortMenu",
      "addr": "0x7339d66728",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaSortMenu",
      "addr": "0x7339d667c4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaSortAsc",
      "addr": "0x7339d667f4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaSortAsc",
      "addr": "0x7339d66874",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaFilterMenuArray",
      "addr": "0x7339d6689c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaFilterMenuArray",
      "addr": "0x7339d6690c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaFactorRarityMinStatus",
      "addr": "0x7339d66928",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaFactorRarityMinStatus",
      "addr": "0x7339d669c4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaFactorRarityMinProper",
      "addr": "0x7339d669ec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaFactorRarityMinProper",
      "addr": "0x7339d66a88",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaFactorRarityMinUnique",
      "addr": "0x7339d66ab8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaFactorRarityMinUnique",
      "addr": "0x7339d66b54",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaNeedSuccessionFactorStatus",
      "addr": "0x7339d66b84",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaNeedSuccessionFactorStatus",
      "addr": "0x7339d66c04",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaNeedSuccessionFactorProper",
      "addr": "0x7339d66c2c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaNeedSuccessionFactorProper",
      "addr": "0x7339d66cac",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaNeedSuccessionFactorUnique",
      "addr": "0x7339d66ccc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaNeedSuccessionFactorUnique",
      "addr": "0x7339d66d4c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaFactorIdUnique",
      "addr": "0x7339d66d74",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaFactorIdUnique",
      "addr": "0x7339d66e10",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCommonFactorIdArray",
      "addr": "0x7339d66e38",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCommonFactorIdArray",
      "addr": "0x7339d66ea8",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCommonFactorRarityMinArray",
      "addr": "0x7339d66ec4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCommonFactorRarityMinArray",
      "addr": "0x7339d66f34",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaNeedSuccessionCommonFactorArray",
      "addr": "0x7339d66f50",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaNeedSuccessionCommonFactorArray",
      "addr": "0x7339d66fc0",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaViewFactor",
      "addr": "0x7339d66fdc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaViewFactor",
      "addr": "0x7339d6705c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeSuccessionCharaIdFirst",
      "addr": "0x7339d6707c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeSuccessionCharaIdFirst",
      "addr": "0x7339d67118",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeSuccessionCharaOwnerIdFirst",
      "addr": "0x7339d67148",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeSuccessionCharaOwnerIdFirst",
      "addr": "0x7339d671e4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeSuccessionEventRentalFirst",
      "addr": "0x7339d67210",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeSuccessionEventRentalFirst",
      "addr": "0x7339d67290",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeSuccessionCharaIdSecond",
      "addr": "0x7339d672b0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeSuccessionCharaIdSecond",
      "addr": "0x7339d6734c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeSuccessionCharaOwnerIdSecond",
      "addr": "0x7339d6737c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeSuccessionCharaOwnerIdSecond",
      "addr": "0x7339d67418",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeSuccessionEventRentalSecond",
      "addr": "0x7339d67448",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeSuccessionEventRentalSecond",
      "addr": "0x7339d674c8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "get_SingleModeSuccessionDeckHistoryHandler",
      "addr": "0x7339d674e8",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCircleSortMenu",
      "addr": "0x7339d675f8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCircleSortMenu",
      "addr": "0x7339d67694",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCircleSortAsc",
      "addr": "0x7339d676c4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCircleSortAsc",
      "addr": "0x7339d67744",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCircleFilterMenuArray",
      "addr": "0x7339d6776c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCircleFilterMenuArray",
      "addr": "0x7339d677dc",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCircleFactorRarityMinStatus",
      "addr": "0x7339d677f8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCircleFactorRarityMinStatus",
      "addr": "0x7339d67894",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCircleFactorRarityMinProper",
      "addr": "0x7339d678bc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCircleFactorRarityMinProper",
      "addr": "0x7339d67958",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCircleFactorRarityMinUnique",
      "addr": "0x7339d67988",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCircleFactorRarityMinUnique",
      "addr": "0x7339d67a24",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCircleNeedSuccessionFactorStatus",
      "addr": "0x7339d67a54",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCircleNeedSuccessionFactorStatus",
      "addr": "0x7339d67ad4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCircleNeedSuccessionFactorProper",
      "addr": "0x7339d67afc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCircleNeedSuccessionFactorProper",
      "addr": "0x7339d67b7c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCircleNeedSuccessionFactorUnique",
      "addr": "0x7339d67b9c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCircleNeedSuccessionFactorUnique",
      "addr": "0x7339d67c1c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCircleFactorIdUnique",
      "addr": "0x7339d67c44",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCircleFactorIdUnique",
      "addr": "0x7339d67ce0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCircleCommonFactorIdArray",
      "addr": "0x7339d67d08",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCircleCommonFactorIdArray",
      "addr": "0x7339d67d78",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCircleCommonFactorRarityMinArray",
      "addr": "0x7339d67d94",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCircleCommonFactorRarityMinArray",
      "addr": "0x7339d67e04",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaCircleNeedSuccessionCommonFactorArray",
      "addr": "0x7339d67e20",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaCircleNeedSuccessionCommonFactorArray",
      "addr": "0x7339d67e90",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaEventSortMenu",
      "addr": "0x7339d67eac",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaEventSortMenu",
      "addr": "0x7339d67f48",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaEventSortAsc",
      "addr": "0x7339d67f70",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaEventSortAsc",
      "addr": "0x7339d67ff0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaEventFilterMenuArray",
      "addr": "0x7339d68018",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaEventFilterMenuArray",
      "addr": "0x7339d68088",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaEventFactorRarityMinStatus",
      "addr": "0x7339d680a4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaEventFactorRarityMinStatus",
      "addr": "0x7339d68140",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaEventFactorRarityMinProper",
      "addr": "0x7339d68170",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaEventFactorRarityMinProper",
      "addr": "0x7339d6820c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaEventFactorRarityMinUnique",
      "addr": "0x7339d6823c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaEventFactorRarityMinUnique",
      "addr": "0x7339d682d8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaEventNeedSuccessionFactorStatus",
      "addr": "0x7339d68300",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaEventNeedSuccessionFactorStatus",
      "addr": "0x7339d68380",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaEventNeedSuccessionFactorProper",
      "addr": "0x7339d683a8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaEventNeedSuccessionFactorProper",
      "addr": "0x7339d68428",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaEventNeedSuccessionFactorUnique",
      "addr": "0x7339d68448",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaEventNeedSuccessionFactorUnique",
      "addr": "0x7339d684c8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaEventFactorIdUnique",
      "addr": "0x7339d684f0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaEventFactorIdUnique",
      "addr": "0x7339d6858c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaEventCommonFactorIdArray",
      "addr": "0x7339d685bc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaEventCommonFactorIdArray",
      "addr": "0x7339d6862c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaEventCommonFactorRarityMinArray",
      "addr": "0x7339d68648",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaEventCommonFactorRarityMinArray",
      "addr": "0x7339d686b8",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeInheritCharaEventNeedSuccessionCommonFactorArray",
      "addr": "0x7339d686d4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeInheritCharaEventNeedSuccessionCommonFactorArray",
      "addr": "0x7339d68744",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeStartSuccessionDeckCharaSortMenu",
      "addr": "0x7339d68760",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartSuccessionDeckCharaSortMenu",
      "addr": "0x7339d687fc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeStartSuccessionDeckCharaSortAsc",
      "addr": "0x7339d6882c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartSuccessionDeckCharaSortAsc",
      "addr": "0x7339d688ac",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeStartSuccessionDeckCharaFilterMenuArray",
      "addr": "0x7339d688d4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartSuccessionDeckCharaFilterMenuArray",
      "addr": "0x7339d68944",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "get_SingleModeSuccessionDeckCharaAdvancedFilterSettingHandler",
      "addr": "0x7339d68960",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set_SingleModeStartSuccessionDeckCharaRentalSortMenu",
      "addr": "0x7339d68a70",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartSuccessionDeckCharaRentalSortMenu",
      "addr": "0x7339d68b0c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeStartSuccessionDeckCharaRentalSortAsc",
      "addr": "0x7339d68b3c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartSuccessionDeckCharaRentalSortAsc",
      "addr": "0x7339d68bbc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeStartSuccessionDeckCharaRentalFilterMenuArray",
      "addr": "0x7339d68be4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartSuccessionDeckCharaRentalFilterMenuArray",
      "addr": "0x7339d68c54",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "get_SingleModeSuccessionDeckCharaRentalAdvancedFilterSettingHandler",
      "addr": "0x7339d68c70",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set_SingleModeStartSuccessionDeckCharaEventRentalSortMenu",
      "addr": "0x7339d68d80",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartSuccessionDeckCharaEventRentalSortMenu",
      "addr": "0x7339d68e1c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeStartSuccessionDeckCharaEventRentalSortAsc",
      "addr": "0x7339d68e4c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartSuccessionDeckCharaEventRentalSortAsc",
      "addr": "0x7339d68ecc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeStartSuccessionDeckCharaEventRentalFilterMenuArray",
      "addr": "0x7339d68ef4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartSuccessionDeckCharaEventRentalFilterMenuArray",
      "addr": "0x7339d68f64",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "get_SingleModeSuccessionDeckCharaEventRentalAdvancedFilterSettingHandler",
      "addr": "0x7339d68f80",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set_SingleModeStartSupportSelectSortMenu",
      "addr": "0x7339d69090",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartSupportSelectSortMenu",
      "addr": "0x7339d6912c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeStartSupportSelectSortAsc",
      "addr": "0x7339d6915c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartSupportSelectSortAsc",
      "addr": "0x7339d691dc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeStartSupportSelectFilterMenuArray",
      "addr": "0x7339d69204",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeStartSupportSelectFilterMenuArray",
      "addr": "0x7339d69274",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeFriendSupportSortMenu",
      "addr": "0x7339d69290",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeFriendSupportSortMenu",
      "addr": "0x7339d6932c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeFriendSupportSortAsc",
      "addr": "0x7339d69354",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeFriendSupportSortAsc",
      "addr": "0x7339d693d4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeFriendSupportFilterMenuArray",
      "addr": "0x7339d693fc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeFriendSupportFilterMenuArray",
      "addr": "0x7339d6946c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_FriendSupportDeckEditSortMenu",
      "addr": "0x7339d69488",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendSupportDeckEditSortMenu",
      "addr": "0x7339d69524",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FriendSupportDeckEditSortAsc",
      "addr": "0x7339d69554",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendSupportDeckEditSortAsc",
      "addr": "0x7339d695d4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FriendSupportDeckEditFilterMenuArray",
      "addr": "0x7339d695fc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendSupportDeckEditFilterMenuArray",
      "addr": "0x7339d6966c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeFriendSupportRentalDeckSortMenu",
      "addr": "0x7339d69688",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeFriendSupportRentalDeckSortMenu",
      "addr": "0x7339d69724",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeFriendSupportRentalDeckSortAsc",
      "addr": "0x7339d6974c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeFriendSupportRentalDeckSortAsc",
      "addr": "0x7339d697cc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeFriendSupportRentalDeckFilterMenuArray",
      "addr": "0x7339d697f4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeFriendSupportRentalDeckFilterMenuArray",
      "addr": "0x7339d69864",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_FriendSupportDeckEditRentalDeckSortMenu",
      "addr": "0x7339d69880",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendSupportDeckEditRentalDeckSortMenu",
      "addr": "0x7339d6991c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FriendSupportDeckEditRentalDeckSortAsc",
      "addr": "0x7339d6994c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendSupportDeckEditRentalDeckSortAsc",
      "addr": "0x7339d699cc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FriendSupportDeckEditRentalDeckFilterMenuArray",
      "addr": "0x7339d699f4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendSupportDeckEditRentalDeckFilterMenuArray",
      "addr": "0x7339d69a64",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_GalleryCharacterCardtSortMenu",
      "addr": "0x7339d69a80",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GalleryCharacterCardtSortMenu",
      "addr": "0x7339d69b1c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GalleryCharacterCardtSortAsc",
      "addr": "0x7339d69b44",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GalleryCharacterCardtSortAsc",
      "addr": "0x7339d69bc4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GalleryCharacterCardtFilterMenuArray",
      "addr": "0x7339d69bec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GalleryCharacterCardtFilterMenuArray",
      "addr": "0x7339d69c5c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_GalleryCharacterCardLastReadStoryId",
      "addr": "0x7339d69c78",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GalleryCharacterCardLastReadStoryId",
      "addr": "0x7339d69d14",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GalleryCharacterCardLastReadCardId",
      "addr": "0x7339d69d44",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GalleryCharacterCardLastReadCardId",
      "addr": "0x7339d69de0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GallerySupportCardtSortMenu",
      "addr": "0x7339d69e10",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GallerySupportCardtSortMenu",
      "addr": "0x7339d69eac",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GallerySupportCardtSortAsc",
      "addr": "0x7339d69ed4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GallerySupportCardtSortAsc",
      "addr": "0x7339d69f54",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GallerySupportCardtFilterMenuArray",
      "addr": "0x7339d69f7c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GallerySupportCardtFilterMenuArray",
      "addr": "0x7339d69fec",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_TalkGalleryCharacterSortMenu",
      "addr": "0x7339d6a008",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TalkGalleryCharacterSortMenu",
      "addr": "0x7339d6a0a4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TalkGalleryCharacterSortAsc",
      "addr": "0x7339d6a0d4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TalkGalleryCharacterSortAsc",
      "addr": "0x7339d6a154",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChampionsEntryRecordSortMenu",
      "addr": "0x7339d6a17c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChampionsEntryRecordSortMenu",
      "addr": "0x7339d6a218",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChampionsEntryRecordSortAsc",
      "addr": "0x7339d6a248",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChampionsEntryRecordSortAsc",
      "addr": "0x7339d6a2c8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChampionsEntryRecordFilterMenuArray",
      "addr": "0x7339d6a2f0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChampionsEntryRecordFilterMenuArray",
      "addr": "0x7339d6a360",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeScenarioTeamRaceSupCharacterSelectSortMenu",
      "addr": "0x7339d6a37c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeScenarioTeamRaceSupCharacterSelectSortMenu",
      "addr": "0x7339d6a418",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeScenarioTeamRaceCharaSelectSortAsc",
      "addr": "0x7339d6a440",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeScenarioTeamRaceCharaSelectSortAsc",
      "addr": "0x7339d6a4c0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeScenarioTeamRaceSupCharacterSelectFilterArray",
      "addr": "0x7339d6a4e8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeScenarioTeamRaceSupCharacterSelectFilterArray",
      "addr": "0x7339d6a558",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_TransferEventDetailIsFirst",
      "addr": "0x7339d6a574",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TransferEventDetailIsFirst",
      "addr": "0x7339d6a5f4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TransferEventDetailSortMenu",
      "addr": "0x7339d6a614",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TransferEventDetailSortMenu",
      "addr": "0x7339d6a6b0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TransferEventDetailSortAsc",
      "addr": "0x7339d6a6e0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TransferEventDetailSortAsc",
      "addr": "0x7339d6a760",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryOwnSortMenu",
      "addr": "0x7339d6a780",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryOwnSortMenu",
      "addr": "0x7339d6a81c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryOwnSortAsc",
      "addr": "0x7339d6a84c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryOwnSortAsc",
      "addr": "0x7339d6a8cc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryOwnFilterArray",
      "addr": "0x7339d6a8ec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryOwnFilterArray",
      "addr": "0x7339d6a95c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryPartnerSortMenu",
      "addr": "0x7339d6a978",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryPartnerSortMenu",
      "addr": "0x7339d6aa14",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryPartnerSortAsc",
      "addr": "0x7339d6aa3c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryPartnerSortAsc",
      "addr": "0x7339d6aabc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryPartnerFilterArray",
      "addr": "0x7339d6aae4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryPartnerFilterArray",
      "addr": "0x7339d6ab54",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryOwnMultiSortMenu",
      "addr": "0x7339d6ab70",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryOwnMultiSortMenu",
      "addr": "0x7339d6ac0c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryOwnMultiSortAsc",
      "addr": "0x7339d6ac3c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryOwnMultiSortAsc",
      "addr": "0x7339d6acbc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryOwnMultiFilterArray",
      "addr": "0x7339d6ace4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryOwnMultiFilterArray",
      "addr": "0x7339d6ad54",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryPartnerMultiSortMenu",
      "addr": "0x7339d6ad70",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryPartnerMultiSortMenu",
      "addr": "0x7339d6ae0c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryPartnerMultiSortAsc",
      "addr": "0x7339d6ae34",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryPartnerMultiSortAsc",
      "addr": "0x7339d6aeb4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryPartnerMultiFilterArray",
      "addr": "0x7339d6aedc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryPartnerMultiFilterArray",
      "addr": "0x7339d6af4c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryTrialMultiSortMenu",
      "addr": "0x7339d6af68",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryTrialMultiSortMenu",
      "addr": "0x7339d6b004",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryTrialMultiSortAsc",
      "addr": "0x7339d6b034",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryTrialMultiSortAsc",
      "addr": "0x7339d6b0b4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticeRaceEntryTrialMultiFilterArray",
      "addr": "0x7339d6b0dc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeRaceEntryTrialMultiFilterArray",
      "addr": "0x7339d6b14c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_PracticeShareTrainedCharacterSortMenu",
      "addr": "0x7339d6b168",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeShareTrainedCharacterSortMenu",
      "addr": "0x7339d6b204",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticeShareTrainedCharacterSortAsc",
      "addr": "0x7339d6b22c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeShareTrainedCharacterSortAsc",
      "addr": "0x7339d6b2ac",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticeShareTrainedCharacterFilterArray",
      "addr": "0x7339d6b2d4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticeShareTrainedCharacterFilterArray",
      "addr": "0x7339d6b344",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_PracticePartnerSearchSortMenu",
      "addr": "0x7339d6b360",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticePartnerSearchSortMenu",
      "addr": "0x7339d6b3fc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticePartnerSearchSortAsc",
      "addr": "0x7339d6b42c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticePartnerSearchSortAsc",
      "addr": "0x7339d6b4ac",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticePartnerSearchFilterArray",
      "addr": "0x7339d6b4d4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticePartnerSearchFilterArray",
      "addr": "0x7339d6b544",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_PracticePartnerSearchFactorRarityMinStatus",
      "addr": "0x7339d6b560",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticePartnerSearchFactorRarityMinStatus",
      "addr": "0x7339d6b5fc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticePartnerSearchFactorRarityMinProper",
      "addr": "0x7339d6b624",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticePartnerSearchFactorRarityMinProper",
      "addr": "0x7339d6b6c0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticePartnerSearchFactorRarityMinUnique",
      "addr": "0x7339d6b6f0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticePartnerSearchFactorRarityMinUnique",
      "addr": "0x7339d6b78c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticePartnerSearchNeedSuccessionFactorStatus",
      "addr": "0x7339d6b7bc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticePartnerSearchNeedSuccessionFactorStatus",
      "addr": "0x7339d6b83c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticePartnerSearchNeedSuccessionFactorProper",
      "addr": "0x7339d6b864",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticePartnerSearchNeedSuccessionFactorProper",
      "addr": "0x7339d6b8e4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PracticePartnerSearchNeedSuccessionFactorUnique",
      "addr": "0x7339d6b904",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PracticePartnerSearchNeedSuccessionFactorUnique",
      "addr": "0x7339d6b984",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchEntrySortMenu",
      "addr": "0x7339d6b9ac",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchEntrySortMenu",
      "addr": "0x7339d6ba48",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchEntrySortAsc",
      "addr": "0x7339d6ba70",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchEntrySortAsc",
      "addr": "0x7339d6baf0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchEntryFilterMenuArray",
      "addr": "0x7339d6bb18",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchEntryFilterMenuArray",
      "addr": "0x7339d6bb88",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ProfileCardSupportCardSortMenu",
      "addr": "0x7339d6bba4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileCardSupportCardSortMenu",
      "addr": "0x7339d6bc40",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileCardSupportCardSortMenuAsc",
      "addr": "0x7339d6bc68",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileCardSupportCardSortMenuAsc",
      "addr": "0x7339d6bce8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileCardSupportCardFilterMenuArray",
      "addr": "0x7339d6bd10",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileCardSupportCardFilterMenuArray",
      "addr": "0x7339d6bd80",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_DressChangeCharacterSortMenu",
      "addr": "0x7339d6bd9c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DressChangeCharacterSortMenu",
      "addr": "0x7339d6be38",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_DressChangeCharacterSortAsc",
      "addr": "0x7339d6be60",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DressChangeCharacterSortAsc",
      "addr": "0x7339d6bee0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_DressChangeCharacterFilterMenuArray",
      "addr": "0x7339d6bf08",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DressChangeCharacterFilterMenuArray",
      "addr": "0x7339d6bf78",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchEntrySortMenuHard3",
      "addr": "0x7339d6bf94",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchEntrySortMenuHard3",
      "addr": "0x7339d6c030",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchEntrySortAscHard3",
      "addr": "0x7339d6c060",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchEntrySortAscHard3",
      "addr": "0x7339d6c0e0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchEntryFilterMenuArrayHard3",
      "addr": "0x7339d6c108",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchEntryFilterMenuArrayHard3",
      "addr": "0x7339d6c178",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchEntrySortMenuVeryHard",
      "addr": "0x7339d6c194",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchEntrySortMenuVeryHard",
      "addr": "0x7339d6c230",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchEntrySortAscVeryHard",
      "addr": "0x7339d6c258",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchEntrySortAscVeryHard",
      "addr": "0x7339d6c2d8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchEntryFilterMenuArrayVeryHard",
      "addr": "0x7339d6c300",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchEntryFilterMenuArrayVeryHard",
      "addr": "0x7339d6c370",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchEntrySortMenuExtreme",
      "addr": "0x7339d6c38c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchEntrySortMenuExtreme",
      "addr": "0x7339d6c428",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchEntrySortAscExtreme",
      "addr": "0x7339d6c458",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchEntrySortAscExtreme",
      "addr": "0x7339d6c4d8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchEntryFilterMenuArrayExtreme",
      "addr": "0x7339d6c500",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchEntryFilterMenuArrayExtreme",
      "addr": "0x7339d6c570",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_RatingRaceShortEntrySortMenu",
      "addr": "0x7339d6c58c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceShortEntrySortMenu",
      "addr": "0x7339d6c628",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceShortEntrySortAsc",
      "addr": "0x7339d6c658",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceShortEntrySortAsc",
      "addr": "0x7339d6c6d8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceShortEntryFilterMenuArray",
      "addr": "0x7339d6c700",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceShortEntryFilterMenuArray",
      "addr": "0x7339d6c770",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_RatingRaceShortEntryTrainedCharaId",
      "addr": "0x7339d6c78c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceShortEntryTrainedCharaId",
      "addr": "0x7339d6c828",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceMileEntrySortMenu",
      "addr": "0x7339d6c850",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceMileEntrySortMenu",
      "addr": "0x7339d6c8ec",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceMileEntrySortAsc",
      "addr": "0x7339d6c91c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceMileEntrySortAsc",
      "addr": "0x7339d6c99c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceMileEntryFilterMenuArray",
      "addr": "0x7339d6c9bc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceMileEntryFilterMenuArray",
      "addr": "0x7339d6ca2c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_RatingRaceMileEntryTrainedCharaId",
      "addr": "0x7339d6ca48",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceMileEntryTrainedCharaId",
      "addr": "0x7339d6cae4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceMiddleEntrySortMenu",
      "addr": "0x7339d6cb0c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceMiddleEntrySortMenu",
      "addr": "0x7339d6cba8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceMiddleEntrySortAsc",
      "addr": "0x7339d6cbd8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceMiddleEntrySortAsc",
      "addr": "0x7339d6cc58",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceMiddleEntryFilterMenuArray",
      "addr": "0x7339d6cc78",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceMiddleEntryFilterMenuArray",
      "addr": "0x7339d6cce8",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_RatingRaceMiddleEntryTrainedCharaId",
      "addr": "0x7339d6cd04",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceMiddleEntryTrainedCharaId",
      "addr": "0x7339d6cda0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceLongEntrySortMenu",
      "addr": "0x7339d6cdc8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceLongEntrySortMenu",
      "addr": "0x7339d6ce64",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceLongEntrySortAsc",
      "addr": "0x7339d6ce94",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceLongEntrySortAsc",
      "addr": "0x7339d6cf14",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceLongEntryFilterMenuArray",
      "addr": "0x7339d6cf34",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceLongEntryFilterMenuArray",
      "addr": "0x7339d6cfa4",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_RatingRaceLongEntryTrainedCharaId",
      "addr": "0x7339d6cfc0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceLongEntryTrainedCharaId",
      "addr": "0x7339d6d05c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceDirtEntrySortMenu",
      "addr": "0x7339d6d084",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceDirtEntrySortMenu",
      "addr": "0x7339d6d120",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceDirtEntrySortAsc",
      "addr": "0x7339d6d150",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceDirtEntrySortAsc",
      "addr": "0x7339d6d1d0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceDirtEntryFilterMenuArray",
      "addr": "0x7339d6d1f0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceDirtEntryFilterMenuArray",
      "addr": "0x7339d6d260",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_RatingRaceDirtEntryTrainedCharaId",
      "addr": "0x7339d6d27c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceDirtEntryTrainedCharaId",
      "addr": "0x7339d6d318",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceLatestCategoryId",
      "addr": "0x7339d6d340",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceLatestCategoryId",
      "addr": "0x7339d6d3dc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RatingRaceLatestRatingRaceDataId",
      "addr": "0x7339d6d40c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RatingRaceLatestRatingRaceDataId",
      "addr": "0x7339d6d4a8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SupportCardRentalSingleModeStartSortMenu",
      "addr": "0x7339d6d4d8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardRentalSingleModeStartSortMenu",
      "addr": "0x7339d6d574",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SupportCardRentalSingleModeStartSortAsc",
      "addr": "0x7339d6d5a4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardRentalSingleModeStartSortAsc",
      "addr": "0x7339d6d624",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SupportCardRentalSingleModeStartFilterMenuArray",
      "addr": "0x7339d6d644",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardRentalSingleModeStartFilterMenuArray",
      "addr": "0x7339d6d6b4",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SupportCardRentalSupportDeckEditSortMenu",
      "addr": "0x7339d6d6d0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardRentalSupportDeckEditSortMenu",
      "addr": "0x7339d6d76c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SupportCardRentalSupportDeckEditSortAsc",
      "addr": "0x7339d6d79c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardRentalSupportDeckEditSortAsc",
      "addr": "0x7339d6d81c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SupportCardRentalSupportDeckEditFilterMenuArray",
      "addr": "0x7339d6d844",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportCardRentalSupportDeckEditFilterMenuArray",
      "addr": "0x7339d6d8b4",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_CampaignWalkingCharaSelectSortMenu",
      "addr": "0x7339d6d8d0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CampaignWalkingCharaSelectSortMenu",
      "addr": "0x7339d6d96c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CampaignWalkingCharaSelectSortAsc",
      "addr": "0x7339d6d99c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CampaignWalkingCharaSelectSortAsc",
      "addr": "0x7339d6da1c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CampaignWalkingCharaSelectFilterMenuArray",
      "addr": "0x7339d6da44",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CampaignWalkingCharaSelectFilterMenuArray",
      "addr": "0x7339d6dab4",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_TeamBuildingCaptainSelectSortMenu",
      "addr": "0x7339d6dad0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamBuildingCaptainSelectSortMenu",
      "addr": "0x7339d6db6c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TeamBuildingCaptainSelectSortAsc",
      "addr": "0x7339d6db94",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamBuildingCaptainSelectSortAsc",
      "addr": "0x7339d6dc14",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TeamBuildingCaptainSelectFilterMenuArray",
      "addr": "0x7339d6dc3c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamBuildingCaptainSelectFilterMenuArray",
      "addr": "0x7339d6dcac",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_TeamBuildingScoutSelectSortMenu",
      "addr": "0x7339d6dcc8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamBuildingScoutSelectSortMenu",
      "addr": "0x7339d6dd64",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TeamBuildingScoutSelectSortAsc",
      "addr": "0x7339d6dd94",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamBuildingScoutSelectSortAsc",
      "addr": "0x7339d6de14",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TeamBuildingScoutSelectFilterMenuArray",
      "addr": "0x7339d6de3c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamBuildingScoutSelectFilterMenuArray",
      "addr": "0x7339d6deac",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_TeamBuildingReplaceSelectSortMenu",
      "addr": "0x7339d6dec8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamBuildingReplaceSelectSortMenu",
      "addr": "0x7339d6df64",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TeamBuildingReplaceSelectSortAsc",
      "addr": "0x7339d6df8c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamBuildingReplaceSelectSortAsc",
      "addr": "0x7339d6e00c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TeamBuildingReplaceSelectFilterMenuArray",
      "addr": "0x7339d6e034",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamBuildingReplaceSelectFilterMenuArray",
      "addr": "0x7339d6e0a4",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_TeamBuildingIsEnableCaptainProperConfirm",
      "addr": "0x7339d6e0c0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamBuildingIsEnableCaptainProperConfirm",
      "addr": "0x7339d6e140",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TeamBuildingIsEnableScoutCancelConfirm",
      "addr": "0x7339d6e160",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamBuildingIsEnableScoutCancelConfirm",
      "addr": "0x7339d6e1e0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectSortMenu",
      "addr": "0x7339d6e208",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectSortMenu",
      "addr": "0x7339d6e2a4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectSortAsc",
      "addr": "0x7339d6e2d4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectSortAsc",
      "addr": "0x7339d6e354",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectFilterMenuArray",
      "addr": "0x7339d6e37c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectFilterMenuArray",
      "addr": "0x7339d6e3ec",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectFactorRarityMinStatus",
      "addr": "0x7339d6e408",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectFactorRarityMinStatus",
      "addr": "0x7339d6e4a4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectFactorRarityMinProper",
      "addr": "0x7339d6e4cc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectFactorRarityMinProper",
      "addr": "0x7339d6e568",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectFactorRarityMinUnique",
      "addr": "0x7339d6e598",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectFactorRarityMinUnique",
      "addr": "0x7339d6e634",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectNeedSuccessionFactorStatus",
      "addr": "0x7339d6e664",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectNeedSuccessionFactorStatus",
      "addr": "0x7339d6e6e4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectNeedSuccessionFactorProper",
      "addr": "0x7339d6e70c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectNeedSuccessionFactorProper",
      "addr": "0x7339d6e78c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectNeedSuccessionFactorUnique",
      "addr": "0x7339d6e7ac",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectNeedSuccessionFactorUnique",
      "addr": "0x7339d6e82c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectIsFullMatchFactorRarityStatus",
      "addr": "0x7339d6e854",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectIsFullMatchFactorRarityStatus",
      "addr": "0x7339d6e8d4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectIsFullMatchFactorRarityProper",
      "addr": "0x7339d6e8f4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectIsFullMatchFactorRarityProper",
      "addr": "0x7339d6e974",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectIsFullMatchFactorRarityUnique",
      "addr": "0x7339d6e99c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectIsFullMatchFactorRarityUnique",
      "addr": "0x7339d6ea1c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectFactorIdUnique",
      "addr": "0x7339d6ea3c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectFactorIdUnique",
      "addr": "0x7339d6ead8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectFactorIdCommonArray",
      "addr": "0x7339d6eb08",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectFactorIdCommonArray",
      "addr": "0x7339d6eb78",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectFactorRarityMinCommonArray",
      "addr": "0x7339d6eb94",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectFactorRarityMinCommonArray",
      "addr": "0x7339d6ec04",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectNeedSuccessionFactorCommonArray",
      "addr": "0x7339d6ec20",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectNeedSuccessionFactorCommonArray",
      "addr": "0x7339d6ec90",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_FactorResearchSelectIsFullMatchFactorRarityCommonArray",
      "addr": "0x7339d6ecac",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchSelectIsFullMatchFactorRarityCommonArray",
      "addr": "0x7339d6ed1c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_FactorResearchTrainedCharaViewFactor",
      "addr": "0x7339d6ed38",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchTrainedCharaViewFactor",
      "addr": "0x7339d6edb8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PlushieCatalogSortMenu",
      "addr": "0x7339d6edd8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PlushieCatalogSortMenu",
      "addr": "0x7339d6ee74",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PlushieCatalogSortAsc",
      "addr": "0x7339d6eea4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PlushieCatalogSortAsc",
      "addr": "0x7339d6ef24",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SelectStepupGachaCharaSortMenu",
      "addr": "0x7339d6ef44",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SelectStepupGachaCharaSortMenu",
      "addr": "0x7339d6efe0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SelectStepupGachaCharaSortAsc",
      "addr": "0x7339d6f010",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SelectStepupGachaCharaSortAsc",
      "addr": "0x7339d6f090",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SelectStepupGachaCharaFilterMenuArray",
      "addr": "0x7339d6f0b0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SelectStepupGachaCharaFilterMenuArray",
      "addr": "0x7339d6f120",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SelectStepupGachaSupportCardSortMenu",
      "addr": "0x7339d6f13c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SelectStepupGachaSupportCardSortMenu",
      "addr": "0x7339d6f1d8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SelectStepupGachaSupportCardSortAsc",
      "addr": "0x7339d6f200",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SelectStepupGachaSupportCardSortAsc",
      "addr": "0x7339d6f280",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SelectStepupGachaSupportCardFilterMenuArray",
      "addr": "0x7339d6f2a8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SelectStepupGachaSupportCardFilterMenuArray",
      "addr": "0x7339d6f318",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_RaceAnalyzeRaceEventListSortMenu",
      "addr": "0x7339d6f334",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RaceAnalyzeRaceEventListSortMenu",
      "addr": "0x7339d6f3d0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RaceAnalyzeRaceEventListSortAsc",
      "addr": "0x7339d6f3f8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RaceAnalyzeRaceEventListSortAsc",
      "addr": "0x7339d6f478",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RaceAnalyzeRaceEventListFilterMenuArray",
      "addr": "0x7339d6f4a0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RaceAnalyzeRaceEventListFilterMenuArray",
      "addr": "0x7339d6f510",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_RaceAnalyzeFilterIsIncludeRaceStatusChange",
      "addr": "0x7339d6f52c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RaceAnalyzeFilterIsIncludeRaceStatusChange",
      "addr": "0x7339d6f5ac",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RaceAnalyzeFilterSkillNameArray",
      "addr": "0x7339d6f5cc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RaceAnalyzeFilterSkillNameArray",
      "addr": "0x7339d6f63c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_PrevSupportSelectPickupGachaSettingConfirmOpenTime",
      "addr": "0x7339d6f658",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PrevSupportSelectPickupGachaSettingConfirmOpenTime",
      "addr": "0x7339d6f6f4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PrevCardStepupGachaSettingConfirmOpenTime",
      "addr": "0x7339d6f724",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PrevCardStepupGachaSettingConfirmOpenTime",
      "addr": "0x7339d6f7c0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PrevSupportStepupGachaSettingConfirmOpenTime",
      "addr": "0x7339d6f7ec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PrevSupportStepupGachaSettingConfirmOpenTime",
      "addr": "0x7339d6f888",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SupportSelectPickupGachaNotifyDialogGachaId",
      "addr": "0x7339d6f8b8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportSelectPickupGachaNotifyDialogGachaId",
      "addr": "0x7339d6f954",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_StampSheetGachaNotifyDialogGachaId",
      "addr": "0x7339d6f97c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_StampSheetGachaNotifyDialogGachaId",
      "addr": "0x7339d6fa18",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_StepupGachaNotifyDialogGachaIdArray",
      "addr": "0x7339d6fa48",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_StepupGachaNotifyDialogGachaIdArray",
      "addr": "0x7339d6fab8",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_JobsTopLastLimitedScheduleId",
      "addr": "0x7339d6fad4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_JobsTopLastLimitedScheduleId",
      "addr": "0x7339d6fb70",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_JobsTopLastTabIndex",
      "addr": "0x7339d6fba0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_JobsTopLastTabIndex",
      "addr": "0x7339d6fc3c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_JobsCharaSelectSortMenu",
      "addr": "0x7339d6fc6c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_JobsCharaSelectSortMenu",
      "addr": "0x7339d6fd08",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_JobsCharaSelectSortAsc",
      "addr": "0x7339d6fd38",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_JobsCharaSelectSortAsc",
      "addr": "0x7339d6fdb8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_JobsCharaSelectFilterMenuArray",
      "addr": "0x7339d6fdd8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_JobsCharaSelectFilterMenuArray",
      "addr": "0x7339d6fe48",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_JobsIsCheckAutoUseRecoveryItem",
      "addr": "0x7339d6fe64",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_JobsIsCheckAutoUseRecoveryItem",
      "addr": "0x7339d6fee4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_AddHomeStoryIdArray",
      "addr": "0x7339d6ff04",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_AddHomeStoryIdArray",
      "addr": "0x7339d6ff74",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_AddShortStoryIdArray",
      "addr": "0x7339d6ff90",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_AddShortStoryIdArray",
      "addr": "0x7339d70000",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_AddHomePosterIdArray",
      "addr": "0x7339d7001c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_AddHomePosterIdArray",
      "addr": "0x7339d7008c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_AddTipsIdArray",
      "addr": "0x7339d700a8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_AddTipsIdArray",
      "addr": "0x7339d70118",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_AddStoryReleaseIdArray",
      "addr": "0x7339d70134",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_AddStoryReleaseIdArray",
      "addr": "0x7339d701a4",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_AddHomeBannerIdArray",
      "addr": "0x7339d701c0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_AddHomeBannerIdArray",
      "addr": "0x7339d70230",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_StoryPlayHistoryJson",
      "addr": "0x7339d7024c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_StoryPlayHistoryJson",
      "addr": "0x7339d702bc",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_AddProductOpenNoticeIdArray",
      "addr": "0x7339d702d8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_AddProductOpenNoticeIdArray",
      "addr": "0x7339d70348",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_AddSeasonEventStoryExtraIdArray",
      "addr": "0x7339d70364",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_AddSeasonEventStoryExtraIdArray",
      "addr": "0x7339d703d4",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_AddSystemVoiceIdArray",
      "addr": "0x7339d703f0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_AddSystemVoiceIdArray",
      "addr": "0x7339d70460",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "SaveAddSystemVoiceId",
      "addr": "0x7339d43204",
      "params": 2,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "CreateNoteDataForRegistArray",
      "addr": "0x7339d7047c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "ClearAddVoice",
      "addr": "0x7339d70700",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "set_LastSingleModeTrainingCard",
      "addr": "0x7339d707e8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastSingleModeTrainingCard",
      "addr": "0x7339d70884",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LastSingleModeScenarioId",
      "addr": "0x7339d708ac",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastSingleModeScenarioId",
      "addr": "0x7339d70948",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PlayedSingleModeScenarioIdArray",
      "addr": "0x7339d70978",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PlayedSingleModeScenarioIdArray",
      "addr": "0x7339d709e8",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SingleModeNotifyNewScenarioIdArray",
      "addr": "0x7339d70a04",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeNotifyNewScenarioIdArray",
      "addr": "0x7339d70a74",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_NotifySingleModeUpdateIdArray",
      "addr": "0x7339d70a90",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_NotifySingleModeUpdateIdArray",
      "addr": "0x7339d70b00",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_NewSingleModeDifficultyId",
      "addr": "0x7339d70b1c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_NewSingleModeDifficultyId",
      "addr": "0x7339d70bb8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_NewSingleModeDifficultyIndex",
      "addr": "0x7339d70be8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_NewSingleModeDifficultyIndex",
      "addr": "0x7339d70c84",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LastSingleModeDifficultyId",
      "addr": "0x7339d70cac",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastSingleModeDifficultyId",
      "addr": "0x7339d70d48",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LastSingleModeDifficulty",
      "addr": "0x7339d70d78",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastSingleModeDifficulty",
      "addr": "0x7339d70e14",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_NewSingleModeDifficulty",
      "addr": "0x7339d70e44",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_NewSingleModeDifficulty",
      "addr": "0x7339d70ee0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SingleModeAoharuAutoBuildTeam",
      "addr": "0x7339d70f10",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SingleModeAoharuAutoBuildTeam",
      "addr": "0x7339d70f90",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ShowAoharuAutoBuildInfoDialog",
      "addr": "0x7339d70fb0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ShowAoharuAutoBuildInfoDialog",
      "addr": "0x7339d71030",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CheckedShopItemIdArray",
      "addr": "0x7339d71058",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CheckedShopItemIdArray",
      "addr": "0x7339d710c8",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "ClearCheckedShopItemIdArray",
      "addr": "0x7339d710e4",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "UpdatePlayedScenarioId",
      "addr": "0x7339d71140",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "set_IsEnableScenarioLinkHighlight",
      "addr": "0x7339d712e8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableScenarioLinkHighlight",
      "addr": "0x7339d7139c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_ScenarioListDialogSortAsc",
      "addr": "0x7339d7140c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ScenarioListDialogSortAsc",
      "addr": "0x7339d7148c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CircleChatReadMessageId",
      "addr": "0x7339d714ac",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleChatReadMessageId",
      "addr": "0x7339d71548",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CircleItemDonateNum",
      "addr": "0x7339d71578",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleItemDonateNum",
      "addr": "0x7339d71614",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TransferEventLastCheckTime",
      "addr": "0x7339d7163c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TransferEventLastCheckTime",
      "addr": "0x7339d716d8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TransferEventIsMultiSelectModeArray",
      "addr": "0x7339d71708",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TransferEventIsMultiSelectModeArray",
      "addr": "0x7339d71778",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_TransferRotationLastCheckTime",
      "addr": "0x7339d71794",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TransferRotationLastCheckTime",
      "addr": "0x7339d71830",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TransferRotationIsMultiSelectModeArray",
      "addr": "0x7339d71860",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TransferRotationIsMultiSelectModeArray",
      "addr": "0x7339d718d0",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_LiveTheaterLastPlayId",
      "addr": "0x7339d718ec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LiveTheaterLastPlayId",
      "addr": "0x7339d71988",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_MainStoryListPosY",
      "addr": "0x7339d719b8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_MainStoryListPosY",
      "addr": "0x7339d71a54",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CircleItemRequestListPosY",
      "addr": "0x7339d71a84",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleItemRequestListPosY",
      "addr": "0x7339d71b20",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CircleItemRequestToggleIndex",
      "addr": "0x7339d71b50",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleItemRequestToggleIndex",
      "addr": "0x7339d71bec",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LastCraneGameResult",
      "addr": "0x7339d71c1c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastCraneGameResult",
      "addr": "0x7339d71cb8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LastCraneGameSingleModeCharaId",
      "addr": "0x7339d71ce0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastCraneGameSingleModeCharaId",
      "addr": "0x7339d71d7c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LastCraneGameTurn",
      "addr": "0x7339d71dac",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastCraneGameTurn",
      "addr": "0x7339d71e48",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LastCraneGameGetPrize",
      "addr": "0x7339d71e78",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastCraneGameGetPrize",
      "addr": "0x7339d71ee8",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "ClearLastCraneGameValues",
      "addr": "0x7339d71f04",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SaveLastCrameGameValues",
      "addr": "0x7339d72030",
      "params": 4,
      "return_type": "void",
      "static": false
    },
    {
      "name": "set_LastPermanentCraneGamePlayCharaId",
      "addr": "0x7339d721b0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastPermanentCraneGamePlayCharaId",
      "addr": "0x7339d7224c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_DailyRaceLogGroupIdArray",
      "addr": "0x7339d7227c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyRaceLogGroupIdArray",
      "addr": "0x7339d722ec",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_DailyRaceLogTrainedCharaIdArray",
      "addr": "0x7339d72308",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyRaceLogTrainedCharaIdArray",
      "addr": "0x7339d72378",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_LegendRaceLogGroupId",
      "addr": "0x7339d72394",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LegendRaceLogGroupId",
      "addr": "0x7339d72430",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LegendRaceLogTrainedCharaId",
      "addr": "0x7339d72458",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LegendRaceLogTrainedCharaId",
      "addr": "0x7339d724f4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_DailyLegendEntryIsFirst",
      "addr": "0x7339d72524",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyLegendEntryIsFirst",
      "addr": "0x7339d725a4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_DailyLegendRaceLatestGroupId",
      "addr": "0x7339d725c4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyLegendRaceLatestGroupId",
      "addr": "0x7339d72660",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_DailyLegendRaceLogGroupIdArray",
      "addr": "0x7339d72690",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyLegendRaceLogGroupIdArray",
      "addr": "0x7339d72700",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_DailyLegendRaceLogTrainedCharaIdArray",
      "addr": "0x7339d7271c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DailyLegendRaceLogTrainedCharaIdArray",
      "addr": "0x7339d7278c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_IsTeamDeckUnlockedClass2Already",
      "addr": "0x7339d727a8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsTeamDeckUnlockedClass2Already",
      "addr": "0x7339d72828",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsTeamDeckUnlockedClass3Already",
      "addr": "0x7339d72848",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsTeamDeckUnlockedClass3Already",
      "addr": "0x7339d728c8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsEnableTeamStadiumAllSkipRace",
      "addr": "0x7339d728f0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableTeamStadiumAllSkipRace",
      "addr": "0x7339d72970",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TeamStadiumRaceHistoryJson",
      "addr": "0x7339d72990",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamStadiumRaceHistoryJson",
      "addr": "0x7339d72a00",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_ChampionsLobbyVoiceIndexArray",
      "addr": "0x7339d72a1c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChampionsLobbyVoiceIndexArray",
      "addr": "0x7339d72a8c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_IsChampionsLobbyFirstTime",
      "addr": "0x7339d72aa8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsChampionsLobbyFirstTime",
      "addr": "0x7339d72b28",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChampionsRewardInfoDefaultLeague",
      "addr": "0x7339d72b48",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChampionsRewardInfoDefaultLeague",
      "addr": "0x7339d72be4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChampionsRewardInfoDefaultId",
      "addr": "0x7339d72c14",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChampionsRewardInfoDefaultId",
      "addr": "0x7339d72cb0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChampionsGoalCaptureHash",
      "addr": "0x7339d72cd8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChampionsGoalCaptureHash",
      "addr": "0x7339d72d48",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_ChampionsGoalCaptureEventId",
      "addr": "0x7339d72d64",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChampionsGoalCaptureEventId",
      "addr": "0x7339d72e00",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChampionsGoalCaptureDressIdArray",
      "addr": "0x7339d72e28",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChampionsGoalCaptureDressIdArray",
      "addr": "0x7339d72e98",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchLogChallengeMatchId",
      "addr": "0x7339d72eb4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchLogChallengeMatchId",
      "addr": "0x7339d72f50",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchLogTrainedCharaId",
      "addr": "0x7339d72f78",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchLogTrainedCharaId",
      "addr": "0x7339d73014",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ChallengeMatchParticipationChallengeMatchId",
      "addr": "0x7339d73044",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ChallengeMatchParticipationChallengeMatchId",
      "addr": "0x7339d730e0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TeamBuildingParticipationEventId",
      "addr": "0x7339d73110",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamBuildingParticipationEventId",
      "addr": "0x7339d731ac",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TeamBuildingEndingAlreadyPlayed",
      "addr": "0x7339d731dc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TeamBuildingEndingAlreadyPlayed",
      "addr": "0x7339d7325c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_HeroesParticipationEventId",
      "addr": "0x7339d7327c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeroesParticipationEventId",
      "addr": "0x7339d73318",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_WebViewFontSavedPath",
      "addr": "0x7339d73348",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_WebViewFontSavedPath",
      "addr": "0x7339d733b8",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_PlatformAutoLoginFlag",
      "addr": "0x7339d733d4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PlatformAutoLoginFlag",
      "addr": "0x7339d73470",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CampaignTitleLogoChangeEndTime",
      "addr": "0x7339d734a0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CampaignTitleLogoChangeEndTime",
      "addr": "0x7339d7353c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CampaignTitleLogoChangeId",
      "addr": "0x7339d73568",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CampaignTitleLogoChangeId",
      "addr": "0x7339d73604",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "ResetCampaignTitleLogoData",
      "addr": "0x7339d73634",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "set_AdjustViewerIdOverWriteFlag",
      "addr": "0x7339d73714",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_AdjustViewerIdOverWriteFlag",
      "addr": "0x7339d73794",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_BusinessCardHashArray",
      "addr": "0x7339d737bc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_BusinessCardHashArray",
      "addr": "0x7339d7382c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_PhotoFavoriteArray",
      "addr": "0x7339d73848",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PhotoFavoriteArray",
      "addr": "0x7339d738b8",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ContactCardPhotoName",
      "addr": "0x7339d738d4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ContactCardPhotoName",
      "addr": "0x7339d7394c",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_CircleContactCardPhotoName",
      "addr": "0x7339d73968",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CircleContactCardPhotoName",
      "addr": "0x7339d739e0",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_PhotoLibralyFilter",
      "addr": "0x7339d739fc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PhotoLibralyFilter",
      "addr": "0x7339d73a98",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PhotoLibralySortAsc",
      "addr": "0x7339d73ac8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PhotoLibralySortAsc",
      "addr": "0x7339d73b48",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ContactCardPhotoFilter",
      "addr": "0x7339d73b70",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ContactCardPhotoFilter",
      "addr": "0x7339d73c0c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ContactCardPhotoSortAsc",
      "addr": "0x7339d73c3c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ContactCardPhotoSortAsc",
      "addr": "0x7339d73cbc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CharacterBgFilter",
      "addr": "0x7339d73ce4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CharacterBgFilter",
      "addr": "0x7339d73d80",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CharacterBgSortAsc",
      "addr": "0x7339d73db0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CharacterBgSortAsc",
      "addr": "0x7339d73e30",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsSelectedNumberCollaborationFilter",
      "addr": "0x7339d73e58",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsSelectedNumberCollaborationFilter",
      "addr": "0x7339d73ed8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_DisplayUIByPhotoCheck",
      "addr": "0x7339d73ef8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DisplayUIByPhotoCheck",
      "addr": "0x7339d73f78",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ValentineReceiveHistorySortMenu",
      "addr": "0x7339d73fa0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ValentineReceiveHistorySortMenu",
      "addr": "0x7339d7403c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ValentineReceiveHistorySortAsc",
      "addr": "0x7339d74068",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ValentineReceiveHistorySortAsc",
      "addr": "0x7339d740e8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ProfileTrainedCharaViewFactor",
      "addr": "0x7339d74108",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ProfileTrainedCharaViewFactor",
      "addr": "0x7339d74188",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LastRoomIdFromUrlScheme",
      "addr": "0x7339d741a8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastRoomIdFromUrlScheme",
      "addr": "0x7339d74244",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LastPartnerIdFromUrlScheme",
      "addr": "0x7339d74274",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastPartnerIdFromUrlScheme",
      "addr": "0x7339d74310",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LastCircleIdFromUrlScheme",
      "addr": "0x7339d74340",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastCircleIdFromUrlScheme",
      "addr": "0x7339d743dc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LatestCampaignWalkingDataId",
      "addr": "0x7339d74404",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LatestCampaignWalkingDataId",
      "addr": "0x7339d744a0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CampaignWalkingdLoginCutinLastCheckedTime",
      "addr": "0x7339d744d0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CampaignWalkingdLoginCutinLastCheckedTime",
      "addr": "0x7339d7456c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CampaignGalWalkingCurrentFriendCharaId",
      "addr": "0x7339d74598",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CampaignGalWalkingCurrentFriendCharaId",
      "addr": "0x7339d74634",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CampaignWalkingSuspendedCuttInfoJson",
      "addr": "0x7339d74664",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CampaignWalkingSuspendedCuttInfoJson",
      "addr": "0x7339d746d4",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_SaveCampaignRaffleLoginLastCheckedTime",
      "addr": "0x7339d746f0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SaveCampaignRaffleLoginLastCheckedTime",
      "addr": "0x7339d7478c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CampaignValentineNormalVoiceCheckReserveFlag",
      "addr": "0x7339d747bc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CampaignValentineNormalVoiceCheckReserveFlag",
      "addr": "0x7339d7483c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CampaignValentineSpecialVoiceCheckReserveFlag",
      "addr": "0x7339d7485c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CampaignValentineSpecialVoiceCheckReserveFlag",
      "addr": "0x7339d748dc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_StoryEventIsEnableContinuousRoulette",
      "addr": "0x7339d74904",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_StoryEventIsEnableContinuousRoulette",
      "addr": "0x7339d749b8",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_StoryEventContinuousRouletteSetting",
      "addr": "0x7339d74a28",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_StoryEventContinuousRouletteSetting",
      "addr": "0x7339d74b04",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ExtraStoryEventShioriEventSortMenu",
      "addr": "0x7339d74b9c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ExtraStoryEventShioriEventSortMenu",
      "addr": "0x7339d74c38",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ExtraStoryEventShioriEventSortAsc",
      "addr": "0x7339d74c68",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ExtraStoryEventShioriEventSortAsc",
      "addr": "0x7339d74ce8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ExtraStoryEventShioriEventFilterMenuArray",
      "addr": "0x7339d74d10",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ExtraStoryEventShioriEventFilterMenuArray",
      "addr": "0x7339d74d80",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_IsNewExtraStoryEventMovie",
      "addr": "0x7339d74d9c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsNewExtraStoryEventMovie",
      "addr": "0x7339d74e50",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_TrainingChallengeResultNoticeSuspend",
      "addr": "0x7339d74ec0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainingChallengeResultNoticeSuspend",
      "addr": "0x7339d74f9c",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_TrainingChallengeResultNoticeSuspendEventId",
      "addr": "0x7339d75034",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainingChallengeResultNoticeSuspendEventId",
      "addr": "0x7339d75110",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_TrainingChallengeSingleModeLastSelectedId",
      "addr": "0x7339d751a0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainingChallengeSingleModeLastSelectedId",
      "addr": "0x7339d7523c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrainingChallengeCardRankingEventId",
      "addr": "0x7339d7526c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainingChallengeCardRankingEventId",
      "addr": "0x7339d75308",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrainingChallengeCardRankingCharaSelectSortMenu",
      "addr": "0x7339d75338",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainingChallengeCardRankingCharaSelectSortMenu",
      "addr": "0x7339d753d4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrainingChallengeCardRankingCharaSelectSortAsc",
      "addr": "0x7339d75404",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainingChallengeCardRankingCharaSelectSortAsc",
      "addr": "0x7339d75484",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_TrainingChallengeCardRankingCharaSelectFilterMenuArray",
      "addr": "0x7339d754a4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainingChallengeCardRankingCharaSelectFilterMenuArray",
      "addr": "0x7339d75514",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_IsTrainingChallengeCardRankingEntryGuidanceRead",
      "addr": "0x7339d75530",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsTrainingChallengeCardRankingEntryGuidanceRead",
      "addr": "0x7339d755b0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_HideTrainingChallengeMismatchSingleModeCardConfirmEventId",
      "addr": "0x7339d755d8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HideTrainingChallengeMismatchSingleModeCardConfirmEventId",
      "addr": "0x7339d75674",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LastPhotoStudioSelectCard",
      "addr": "0x7339d756a4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastPhotoStudioSelectCard",
      "addr": "0x7339d75740",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsPhotoStudioPlayLoopCut",
      "addr": "0x7339d75768",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsPhotoStudioPlayLoopCut",
      "addr": "0x7339d757e8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PhotoStudioTopCardSelectSortMenu",
      "addr": "0x7339d75810",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PhotoStudioTopCardSelectSortMenu",
      "addr": "0x7339d758ac",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PhotoStudioTopCardSelectSortAsc",
      "addr": "0x7339d758d4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PhotoStudioTopCardSelectSortAsc",
      "addr": "0x7339d75954",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PhotoStudioTopCardSelectFilterMenuArray",
      "addr": "0x7339d7597c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PhotoStudioTopCardSelectFilterMenuArray",
      "addr": "0x7339d759ec",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_LatestMapEventIdThatOpenedUnlockAreaDialog",
      "addr": "0x7339d75a08",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LatestMapEventIdThatOpenedUnlockAreaDialog",
      "addr": "0x7339d75aa4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LatestMapEventAreaIdThatOpenedUnlockAreaDialog",
      "addr": "0x7339d75acc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LatestMapEventAreaIdThatOpenedUnlockAreaDialog",
      "addr": "0x7339d75b68",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LimitedSalesLastCheckOpenCount",
      "addr": "0x7339d75b98",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LimitedSalesLastCheckOpenCount",
      "addr": "0x7339d75c34",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ShopTicketExchangeCharaSortMenu",
      "addr": "0x7339d75c64",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ShopTicketExchangeCharaSortMenu",
      "addr": "0x7339d75d00",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ShopTicketExchangeCharaSortAsc",
      "addr": "0x7339d75d30",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ShopTicketExchangeCharaSortAsc",
      "addr": "0x7339d75db0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ShopTicketExchangeCharaFilterMenuArray",
      "addr": "0x7339d75dd0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ShopTicketExchangeCharaFilterMenuArray",
      "addr": "0x7339d75e40",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ShopTicketExchangeSupportCardSortMenu",
      "addr": "0x7339d75e5c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ShopTicketExchangeSupportCardSortMenu",
      "addr": "0x7339d75ef8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ShopTicketExchangeSupportCardSortAsc",
      "addr": "0x7339d75f28",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ShopTicketExchangeSupportCardSortAsc",
      "addr": "0x7339d75fa8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ShopTicketExchangeSupportCardFilterMenuArray",
      "addr": "0x7339d75fd0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ShopTicketExchangeSupportCardFilterMenuArray",
      "addr": "0x7339d76040",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ShopSupportCardTicketSortMenu",
      "addr": "0x7339d7605c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ShopSupportCardTicketSortMenu",
      "addr": "0x7339d760f8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ShopSupportCardTicketSortAsc",
      "addr": "0x7339d76120",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ShopSupportCardTicketSortAsc",
      "addr": "0x7339d761a0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ShopSupportCardTicketFilterMenuArray",
      "addr": "0x7339d761c8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ShopSupportCardTicketFilterMenuArray",
      "addr": "0x7339d76238",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ShopSupportCardLimitBreakSortMenu",
      "addr": "0x7339d76254",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ShopSupportCardLimitBreakSortMenu",
      "addr": "0x7339d762f0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ShopSupportCardLimitBreakSortAsc",
      "addr": "0x7339d76320",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ShopSupportCardLimitBreakSortAsc",
      "addr": "0x7339d763a0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ShopSupportCardLimitBreakFilterMenuArray",
      "addr": "0x7339d763c8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ShopSupportCardLimitBreakFilterMenuArray",
      "addr": "0x7339d76438",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_FactorUpdatePaidHomeStartNotifyExpirationLastCheckedTime",
      "addr": "0x7339d76454",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorUpdatePaidHomeStartNotifyExpirationLastCheckedTime",
      "addr": "0x7339d764f0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_FactorResearchNoticeLastCheckedTime",
      "addr": "0x7339d7651c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FactorResearchNoticeLastCheckedTime",
      "addr": "0x7339d765b8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GroupSelectGachaDialogAutoOpenHistory",
      "addr": "0x7339d765e8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GroupSelectGachaDialogAutoOpenHistory",
      "addr": "0x7339d76658",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_TrainingReportPeriodNotifyLastShowTime",
      "addr": "0x7339d76674",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_TrainingReportPeriodNotifyLastShowTime",
      "addr": "0x7339d76710",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_IsEnableRaceFitAssistancePopup",
      "addr": "0x7339d76740",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableRaceFitAssistancePopup",
      "addr": "0x7339d767f4",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsEnableRaceFitAssistanceCheck",
      "addr": "0x7339d76864",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsEnableRaceFitAssistanceCheck",
      "addr": "0x7339d76918",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_LastRaceSpRuleNoticeChampionsId",
      "addr": "0x7339d76990",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastRaceSpRuleNoticeChampionsId",
      "addr": "0x7339d76a6c",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_LastConfirmRaceSpRuleScheduleId",
      "addr": "0x7339d76b04",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastConfirmRaceSpRuleScheduleId",
      "addr": "0x7339d76be0",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_ConfirmedInRaceFitAssistanceByRaceSpRuleScheduleIdArray",
      "addr": "0x7339d76c78",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ConfirmedInRaceFitAssistanceByRaceSpRuleScheduleIdArray",
      "addr": "0x7339d76ce8",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_RaceSpRuleBanSkillGetConfirm",
      "addr": "0x7339d76d04",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RaceSpRuleBanSkillGetConfirm",
      "addr": "0x7339d76d84",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ConstWalkingViewedSortMenu",
      "addr": "0x7339d76da4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ConstWalkingViewedSortMenu",
      "addr": "0x7339d76e40",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ConstWalkingViewedSortAsc",
      "addr": "0x7339d76e70",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ConstWalkingViewedSortAsc",
      "addr": "0x7339d76ef0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ConstWalkingViewedFilterMenuArray",
      "addr": "0x7339d76f10",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ConstWalkingViewedFilterMenuArray",
      "addr": "0x7339d76f80",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ConstWalkingCharaSelectSortMenu",
      "addr": "0x7339d76f9c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ConstWalkingCharaSelectSortMenu",
      "addr": "0x7339d77038",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ConstWalkingCharaSelectSortAsc",
      "addr": "0x7339d77060",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ConstWalkingCharaSelectSortAsc",
      "addr": "0x7339d770e0",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ConstWalkingCharaSelectFilterMenuArray",
      "addr": "0x7339d77108",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ConstWalkingCharaSelectFilterMenuArray",
      "addr": "0x7339d77178",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ConstWalkingReleasedCuttListSortAsc",
      "addr": "0x7339d77194",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ConstWalkingReleasedCuttListSortAsc",
      "addr": "0x7339d77214",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ConstWalkingReleasedCuttListFilterMenuArray",
      "addr": "0x7339d77234",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ConstWalkingReleasedCuttListFilterMenuArray",
      "addr": "0x7339d772a4",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_ConstGalWalkingCurrentFriendCharaId",
      "addr": "0x7339d772c0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ConstGalWalkingCurrentFriendCharaId",
      "addr": "0x7339d7735c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_ConstWalkingSuspendedCuttInfoJson",
      "addr": "0x7339d77384",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ConstWalkingSuspendedCuttInfoJson",
      "addr": "0x7339d773f4",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_IsAutoConstWalking",
      "addr": "0x7339d77410",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsAutoConstWalking",
      "addr": "0x7339d77490",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SuccessionOnlyCharaSortMenu",
      "addr": "0x7339d774b0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SuccessionOnlyCharaSortMenu",
      "addr": "0x7339d7754c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SuccessionOnlyCharaSortAsc",
      "addr": "0x7339d7757c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SuccessionOnlyCharaSortAsc",
      "addr": "0x7339d775fc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_SuccessionOnlyCharaFilterMenuArray",
      "addr": "0x7339d7761c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SuccessionOnlyCharaFilterMenuArray",
      "addr": "0x7339d7768c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "get_SuccessionOnlyCharaAdvancedFilterSettingHandler",
      "addr": "0x7339d776a8",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set_GeneratedSuccessionOnlyCharaSortMenu",
      "addr": "0x7339d777b8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GeneratedSuccessionOnlyCharaSortMenu",
      "addr": "0x7339d77854",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GeneratedSuccessionOnlyCharaSortAsc",
      "addr": "0x7339d7787c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GeneratedSuccessionOnlyCharaSortAsc",
      "addr": "0x7339d778fc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GeneratedSuccessionOnlyCharaFilterMenuArray",
      "addr": "0x7339d77924",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GeneratedSuccessionOnlyCharaFilterMenuArray",
      "addr": "0x7339d77994",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "get_GeneratedSuccessionOnlyCharaAdvancedFilterSettingHandler",
      "addr": "0x7339d779b0",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartCardSelectSortMenu",
      "addr": "0x7339d77ac0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartCardSelectSortMenu",
      "addr": "0x7339d77b5c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartCardSelectSortAsc",
      "addr": "0x7339d77b84",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartCardSelectSortAsc",
      "addr": "0x7339d77c04",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartCardSelectFilterMenuArray",
      "addr": "0x7339d77c2c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartCardSelectFilterMenuArray",
      "addr": "0x7339d77c9c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartCardSelectAdvancedFilterSettingHandler",
      "addr": "0x7339d77cb8",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartSuccessionCharaSelectSortMenu",
      "addr": "0x7339d77dc8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionCharaSelectSortMenu",
      "addr": "0x7339d77e64",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartSuccessionCharaSelectSortAsc",
      "addr": "0x7339d77e8c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionCharaSelectSortAsc",
      "addr": "0x7339d77f0c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartSuccessionCharaSelectFilterMenuArray",
      "addr": "0x7339d77f34",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionCharaSelectFilterMenuArray",
      "addr": "0x7339d77fa4",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionCharaSelectAdvancedFilterSettingHandler",
      "addr": "0x7339d77fc0",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartSuccessionCharaRentalSelectSortMenu",
      "addr": "0x7339d780d0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionCharaRentalSelectSortMenu",
      "addr": "0x7339d7816c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartSuccessionCharaRentalSelectSortAsc",
      "addr": "0x7339d78194",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionCharaRentalSelectSortAsc",
      "addr": "0x7339d78214",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartSuccessionCharaRentalSelectFilterMenuArray",
      "addr": "0x7339d7823c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionCharaRentalSelectFilterMenuArray",
      "addr": "0x7339d782ac",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionCharaRentalSelectAdvancedFilterSettingHandler",
      "addr": "0x7339d782c8",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartSuccessionDeckCharaSelectSortMenu",
      "addr": "0x7339d783d8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionDeckCharaSelectSortMenu",
      "addr": "0x7339d78474",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartSuccessionDeckCharaSelectSortAsc",
      "addr": "0x7339d7849c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionDeckCharaSelectSortAsc",
      "addr": "0x7339d7851c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartSuccessionDeckCharaSelectFilterMenuArray",
      "addr": "0x7339d78544",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionDeckCharaSelectFilterMenuArray",
      "addr": "0x7339d785b4",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionDeckCharaSelectAdvancedFilterSettingHandler",
      "addr": "0x7339d785d0",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartSuccessionDeckCharaRentalSelectSortMenu",
      "addr": "0x7339d786e0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionDeckCharaRentalSelectSortMenu",
      "addr": "0x7339d7877c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartSuccessionDeckCharaRentalSelectSortAsc",
      "addr": "0x7339d787a4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionDeckCharaRentalSelectSortAsc",
      "addr": "0x7339d78824",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionStartSuccessionDeckCharaRentalSelectFilterMenuArray",
      "addr": "0x7339d7884c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionDeckCharaRentalSelectFilterMenuArray",
      "addr": "0x7339d788bc",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionStartSuccessionDeckCharaRentalSelectAdvancedFilterSettingHandler",
      "addr": "0x7339d788d8",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionSupportSelectSortMenu",
      "addr": "0x7339d789e8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionSupportSelectSortMenu",
      "addr": "0x7339d78a84",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionSupportSelectSortAsc",
      "addr": "0x7339d78aac",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionSupportSelectSortAsc",
      "addr": "0x7339d78b2c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionSupportSelectFilterMenuArray",
      "addr": "0x7339d78b54",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionSupportSelectFilterMenuArray",
      "addr": "0x7339d78bc4",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionSupportSelectAdvancedFilterSettingHandler",
      "addr": "0x7339d78be0",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionFriendSupportSelectSortMenu",
      "addr": "0x7339d78cf0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionFriendSupportSelectSortMenu",
      "addr": "0x7339d78d8c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionFriendSupportSelectSortAsc",
      "addr": "0x7339d78db4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionFriendSupportSelectSortAsc",
      "addr": "0x7339d78e34",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_GenerateSuccessionFriendSupportSelectFilterMenuArray",
      "addr": "0x7339d78e5c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionFriendSupportSelectFilterMenuArray",
      "addr": "0x7339d78ecc",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionFriendSupportSelectAdvancedFilterSettingHandler",
      "addr": "0x7339d78ee8",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "<get_SingleModeSuccessionDeckHistoryHandler>b__949_0",
      "addr": "0x7339d78ff8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<get_SingleModeSuccessionDeckHistoryHandler>b__949_1",
      "addr": "0x7339d79068",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<get_SingleModeSuccessionDeckCharaAdvancedFilterSettingHandler>b__1039_0",
      "addr": "0x7339d79084",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<get_SingleModeSuccessionDeckCharaAdvancedFilterSettingHandler>b__1039_1",
      "addr": "0x7339d790f4",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<get_SingleModeSuccessionDeckCharaRentalAdvancedFilterSettingHandler>b__1051_0",
      "addr": "0x7339d79110",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<get_SingleModeSuccessionDeckCharaRentalAdvancedFilterSettingHandler>b__1051_1",
      "addr": "0x7339d79180",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<get_SingleModeSuccessionDeckCharaEventRentalAdvancedFilterSettingHandler>b__1063_0",
      "addr": "0x7339d7919c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<get_SingleModeSuccessionDeckCharaEventRentalAdvancedFilterSettingHandler>b__1063_1",
      "addr": "0x7339d7920c",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<get_SuccessionOnlyCharaAdvancedFilterSettingHandler>b__2029_0",
      "addr": "0x7339d79228",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<get_SuccessionOnlyCharaAdvancedFilterSettingHandler>b__2029_1",
      "addr": "0x7339d79298",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<get_GeneratedSuccessionOnlyCharaAdvancedFilterSettingHandler>b__2041_0",
      "addr": "0x7339d792b4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<get_GeneratedSuccessionOnlyCharaAdvancedFilterSettingHandler>b__2041_1",
      "addr": "0x7339d79324",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<get_GenerateSuccessionStartCardSelectAdvancedFilterSettingHandler>b__2053_0",
      "addr": "0x7339d79340",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<get_GenerateSuccessionStartCardSelectAdvancedFilterSettingHandler>b__2053_1",
      "addr": "0x7339d793b0",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<get_GenerateSuccessionStartSuccessionCharaSelectAdvancedFilterSettingHandler>b__2065_0",
      "addr": "0x7339d793cc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<get_GenerateSuccessionStartSuccessionCharaSelectAdvancedFilterSettingHandler>b__2065_1",
      "addr": "0x7339d7943c",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<get_GenerateSuccessionStartSuccessionCharaRentalSelectAdvancedFilterSettingHandler>b__2077_0",
      "addr": "0x7339d79458",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<get_GenerateSuccessionStartSuccessionCharaRentalSelectAdvancedFilterSettingHandler>b__2077_1",
      "addr": "0x7339d794c8",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<get_GenerateSuccessionStartSuccessionDeckCharaSelectAdvancedFilterSettingHandler>b__2089_0",
      "addr": "0x7339d794e4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<get_GenerateSuccessionStartSuccessionDeckCharaSelectAdvancedFilterSettingHandler>b__2089_1",
      "addr": "0x7339d79554",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<get_GenerateSuccessionStartSuccessionDeckCharaRentalSelectAdvancedFilterSettingHandler>b__2101_0",
      "addr": "0x7339d79570",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<get_GenerateSuccessionStartSuccessionDeckCharaRentalSelectAdvancedFilterSettingHandler>b__2101_1",
      "addr": "0x7339d795e0",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<get_GenerateSuccessionSupportSelectAdvancedFilterSettingHandler>b__2113_0",
      "addr": "0x7339d795fc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<get_GenerateSuccessionSupportSelectAdvancedFilterSettingHandler>b__2113_1",
      "addr": "0x7339d7966c",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<get_GenerateSuccessionFriendSupportSelectAdvancedFilterSettingHandler>b__2125_0",
      "addr": "0x7339d79688",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<get_GenerateSuccessionFriendSupportSelectAdvancedFilterSettingHandler>b__2125_1",
      "addr": "0x7339d796f8",
      "params": 0,
      "return_type": "byref",
      "static": false
    }
  ]
}
```

## Gallop.ObscuredSuccessionDeckLastUsed

```json
{
  "class": "ObscuredSuccessionDeckLastUsed",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 8,
  "methods": [
    {
      "name": "get_CardId",
      "addr": "0x7339daa438",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_CardId",
      "addr": "0x7339daa44c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_LastStartTime",
      "addr": "0x7339daa460",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_LastStartTime",
      "addr": "0x7339daa478",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SuccessionTrainedChara1",
      "addr": "0x7339daa490",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_SuccessionTrainedChara1",
      "addr": "0x7339daa498",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SuccessionTrainedChara2",
      "addr": "0x7339daa4a0",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_SuccessionTrainedChara2",
      "addr": "0x7339daa4a8",
      "params": 1,
      "return_type": "void",
      "static": false
    }
  ]
}
```

## Gallop.ObscuredSuccessionDeckLastUsedExtensions

```json
{
  "class": "ObscuredSuccessionDeckLastUsedExtensions",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 3,
  "methods": [
    {
      "name": "AsObscured",
      "addr": "0x7339daa5c4",
      "params": 1,
      "return_type": "byref",
      "static": true
    },
    {
      "name": "AsObscuredArray",
      "addr": "0x7339daa628",
      "params": 1,
      "return_type": "fnptr",
      "static": true
    },
    {
      "name": "AsObscuredArrayOrEmpty",
      "addr": "0x7339daa718",
      "params": 1,
      "return_type": "fnptr",
      "static": true
    }
  ]
}
```

## Gallop.WorkSuccessionDeckData

```json
{
  "class": "WorkSuccessionDeckData",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 13,
  "methods": [
    {
      "name": "get_UsingTrainedCharaIdListAtLogin",
      "addr": "0x7339ea5f00",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "get_UsingGenerateSuccessionCharaIdListAtLogin",
      "addr": "0x7339ea5f08",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "get_UsingGenerateSuccessionExecCharaIdListAtLogin",
      "addr": "0x7339ea5f10",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "get_SuccessionDeckList",
      "addr": "0x7339ea5f18",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "get_GenerateSuccessionDeckList",
      "addr": "0x7339ea5f20",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "get_SuccessionDeckLastUsedExistCardIdList",
      "addr": "0x7339ea5f28",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "ApplyUsingTrainedCharaIdArrayAtLogin",
      "addr": "0x7339ea5f30",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "ApplyUsingGenerateSuccessionCharaIdArrayAtLogin",
      "addr": "0x7339ea5f50",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "ApplyUsingGenerateSuccessionExecCharaIdArrayAtLogin",
      "addr": "0x7339ea5f70",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Apply",
      "addr": "0x7339ea5f90",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "ApplySuccessionDeckLastUsedExistCardId",
      "addr": "0x7339ea60cc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "ApplySuccessionDeckLastUsed",
      "addr": "0x7339ea60ec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "GetSuccessionDeckLastUsedByCardId",
      "addr": "0x7339ea6404",
      "params": 1,
      "return_type": "byref",
      "static": false
    }
  ]
}
```

## .<>c__DisplayClass19_0

```json
{
  "class": "<>c__DisplayClass19_0",
  "ns": "",
  "is_enum": false,
  "method_count": 1,
  "methods": [
    {
      "name": "<TryGetTrainedCharaData>b__0",
      "addr": "0x733a1ca31c",
      "params": 1,
      "return_type": "void",
      "static": false
    }
  ]
}
```

## Gallop.DialogRaceFitAssistanceFriendSelect

```json
{
  "class": "DialogRaceFitAssistanceFriendSelect",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 12,
  "methods": [
    {
      "name": "GetFormType",
      "addr": "0x733a1c90e4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "GetParentType",
      "addr": "0x733a1c90ec",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "get_CanFollow",
      "addr": "0x733a1c90f4",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "Open",
      "addr": "0x733a1c9240",
      "params": 1,
      "return_type": "void",
      "static": true
    },
    {
      "name": "Setup",
      "addr": "0x733a1c9308",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetupReloadButton",
      "addr": "0x733a1c9324",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetupFriendCardList",
      "addr": "0x733a1c9460",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "CreateList",
      "addr": "0x733a1c9464",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "UpdateFriendListItem",
      "addr": "0x733a1c9770",
      "params": 4,
      "return_type": "void",
      "static": false
    },
    {
      "name": "TryGetTrainedCharaData",
      "addr": "0x733a1c98bc",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnClickReloadButton",
      "addr": "0x733a1c99e4",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "RegisterDownload",
      "addr": "0x733a1c9aec",
      "params": 1,
      "return_type": "void",
      "static": true
    }
  ]
}
```

## .<>c

```json
{
  "class": "<>c",
  "ns": "",
  "is_enum": false,
  "method_count": 15,
  "methods": [
    {
      "name": "<SetSuccessionDeckUserInfoArray>b__53_0",
      "addr": "0x73386e9b00",
      "params": 1,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "<SetSuccessionDeckLastUsedUserInfoArray>b__54_0",
      "addr": "0x73386e9b0c",
      "params": 1,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "<SetupRaceInfo>b__80_2",
      "addr": "0x73386e9b18",
      "params": 1,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "<GetScenarioRouteRaceList>b__90_0",
      "addr": "0x73386e9b34",
      "params": 1,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "<SetRestrictTurnList>b__94_0",
      "addr": "0x73386e9b54",
      "params": 1,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "<SetPresetRace>b__99_1",
      "addr": "0x73386e9b6c",
      "params": 1,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "<GetRaceInfo>b__101_0",
      "addr": "0x73386e9b90",
      "params": 1,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "<GetMaxConsecutiveCount>b__110_1",
      "addr": "0x73386e9ba8",
      "params": 1,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "<GetFactorGroupMasterList>b__125_0",
      "addr": "0x73386e9bb0",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<GetFactorGroupMasterList>b__125_1",
      "addr": "0x73386e9d30",
      "params": 1,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "<GetNewestScenarioId>b__129_0",
      "addr": "0x73386e9d48",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<GetNewestScenarioId>b__129_2",
      "addr": "0x73386e9e30",
      "params": 1,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "<GetGenerateSuccessionStartCharaBase>b__133_0",
      "addr": "0x73386e9e48",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<GetGenerateSuccessionStartCharaBase>b__133_2",
      "addr": "0x73386e9ec4",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<CanExec>b__135_0",
      "addr": "0x73386e9f40",
      "params": 1,
      "return_type": "boolean",
      "static": false
    }
  ]
}
```

## .<>c__DisplayClass61_0

```json
{
  "class": "<>c__DisplayClass61_0",
  "ns": "",
  "is_enum": false,
  "method_count": 1,
  "methods": [
    {
      "name": "<Gallop.ISinglemodeStartSuccessionInfoAccessor.FindRentalSuccessionTrainedCharaByCondition>b__0",
      "addr": "0x73386ea6fc",
      "params": 1,
      "return_type": "boolean",
      "static": false
    }
  ]
}
```

## .GenerateInfo

```json
{
  "class": "GenerateInfo",
  "ns": "",
  "is_enum": false,
  "method_count": 80,
  "methods": [
    {
      "name": "get_PresetId",
      "addr": "0x73386e61b8",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_PresetId",
      "addr": "0x73386e61c0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PresetName",
      "addr": "0x73386e61c8",
      "params": 0,
      "return_type": "string",
      "static": false
    },
    {
      "name": "set_PresetName",
      "addr": "0x73386e61d0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ScenarioId",
      "addr": "0x73386e61d8",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_ScenarioId",
      "addr": "0x73386e37fc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Gallop.ISingleModeStartScenarioAccessor.get_ScenarioId",
      "addr": "0x73386e61e0",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "get_CardId",
      "addr": "0x73386e61e8",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_CardId",
      "addr": "0x73386e61f0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Gallop.ISingleModeStartCardAccessor.get_CardId",
      "addr": "0x73386e620c",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "GetCardDressIdSet",
      "addr": "0x73386e6214",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "get_UseEventRental",
      "addr": "0x73386e6300",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set_UseEventRental",
      "addr": "0x73386e6308",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HasUseEventRental",
      "addr": "0x73386e6310",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "Gallop.ISingleModeSuccessionDeckEntityAccessor.get_DeckId",
      "addr": "0x73386e6324",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "Gallop.ISingleModeSuccessionDeckEntityAccessor.get_DeckName",
      "addr": "0x73386e632c",
      "params": 0,
      "return_type": "string",
      "static": false
    },
    {
      "name": "Gallop.ISingleModeSuccessionDeckEntityAccessor.get_CharaFirst",
      "addr": "0x73386e6338",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "Gallop.ISingleModeSuccessionDeckEntityAccessor.get_CharaSecond",
      "addr": "0x73386e63a8",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "get_EnableEventType",
      "addr": "0x73386e6414",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_EnableEventType",
      "addr": "0x73386e641c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Gallop.ISingleModeStartEventAccessor.get_EnableEventType",
      "addr": "0x73386e6424",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "get_RentalTrainedCharaArray",
      "addr": "0x73386e642c",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "get_EventRentalTrainedCharaArray",
      "addr": "0x73386e6434",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "SetRentalTrainedCharaArray",
      "addr": "0x73386e643c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetRentalUserInfoArray",
      "addr": "0x73386e64e8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetSuccessionDeckUserInfoArray",
      "addr": "0x73386e650c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetSuccessionDeckLastUsedUserInfoArray",
      "addr": "0x73386e664c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Gallop.ISinglemodeStartSuccessionInfoAccessor.get_RemainRentalCount",
      "addr": "0x73386e678c",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "Gallop.ISinglemodeStartSuccessionInfoAccessor.get_RentalTrainedCharaArray",
      "addr": "0x73386e6798",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "Gallop.ISinglemodeStartSuccessionInfoAccessor.get_EventRentalTrainedCharaArray",
      "addr": "0x73386e67a0",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "Gallop.ISinglemodeStartSuccessionInfoAccessor.FindRentalSuccessionTrainedCharaByCondition",
      "addr": "0x73386e67a8",
      "params": 3,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "Gallop.ISinglemodeStartSuccessionInfoAccessor.GetUserInfoForRentalSuccessionTrainedChara",
      "addr": "0x73386e68a8",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "GetRentalTrainedChara",
      "addr": "0x73386e68c4",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "get_SupportCardDeckId",
      "addr": "0x73386e6a38",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_SupportCardDeckId",
      "addr": "0x73386e6a40",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SupportSerialIdArray",
      "addr": "0x73386e6a48",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SupportSerialIdArray",
      "addr": "0x73386e6a50",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendSupportCardInfoModel",
      "addr": "0x73386e6a58",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_FriendSupportCardInfoModel",
      "addr": "0x73386e6a60",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendSupportCardPosition",
      "addr": "0x73386e6a68",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_FriendSupportCardPosition",
      "addr": "0x73386e6a70",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetupRaceInfo",
      "addr": "0x73386dc918",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SelectRouteRaceProgramId",
      "addr": "0x73386e7574",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_SelectRouteRaceProgramId",
      "addr": "0x73386e757c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RouteRaceList",
      "addr": "0x73386e7584",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "SetupTargetRace",
      "addr": "0x73386e6aa4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "GetScenarioRouteRaceList",
      "addr": "0x73386e758c",
      "params": 3,
      "return_type": "valuetype",
      "static": true
    },
    {
      "name": "get_RestrictTurnList",
      "addr": "0x73386e7b90",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "SetRestrictTurnList",
      "addr": "0x73386e6f4c",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ReservedProgramList",
      "addr": "0x73386e7b98",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "EditReserveRace",
      "addr": "0x73386ddbb0",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetPresetRace",
      "addr": "0x73386dcd08",
      "params": 1,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "RemoveUnavailableReservedRace",
      "addr": "0x73386e71c8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "GetRaceInfo",
      "addr": "0x73386e78e8",
      "params": 3,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "GetAreaType",
      "addr": "0x73386e7bc0",
      "params": 1,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "NotifyReservedRaceIfChanged",
      "addr": "0x73386dcf98",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "CheckReservedRaceChanged",
      "addr": "0x73386e738c",
      "params": 1,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "get_IsAlreadyShowRaceWarningDialog",
      "addr": "0x73386e7cbc",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsAlreadyShowRaceWarningDialog",
      "addr": "0x73386e7cc4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "GetMaxConsecutiveCount",
      "addr": "0x73386ddf14",
      "params": 2,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "GetProperInfoWithBonus",
      "addr": "0x73386dc608",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "get_Context",
      "addr": "0x73386dc5e4",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "SetContext",
      "addr": "0x73386e7cd8",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_PriorityFactorGroupList",
      "addr": "0x73386e7dec",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "get_IsMaxPriorityFactorGroupSet",
      "addr": "0x73386da8e0",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "EditPriorityFactorGroupList",
      "addr": "0x73386d8b98",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "AddPriorityFactorGroup",
      "addr": "0x73386e7df4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "RefreshPriorityFactorGroupList",
      "addr": "0x73386dd538",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "GetFactorGroupMasterList",
      "addr": "0x73386d8a68",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "ToPresetEntity",
      "addr": "0x73386e8de0",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "GetNewestScenarioId",
      "addr": "0x73386e7fdc",
      "params": 0,
      "return_type": "i4",
      "static": true
    },
    {
      "name": "FindTurn",
      "addr": "0x73386e8e48",
      "params": 4,
      "return_type": "i4",
      "static": true
    },
    {
      "name": "GetGenerateSuccessionStartChara",
      "addr": "0x73386dd158",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "GetGenerateSuccessionStartCharaOnSavePreset",
      "addr": "0x73386daff8",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "GetGenerateSuccessionStartCharaBase",
      "addr": "0x73386e8e6c",
      "params": 2,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "IsEmptyScenarioOrCard",
      "addr": "0x73386e6a80",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "CanExec",
      "addr": "0x73386e96ec",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "GetMergedSupportCardIdArray",
      "addr": "0x73386e968c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "<.ctor>b__127_2",
      "addr": "0x73386e9850",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<GetGenerateSuccessionStartCharaBase>b__133_1",
      "addr": "0x73386e99c0",
      "params": 1,
      "return_type": "boolean",
      "static": false
    }
  ]
}
```

## .<>c

```json
{
  "class": "<>c",
  "ns": "",
  "is_enum": false,
  "method_count": 3,
  "methods": [
    {
      "name": "<SetSuccessionDeckUserInfoArray>b__108_0",
      "addr": "0x7339457d9c",
      "params": 1,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "<SetSuccessionDeckLastUsedUserInfoArray>b__109_0",
      "addr": "0x7339457da8",
      "params": 1,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "<IsUseRentalSupportCard>b__126_0",
      "addr": "0x7339457db4",
      "params": 1,
      "return_type": "boolean",
      "static": false
    }
  ]
}
```

## .<>c__DisplayClass116_0

```json
{
  "class": "<>c__DisplayClass116_0",
  "ns": "",
  "is_enum": false,
  "method_count": 1,
  "methods": [
    {
      "name": "<Gallop.ISinglemodeStartSuccessionInfoAccessor.FindRentalSuccessionTrainedCharaByCondition>b__0",
      "addr": "0x7339457dc0",
      "params": 1,
      "return_type": "boolean",
      "static": false
    }
  ]
}
```

## .EntryInfo

```json
{
  "class": "EntryInfo",
  "ns": "",
  "is_enum": false,
  "method_count": 71,
  "methods": [
    {
      "name": "get_ScenarioId",
      "addr": "0x7339455f10",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_ScenarioId",
      "addr": "0x7339455f18",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Gallop.ISingleModeStartScenarioAccessor.get_ScenarioId",
      "addr": "0x7339455f20",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "get_CardId",
      "addr": "0x7339455f28",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_CardId",
      "addr": "0x7339455f30",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Gallop.ISingleModeStartCardAccessor.get_CardId",
      "addr": "0x7339455f38",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "get_SupportSerialIdArray",
      "addr": "0x7339455f40",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_SupportSerialIdArray",
      "addr": "0x7339455f48",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RentalSupportCardIdArray",
      "addr": "0x7339455f50",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "set_RentalSupportCardIdArray",
      "addr": "0x7339455f58",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendSupportCardInfoModel",
      "addr": "0x7339455f60",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_FriendSupportCardInfoModel",
      "addr": "0x7339455f68",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_FriendSupportCardPosition",
      "addr": "0x7339455f70",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_FriendSupportCardPosition",
      "addr": "0x7339455f78",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RentalDeckId",
      "addr": "0x7339455f80",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_RentalDeckId",
      "addr": "0x7339455f88",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_UseEventRental",
      "addr": "0x7339455f90",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set_UseEventRental",
      "addr": "0x7339455f98",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HasUseEventRental",
      "addr": "0x7339455fa0",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "Gallop.ISingleModeSuccessionDeckEntityAccessor.get_DeckId",
      "addr": "0x7339455fb4",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "Gallop.ISingleModeSuccessionDeckEntityAccessor.get_DeckName",
      "addr": "0x7339455fbc",
      "params": 0,
      "return_type": "string",
      "static": false
    },
    {
      "name": "Gallop.ISingleModeSuccessionDeckEntityAccessor.get_CharaFirst",
      "addr": "0x7339455fc8",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "Gallop.ISingleModeSuccessionDeckEntityAccessor.get_CharaSecond",
      "addr": "0x7339456038",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "get_UseTp",
      "addr": "0x73394560a4",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_UseTp",
      "addr": "0x73394560ac",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_UseTpWhenBoostMode",
      "addr": "0x73394560b4",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_UseTpWhenBoostMode",
      "addr": "0x73394560bc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_EnableTpBoost",
      "addr": "0x73394560c4",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_EnableTpBoost",
      "addr": "0x73394560cc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_DifficultyId",
      "addr": "0x73394560d8",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_DifficultyId",
      "addr": "0x73394560e0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_Difficulty",
      "addr": "0x73394560e8",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_Difficulty",
      "addr": "0x73394560f0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_EnableEventType",
      "addr": "0x73394560f8",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_EnableEventType",
      "addr": "0x7339456100",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Gallop.ISingleModeStartEventAccessor.get_EnableEventType",
      "addr": "0x7339456108",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "ContainsApplyCampaignId",
      "addr": "0x7339456110",
      "params": 1,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "AddApplyCampaignId",
      "addr": "0x7339456168",
      "params": 1,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "RemoveApplyCampaignId",
      "addr": "0x73394561c0",
      "params": 1,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "get_IsIndexRequested",
      "addr": "0x7339456218",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsIndexRequested",
      "addr": "0x7339456220",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_IsIndexPartsSuccessions",
      "addr": "0x733945622c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsIndexPartsSuccessions",
      "addr": "0x7339456234",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_SuccessionDeckLastUsedFetchedCardIdHashSet",
      "addr": "0x7339456240",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "get_RunningStyle",
      "addr": "0x7339456248",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_RunningStyle",
      "addr": "0x7339456250",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_RouteRaceProgramId",
      "addr": "0x7339456258",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "set_RouteRaceProgramId",
      "addr": "0x7339456260",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "MarkAsIndexRequested",
      "addr": "0x7339456614",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "GetSuccessionTrainedCharaData",
      "addr": "0x7339456620",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "SetRentalTrainedCharaArray",
      "addr": "0x7339456638",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetEventRentalTrainedCharaArray",
      "addr": "0x73394566e4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetRentalUserInfoArray",
      "addr": "0x7339456790",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetEventRentalUserInfoArray",
      "addr": "0x73394567b4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetSuccessionDeckUserInfoArray",
      "addr": "0x73394567d8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetSuccessionDeckLastUsedUserInfoArray",
      "addr": "0x7339456918",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Gallop.ISinglemodeStartSuccessionInfoAccessor.get_RemainRentalCount",
      "addr": "0x7339456a58",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "Gallop.ISinglemodeStartSuccessionInfoAccessor.get_RentalTrainedCharaArray",
      "addr": "0x7339456a60",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "Gallop.ISinglemodeStartSuccessionInfoAccessor.get_EventRentalTrainedCharaArray",
      "addr": "0x7339456a68",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "Gallop.ISinglemodeStartSuccessionInfoAccessor.FindRentalSuccessionTrainedCharaByCondition",
      "addr": "0x7339456a70",
      "params": 3,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "Gallop.ISinglemodeStartSuccessionInfoAccessor.GetUserInfoForRentalSuccessionTrainedChara",
      "addr": "0x7339456b70",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "GetRentalTrainedChara",
      "addr": "0x7339456b8c",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "GetMasterCardData",
      "addr": "0x7339456d00",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "GetCardDressIdSet",
      "addr": "0x7339456d88",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "GetCharaId",
      "addr": "0x7339456e74",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "GetSingleModeStartChara",
      "addr": "0x7339456e8c",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "GetUseTpWhenBoostMode",
      "addr": "0x7339457828",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "GetMergedSupportCardIdArray",
      "addr": "0x733945762c",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "GetCampaignRentalSupportCardIdArray",
      "addr": "0x7339457640",
      "params": 0,
      "return_type": "fnptr",
      "static": false
    },
    {
      "name": "IsUseRentalSupportCard",
      "addr": "0x7339457c38",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "<GetUseTpWhenBoostMode>g__TryGetUseTpBoost|123_0",
      "addr": "0x73394579dc",
      "params": 2,
      "return_type": "boolean",
      "static": false
    }
  ]
}
```

## .<>c__DisplayClass102_0

```json
{
  "class": "<>c__DisplayClass102_0",
  "ns": "",
  "is_enum": false,
  "method_count": 1,
  "methods": [
    {
      "name": "<FetchSuccessionDeckLastUsed>b__0",
      "addr": "0x7339458054",
      "params": 1,
      "return_type": "void",
      "static": false
    }
  ]
}
```

## Gallop.SingleModeStartViewController

```json
{
  "class": "SingleModeStartViewController",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 63,
  "methods": [
    {
      "name": "get_SingleModeStartModel",
      "addr": "0x7339453388",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "get_IsBegin",
      "addr": "0x73394533ec",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsBegin",
      "addr": "0x73394533f4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_Entry",
      "addr": "0x7339453400",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_Entry",
      "addr": "0x7339453408",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_CurrentStep",
      "addr": "0x7339453410",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "get_PrevStep",
      "addr": "0x7339453418",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "set_PrevStep",
      "addr": "0x7339453420",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_StepUIDic",
      "addr": "0x7339453428",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "get_CharaViewer",
      "addr": "0x73394534a0",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_CharaViewer",
      "addr": "0x73394534a8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_OpenDialogAction",
      "addr": "0x73394534b0",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "set_OpenDialogAction",
      "addr": "0x73394534b8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_ExitSingleMode",
      "addr": "0x73394534c0",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_ExitSingleMode",
      "addr": "0x73394534c8",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "RegisterDownload",
      "addr": "0x73394534d4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "InitializeView",
      "addr": "0x7339453844",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "InitializeEachPlayIn",
      "addr": "0x73394538cc",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "CreateCharaViewer",
      "addr": "0x7339453954",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "PlayInView",
      "addr": "0x7339453c34",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "CoroutineDoTweenTimeScale",
      "addr": "0x7339453cbc",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "BeginView",
      "addr": "0x7339453d3c",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "UpdateView",
      "addr": "0x7339453da4",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "PlayOutView",
      "addr": "0x7339453efc",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "EndView",
      "addr": "0x7339453f84",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "FinalizeView",
      "addr": "0x733945400c",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "SetBg",
      "addr": "0x7339454094",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "GetCurrentScenarioBgOffset",
      "addr": "0x73394543e8",
      "params": 0,
      "return_type": "ptr",
      "static": true
    },
    {
      "name": "GetInitializeScenarioBgOffset",
      "addr": "0x73394545f8",
      "params": 0,
      "return_type": "ptr",
      "static": true
    },
    {
      "name": "GetScenarioBgOffset",
      "addr": "0x7339454264",
      "params": 1,
      "return_type": "ptr",
      "static": true
    },
    {
      "name": "GetCurrentScenarioBgPath",
      "addr": "0x733945466c",
      "params": 0,
      "return_type": "string",
      "static": true
    },
    {
      "name": "GetInitializeScenarioBgPath",
      "addr": "0x733945467c",
      "params": 0,
      "return_type": "string",
      "static": true
    },
    {
      "name": "GetScenarioBgPath",
      "addr": "0x7339454158",
      "params": 1,
      "return_type": "string",
      "static": true
    },
    {
      "name": "GetCurrentScenarioBgFadePath",
      "addr": "0x733945468c",
      "params": 0,
      "return_type": "string",
      "static": true
    },
    {
      "name": "GetInitializeScenarioBgFadePath",
      "addr": "0x73394546ec",
      "params": 0,
      "return_type": "string",
      "static": true
    },
    {
      "name": "GetCurrentScenarioId",
      "addr": "0x73394543f8",
      "params": 0,
      "return_type": "i4",
      "static": true
    },
    {
      "name": "GetInitializeScenarioId",
      "addr": "0x7339454608",
      "params": 0,
      "return_type": "i4",
      "static": true
    },
    {
      "name": "UpdateBg",
      "addr": "0x7339453e44",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "CreateStepUI",
      "addr": "0x73394547ec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetStep",
      "addr": "0x733945499c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "GetStepUI",
      "addr": "0x73394550b4",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "SetupDifficultyCampaign",
      "addr": "0x7339454c78",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetupSupportCardRankingButton",
      "addr": "0x7339454f5c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetupRaceFitAssistanceLabel",
      "addr": "0x7339454b2c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "PlaySelectedMotion",
      "addr": "0x733945512c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "PlayTapMotion",
      "addr": "0x73394552a4",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "StopVoice",
      "addr": "0x73394554a8",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "GetMainBgRect",
      "addr": "0x733945433c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetMainBgRect",
      "addr": "0x7339454754",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "MoveToCamera",
      "addr": "0x7339453b64",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "MoveToCamera",
      "addr": "0x73394555c0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetupCharacterFromTrainedData",
      "addr": "0x73394555f4",
      "params": 3,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SendPreSingleModeIndex",
      "addr": "0x7339455774",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SendUpdateRequestSuccessionDeck",
      "addr": "0x7339455784",
      "params": 3,
      "return_type": "void",
      "static": false
    },
    {
      "name": "FetchSuccessionDeckLastUsed",
      "addr": "0x733945585c",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnClickBackButton",
      "addr": "0x7339455ab8",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnClickOsBackKey",
      "addr": "0x7339455ac8",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<>n__0",
      "addr": "0x7339455b84",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<>n__1",
      "addr": "0x7339455bcc",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<>n__2",
      "addr": "0x7339455c14",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<>n__3",
      "addr": "0x7339455c5c",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<SetupSupportCardRankingButton>b__90_0",
      "addr": "0x7339455ca4",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<PlayTapMotion>b__93_0",
      "addr": "0x7339455e30",
      "params": 0,
      "return_type": "void",
      "static": false
    }
  ]
}
```

## Gallop.SingleModeStartStepCardSelect

```json
{
  "class": "SingleModeStartStepCardSelect",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 40,
  "methods": [
    {
      "name": "get__memberCardList",
      "addr": "0x73394605e4",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "set__memberCardList",
      "addr": "0x73394605ec",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "get_HeaderTextId",
      "addr": "0x73394605f4",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "get_GuideId",
      "addr": "0x73394605fc",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "get_IsRibbonAnimFirstCalled",
      "addr": "0x7339460604",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "set_IsRibbonAnimFirstCalled",
      "addr": "0x733946060c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Initialize",
      "addr": "0x7339460618",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "CreateBackInfo",
      "addr": "0x7339460b10",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "SetActionInBackInfo",
      "addr": "0x7339460d1c",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "CreateDetailDialogSetupParameter",
      "addr": "0x7339460df8",
      "params": 3,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "OnCloseCharacterCardDetailDialog",
      "addr": "0x7339461144",
      "params": 3,
      "return_type": "void",
      "static": false
    },
    {
      "name": "InitializeEachPlayIn",
      "addr": "0x73394616b4",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "EndView",
      "addr": "0x73394616f4",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnUpdateCard",
      "addr": "0x7339461778",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Show",
      "addr": "0x7339461bfc",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Hide",
      "addr": "0x73394623fc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "LoadMemberInfo",
      "addr": "0x73394614f0",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "FindAvailableMemberCardId",
      "addr": "0x7339461fb0",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "CreateCardModel",
      "addr": "0x733946243c",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "PlaySelectedMotion",
      "addr": "0x73394627ac",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "PlayRibbonAnimation",
      "addr": "0x73394625bc",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnTapCharaListButton",
      "addr": "0x7339462860",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "UpdateEntryInfo",
      "addr": "0x7339461304",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "CreateCharaList",
      "addr": "0x7339460908",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetupRecommendLabel",
      "addr": "0x733946193c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OpenCharaDetailDialog",
      "addr": "0x73394628e4",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnClickNextButton",
      "addr": "0x7339462a54",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OpenDialogTrainingChallengeCardRankingConfirmIfNeed",
      "addr": "0x7339462a58",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OpenRaceFitAssistanceProperAlert",
      "addr": "0x73394622d0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OpenDialogLovePointUpCampaignIfNeed",
      "addr": "0x7339462ae0",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "FetchSuccessionDeckLastUsed",
      "addr": "0x7339462c94",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "TransitionSuccessionSelect",
      "addr": "0x7339462d30",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnClickBackButton",
      "addr": "0x7339462d4c",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Release",
      "addr": "0x7339462d78",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "GetTutorialChooseCardButton",
      "addr": "0x7339462df0",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "<PlayRibbonAnimation>b__38_0",
      "addr": "0x7339462f9c",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<CreateCharaList>g__SaveSortFilterSetting|41_0",
      "addr": "0x7339463004",
      "params": 1,
      "return_type": "void",
      "static": true
    },
    {
      "name": "<OpenDialogTrainingChallengeCardRankingConfirmIfNeed>b__45_0",
      "addr": "0x733946306c",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<OpenRaceFitAssistanceProperAlert>b__46_0",
      "addr": "0x7339463074",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<OpenDialogLovePointUpCampaignIfNeed>b__47_0",
      "addr": "0x73394630b4",
      "params": 1,
      "return_type": "void",
      "static": false
    }
  ]
}
```

## Gallop.ISinglemodeStartSuccessionInfoAccessor

```json
{
  "class": "ISinglemodeStartSuccessionInfoAccessor",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 5,
  "methods": [
    {
      "name": "get_RemainRentalCount",
      "addr": "0x7334f63b9c",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "get_RentalTrainedCharaArray",
      "addr": "0x7334f63ee8",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "get_EventRentalTrainedCharaArray",
      "addr": "0x7334f63ee8",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "FindRentalSuccessionTrainedCharaByCondition",
      "addr": "0x0",
      "params": 3,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "GetUserInfoForRentalSuccessionTrainedChara",
      "addr": "0x0",
      "params": 1,
      "return_type": "byref",
      "static": false
    }
  ]
}
```

## Gallop.SingleModeSuccessionDeckLastUsedEntity

```json
{
  "class": "SingleModeSuccessionDeckLastUsedEntity",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 5,
  "methods": [
    {
      "name": "get_DeckId",
      "addr": "0x7339471eb8",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "get_DeckName",
      "addr": "0x7339471ec0",
      "params": 0,
      "return_type": "string",
      "static": false
    },
    {
      "name": "get_CharaFirst",
      "addr": "0x7339471ecc",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "get_CharaSecond",
      "addr": "0x7339471ed4",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "get_LastSingleModeStartedTimestamp",
      "addr": "0x7339471edc",
      "params": 0,
      "return_type": "i8",
      "static": false
    }
  ]
}
```

## Gallop.SingleModeSuccessionDeckSetEntity

```json
{
  "class": "SingleModeSuccessionDeckSetEntity",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 10,
  "methods": [
    {
      "name": "GetDeckByDeckId",
      "addr": "0x733946f97c",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "GetDecksBySetId",
      "addr": "0x7339472028",
      "params": 1,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "GetAllDecks",
      "addr": "0x7339472120",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "SetDeckName",
      "addr": "0x7339472128",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetSuccessionCharaFirst",
      "addr": "0x7339472144",
      "params": 3,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetSuccessionCharaSecond",
      "addr": "0x73394721d8",
      "params": 3,
      "return_type": "void",
      "static": false
    },
    {
      "name": "CopyDeckChara",
      "addr": "0x733947226c",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "IsUsingTrainedChara",
      "addr": "0x7339472398",
      "params": 2,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "FindTrainedCharaByCondition",
      "addr": "0x733947247c",
      "params": 3,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "GetDeck",
      "addr": "0x7339471f54",
      "params": 1,
      "return_type": "byref",
      "static": false
    }
  ]
}
```

## Gallop.DialogSingleModeStartSuccessionDeckActionResultNotice

```json
{
  "class": "DialogSingleModeStartSuccessionDeckActionResultNotice",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 7,
  "methods": [
    {
      "name": "GetFormType",
      "addr": "0x7339475b54",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "GetParentType",
      "addr": "0x7339475b5c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "PushDialogForSave",
      "addr": "0x73394751e4",
      "params": 2,
      "return_type": "void",
      "static": true
    },
    {
      "name": "PushDialogForLoad",
      "addr": "0x733946d440",
      "params": 2,
      "return_type": "void",
      "static": true
    },
    {
      "name": "PushDialogForRemove",
      "addr": "0x733946ebec",
      "params": 0,
      "return_type": "void",
      "static": true
    },
    {
      "name": "CreateDialogData",
      "addr": "0x7339475b64",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "Setup",
      "addr": "0x7339475ba8",
      "params": 3,
      "return_type": "void",
      "static": false
    }
  ]
}
```

## Gallop.DialogSingleModeStartSuccessionDeckCharacterSelect

```json
{
  "class": "DialogSingleModeStartSuccessionDeckCharacterSelect",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 22,
  "methods": [
    {
      "name": "GetFormType",
      "addr": "0x7339475f80",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "GetParentType",
      "addr": "0x7339475f88",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "PushDialog",
      "addr": "0x7339474ca8",
      "params": 10,
      "return_type": "void",
      "static": true
    },
    {
      "name": "CreateDialogData",
      "addr": "0x7339476048",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "Setup",
      "addr": "0x7339475f90",
      "params": 10,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetupSuccessionCharaSelector",
      "addr": "0x733947632c",
      "params": 7,
      "return_type": "void",
      "static": false
    },
    {
      "name": "GetSuccessionTrainedCharaData",
      "addr": "0x7339476144",
      "params": 2,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "SetupFacterListButton",
      "addr": "0x733947644c",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OpenDialogSingleModeFactorList",
      "addr": "0x7339476c2c",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetupDecideButtonStatus",
      "addr": "0x7339476608",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnUpdateSelectedCharacter",
      "addr": "0x7339476e34",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnTapUnSelectCharacter",
      "addr": "0x7339476f18",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnUpdateToggleCharacterType",
      "addr": "0x7339476f64",
      "params": 3,
      "return_type": "void",
      "static": false
    },
    {
      "name": "UpdateSelectedStatus",
      "addr": "0x7339476eec",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnClickDecideButton",
      "addr": "0x733947701c",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnClickCancelButton",
      "addr": "0x7339477068",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Gallop.PartsSingleModeStartSuccessionCharaSelector.IEventDispatcher.OnUpdateSelectedStatus",
      "addr": "0x73394770ac",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Gallop.PartsSingleModeStartSuccessionCharaSelector.IEventDispatcher.OnTapUnselect",
      "addr": "0x73394770b4",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Gallop.PartsSingleModeStartSuccessionCharaSelector.IEventDispatcher.OnUpdateToggle",
      "addr": "0x73394770b8",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "Gallop.PartsSingleModeStartSuccessionCharaSelector.IEventDispatcher.OnClickFriendSearchButton",
      "addr": "0x73394770c0",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<CreateDialogData>b__13_0",
      "addr": "0x7339477114",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<CreateDialogData>b__13_1",
      "addr": "0x7339477118",
      "params": 1,
      "return_type": "void",
      "static": false
    }
  ]
}
```

## Gallop.DialogSingleModeStartSuccessionDeckListModel

```json
{
  "class": "DialogSingleModeStartSuccessionDeckListModel",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 7,
  "methods": [
    {
      "name": "get_RemainRentalCount",
      "addr": "0x7339477284",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "get_IsAvailableEventRental",
      "addr": "0x733947728c",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "GetDecksBySetId",
      "addr": "0x7339477294",
      "params": 1,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "SetDeckName",
      "addr": "0x73394772ac",
      "params": 3,
      "return_type": "void",
      "static": false
    },
    {
      "name": "ExecCharaChange",
      "addr": "0x733947747c",
      "params": 5,
      "return_type": "void",
      "static": false
    },
    {
      "name": "ExecCopy",
      "addr": "0x73394776bc",
      "params": 3,
      "return_type": "void",
      "static": false
    },
    {
      "name": "UpdateRentalCharaList",
      "addr": "0x733947782c",
      "params": 1,
      "return_type": "void",
      "static": false
    }
  ]
}
```

## Gallop.DialogSingleModeStartSuccessionDeckList

```json
{
  "class": "DialogSingleModeStartSuccessionDeckList",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 19,
  "methods": [
    {
      "name": "GetFormType",
      "addr": "0x7339478128",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "GetParentType",
      "addr": "0x7339478130",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "GetSortFilterSettingSaveTagDict",
      "addr": "0x7339478138",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "PushDialog",
      "addr": "0x733946e7a4",
      "params": 5,
      "return_type": "void",
      "static": true
    },
    {
      "name": "CreateDialogData",
      "addr": "0x73394784e8",
      "params": 0,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "Setup",
      "addr": "0x73394781f8",
      "params": 5,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetupDeckList",
      "addr": "0x73394786e0",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnSelectSet",
      "addr": "0x7339478d28",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnClickDeckNameChange",
      "addr": "0x7339478e1c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnClickDeckCharaChange",
      "addr": "0x7339478f94",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "PushDialogSuccessionDeckCharacterSelect",
      "addr": "0x733947906c",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnClickCopy",
      "addr": "0x7339479254",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnClickClose",
      "addr": "0x733947952c",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnClickLoad",
      "addr": "0x7339479568",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnCompleteDeckCharaChange",
      "addr": "0x7339479648",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "OnUpdateSuccessionDeck",
      "addr": "0x7339479678",
      "params": 0,
      "return_type": "void",
      "static": false
    },
    {
      "name": "GetFollowChangeCheckActions",
      "addr": "0x73394785a0",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "<Setup>b__21_1",
      "addr": "0x7339479770",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<GetFollowChangeCheckActions>b__32_1",
      "addr": "0x7339479778",
      "params": 0,
      "return_type": "void",
      "static": false
    }
  ]
}
```

## Gallop.DialogSingleModeStartSuccessionDeckOverwriteConfirm

```json
{
  "class": "DialogSingleModeStartSuccessionDeckOverwriteConfirm",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 6,
  "methods": [
    {
      "name": "GetFormType",
      "addr": "0x7339479d8c",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "GetParentType",
      "addr": "0x7339479d94",
      "params": 0,
      "return_type": "ptr",
      "static": false
    },
    {
      "name": "PushDialog",
      "addr": "0x733946e9b0",
      "params": 8,
      "return_type": "void",
      "static": true
    },
    {
      "name": "CreateDialogData",
      "addr": "0x7339479d9c",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "Setup",
      "addr": "0x7339479e5c",
      "params": 7,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<CreateDialogData>b__8_0",
      "addr": "0x7339479f94",
      "params": 1,
      "return_type": "void",
      "static": false
    }
  ]
}
```

## .SuccessionCharaSelectorModel

```json
{
  "class": "SuccessionCharaSelectorModel",
  "ns": "",
  "is_enum": false,
  "method_count": 18,
  "methods": [
    {
      "name": "get_IsFirst",
      "addr": "0x733947d9cc",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "get_SingleModeCharaId",
      "addr": "0x733947d9d4",
      "params": 0,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "get_SaveTagDict",
      "addr": "0x733947d9dc",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "get_AllowSelectSingleModeChara",
      "addr": "0x733947d9e4",
      "params": 0,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "get_SelectedDataOnSetup",
      "addr": "0x733947d9ec",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "get_OtherSelectedData",
      "addr": "0x733947da14",
      "params": 0,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "SetSelectedCharacter",
      "addr": "0x733947db94",
      "params": 2,
      "return_type": "void",
      "static": false
    },
    {
      "name": "GetSelectedCharacter",
      "addr": "0x733947dbfc",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "GetTrainedCharaDataListByToggleType",
      "addr": "0x733947dc54",
      "params": 1,
      "return_type": "valuetype",
      "static": false
    },
    {
      "name": "TryGetToggleType",
      "addr": "0x733947e430",
      "params": 3,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "GetUserInfoForRentalSuccessionTrainedChara",
      "addr": "0x733947e628",
      "params": 1,
      "return_type": "byref",
      "static": false
    },
    {
      "name": "CalcRelationPoint",
      "addr": "0x733947e6dc",
      "params": 1,
      "return_type": "i4",
      "static": false
    },
    {
      "name": "IsSameTrainedCharaData",
      "addr": "0x733947eb74",
      "params": 2,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "UpdateRentalCharaList",
      "addr": "0x733947ec14",
      "params": 1,
      "return_type": "void",
      "static": false
    },
    {
      "name": "<GetTrainedCharaDataListByToggleType>g__IsRental|23_0",
      "addr": "0x733947e0dc",
      "params": 1,
      "return_type": "boolean",
      "static": true
    },
    {
      "name": "<GetTrainedCharaDataListByToggleType>g__IsEventRental|23_1",
      "addr": "0x733947e288",
      "params": 1,
      "return_type": "boolean",
      "static": true
    },
    {
      "name": "<GetTrainedCharaDataListByToggleType>g__ContainsRentalTrainedCharaArray|23_2",
      "addr": "0x733947e10c",
      "params": 1,
      "return_type": "boolean",
      "static": false
    },
    {
      "name": "<GetTrainedCharaDataListByToggleType>g__ContainsInEventRentalTrainedCharaArray|23_3",
      "addr": "0x733947e2b4",
      "params": 1,
      "return_type": "boolean",
      "static": false
    }
  ]
}
```

## Gallop.PartsSingleModeStartSuccessionDeckListItem

```json
{
  "class": "PartsSingleModeStartSuccessionDeckListItem",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 3,
  "methods": [
    {
      "name": "Setup",
      "addr": "0x733947ff30",
      "params": 11,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetupSuccessionSlot",
      "addr": "0x7339480394",
      "params": 9,
      "return_type": "void",
      "static": false
    },
    {
      "name": "SetLoadButtonEnabled",
      "addr": "0x73394804d4",
      "params": 1,
      "return_type": "void",
      "static": false
    }
  ]
}
```

## Gallop.SingleModeSuccessionDeckLastUsedRepository

```json
{
  "class": "SingleModeSuccessionDeckLastUsedRepository",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 2,
  "methods": [
    {
      "name": "GetByCardId",
      "addr": "0x7339480b68",
      "params": 1,
      "return_type": "byref",
      "static": true
    },
    {
      "name": "GetTrainedCharaData",
      "addr": "0x7339480dec",
      "params": 1,
      "return_type": "byref",
      "static": true
    }
  ]
}
```

## Gallop.SingleModeSuccessionDeckSetRepository

```json
{
  "class": "SingleModeSuccessionDeckSetRepository",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 3,
  "methods": [
    {
      "name": "Get",
      "addr": "0x73394810a4",
      "params": 1,
      "return_type": "byref",
      "static": true
    },
    {
      "name": "CreateDeckEntity",
      "addr": "0x7339481244",
      "params": 1,
      "return_type": "byref",
      "static": true
    },
    {
      "name": "GetTrainedCharaData",
      "addr": "0x733948154c",
      "params": 1,
      "return_type": "byref",
      "static": true
    }
  ]
}
```

## Gallop.SingleModeStartSuccessionDeckConnect

```json
{
  "class": "SingleModeStartSuccessionDeckConnect",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 4,
  "methods": [
    {
      "name": "SendSingleModeUpdateRequestSuccessionDeck",
      "addr": "0x733948360c",
      "params": 3,
      "return_type": "void",
      "static": true
    },
    {
      "name": "SendGenerateSuccessionUpdateRequestSuccessionDeck",
      "addr": "0x7339483884",
      "params": 3,
      "return_type": "void",
      "static": true
    },
    {
      "name": "CreateSuccessionDeckForRequest",
      "addr": "0x7339483afc",
      "params": 1,
      "return_type": "byref",
      "static": true
    },
    {
      "name": "CreateSuccessionDeckCharaForRequest",
      "addr": "0x7339483d64",
      "params": 1,
      "return_type": "byref",
      "static": true
    }
  ]
}
```

## Gallop.SingleModeStartSuccessionDeckDefine

```json
{
  "class": "SingleModeStartSuccessionDeckDefine",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 0,
  "methods": []
}
```

## Gallop.SingleModeStartSuccessionDeckRestorer

```json
{
  "class": "SingleModeStartSuccessionDeckRestorer",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 3,
  "methods": [
    {
      "name": "Restore",
      "addr": "0x733948401c",
      "params": 2,
      "return_type": "valuetype",
      "static": true
    },
    {
      "name": "SaveDataForRestore",
      "addr": "0x733948431c",
      "params": 2,
      "return_type": "void",
      "static": true
    },
    {
      "name": "ConvertToDeckChara",
      "addr": "0x7339484258",
      "params": 4,
      "return_type": "byref",
      "static": true
    }
  ]
}
```

## .<>c__DisplayClass6_0

```json
{
  "class": "<>c__DisplayClass6_0",
  "ns": "",
  "is_enum": false,
  "method_count": 1,
  "methods": [
    {
      "name": "<TryGetTrainedCharaData>b__0",
      "addr": "0x7339484bf8",
      "params": 1,
      "return_type": "boolean",
      "static": false
    }
  ]
}
```

## Gallop.SingleModeStartSuccessionDeckUtils

```json
{
  "class": "SingleModeStartSuccessionDeckUtils",
  "ns": "Gallop",
  "is_enum": false,
  "method_count": 7,
  "methods": [
    {
      "name": "IsUsingTrainedCharaForSuccessionDeck",
      "addr": "0x7339484630",
      "params": 1,
      "return_type": "boolean",
      "static": true
    },
    {
      "name": "IsUsingTrainedCharaForGenerateSuccessionExec",
      "addr": "0x733948499c",
      "params": 1,
      "return_type": "boolean",
      "static": true
    },
    {
      "name": "IsUsingTrainedCharaForSingleModeDeck",
      "addr": "0x7339484660",
      "params": 1,
      "return_type": "boolean",
      "static": true
    },
    {
      "name": "IsUsingTrainedCharaForGenerateSuccessionDeck",
      "addr": "0x73394847f8",
      "params": 1,
      "return_type": "boolean",
      "static": true
    },
    {
      "name": "GetUserInfos",
      "addr": "0x7339484aa8",
      "params": 1,
      "return_type": "valuetype",
      "static": true
    },
    {
      "name": "GetUserInfos",
      "addr": "0x7339484b4c",
      "params": 1,
      "return_type": "valuetype",
      "static": true
    },
    {
      "name": "TryGetTrainedCharaData",
      "addr": "0x73394843c0",
      "params": 7,
      "return_type": "boolean",
      "static": true
    }
  ]
}
```

