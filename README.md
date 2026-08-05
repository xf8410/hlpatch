# hlpatch

**赛马娘 Android 本地运行时观测、协议分析与 IL2CPP 逆向插件**

`hlpatch` 运行于 Hachimi 插件环境，将游戏 IL2CPP 运行时数据、训练状态、事件观测、协议请求与响应、Hook 状态和调试结果通过本机 HTTP 服务提供给浏览器、`uma-juece` 与 Agora Workbench。

核心产物：

```text
libhachimi_ura.so
```

## 功能

### 育成状态

- 角色属性、体力、干劲、技能点、粉丝和回合
- 当前训练命令、训练收益、参与伙伴及羁绊
- 通用育成摘要和剧本专用数据
- 训练结果、行动记录和运行时差分

### 事件与剧本

- Story ID、角色 ID、事件 ID 和事件来源
- 事件选项、选择结果和时间顺序
- 拉面杯等剧本的数据集、地区、资源、效果和事务状态
- Hook 安装状态、调用观测和运行时日志

### 协议观测

- 请求路径、请求头、Cookie 和查询参数
- MessagePack 请求体与响应体
- 请求和响应原始十六进制
- 会话、账号、设备、版本、平台和操作遥测字段
- 请求—响应关联 ID 与时间顺序

### IL2CPP

- 类、字段和方法查询
- 静态字段与实例字段读取
- 方法地址、反汇编和运行时调用
- 进程内存、字符串、对象与资源文件读取
- MDB 表、Schema 和原始数据导出

## 数据原则

协议和运行时端点输出实际捕获到的原始字段和值。结构化解析作为附加结果，不替代原始请求、响应、字节、字段顺序和关联信息。读取受端点状态、缓冲区状态或工具响应范围限制时，以端点返回的实际结果为准。

## 版本

插件版本定义于：

```text
hachimi_ura_plugin/Cargo.toml
```

核对构建时使用：

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

## 架构

```text
赛马娘 Android 进程
  └─ Hachimi
      └─ libhachimi_ura.so
          ├─ IL2CPP 运行时读取与调用
          ├─ 育成、事件和剧本 Hook
          ├─ 协议请求与响应观测
          ├─ MDB、文件和内存端点
          └─ HTTP 服务 127.0.0.1:18765
              ├─ 浏览器
              ├─ uma-juece
              └─ Agora Workbench
```

主要源码：

```text
hachimi_ura_plugin/src/lib.rs
```

## HTTP 服务

默认地址：

```text
http://127.0.0.1:18765
```

基础状态：

```text
GET /health
GET /status
```

完整端点索引：

```text
hlpatch_endpoints.txt
```

主要类别：

- `/api/sniff/*`：请求、响应、请求头、正文和协议 Hook
- `/api/event/*`：事件选项和事件观测
- `/il2cpp/*`：类、字段、方法、内存、调用和反汇编
- `/debug/*`：运行时、Hook、MDB、资源和剧本诊断
- `/mdb/*`：MDB Schema、查询和原始数据
- `/summary`、`/data`、`/scenario`、`/ramen`：育成与剧本状态

## 构建

### GitHub Actions

```text
.github/workflows/build-ura.yml
```

输出：

- `libhachimi_ura.so`
- `SHA256SUMS`
- `BUILD-MANIFEST.txt`

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

发布工作流从指定源码构建版本产物，创建版本 tag 与 GitHub Release，并校验发布资产 SHA-256。已存在的同名 Release 不覆盖。

## 安装检查

1. 下载对应版本的 `libhachimi_ura.so`。
2. 按 Hachimi 插件目录结构放置文件。
3. 启动游戏。
4. 访问：

```text
http://127.0.0.1:18765/health
http://127.0.0.1:18765/status
```

5. 按任务读取对应的协议、IL2CPP、MDB、事件或剧本端点。

## 运行时验证

CI 成功表示源码完成 Rust/NDK 构建。Hook 地址、ABI、方法签名、字段布局和业务语义继续通过当前游戏构建的运行时结果验证。

数据为空时依次检查：

- 游戏是否处于对应场景
- 插件版本和游戏版本
- Hook 安装状态
- 对象链是否存在
- 端点返回的原始错误

## 目录

```text
hachimi_ura_plugin/   Rust 插件主体
.github/workflows/    构建与发布流程
config/               Hook 与 HTTP 配置
docs/                 研究记录和工作单
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
