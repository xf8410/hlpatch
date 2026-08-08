# 下一代 SO：v3.27.11 源码锚点、Hook 占用审计与接口契约

日期：2026-08-08  
仓库：`xf8410/hlpatch`  
审计基线：`main@73748b84dbecec19636c6af6e12d95bcf5f13b03`  
已发布运行时基线：`v3.27.11`，发布功能源码由累计生成器产出

## 1. 本阶段边界

本阶段只冻结下一代 SO 的源码入口、现有 Hook 占用、已确认冲突风险和外部接口契约，不修改 Hook 回调、采集时点、协议关联、队列、持久化或 UI 业务逻辑。

后续实现必须继续满足：

- SO 自动管理 Hook 安装；用户只控制是否记录。
- capture 关闭时 Hook 仅调用原方法，不复制载荷、不解析、不落盘。
- 原始协议保持实际捕获的完整 URL、Headers、Cookie、请求体和响应体；派生解析不能替代原始字节。
- 不增加第二套 Compress、Decompress、Post、Unity Send 或 Completion Hook。
- 先对现行同步链取得设备压力测试原始指标，再决定是否迁移到分块池、有界队列和单写入 worker。

## 2. 构建与发布基线

### 2.1 仓库源码不是已发布 SO 的直接单步输入

当前 `main` 中：

- `hachimi_ura_plugin/Cargo.toml` 为 `3.27.9`；
- `hachimi_ura_plugin/Cargo.lock` 中 `hachimi_ura` 为 `3.27.4`；
- `scripts/run_generated_succession_l_cumulative.py` 会把两者提升到 `3.27.11`，再依次应用 A～P 累计补丁；
- v3.27.11 Release 工作流固定从功能源码提交 `07adec9d24d721b1341191f8ffa8e15bc88ab931` 构建，并先运行上述累计生成器。

因此，下一代功能不能把未生成的 `main/hachimi_ura_plugin/src/lib.rs` 直接误认成 v3.27.11 最终源码。所有补丁锚点必须针对“累计生成后源码”验证，并执行两遍幂等检查。

### 2.2 当前标准 build 工作流基线未通过

`Build Hachimi URA Plugin` Run `31250258657` 在 `main@73748b84...` 的 `Build exact checked-out source` 步骤失败。当前工具返回了失败步骤，但没有原始 Cargo 日志，故失败原因不在本文中推断。

静态审计另行确认 `Cargo.toml` 与 `Cargo.lock` 的包版本不一致，而该步骤使用 `cargo build --locked`。开始功能实现前必须先恢复唯一活动分支上的可重复构建基线，不能把既有失败混入新功能 CI 结果。

## 3. 现有 Hook 占用清单

| 域 | 当前目标 | 当前安装入口 | 当前记录门控 | 已确认边界 |
|---|---|---|---|---|
| 协议请求 DTO | `Gallop.HttpHelper.CompressRequest(byte[])` | `install_api_sniff_hooks` | `SNIFF_ENABLED` 只门控部分记录逻辑 | 已占用，禁止新增第二 Hook |
| 协议响应 DTO | `Gallop.HttpHelper.DecompressResponse(byte[])` | `install_api_sniff_hooks` | `SNIFF_ENABLED` | 已占用，禁止新增第二 Hook |
| HTTP Post | `Cute.Http.WWWRequest.Post(...)` | `install_api_sniff_hooks` | `SNIFF_ENABLED` | 已占用，当前请求关联含全局 pending 状态 |
| Unity 请求入口 | `UnityWebRequest.SendWebRequest()` | `install_api_sniff_hooks` | `SNIFF_ENABLED` | 已占用 |
| Unity 完成入口 | `AsyncOperation.InvokeCompletionEvent()` | `install_api_sniff_hooks` | 由累计 H 补丁接入 | 已占用；用于响应 Headers 关联 |
| 摘要散列 | `Cryptographer.MakeMd5`、`ComputeHash` | `install_api_sniff_hooks` | 独立内存记录 | 已占用 |
| UI 文本 | `Gallop.TextCommon.set_text(string)` | N-stage 从 `install_api_sniff_hooks` 内重试 | 当前没有独立 UI capture 门控 | 不能继续作为 Scenario 14 唯一入口 |
| 事件选项 | `StoryChoiceController.Choice`、`AddChoiceButton` | `install_event_choice_hook` | 独立事件内存状态 | 已占用 |
| 剧情切换 | `StoryManager.SetStory` | `install_event_choice_hook` | 独立事件内存状态 | 已占用 |
| 训练 | `install_training_hook`、`install_exec_training_hook`、`install_failure_rate_hook` 所解析目标 | 初始化回调和 fallback probe | 各自旧状态 | 已占用，需在注册表中纳管 |

