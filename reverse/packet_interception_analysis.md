# 服务器请求拦截分析

**分析日期**: 2026-07-12

---

## 一、通信架构

```
游戏客户端
    │
    ├─ Cute.Http.Assembly.dll (Cygames 自研 HTTP 库)
    │   ├─ WWWRequest.Post(url, data, headers)  ← 发送请求
    │   ├─ CompressRequest(byte[]) → byte[]     ← 请求体压缩
    │   └─ DecompressResponse(byte[]) → byte[]  ← 响应体解压
    │
    ├─ libnative.so (Native 通信层)
    │   ├─ curl (HTTP/HTTPS 客户端)
    │   ├─ mbedTLS (TLS 加密)
    │   ├─ nghttp2 (HTTP/2)
    │   └─ SQLite (master.mdb 读取)
    │
    ├─ MessagePack.dll (序列化)
    │   └─ 请求/响应体用 MessagePack 编码
    │
    └─ Gallop.SingleModeAPI
        ├─ SendExecCommand (训练请求)
        ├─ SendCheckEvent (检查事件/获取训练结果)
        ├─ SendLoad (加载存档)
        ├─ SendStart (开始育成)
        └─ SendFinish (结束育成)
```

## 二、当前拦截能力（已实现）

### 2.1 三个 Hook 点

| Hook 点 | 函数 | 能抓到什么 | 状态 |
|---------|------|-----------|------|
| CompressRequest | `Cute.Http.WWWRequest::CompressRequest(byte[])` | 请求体(压缩前) | ✅ 已实现 |
| DecompressResponse | `Cute.Http.WWWRequest::DecompressResponse(byte[])` | 响应体(解压后) | ✅ 已实现 |
| WWWRequest.Post | `Cute.Http.WWWRequest::Post(url, data, headers)` | URL + Headers | ✅ 已实现 |

### 2.2 拦截流程

```
1. 游戏调用 CompressRequest(body) → 压缩请求体
   ★ Hook 拦截: 保存原始(未压缩)请求体
   
2. 游戏调用 WWWRequest.Post(url, compressedBody, headers)
   ★ Hook 拦截: 保存 URL + Headers
   ★ 用时间戳匹配 CompressRequest 保存的请求体
   
3. 服务器返回响应 → 游戏调用 DecompressResponse(compressedResp)
   ★ Hook 拦截: 保存解压后的响应体
   
4. /api/sniff 端点返回所有抓到的 (URL, Headers, Request, Response)
```

### 2.3 API 端点

| 端点 | 功能 |
|------|------|
| `/api/sniff/toggle` | 开关拦截 |
| `/api/sniff/clear` | 清空缓冲区 |
| `/api/sniff` | 获取所有抓到的请求/响应 |
| `/api/sniff/diag` | 诊断(hook状态/地址) |

## 三、能不能拦包？—— 能

### 3.1 已经能拦的
- ✅ **请求 URL** — WWWRequest.Post 的第一个参数
- ✅ **请求 Headers** — WWWRequest.Post 的第三个参数
- ✅ **请求体(明文)** — CompressRequest 拦截压缩前数据
- ✅ **响应体(明文)** — DecompressResponse 拦截解压后数据

### 3.2 拦截限制
- ⚠️ **HTTPS 内容**：curl+mbedTLS 做了 TLS 加密，但我们 hook 的是 **C# 层**（加密前/解压后），所以 **不需要破解 TLS**
- ⚠️ **URL 动态拼接**：URL 不在 SO 静态字符串中，运行时拼接。但 WWWRequest.Post 的参数就是完整 URL
- ⚠️ **MessagePack 编码**：请求/响应体是 MessagePack 格式，需要反序列化才能读懂
- ⚠️ **缓冲区限制**：SNIFF_MAX=20，最多保存20条记录

### 3.3 SSL Pinning 情况
- libnative.so 有 mbedTLS 完整套件
- 搜索 `pinning`/`pinned`/`SSL_PINNED` → **未找到明确的证书固定代码**
- curl 的 `CURLOPT_SSL_VERIFYPEER` 可能启用（默认 true）
- **但我们不需要绕过 SSL** — 因为 hook 在 C# 层，TLS 加密前

## 四、关键发现

### 4.1 透视原理
训练结果通过 `SendCheckEvent` → 服务器返回 → `DecompressResponse` 解压 → MessagePack 反序列化 → `WorkSingleModeChangeParameterInfo` 存储。

**Hook DecompressResponse 就能截获训练结果（透视）**，这已经实现了。

### 4.2 请求体编码
- 请求体: MessagePack 格式 + 压缩(LZ4/Gzip)
- CompressRequest hook 拦截的是压缩**前**的数据 → MessagePack 明文
- 需要 MessagePack 反序列化才能读取具体字段

### 4.3 服务器端点
URL 在运行时拼接，不在静态文件中。但通过 sniff 可以运行时获取：
- 启动 `/api/sniff/toggle` → 打游戏 → 查 `/api/sniff` 就能看到所有请求 URL

## 五、增强建议

### 5.1 MessagePack 解码
当前 sniff 只抓到原始字节。可以加一个 MessagePack 解码器：
- 解析 MessagePack → JSON
- 这样就能直接看到请求/响应的具体字段

### 5.2 自动匹配请求-响应
当前用时间戳匹配。可以改为用 WWWRequest.Post 的返回值（HTTP响应）来精确匹配。

### 5.3 请求修改（重放/篡改）
当前只读不写。如果要修改请求/响应：
- CompressRequest hook: 修改原始 body → 重新压缩
- DecompressResponse hook: 修改解压后的 body
- **但修改后服务器可能校验不一致 → 封号风险**

### 5.4 训练结果预测
如果 MessagePack 解码后能看到训练结果的结构，就能在 `DecompressResponse` 拦截时就知道训练结果，比当前读 `WorkSingleModeChangeParameterInfo` 更早。

## 六、总结

| 能力 | 状态 | 方法 |
|------|------|------|
| 拦截请求 URL | ✅ 已实现 | hook WWWRequest.Post |
| 拦截请求 Headers | ✅ 已实现 | hook WWWRequest.Post |
| 拦截请求体(明文) | ✅ 已实现 | hook CompressRequest |
| 拦截响应体(明文) | ✅ 已实现 | hook DecompressResponse |
| MessagePack 解码 | ❌ 未实现 | 需加解码器 |
| 请求修改/篡改 | ❌ 未实现 | 风险高 |
| SSL Pinning 绕过 | ✅ 不需要 | hook 在 C# 层 |

**结论：拦包能力已经完整实现。三个 Hook 点覆盖了请求/响应的全链路。**

*分析: 2026-07-12*
