# 诊断候选的 managed Hook 准入条件

当前 `3.28.2-recovery.3` 是诊断候选，不是完整嗅探修复版。目标游戏、Unity、宿主
版本及生成函数的隐藏参数没有共同验证证据。`hook_abi::verified_profiles()` 与
`implemented_thunk()` 当前为空，所有 13 个 managed Hook 在 `interceptor_hook`
入口返回 `unsupported_native_abi`，发生在原生安装及读取目标 prologue 之前。
HTTP 的 `enabled=1`、已有配置或元数据成功都不能绕过这一条件。
安装器还在程序集/类/方法查询之前执行同一 profile/thunk 准入检查，避免 HTTP
嗅探开关或 MD5 安装入口变成第二条原生元数据读取路径。

这包括训练结果、ExecTraining、失败率、事件选项、SetStory、Unity 请求与完成回调、
MakeMd5、ComputeHash、CompressRequest、DecompressResponse、Post。原生 C Hook
另有显式白名单；镜像默认关闭且启动时不安装，TextCommon 观察器不安装。

## 已确认与未确认的依据

Unity 官方 [A tour of generated code](https://unity.com/blog/engine-platform/il2cpp-internals-a-tour-of-generated-code)
展示包含 `MethodInfo*` 的生成方法；文首明确示例使用 Unity 5.0.1p1，属于会随版本改变的实现细节。
[Method calls](https://unity.com/blog/engine-platform/il2cpp-internals-method-calls)
同样展示隐藏元数据参数的传递。因此不能直接将旧文章的静态 `this` 规则套到目标游戏，
也不能把仅保留当前参数的宿主 spy 测试当作隐藏参数已正确转发的证据。

已固定 Hachimi-Edge 提交 `bbff85c25008c884fb6c7e825ed4e85044d8f5c3`：

- [interceptor.rs](https://github.com/kairusds/Hachimi-Edge/blob/bbff85c25008c884fb6c7e825ed4e85044d8f5c3/src/core/interceptor.rs)：
  Hook map 以 handler 为 key；安装返回 trampoline；unhook 从 map 移除后才调用底层恢复，
  恢复失败仍可能返回记录。因此不能仅用 API 返回值宣称回滚成功。
- [plugin_api.rs](https://github.com/kairusds/Hachimi-Edge/blob/bbff85c25008c884fb6c7e825ed4e85044d8f5c3/src/core/plugin_api.rs)：
  固定了此次核对的导出 API 原型，但这个提交尚未确认就是设备宿主版本。

UnityCsReference 固定提交 `88ce7b60434ba7a8ca0218590a4cb509971788ad` 的
[SendWebRequest](https://github.com/Unity-Technologies/UnityCsReference/blob/88ce7b60434ba7a8ca0218590a4cb509971788ad/Modules/UnityWebRequest/Public/UnityWebRequest.bindings.cs#L878)
和 [InvokeCompletionEvent](https://github.com/Unity-Technologies/UnityCsReference/blob/88ce7b60434ba7a8ca0218590a4cb509971788ad/Runtime/Export/Scripting/AsyncOperation.cs#L26)
分别支持 instance／零 managed 参数／引用返回和 instance／零 managed 参数／void 的
托管声明。它们不是目标游戏生成函数的原生签名证明。其他游戏方法目前也没有足够的
static、参数类型、返回类型和 hidden MethodInfo 证据；现有 callback 的类型只是待核验假设。

## 只读元数据诊断

`/api/hooks/abi` 仅返回采样线程已缓存的反射结果。HTTP 不执行 IL2CPP 读取、不调用
盘面 getter、不安装 Hook。采样线程先通过有限的 runtime-ready 检查，刷新函数再
检查 domain、当前线程附着和游戏程序集；宿主初始化回调是否收到单独显示为
`callback_received`，不会使遗漏回调的续养场景永远无法取证。

结果包含实际 declaring class、方法名、flags、implementation flags、static、参数
数量与类型、返回类型、generic、inflated、地址及解析错误。请求中的类和 arity 假设
单独列在 `requested`，不会伪装为实际读到的值。缺失值保持 null。
只枚举确定的 13 个目标，每个类最多扫描 4096 个方法，每个目标最多返回 4 个重载，
最多展开 8 个参数，元数据字符串限制为 256 字节。完整成功后缓存；失败或部分结果
间隔至少 30 秒重试，整个进程最多 3 次。元数据成功仍显示 `native_abi_verified:false`。

## 作者需回传的验证材料

1. 游戏完整版本、APK 与 libil2cpp 的 SHA256、Unity 版本、CPU ABI；设备使用的
   Hachimi 版本、精确源码提交或可核对的构建清单及宿主 SO SHA256。
2. `/api/hooks/abi` 原始 JSON，以及该次运行的 `/api/diagnostics/recovery`。
   这些诊断不需要上传账号、密钥、抓包原文或真实育成记录。
3. 对每个拟启用方法，提供与上述游戏二进制一致的生成 C++ 声明或可审查的反汇编／
   调用点证据，明确 instance/static、managed 参数类型、返回类型、隐藏 MethodInfo、
   泛型共享／值类型 thunk 以及寄存器和栈参数位置。不能只提供托管 arity。
4. 对匹配声明实现专用 forwarding thunk，验证原调用恰好一次、原参数和隐藏参数原样传递、
   观察关闭或异常时原返回值不变；再进行目标设备冷启动、开关嗅探及中途续养对照。

审核完成后才可增加编译期 profile；它必须绑定游戏/Unity/宿主/二进制/ABI、完整托管
签名、native argument layout、实现的 thunk ID 和证据 SHA256。生产匹配还要求实际构建
身份已观测、元数据完整且非歧义。无 profile、缺字段、版本变化、类型变化、generic/inflated
不受支持或 thunk 缺失均拒绝安装。不能用手工开关替代这些验证。

当前实际构建身份尚未接线，专用 thunk 也未实现。取得证据后仍需修改、构建和验证
这些实现；仅提交一份 profile 不能使本候选恢复真实嗅探。

## 自动化验证范围

运行 `python scripts/recovery/run_checks.py`。ABI 测试包含合成契约的成功匹配、未知或
变化的构建拒绝、字段/类型/static/隐藏布局不符拒绝、缺 profile/thunk 拒绝，以及提取
生产 `interceptor_hook` 的 spy：13 个 managed callback 均在 native hook、lookup、unhook
和 prologue 读取之前被拒绝。另验证无初始化回调时仍能缓存元数据及失败重试上限。
这些测试不替代实际设备的生成 ABI 和运行验证。
