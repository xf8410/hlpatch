# hlpatch

**赛马娘 Android 本地运行时观测、分层协议分析、IL2CPP 逆向与游戏内发送链研究插件**

`hlpatch` 运行于 Hachimi 插件环境，在游戏进程内观测 IL2CPP 对象、育成状态、事件、剧本数据以及网络请求与响应，并通过本机 HTTP 服务提供结构化查询、原始证据、Hook 诊断、持久归档和逆向研究接口。

它面向浏览器、`uma-juece`、Agora Workbench 与自动化研究工具。当前主动发包仍默认关闭；项目正在先识别和验证游戏原生业务 API、MessagePack、Header、压缩、发送及响应链，为后续受控的游戏内 `prepare/commit` 和无头协议客户端建立证据。

核心产物：

```text
libhachimi_ura.so
```

## 核心能力

### 育成状态与剧本观测

- 角色属性、体力、干劲、技能点、粉丝和回合
- 当前训练命令、训练收益、参与伙伴及羁绊
- 事件选项、选择结果、奖励和时间顺序
- 通用育成摘要及拉面杯等剧本专用数据
- 训练结果、行动记录、运行时快照与差分

### 分层协议观测与解码

- 请求路径、查询参数、请求头、响应头、Cookie 与 token
- MessagePack 请求体和响应体的结构化解析
- `request_plain`、`request_wire`、`response_wire`、`response_plain` 来源语义
- 明文边界与启发式候选分级，避免将密文或重复字节冒充解密正文
- 带前缀及 binary 字段中的嵌套 MessagePack/UTF-8 探测
- 无法完整解码时继续提取路径、类名、字符串和字节模式候选
- 原始 payload、完整 URL、Headers、关联 ID、时间线与持久会话归档

`wire` 是来源标签，不是解析禁令。Unity 上传/下载层的数据仍可用于嵌套协议、endpoint、类名和封装层研究；启发式结果不会冒充已验证明文。

### 游戏内发送链发现

- 展示已知原生链：`CompressRequest → WWWRequest.Post → UnityWebRequest → DecompressResponse`
- 从真实游戏流量枚举路由候选和最近 request ID
- 输出单次请求的明文捕获、发送边界、响应与风险证据
- 标记尚未解析的业务 API、formatter 和主线程调度环节
- 默认拒绝主动重放，不根据路径名称自动认定请求无副作用

当前端点只做被动发现，不会自行发送网络请求：

```text
GET /api/protocol/send/discovery
GET /api/protocol/send/candidates
GET /api/protocol/send/evidence?request_id=...
```

后续路线：

```text
真实游戏调用取证
→ 精确业务 API / formatter / 线程模型
→ 游戏内 prepare
→ 一次性 commit
→ status/result
→ 半无头与无头协议客户端研究
```

### IL2CPP 与本地数据

- 类、字段、方法和嵌套类型查询
- 静态字段与实例字段读取
- 方法地址、反汇编和受控运行时调用
- 进程内存、字符串、对象和资源文件读取
- MDB 表、Schema、查询和原始数据导出
- Hook 注册、安装状态、调用观测与错误诊断

## 数据与安全原则

1. **原始证据优先**：结构化解析是派生结果，不替代实际捕获的 URL、Headers、Cookie、字节、字段顺序和关联信息。
2. **来源不限制分析**：`wire` 与 `plain` 描述捕获边界，不决定是否允许搜索、递归探测或下载。
3. **不把候选冒充事实**：游戏明文边界标记为已验证；偏移扫描、嵌套探测和字符串识别标记为启发式候选。
4. **控制端点体积**：紧凑视图限制递归、容器、字符串和 hex 预览；完整原始数据通过归档和下载接口读取。
5. **主动发送默认关闭**：发送链发现阶段不产生新网络请求；未来 commit 必须使用显式确认、一次性凭据、hash 校验和审计记录。
6. **运行时结果为准**：CI 成功只证明源码可构建；Hook 地址、ABI、签名、字段布局和业务语义必须在当前游戏版本验证。

分层解码契约：

```text
docs/PROTOCOL_LAYERED_DECODE_CONTRACT.md
```

## 架构

```text
赛马娘 Android 进程
  └─ Hachimi
      └─ libhachimi_ura.so
          ├─ IL2CPP 运行时读取、方法定位与调用
          ├─ 育成、事件和剧本 Hook
          ├─ 请求/响应明文与 Unity transport 观测
          ├─ 分层解码、嵌套探测和协议归档
          ├─ 游戏原生发送链被动发现
          ├─ MDB、文件、资源和内存端点
          └─ HTTP 服务 127.0.0.1:18765
              ├─ 浏览器
              ├─ uma-juece
              └─ Agora Workbench
```

主要源码：

