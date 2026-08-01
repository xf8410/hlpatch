# hlpatch

**赛马娘 Android 本地运行时观测与诊断插件**

`hlpatch` 是运行在 Hachimi 插件环境中的 ARM64 Android 原生插件。它在用户自己的设备上连接游戏 IL2CPP 运行时，将经过约束的育成状态、事件观测、Hook 诊断和协议元数据通过本机 HTTP 服务提供给调试页面、`uma-juece`、Agora Workbench 等本地客户端。

本仓库当前的核心产物是：

```text
libhachimi_ura.so
```

> 本项目是持续演进的个人研究与实机诊断工程，不是官方插件，也不是通用游戏修改框架。运行时结构会随游戏版本变化；源码可编译不代表所有 Hook 已通过当前游戏版本的实机验证。

## 项目定位

hlpatch 目前承担四类工作：

1. **育成状态观测**
   - 读取角色属性、体力、干劲、技能点、粉丝、回合和训练候选等本地运行时状态。
   - 提供通用育成摘要及部分剧本专用的有界快照。

2. **事件与剧本诊断**
   - 记录事件选项、选择后的时间关联观测和 Hook 安装状态。
   - 为拉面杯等剧本保留专用的结构化观测，但不把候选映射或时间相关性冒充为已证实因果。

3. **IL2CPP 定点自省**
   - 对明确指定的类名、字段名或方法名进行目标化查询。
   - 用于游戏升级后的结构核对、字段链验证和低风险诊断。

4. **本地协议观测**
   - 维护有界请求/响应缓存和脱敏元数据。
   - 支持请求路径、方向、大小、时间和本地关联 ID 等诊断信息。
   - 原始正文可能包含账号或会话敏感信息，不应上传到聊天、GitHub 或公开日志。

## 当前版本与真实性来源

Rust crate 的当前版本定义在：

```text
hachimi_ura_plugin/Cargo.toml
```

当前 main 显示：

```text
3.25.1
```

但版本号、main 源码、Actions Artifact 和 GitHub Release 是四个不同概念：

- main 提交代表当前源码；
- `Build Hachimi URA Plugin` 生成该提交的测试 Artifact；
-版本标签固定某个不可变源码提交；
- `Release Hachimi URA Plugin` 只从已存在的版本标签重新构建并发布不可覆盖的 Release。

安装或比较 SO 时，应同时核对：

- Git tag
- `source_commit`
- Cargo version
- 文件大小
- SHA-256
- Actions run ID

Release 中的 `BUILD-MANIFEST.txt` 和 `SHA256SUMS` 是二进制来源的主要依据。不要仅凭文件名或“Latest”标记判断新旧。

## 架构

```text
赛马娘 Android 进程
  └─ Hachimi 插件宿主
      └─ libhachimi_ura.so
          ├─ IL2CPP 定点读取与运行时缓存
          ├─ 事件/剧本 Hook 与有界观测环
          ├─ 协议 Hook、脱敏元数据和诊断
          ├─ 本机 HTTP 服务 127.0.0.1:18765
          └─ 可选本地客户端
              ├─ 手机浏览器（人工诊断）
              ├─ uma-juece
              └─ Agora Workbench
```

主要源码：

```text
hachimi_ura_plugin/src/lib.rs
```

当前源码规模较大且仍偏单体化。后续应按 HTTP 路由、IL2CPP、自省、育成摘要、事件、协议、剧本和发布元数据逐步拆分模块；README 不再把历史版本积累的每个字段偏移当成永久 API 文档。

## 本机服务

默认地址：

```text
http://127.0.0.1:18765
```

基础检查：

```text
GET /health
GET /status
```

常用的低风险、有界接口类别：

- 育成摘要和当前状态
- 已完成事件观测
- Hook 诊断
- 协议脱敏元数据
- 明确类名的字段或方法查询

具体路由以当前源码和 `/health` 返回的版本为准。游戏版本升级后，应先检查健康状态和 Hook 诊断，再判断某项数据是否可信。

## 安全分级

### 日常可用

- `/health`、`/status`
- 有界摘要和结构变化
- 已完成事件观测
- Hook 安装诊断
- 脱敏协议元数据
- 明确类名的定点只读查询

### 仅人工诊断

仓库中仍可能保留原始 sniff、配置写入、文件诊断、较大响应或实验性探针。它们不应默认交给 AI，也不应在不了解参数和数据范围时调用。

### 禁止常规使用

- 全量 IL2CPP 类扫描
- 全量 singleton 或大型对象遍历
- 任意裸内存读取
- 任意方法调用
- 未知 getter 自动执行
- 递归对象 dump
- 将原始请求、响应、认证头、Cookie、SID、Token 或私有文件上传到 GitHub/聊天

这些能力即使在历史源码中存在，也不代表是推荐接口。后续重构目标是将危险能力显式隔离、默认关闭，并让服务在版本或参数不明确时 fail closed。

## 协议数据说明

协议观测分为两层：

### 脱敏元数据

可用于普通诊断，典型字段包括：

- 本地 observation ID
- request ID
- 时间
- 请求或响应方向
- 去除 query 的路径
- 大小