## 4. 当前安装模型的结构性缺口

### 4.1 Hook 安装与 capture 没有彻底分离

`SNIFF_ENABLED` 当前默认 `true`；`/api/sniff/toggle` 同时触发 Hook 安装重试并改变记录开关。下一代契约必须拆成：

```text
hooks_installed = SO 后台状态机管理
capture_enabled = 用户控制是否生成记录
```

关闭 capture 不得卸载 Hook，也不得改变 trampoline；Hook 回调只转发原调用。

### 4.2 安装目标未按完整签名验证

现有代码大量使用：

```text
assembly + namespace + class + method name + parameter count
```

并保留 `find_class_fuzzy`、`find_method_fuzzy` 回退。该模型不能证明重载的参数类型和返回类型唯一，也不能识别程序集 generation 变化后的旧地址。

下一代 HookKey 固定为：

```text
assembly
namespace
declaring_type（含嵌套类型链）
method
parameter_types（有序完整类型名）
return_type
```

### 4.3 没有统一所有权与冲突判定

现有 `interceptor_hook` 只依据 Interceptor 返回值判断成功；`install_hook_safe` 在 Interceptor 失败后会复制 16 字节 prologue 并调用 `write_hook_bytes`。当前没有统一保存原始/当前 prologue、目标模块、MethodInfo、trampoline、owner 或 generation，也没有在入口已被其他插件改写时拒绝二次覆盖。

后续所有新安装必须经过：

```text
Resolve → Validate → Commit
```

Validate 失败时只记录精确目标与原始错误，不进入直接改写回退。

### 4.4 安装重试分散

现有安装来自：

- `on_game_initialized`；
- `push_loop` 的 summary fallback probe；
- `/api/sniff/toggle`；
- 个别读取端点的惰性重试。

下一代必须由单一后台状态机驱动，HTTP 读取或 capture 开关不能承担“使 Hook 最终装上”的必要职责。

### 4.5 capture 关闭契约目前不覆盖全部观察者

协议回调多处检查 `SNIFF_ENABLED`，但 TextCommon、事件、训练和 MD5 观察各自使用独立状态或无 capture 门控。后续需建立统一 capture policy，并按运行模式明确哪些观察域启用；不能因某一域关闭而改变已经捕获的协议原始数据语义。

## 5. 当前同步采集与压力测试锚点

以下现行路径必须先保留并测量，取得原始指标前不做队列化重构：

1. `read_il2cpp_byte_array` 在 Hook 现场复制 IL2CPP byte[]。
2. `persist_protocol_capture` 依次写 `url.txt`、`headers.raw`、`payload.bin`，每个临时文件执行 `sync_data` 后 rename。
3. SQLite `observation_files` 索引在原始文件提交后写入。
4. `append_global_observation` 同步追加 `timeline.ndjson`；critical 记录执行 `sync_data`。
5. O-stage 在协议提交路径中同步递归扫描 MessagePack 并写派生时间线。
6. N-stage 在 `TextCommon.set_text` 原方法返回后立即调用同对象 `get_text`，再同步写 UI 记录。

这些锚点是压力测试的计时边界。测试至少记录 Hook 回调总耗时、IL2CPP 数据复制耗时、三文件写入与每次 `sync_data`、SQLite 事务、MessagePack 派生解析、timeline 写入、每分钟字节数、RSS/PSS、CPU、帧时间和 HTTP 可达性。

## 6. 已确认的数据完整性/关联缺口

### 6.1 旧内存展示层仍存在固定容量

源码保留：

