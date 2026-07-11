# libil2cpp.so 字符串搜索报告

**文件**: libil2cpp.so (208MB, ARM64)
**总字符串数(>=8字符)**: 8901
---

## "Cook" (4匹配)

- `@0x01e13abf`: `UnityEngine.MeshCollider::get_cookingOptions()`
- `@0x01e16925`: `UnityEngine.Light::get_cookie()`
- `@0x01e22a8f`: `UnityEngine.MeshCollider::set_cookingOptions(UnityEngine.MeshColliderCookingOptions)`
- `@0x01e456c1`: `UnityEngine.Light::get_cookieSize()`

## "Arc" (17匹配)

- `@0x0001deb8`: `mono_arch_clear_breakpoint`
- `@0x0001ded3`: `mono_arch_context_get_int_reg`
- `@0x0001def1`: `mono_arch_context_set_int_reg`
- `@0x0001df0f`: `mono_arch_set_breakpoint`
- `@0x0001df28`: `mono_arch_setup_resume_sighandler_ctx`
- `@0x0001df4e`: `mono_arch_skip_breakpoint`
- `@0x0001df68`: `mono_arch_skip_single_step`
- `@0x0001df83`: `mono_arch_start_single_stepping`
- `@0x0001dfa3`: `mono_arch_stop_single_stepping`
- `@0x01e0f038`: `UnityEngine.Camera::get_farClipPlane()`
- `@0x01e1e3c9`: `UnityEngine.Camera::get_nearClipPlane()`
- `@0x01e22316`: `UnityEngine.Transform::internal_getHierarchyCount()`
- `@0x01e2d192`: `Unknown DWARF encoding for search table.`
- `@0x01e40309`: `UnityEngine.GameObject::get_activeInHierarchy()`
- `@0x01e41695`: `UnityEngine.Camera::set_farClipPlane(System.Single)`
- `@0x01e433df`: `UnityEngine.Camera::set_nearClipPlane(System.Single)`
- `@0x01e46c2c`: `Can't binary search on variable length encoded data.`

## "Live" (6匹配)

- `@0x0001dd72`: `il2cpp_unity_liveness_allocate_struct`
- `@0x0001dd98`: `il2cpp_unity_liveness_calculation_from_root`
- `@0x0001ddc4`: `il2cpp_unity_liveness_calculation_from_statics`
- `@0x0001ddf3`: `il2cpp_unity_liveness_finalize`
- `@0x0001de12`: `il2cpp_unity_liveness_free_struct`
- `@0x01e1927d`: `UnityEngine.ParticleSystem::IsAlive(System.Boolean)`

## "Sport" (1匹配)

- `@0x0001d80f`: `il2cpp_register_debugger_agent_transport`

## "URA" (21匹配)

- `@0x0001eb10`: `_ZNSt6__ndk111this_thread9sleep_forERKNS_6chrono8durationIxNS_5ratioILl1ELl1000000000EEEEE`
- `@0x0001ec38`: `_ZNSt6__ndk118condition_variable15__do_timed_waitERNS_11unique_lockINS_5mutexEEENS_6chrono10time_pointINS5_12system_clockENS5_8durationIxNS_5ratioILl1ELl1000000000EEEEEEE`
- `@0x01e0f881`: `UnityEngine.Rendering.CommandBuffer::Internal_DrawProcedural_Injected(UnityEngine.Matrix4x4&,UnityEngine.Material,System.Int32,UnityEngine.MeshTopology,System.Int32,System.Int32,UnityEngine.MaterialPropertyBlock)`
- `@0x01e1ae1a`: `UnityEngine.Playables.PlayableHandle::SetDuration_Injected(UnityEngine.Playables.PlayableHandle&,System.Double)`
- `@0x01e21197`: `/dev/urandom`
- `@0x01e2160c`: `criAtomExAsr_EnableBinauralizer`
- `@0x01e2162c`: `criAtomExAsr_IsEnabledBinauralizer`
- `@0x01e2486e`: `UnityEngine.Playables.PlayableHandle::GetDuration_Injected(UnityEngine.Playables.PlayableHandle&)`
- `@0x01e24a65`: `UnityEngine.Playables.PlayableDirector::get_duration()`
- `@0x01e2e99b`: `UnityEngine.ParticleSystem/MainModule::get_duration_Injected(UnityEngine.ParticleSystem/MainModule&)`
- `@0x01e2f45e`: `criAtomExOutputPort_SetMonauralMix`
- `@0x01e30700`: `UnityEngine.ParticleSystem/MainModule::set_duration_Injected(UnityEngine.ParticleSystem/MainModule&,System.Single)`
- `@0x01e3b011`: `/Users/gallop/unity_workspace/release_project1/GallopClient/Temp/coneshellWork/IL2CPP/libil2cpp/icalls/mscorlib/System/AppDomain.cpp(209) : Unsupported internal call for IL2CPP:AppDomain::InternalGetProcessGuid - "This icall is only used in System.Runtime.Remoting.RemotingConfiguraiton.ProcessId."`
- `@0x01e3eddd`: `System.Configuration.dll`
- `@0x026bff6e`: `Spanish (Honduras)`
- `@0x026bff87`: `ol (Honduras)`
- `@0x026c12c1`: `Honduras`
- `@0x026c12ce`: `Honduran Lempira`
- `@0x026cac0e`: `Faburairu`
- `@0x026cb12d`: `Guraandhala`
- `@0x026cbbae`: `Gicuransi`

