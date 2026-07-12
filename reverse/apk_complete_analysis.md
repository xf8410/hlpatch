# APK 完整分析报告

**分析日期**: 2026-07-12
**APK**: base.apk (107MB) + split_config.arm64_v8a.apk (77MB)

---

## APK 文件结构

### base.apk (107MB, 3801个文件)
| 目录 | 文件数 | 大小 | 内容 |
|------|--------|------|------|
| assets/bin/Data/ | 3,726 | 309.8MB | Unity 资源 (hash命名的 .dat) |
| assets/PreinResource/Movie/ | 1 | 34.1MB | 标题视频 (CriWare USM) |
| assets/PreinResource/Sound/ | 5 | 4.0MB | 预加载音频 (BGM/语音/SE) |
| assets/bin/Data/Managed/ | 3 | 1.5MB | DLL resources.dat |
| (root) | 5 | 8.6MB | classes.dex + AndroidManifest + resources.arsc |
| META-INF/ | 53 | <1MB | 签名文件 |

### split_config.arm64_v8a.apk (77MB, Native库)
| 库 | 大小 | 用途 | 已扒 |
|---|---|---|---|
| libil2cpp.so | 209MB | IL2CPP 游戏代码 | ✅ |
| libunity.so | 18MB | Unity 引擎 | ❌ |
| libnative.so | 2.5MB | Cygames native (curl+sqlite+nghttp2+mbedTLS) | ✅ |
| libcyspringandroid.so | 15KB | Cygames 布料物理 (裙摆模拟) | ✅ |
| libmain.so | 6.7KB | Unity 启动入口 | ✅ |
| libcri_ware_unity.so | 2.6MB | CriWare 音频引擎 | ❌ |
| libcriafx_soundxr.so | 5.7MB | CriWare 音效 | ❌ |
| libcrifs_web_installer_curl.so | 5.6MB | CriWare 文件系统 | ❌ |
| libcri_mana_dav1d.so | 740KB | AV1 视频解码 | ❌ |
| libcri_mana_vpx.so | 461KB | VP9 视频解码 | ❌ |
| libcri_lips_unity.so | 1MB | 唇形同步 | ❌ |
| libcriafx_mcdsp.so | 1.1MB | 音频 DSP | ❌ |
| lib_burst_generated.so | 95KB | Unity Burst | ❌ |
| Firebase/Crashlytics | ~10MB | Firebase 服务 | ❌ |

---

## classes.dex 分析

8,227 个类 — 全是 Android 框架层代码：
- 6,127 个混淆类 (default package)
- androidx.* (1,800+)
- com.google.android / com.facebook / com.adjust.sdk (各种第三方SDK)

**没有 Cygames 游戏逻辑** — 游戏代码全在 C# (libil2cpp.so) 里。

关键 Java 层组件：
- 包名: `jp.co.cygames.umamusume`
- Activity: `OMOTENASHI_PUSH_LAUNCH_ACTIVITY`
- Provider: `FacebookInitProvider`, `firebaseinitprovider`
- Receiver: `AdjustReferrerReceiver`, `FacebookAuth/TokenManager`
- 权限: INTERNET, READ/WRITE_EXTERNAL_STORAGE, INSTALL_PACKAGES

---

## libnative.so 分析 (2.5MB)

4,496 个函数, 4,863 条字符串。

**这不是游戏逻辑库** — 它是 Cygames 自研的网络/数据库工具库：

### 功能模块
1. **libcurl** — HTTP/HTTPS 请求 (WebSocket, HTTP/2, nghttp2)
2. **SQLite 3** — 嵌入式数据库 (master.mdb 读取器)
3. **mbedTLS** — TLS/SSL 加密 (证书验证, 签名)
4. **nghttp2** — HTTP/2 协议栈
5. **PKI/Crypto** — X509 证书, PKCS7/12, RSA/ECDSA 签名

### 关键发现
- 完整的 SQLite3 引擎 → master.mdb 读取用这个
- 完整的 curl + WebSocket → 服务器通信用这个
- mbedTLS → 通信加密用这个
- 有 `IntegrityCk` / `integrity_check` / `hmac_check` → 可能是反篡改
- 有 `Object Signing` / `Code Signing` → 代码签名验证
- 无游戏逻辑 (无 training/umamusume/gallop 字符串)

---

## libcyspringandroid.so 分析 (15KB)

6 个函数, 45 条字符串。

**Cygames 布料物理引擎** — 裙摆/衣物物理模拟：
- `NativeClothUpdate` — 布料更新
- `NativeSkirtUpdate` — 裙摆更新
- `NativeClothSkirtUpdate` — 衣裙联合更新
- `NativeClothUpdateInternal` — 内部计算
- `_UpdateSkirtNativePluginCalcRotAngle` — 裙摆旋转角度计算

用 NDK r29 (clang 21.0.0) 编译, 启用了 PGO+BOLT+LTO+MLGO 优化。

---

## libmain.so 分析 (6.7KB)

Unity 启动入口：
- `JNI_OnLoad` → 加载 libunity.so
- `com/unity3d/player/NativeLoader` → Unity Native Loader
- 加载后由 Unity 引擎加载 libil2cpp.so

---

## Unity 资源 (assets/bin/Data/)

### 配置文件
- `boot.config` — Unity 启动配置
  - `gc-max-time-slice=3`
  - `androidStartInFullscreen=1`
  - `build-guid=3379d72d3f2b4379b812c226611af66b`

- `ScriptingAssemblies.json` — 136 个 DLL 列表
  - umamusume.dll (主游戏)
  - umamusume.Http.dll (HTTP通信)
  - Cute.Cri.Assembly.dll (CriWare)
  - Cute.Http.Assembly.dll (HTTP)
  - MessagePack.dll (序列化)
  - _Cyan.dll (可能是反作弊)
  - DebugHook.dll (调试钩子)

- `globalgamemanagers` (69.5MB) — Unity 全局配置/Shader/设置
- `sharedassets*.assets` (32个文件) — 场景资源

### Hash命名的资源文件 (3,726个)
Unity AssetBundle — 内容是贴图/模型/音频/文本等编译后资源。
需要 Unity 运行时或 AssetStudio 工具才能解析内容。

---

## APK 中没有找到的

1. **global-metadata.dat** — 不在 base.apk 里，在运行时从服务器下载或由 libil2cpp.so 解密
2. **master.mdb** — 不在 APK 里，运行时从服务器下载
3. **游戏逻辑 Java 代码** — 全在 C# (libil2cpp.so) 里
4. **反作弊系统** — _Cyan.dll 可能是，但 DLL 在运行时由 IL2CPP 加载，不在 APK 静态文件中

---

## 还能继续扒的

1. **libunity.so (18MB)** — Unity 引擎，包含渲染/物理/音频系统
2. **CriWare 库 (libcri*.so, ~11MB)** — 音频/视频解码
3. **Unity AssetBundle (3,726个 hash 文件)** — 需要专用工具解析
4. **globalgamemanagers (69.5MB)** — Unity 序列化文件，含 Shader 和配置

*分析: 2026-07-12*