- `SNIFF_RAW_MAX=50`；
- `SNIFF_METADATA_MAX=1000`；
- `EVENT_OBSERVATIONS_MAX=16`；
- `EVENT_RESPONSE_PREVIEW_MAX=16 KiB`；
- TextCommon 字符串读取长度上限 4096 UTF-16 code units；
- Unity 观察在内存中长期保存 `body_hex`。

其中 v3.27.11 的持久协议文件已走完整 payload 路径，但内存展示、事件预览和 UI 文本仍需分别标明实际范围，不能把预览或环形缓存当成完整持久事实。下一阶段审计必须逐处区分“历史调试展示”和“原始持久层”，不得通过删除原始数据解决内存压力。

### 6.2 请求和响应仍含全局 pending/FIFO 关联

现有结构包括：

```text
PENDING_REQ_BODY
PENDING_COMPRESSED
PENDING_REQ_ID
PENDING_URL
SNIFF_RESPONSE_QUEUE
```

这不能作为同路径并发下的唯一对象关联。H-stage 已加入 `AsyncOperation → UnityWebRequest → GetResponseHeaders` 观察，但 Exchange 仍需以请求对象、AsyncOperation 和 UnityWebRequest 身份贯穿，而非路径 FIFO。

### 6.3 TextCommon 只能证明一次 setter 调用边界

N-stage 保存：

```text
输入文本 → 原 set_text 返回 → 同组件 get_text
```

它尚不能证明帧末稳定显示，也覆盖不了 Prefab 初始固定文本、BitmapTextCommon、ImageCommon/RawImageCommon 图片文字和资源来源。Scenario 14 UI 后续应优先页面业务生命周期与组件身份，再把 TextCommon 作为其中一种文本事件。

## 7. 冻结接口契约

### 7.1 初始化状态

固定状态枚举：

```text
waiting_module
waiting_domain
waiting_assemblies
probing_core_types
installing_core_hooks
installing_optional_hooks
ready
degraded
```

`ready` 仅在全部核心 Hook 已验证安装后成立；可选 Hook 失败进入 `ready` 时必须同时列出 optional failure。核心 Hook 失败为 `degraded`，不得只因 HTTP 可达或任一 Hook 成功而报告 ready。

建议只读端点：

```text
GET /runtime/init_status
GET /hooks/registry
GET /hooks/diagnostics
```

### 7.2 HookRegistry 记录

每个目标保存：

```text
hook_id
role = core | optional
HookKey 完整签名
method_info
target_address
module_name
module_base
module_generation
mapping_permissions
alignment_valid
original_prologue_bytes
current_prologue_bytes
prologue_fingerprint
external_hook_present
owner
install_generation
install_state
trampoline_address
call_count
last_call_monotonic_ns
last_error_stage
last_error_raw
```

`external_hook_present=true` 且无法证明宿主 Interceptor 支持安全链式注册时，安装结果固定为拒绝提交，不调用 `write_hook_bytes`。

### 7.3 capture 控制

建议端点：

```text
GET  /capture/status
POST /capture/set?enabled=true|false
```

返回至少包含：

```text
hooks_installed
capture_enabled
active_mode
capture_generation
change_sequence
```

capture 从开关后的下一次真实调用开始；关闭前已可靠提交的记录保持不变。

### 7.4 版本指纹

初始化状态绑定：

```text
plugin_version
source_commit
game_version
resource_version
loaded_module build-id / SHA-256（实际可取得者）
assembly name + image identity + generation
MDB SHA-256
hook_registry_schema_version
observation_schema_version
```

无法取得的字段为 `null + 原始错误`，不得填默认值。

### 7.5 状态迁移记录

每次迁移保存：

```text
global_sequence
monotonic_ns
wall_clock_ms
from_state
to_state
probe_or_hook_id
attempt
raw_result
```

状态机只做解析、验证和安装，不开启 capture。

## 8. 下一最小实施单元

在同一分支先恢复标准 Android arm64 可重复构建基线，并保存该 SHA 的 CI 终态；随后只实现“初始化状态数据结构、只读状态端点和 HookRegistry 数据模型”，暂不接管任何现有 Hook 安装调用。这样可先验证接口和线程安全，再单独迁移一个核心 Hook 进入 `Resolve → Validate → Commit`。