## "Evaluation" (1匹配)

- `@0x01e3a8df`: `UnityEngine.Playables.PlayableGraph::SynchronizeEvaluation_Injected(UnityEngine.Playables.PlayableGraph&,UnityEngine.Playables.PlayableGraph&)`

## "CheckPoint" (2匹配)

- `@0x01e21aaa`: `sqlite3_wal_checkpoint_v2`
- `@0x01e3f86f`: `sqlite3_wal_checkpoint`

## "Board" (20匹配)

- `@0x01e110e9`: `UnityEngine.TouchScreenKeyboard::IsRequiredToForceOpen()`
- `@0x01e16dac`: `UnityEngine.TouchScreenKeyboard::set_active(System.Boolean)`
- `@0x01e18c69`: `UnityEngine.TouchScreenKeyboard::SetSelection(System.Int32,System.Int32)`
- `@0x01e1af85`: `UnityEngine.GUIUtility::Internal_SetKeyboardControl(System.Int32)`
- `@0x01e1e9f6`: `UnityEngine.TouchScreenKeyboard::IsInPlaceEditingAllowed()`
- `@0x01e1ea31`: `UnityEngine.TouchScreenKeyboard::get_active()`
- `@0x01e1eda0`: `UnityEngine.GUIUtility::SetKeyboardControlToLastControlId()`
- `@0x01e202df`: `UnityEngine.TouchScreenKeyboard::get_canSetSelection()`
- `@0x01e226ed`: `UnityEngine.GUIUtility::SetKeyboardControlToFirstControlId()`
- `@0x01e243ab`: `UnityEngine.TouchScreenKeyboard::get_status()`
- `@0x01e25e0a`: `UnityEngine.QualitySettings::get_billboardsFaceCameraPosition()`
- `@0x01e2e2fd`: `UnityEngine.TouchScreenKeyboard::get_canGetSelection()`
- `@0x01e35f78`: `UnityEngine.TouchScreenKeyboard::get_text()`
- `@0x01e35fa4`: `UnityEngine.TouchScreenKeyboard::set_hideInput(System.Boolean)`
- `@0x01e3813e`: `UnityEngine.TouchScreenKeyboard::set_text(System.String)`
- `@0x01e3872f`: `UnityEngine.GUIUtility::Internal_GetKeyboardControl()`
- `@0x01e403d3`: `UnityEngine.TouchScreenKeyboard::set_characterLimit(System.Int32)`
- `@0x01e41cdf`: `UnityEngine.TouchScreenKeyboard::TouchScreenKeyboard_InternalConstructorHelper(UnityEngine.TouchScreenKeyboard_InternalConstructorHelperArguments&,System.String,System.String)`
- `@0x01e41d8f`: `UnityEngine.TouchScreenKeyboard::GetSelection(System.Int32&,System.Int32&)`
- `@0x01e49cb1`: `UnityEngine.TouchScreenKeyboard::Internal_Destroy(System.IntPtr)`

## "Heart" (2匹配)

- `@0x01e2b342`: `Firebase_App_CSharp_FirebaseApp_LogHeartbeatInternal`
- `@0x01e33468`: `Firebase_Auth_CSharp_FirebaseAuth_LogHeartbeatInternal`

## "Random" (23匹配)

