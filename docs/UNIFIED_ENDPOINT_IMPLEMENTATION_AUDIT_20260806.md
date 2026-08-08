# 统一观测端点实现前审计（2026-08-06）

## 审计结论

实现不能直接从 `main` 或仅含规划文档的分支修改。当前实际部署的 v3.27.4 源码位于：

```text
workbench/add-il2cpp-signature-and-invoke-guards-20260806
e079097805ecbb6a1db0d5262315e11852f802d7
```

该分支相对 `main` ahead 14，包含运行时参数类型输出、精确 declaring-type 嵌套类解析、调用参数门控和 v3.27.4 发布历史。规划分支从旧 `main` 创建，落后这14个提交；若直接在规划分支实现，会回退已部署能力。

因此新实现分支已从 v3.27.4 实际源码创建：

```text
workbench/unified-observation-endpoint-implementation-20260806
base = e079097805ecbb6a1db0d5262315e11852f802d7
```

规划分支中的两份文档后续需要复制到实现分支，不能以合并旧分支覆盖源码。

## 主体源码结构

`hachimi_ura_plugin/src/lib.rs` 当前约970KB，仍是单文件架构，混合：

- Hachimi API解析；
- IL2CPP FFI与运行时调用；
- 类/字段/方法查询；
- 协议Hook与内存环形缓冲；
- 训练、事件、Ramen和其他业务读取；
- HTTP路由；
- 文件读写、崩溃日志与上传逻辑。

不能用一次完整文件重写实现新端点。A阶段应先新增独立、低耦合的索引结构和路由处理函数，再精确插入现有 `handle_http` 路由。

## 已有IL2CPP基础

当前主体已经动态解析并使用：

```text
il2cpp_image_get_class_count
il2cpp_image_get_class
il2cpp_class_get_methods
il2cpp_method_get_name
il2cpp_class_get_method_from_name
il2cpp_runtime_invoke
il2cpp_method_get_param
il2cpp_method_get_param_name
il2cpp_method_get_return_type
il2cpp_type_get_name
il2cpp_class_get_declaring_type
il2cpp_class_get_namespace
```

v3.27.4 的参数补丁脚本证明 `il2cpp_method_get_param*` 与 `il2cpp_type_get_name` 在当前构建可用。A阶段不需要改Hachimi宿主API，只需继续通过 `il2cpp_resolve_symbol` 解析额外导出。

当前缺少或尚未接入：

```text
il2cpp_class_get_nested_types
il2cpp_class_get_type
il2cpp_type_get_class_or_element_class
il2cpp_class_get_flags / il2cpp_class_is_enum
il2cpp_class_get_field_from_name之外的枚举常量读取辅助
MethodInfo完整声明布局或稳定的method_get_*访问器集合
```

`native/include/il2cpp_api.h` 是独立旧实验路径，不是当前Rust插件的权威FFI；不能只修改该头文件期待Rust SO获得功能。

## MethodInfo地址索引设计

### 数据结构

建立进程生命周期内只读索引：

```text
MethodIndexEntry {
  method_info
  method_pointer
  namespace
  declaring_type_full
  method_name
  return_type
  parameter_types
  flags/static
}
```

索引按 `method_pointer` 排序，并保留：

```text
exact pointer map
method_info map
(class, method, signature) map
```

函数结束地址不能仅用“下一方法地址”无条件推断；只可返回 `next_distinct_pointer_upper_bound`，并标记 `boundary_kind=upper_bound_estimate`。存在共享泛型指针、thunk、null pointer或同址多MethodInfo时返回ambiguous列表。

### 建立时机

- 游戏初始化后惰性构建；
- 使用互斥锁与明确状态 `empty/building/ready/failed`；
- 新游戏进程重新构建；
- 返回image、class、method总数及重复指针统计；
- 禁止在每次HTTP请求中重新全量遍历。

## A阶段端点实现边界

### `/il2cpp/method_by_addr`

先支持：

```text
exact method_pointer
exact MethodInfo pointer
唯一上界范围候选
ambiguous
none
```

查询必须是索引查找，不做请求时全类扫描。

### `/il2cpp/method_detail`

使用 namespace、完整 declaring type、method、参数类型签名精确消歧。输出参数名/类型、返回类型、MethodInfo与方法指针；不按短名命中第一个重载。