```text
hachimi_ura_plugin/src/lib.rs
hachimi_ura_plugin/src/signup_plaintext.rs
hachimi_ura_plugin/src/protocol_send_discovery.rs
```

仓库采用累计生成和补丁脚本构建部分发布功能；修改前应同时检查对应的 `scripts/apply_*.py` 与工作流。

## HTTP 服务

默认地址：

```text
http://127.0.0.1:18765
```

基础状态与诊断：

```text
GET /health
GET /status
GET /runtime/init_status
GET /hooks/registry
GET /hooks/diagnostics
GET /capture/status
```

协议入口：

```text
GET /api/sniff
GET /api/sniff/metadata
GET /api/sniff/signup_plaintext
GET /api/sniff/exchanges?session_id=...
GET /api/sniff/exchange?session_id=...&request_id=...
GET /api/protocol/send/discovery
GET /api/protocol/send/candidates
GET /api/protocol/send/evidence?request_id=...
```

完整端点索引：

```text
hlpatch_endpoints.txt
```

主要类别：

- `/api/sniff/*`：请求、响应、Header、正文、明文解码和协议 Hook
- `/api/protocol/send/*`：游戏原生发送链发现与请求证据；当前不主动发包
- `/storage/*`：持久会话、协议文件、审计、恢复和范围读取
- `/api/event/*`：事件选项和完成事件观测
- `/il2cpp/*`：类、字段、方法、内存、调用和反汇编
- `/hooks/*`、`/runtime/*`、`/capture/*`：初始化与 Hook/capture 状态
- `/debug/*`：运行时、Hook、MDB、资源和剧本诊断
- `/mdb/*`：MDB Schema、查询和原始数据
- `/summary`、`/data`、`/scenario`、`/ramen`：育成与剧本状态

除明确控制或未来发送接口外，当前观测端点主要使用 GET。接口是否可用及参数要求以运行时返回和当前源码为准。

## 构建

### GitHub Actions

标准构建：

```text
.github/workflows/build-ura.yml
```

专项功能还可能通过对应的 build/release 工作流先执行累计生成器和补丁脚本。输出通常包括：

- `libhachimi_ura.so`
- `SHA256SUMS`
- `BUILD-MANIFEST.txt`
- Cargo 构建日志

### 本地构建

需要 Rust、Android NDK r26c 和 ARM64 Android target：

```bash
rustup target add aarch64-linux-android
cd hachimi_ura_plugin
cargo build --locked --release --target aarch64-linux-android
```

产物：

```text
hachimi_ura_plugin/target/aarch64-linux-android/release/libhachimi_ura.so
```

如果目标功能由累计生成器产生，应先按相应工作流执行生成和补丁步骤，不能将仓库中的基础 `lib.rs` 误认为最终发布源码。

## 版本与发布核验

插件包版本定义于：

```text
hachimi_ura_plugin/Cargo.toml
```

发布核验使用：

- Git tag
- source commit
- Cargo version
- Actions run ID
- 文件大小
- SHA-256

Release 附带：

```text
libhachimi_ura.so
SHA256SUMS
BUILD-MANIFEST.txt
```

发布工作流从指定源码构建产物，创建 tag 与 GitHub Release，并校验发布资产。已存在的同名 Release 不覆盖。

## 安装与运行时检查

1. 下载对应版本的 `libhachimi_ura.so`。
2. 按 Hachimi 插件目录结构放置文件。
3. 启动游戏。
4. 访问：

```text
http://127.0.0.1:18765/health
http://127.0.0.1:18765/status
```

5. 检查 Hook 和采集状态：

```text
/runtime/init_status
/hooks/registry
/hooks/diagnostics
/capture/status
/api/sniff/diag
```

6. 正常操作游戏产生对应场景或协议流量，再读取目标端点。

数据为空时依次检查：

- 游戏是否处于对应场景；
- capture 是否启用；
- 核心 Hook 是否已安装并实际命中；
- 插件、游戏和资源版本是否匹配；
- 对象链或请求 ID 是否仍在内存缓冲；
- 持久 session 是否已经 flush；
- 端点返回的原始错误和诊断字段。

## 目录

```text
hachimi_ura_plugin/   Rust 插件主体
scripts/              累计生成与功能补丁脚本
.github/workflows/    构建、审计与发布流程
config/               Hook 与 HTTP 配置
docs/                 接口契约、研究记录和工作单
reverse/              逆向索引与报告
data/                 项目数据和辅助资料
native/               原生实验代码
third_party/          第三方依赖
```

## 关联项目

- `hlpatch`：SO、Hook、IL2CPP、协议观测、运行时缓存和 HTTP 服务
- `uma-juece`：浮窗与决策客户端
- `uma-data`：数据仓库
- `uma-train`：模拟与训练项目
- `uma-ai-context`：项目上下文归档