- `@0x0001a656`: `SystemNative_GetNonCryptographicallySecureRandomBytes`
- `@0x01e0d3db`: `UnityEngine.RenderTexture::set_enableRandomWrite(System.Boolean)`
- `@0x01e10e66`: `UnityEngine.Random::get_state_Injected(UnityEngine.Random/State&)`
- `@0x01e117e4`: `UnityEngine.ParticleSystemForceField::get_rotationRandomness_Injected(UnityEngine.Vector2&)`
- `@0x01e18af2`: `UnityEngine.Random::get_insideUnitSphere_Injected(UnityEngine.Vector3&)`
- `@0x01e1cd49`: `UnityEngine.Random::RandomRangeInt(System.Int32,System.Int32)`
- `@0x01e1eb29`: `UnityEngine.Rendering.CommandBuffer::SetRandomWriteTarget_Texture(System.Int32,UnityEngine.Rendering.RenderTargetIdentifier&)`
- `@0x01e21197`: `/dev/urandom`
- `@0x01e26a53`: `UnityEngine.ParticleSystem::get_randomSeed()`
- `@0x01e28270`: `UnityEngine.Random::Range(System.Single,System.Single)`
- `@0x01e28510`: `UnityEngine.Rendering.CommandBuffer::SetRandomWriteTarget_GraphicsBuffer(System.Int32,UnityEngine.GraphicsBuffer,System.Boolean)`
- `@0x01e2e909`: `UnityEngine.ParticleSystem::set_randomSeed(System.UInt32)`
- `@0x01e38009`: `UnityEngine.Random::set_state_Injected(UnityEngine.Random/State&)`
- `@0x01e3939d`: `criAtomEx_SetRandomSeed`
- `@0x01e3abb9`: `UnityEngine.ParticleSystem::get_useAutoRandomSeed()`
- `@0x01e3c529`: `UnityEngine.Rendering.CommandBuffer::ClearRandomWriteTargets()`
- `@0x01e3d4b8`: `criAtomExPlayer_SetRandomSeed`
- `@0x01e3deba`: `UnityEngine.Random::InitState(System.Int32)`
- `@0x01e3dee6`: `UnityEngine.Random::get_value()`
- `@0x01e42499`: `UnityEngine.ParticleSystemForceField::set_rotationRandomness_Injected(UnityEngine.Vector2&)`
- `@0x01e4486a`: `criAtomEx3dSource_SetRandomPositionList`
- `@0x01e460d2`: `UnityEngine.ParticleSystem::set_useAutoRandomSeed(System.Boolean)`
- `@0x01e48e5b`: `criAtomEx3dSource_SetRandomPositionConfig`

## "Rest" (14匹配)

- `@0x0001e785`: `mono_restore_context`
- `@0x01e0e476`: `unsupported restore location for float register`
- `@0x01e1815c`: `Firebase_App_CSharp_new_FutureString`
- `@0x01e1db5f`: `Bad signal in restart handler`
- `@0x01e1f496`: `unsupported restore location for register`
- `@0x01e25393`: `Cannot set SIG_THR_RESTART handler`
- `@0x01e295d7`: `tempest_restart`
- `@0x01e2d5a0`: `Firebase_App_CSharp_FutureString_SWIGUpcast`
- `@0x01e32e19`: ` restrict`
- `@0x01e352e4`: `Firebase_App_CSharp_FutureString_SWIG_FreeCompletionData`
- `@0x01e3747c`: `Firebase_App_CSharp_FutureString_GetResult`
- `@0x01e3d50d`: `Firebase_App_CSharp_delete_FutureString`
- `@0x01e44abe`: `Firebase_App_CSharp_FutureString_SWIG_OnCompletion`
- `@0x01e493cb`: `tempest_restart_with_reconfig`

## "Race" (10匹配)

- `@0x0001cf88`: `il2cpp_format_stack_trace`
- `@0x0001d5a2`: `il2cpp_native_stack_trace`
- `@0x0001d690`: `il2cpp_override_stack_backtrace`
- `@0x01e14006`: `GC_TRACE`
- `@0x01e1e386`: `UnityEngine.Application::GetStackTraceLogType(UnityEngine.LogType)`
- `@0x01e37be4`: `UnityEngine.Debug::ExtractStackTraceNoAlloc(System.Byte*,System.Int32,System.String)`
- `@0x01e40c45`: `StackTrace`
- `@0x01e44522`: `No native stack trace exists. Make sure this is platform supports native stack traces.`
- `@0x02780256`: `N12_GLOBAL__N_116itanium_demangle10BracedExprE`
- `@0x02780285`: `N12_GLOBAL__N_116itanium_demangle15BracedRangeExprE`

## "Region" (9匹配)

- `@0x0001d36d`: `il2cpp_memory_pool_get_region_size`
- `@0x0001d390`: `il2cpp_memory_pool_set_region_size`
- `@0x01e17f96`: `criAtomEx3dListener_Set3dRegionHn`
- `@0x01e17fb8`: `criAtomEx3dSource_Set3dRegionHn`
- `@0x01e25564`: `criAtomEx3dRegion_Destroy`
- `@0x01e31108`: `criAtomEx3dTransceiver_Set3dRegionHn`
- `@0x01e32f6e`: `criAtomEx3dRegion_Create`
- `@0x01e33c99`: `UnityEngine.Graphics::CopyTexture_Region(UnityEngine.Texture,System.Int32,System.Int32,System.Int32,System.Int32,System.Int32,System.Int32,UnityEngine.Texture,System.Int32,System.Int32,System.Int32,System.Int32)`
- `@0x01e40ed2`: `criAtomEx3dRegion_IsDestroyable`