### 原始缓存

可能包含：

- 请求/响应正文
- 账号标识
- 会话信息
- 设备信息
- 其他认证数据

原始缓存仅允许用户在本机明确打开并短时检查。不得复制到 Issue、README、AI 对话、Actions 日志或上下文归档。

## IL2CPP 查询原则

推荐流程：

1. 先使用同一游戏构建的离线索引确定精确候选类名。
2. 在手机运行时只搜索具体关键词或查询一个明确类。
3. 分开读取字段和方法，不自动调用方法。
4. 对结果标记来源和游戏版本。
5. 对大类设置条目上限，禁止整类大输出进入聊天。

必须区分：

- 找到类名
- 找到字段/方法元数据
- 成功安装 Hook
- 实机获得合理值
- 已验证业务语义

前四项中的任何一项都不能自动证明最后一项。

## 构建

### GitHub Actions

主工作流：

```text
.github/workflows/build-ura.yml
```

它会从精确检出的源码构建 ARM64 Android Release，并上传：

- `libhachimi_ura.so`
- `SHA256SUMS`
- `BUILD-MANIFEST.txt`

Artifact 名包含完整 commit SHA。普通 main 或手动构建不会修改 GitHub Release。

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

## 发布

发布工作流：

```text
.github/workflows/release-ura.yml
```

当前规则：

1. 先创建与 Cargo version 一致的不可变 tag，例如 `v3.25.2`。
2. 发布工作流只检出该 tag。
3. 从标签源码重新构建。
4. 如果同名 Release 已存在则拒绝覆盖。
5. 发布后验证 GitHub 资产 digest 与本地 SHA-256 一致。

旧版本曾出现源码、标签和 Release 二进制不一致的问题，因此禁止恢复“同 tag 覆盖上传”或可变 Latest 二进制流程。

## 安装与验证

1. 从本仓库对应版本 Release 下载 `libhachimi_ura.so`、`SHA256SUMS` 和 `BUILD-MANIFEST.txt`。
2. 在手机本地核对 SHA-256。
3. 按 Hachimi 当前插件目录规范放置 SO；升级前保留旧版本备份。
4. 启动游戏后访问：

```text
http://127.0.0.1:18765/health
```

5. 核对运行版本，再检查：

```text
http://127.0.0.1:18765/status
```

6. 如涉及 Hook，必须查看对应诊断状态；不能只根据 HTTP 服务已启动判断 Hook 成功。

## 常见问题

### HTTP 无法连接

- 确认游戏进程仍在运行。
- 确认 Hachimi 已加载插件。
- 确认当前配置端口为 18765。
- 服务默认只供本机访问，不应暴露到局域网或公网。

### 数据为零或为空

- 当前可能不在育成场景。
- 游戏升级后类名、方法或对象链发生变化。
- Hook 未安装成功。
- 数据仍处于 unknown，不能为了显示而填入猜测值。

### 构建成功但实机无效

CI 成功只证明 Rust/NDK 构建通过，不证明：

- ABI 假设正确
- Hook 地址正确
- 方法签名正确
- 当前游戏版本兼容
- 业务语义已经验证

需要结合 `/health`、`/status`、Hook 诊断和有界实机样本判断。

### 文件看起来还是旧版本

不要只看 SO 文件名。核对 Release tag、manifest 中的 source commit、文件 SHA-256 和运行时 `/health` 版本。

## 仓库目录

```text
hachimi_ura_plugin/   Rust 插件主体
.github/workflows/    构建与不可变发布流程
config/               本地 Hook/HTTP 示例配置
docs/                 历史研究记录和工作单
reverse/              有界逆向索引与报告
data/                 项目数据和辅助资料
native/               原生实验代码
third_party/          第三方依赖
```

历史资料可能包含已过期假设。源码当前行为、不可变构建清单和带证据的最新项目归档优先级更高。

## 关联项目边界

- `hlpatch`：只负责 SO、Hook、IL2CPP、运行时缓存和本机服务。
- `Agora-Workbench`：属于独立 Android 客户端项目；其对话、Token、白屏、GitHub 工具、APK 和 Release 问题不得写入本仓库的 SO 技术文档。
- `uma-juece`：独立浮窗/决策客户端。
- `uma-data`：独立数据仓库。
- `uma-train`：独立模拟与训练项目。
- `uma-ai-context`：上下文归档；hlpatch 内容应放在 `projects/uma-so/`。

跨项目集成必须分别记录两侧实现，不能再把 Agora 客户端故障和 SO 运行时问题写进同一归档文件。

## 隐私与免责声明

- 仅在你有权测试的设备、账号和环境中使用。
- 不提供生产服务任意发包、认证绕过、凭据导出或规避安全机制的使用指导。
- 插件运行在游戏进程中，错误的 Hook、ABI 或指针读取可能导致崩溃和数据损坏；升级前务必备份并保留可回退版本。
- 不要公开原始协议正文、认证数据、私有文件或崩溃日志中的敏感内容。

本 README 描述当前魔改项目的真实定位、构建来源和安全边界，不再把历史字段偏移、全量扫描端点或旧版功能清单当作永久承诺。