### `/il2cpp/nested_types`

调用 `il2cpp_class_get_nested_types`，只枚举一个精确目标类的直接嵌套类型；输出完整 declaring chain。不能退化到 `/il2cpp/classes` 全类筛选。

### `/il2cpp/enum_values`

先确认当前运行时可用的枚举字段与常量读取API。若只能读取字段元数据而无法稳定读取literal值，端点必须返回 `value_status=unresolved`，不能按声明顺序自行编号。

### 暂缓 `/il2cpp/call_targets` 与 `/il2cpp/callers`

它们属于B阶段。必须先有可靠MethodInfo索引和函数边界语义，再复用已有按方法反汇编函数；不能复用禁用的任意地址反汇编入口。

## HTTP与查询解析

现有HTTP服务器手工读取固定8192字节请求并由大段 `if/else` 路由。A阶段应新增：

```text
parse_request_uri
parse_query_pairs
percent_decode
json_escape复用
endpoint handler函数
```

新端点参数全部来自原始URI query；当前 `parse_path` 会丢弃query，因此不能只在现有path变量上实现。

长期应拆分路由模块，但本批不同时进行大规模重构，以免扩大回归面。

## 持久化审计

`Cargo.toml` 已含：

```text
rusqlite 0.31 bundled
```

因此持久会话不需要新增SQLite依赖。当前已有多个硬编码路径和简单覆盖写入：训练日志只在内存保留30条，sniff原始记录上限50、metadata上限1000，超过后删除首项，且没有dropped_count；这些都不满足跨重启要求。

持久根目录应复用 `find_own_so_path()`、包名和可写探测选择实际路径，并由 `/storage/status` 返回。不能直接把现有 `CRASH_LOG_PATH` 当作统一存储根。

## 协议观测审计

当前结构仍是分散状态：

```text
SNIFF_REQUESTS
SNIFF_RESPONSES
SNIFF_METADATA
SNIFF_RESPONSE_QUEUE
PENDING_URL/PENDING_HEADERS/PENDING_REQ_ID/PENDING_REQ_BODY
UNITY_OBSERVATIONS
```

请求和响应依赖时序队列关联，未形成稳定 `exchange_id` 聚合对象；重启即丢失。后续C/D阶段应建立统一Exchange状态机并同步提交关键交换。

## 构建与发布审计

标准 `.github/workflows/build-ura.yml` 在 `main` 是通用构建工作流；v3.27.4实际分支把同文件改成固定分支发布工作流。实现分支构建前应恢复通用构建工作流或增加独立测试工作流，避免误发布新版本。

已确认历史成功：

```text
Build run 31036912410  success
Release run 31049297539 success
```

新实现必须先走普通构建Artifact，不直接创建Release。

## 已发现的代码风险与语义债务

1. `static mut Vec` 与独立Mutex混用，部分状态仅靠外围锁约定；新索引和持久层不得复制此模式，优先 `Mutex<State>`。
2. 多处手工JSON拼接；A阶段复用统一 `json_escape`，所有地址、类型名和参数名必须转义。
3. `find_class_by_short_name` 仍有全类fallback；新精确端点必须优先namespace/full declaring chain，不能调用短名fallback完成消歧。
4. `src/lib.rs` 是约96KB的旧/独立实现，包含手写AI评分表，不是当前 `hachimi_ura_plugin/src/lib.rs` 的构建入口；本任务不能修改错文件。
5. `native/` C++目录是旧实验实现，不参与 `hachimi_ura_plugin` Cargo构建。
6. GitHub代码搜索本批触发API rate limit；阴性搜索结果不能用于证明符号不存在。
7. Local Sandbox未安装，无法本地clone、grep或cargo build；代码变更需通过GitHub精确写入与Actions验证。

## 下一步唯一入口

在实现分支新增一个幂等源码补丁脚本，基于v3.27.4实际 `lib.rs` 的唯一结构锚点插入：

1. MethodIndex状态和FFI类型；
2. 索引构建与查询函数；
3. URI query解析；
4. `method_by_addr`、`method_detail`、`nested_types` 三个首批端点；
5. 普通构建工作流。

`enum_values` 在同批先实现元数据能力检测，只有常量读取API验证通过后才输出整数值。