## "Store" (9匹配)

- `@0x0001e785`: `mono_restore_context`
- `@0x01e0e476`: `unsupported restore location for float register`
- `@0x01e1e9be`: `UnityEngine.SystemInfo::SupportsStoreAndResolveAction()`
- `@0x01e1f496`: `unsupported restore location for register`
- `@0x01e283ee`: `UnityEngine.SystemInfo::UsesLoadStoreActions()`
- `@0x01e2a19e`: `UnityEngine.Rendering.CommandBuffer::SetRenderTargetColorDepth_Internal_Injected(UnityEngine.Rendering.RenderTargetIdentifier&,UnityEngine.Rendering.RenderTargetIdentifier&,UnityEngine.Rendering.RenderBufferLoadAction,UnityEngine.Rendering.RenderBufferStoreAction,UnityEngine.Rendering.RenderBufferLoadAction,UnityEngine.Rendering.RenderBufferStoreAction,UnityEngine.Rendering.RenderTargetFlags)`
- `@0x01e36273`: `UnityEngine.Rendering.CommandBuffer::SetRenderTargetSingle_Internal_Injected(UnityEngine.Rendering.RenderTargetIdentifier&,UnityEngine.Rendering.RenderBufferLoadAction,UnityEngine.Rendering.RenderBufferStoreAction,UnityEngine.Rendering.RenderBufferLoadAction,UnityEngine.Rendering.RenderBufferStoreAction)`
- `@0x01e46ade`: `invalid stored block lengths`
- `@0x01e47e33`: `UnityEngine.Rendering.CommandBuffer::SetRenderTargetMultiSubtarget_Injected(UnityEngine.Rendering.RenderTargetIdentifier[],UnityEngine.Rendering.RenderTargetIdentifier&,UnityEngine.Rendering.RenderBufferLoadAction[],UnityEngine.Rendering.RenderBufferStoreAction[],UnityEngine.Rendering.RenderBufferLoadAction,UnityEngine.Rendering.RenderBufferStoreAction,System.Int32,UnityEngine.CubemapFace,System.Int32)`

## "Shop" (1匹配)

- `@0x01e43fb7`: `UnityEngine.ParticleSystemRenderer::BakeMesh(UnityEngine.Mesh,UnityEngine.Camera,UnityEngine.ParticleSystemBakeMeshOptions)`

## "Director" (19匹配)

- `@0x01e0f9d5`: `UnityEngine.Playables.PlayableDirector::GetGraphHandle_Injected(UnityEngine.Playables.PlayableGraph&)`
- `@0x01e11497`: `UnityEngine.Playables.PlayableDirector::set_time(System.Double)`
- `@0x01e114d7`: `UnityEngine.Playables.PlayableDirector::Play()`
- `@0x01e154d6`: `UnityEngine.Playables.PlayableDirector::Pause()`
- `@0x01e17283`: `UnityEngine.Playables.PlayableDirector::Internal_SetGenericBinding(UnityEngine.Object,UnityEngine.Object)`
- `@0x01e24a65`: `UnityEngine.Playables.PlayableDirector::get_duration()`
- `@0x01e28752`: `UnityEngine.Playables.PlayableHandle::SetTimeWrapMode_Injected(UnityEngine.Playables.PlayableHandle&,UnityEngine.Playables.DirectorWrapMode)`
- `@0x01e29209`: `criFsBinder_BindDirectory`
- `@0x01e2a4c3`: `UnityEngine.Playables.PlayableDirector::Evaluate()`
- `@0x01e2c604`: `UnityEngine.Playables.PlayableDirector::GetPlayState()`
- `@0x01e2c63b`: `UnityEngine.Playables.PlayableDirector::PlayOnFrame_Injected(UnityEngine.Playables.FrameRate&)`
- `@0x01e2e77d`: `UnityEngine.Playables.PlayableDirector::Stop()`
- `@0x01e2e7ac`: `UnityEngine.Playables.PlayableDirector::GetGenericBinding(UnityEngine.Object)`
- `@0x01e3644f`: `UnityEngine.Playables.PlayableGraph::SetTimeUpdateMode_Injected(UnityEngine.Playables.PlayableGraph&,UnityEngine.Playables.DirectorUpdateMode)`
- `@0x01e38626`: `UnityEngine.Playables.PlayableDirector::get_time()`
- `@0x01e38659`: `UnityEngine.Playables.PlayableDirector::Internal_GetPlayableAsset()`
- `@0x01e3c87c`: `UnityEngine.Playables.PlayableDirector::GetWrapMode()`
- `@0x01e43d07`: `UnityEngine.Playables.PlayableDirector::GetReferenceValue_Injected(UnityEngine.PropertyName&,System.Boolean&)`
- `@0x01e4879e`: `UnityEngine.DirectorModule.dll`

