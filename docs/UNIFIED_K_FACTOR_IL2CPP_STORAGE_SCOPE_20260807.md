# K批次：统一观测与继承生产链完整更新范围（2026-08-07）

## 基线与目标

本分支从累计A–J实现分支 `workbench/unified-observation-endpoint-implementation-20260806` 创建。K批次不是单一网络请求或13个追加端点；本文件锁定此前讨论的继承家系、自动/离线育成、绿卡生成、因子生产、IL2CPP调用关系、完整交换记录与跨重启文件访问。所有新增能力进入同一累计补丁、构建和最终Release候选。

## A. 继承、父辈、祖辈与比赛记录

- `/inherit/tree`
- `/inherit/parent_records`
- `/inherit/race_history`
- `/inherit/pair_compat`（统一现有部分实现）
- `/inherit/race_compat`
- `/inherit/full_compat`
- `/inherit/compat_trace`
- `/inherit/factor_tree`
- `/inherit/bonus_params`
- `/inherit/event_trace`
- `/inherit/deck_runtime`
- `/inherit/deck_validate`
- `/inherit/friend_rental_context`
- `/inherit/auto_select_trace`
- `/inherit/selected_parent_runtime`（统一现有部分实现）
- `/inherit/selected_parent_records`（统一现有部分实现）

### 继承数据契约

`tree/parent_records`按两名父辈与四名祖辈槽位输出具体 `trained_chara_id`，并保存card/chara、owner/viewer、租借来源、因子、胜鞍和完整比赛记录。`race_compat/full_compat/compat_trace`分别输出逐项赛事命中、每段家系小计、最终游戏值与本地重算差异；不以角色两两基础相性冒充具体种马完整相性。

## B. 游戏内自动育成与离线育成

- `/autoplay/runtime`
- `/autoplay/plan`
- `/autoplay/action_trace`
- `/autoplay/factor_select_trace`
- `/offline_auto/runtime`
- `/offline_auto/start_request`
- `/offline_auto/race_reserve`
- `/offline_auto/result`

离线育成按游戏真实 `IdleSingleMode` 链实现；不得与客户端 `SingleModeAutoPlayAgent` 混为一类。

## C. 绿卡直接生成继承马

- `/generate_succession/status`
- `/generate_succession/limits`
- `/generate_succession/request`
- `/generate_succession/result`
- `/generate_succession/candidates`
- `/generate_succession/race_reserve`
- `/generate_succession/race_validation`
- `/generate_succession/factor_priority`
- `/generate_succession/factor_order`
- `/generate_succession/probability_trace`
- `/generate_succession/cost_trace`

请求、响应和finish按同一exchange/session关联。服务端未返回胜负或因子概率中间值时，probability_trace明确标 `server_only_not_observed` 并引用完整原始交换，不伪造权重。

## D. 因子生产、历史统计与种马建议

- `/factor/finish_trace`
- `/factor/candidates`
- `/factor/roll_trace`
- `/factor/probability_model`
- `/factor/history`
- `/factor/stats`
- `/factor/breeding_advice`

`candidates`区分未进入候选、进入未命中、命中及星级。`probability_model`每项标 `confirmed_runtime_formula`、`confirmed_server_field`、`confirmed_mdb`、`estimated_from_samples` 或 `unknown`。`history/stats`保留原始样本ID和分层条件；`breeding_advice`只能引用模型中已有证据。

## E. IL2CPP通用单目标能力

- `/il2cpp/call_targets`
- `/il2cpp/callers`
- `/il2cpp/type_detail`
- `/il2cpp/object_dump`

调用关系和类型查询需支持精确类+方法+签名单目标解析，不能依赖永久成功的全量MethodIndex。`object_dump`保留字段原值、数组顺序、空值、混淆结构与解码值；技术上限必须显式报告未读路径，不能静默删除。

## F. 完整交换记录与精确Hook

- `/api/sniff/exchanges`
- `/api/sniff/exchange`
- `/api/hook/install`
- `/api/hook/remove`
- `/api/hook/list`
- `/api/hook/events`

请求创建、序列化、发送、响应、解压、反序列化和回调使用统一 `exchange_id`；保存完整URL、Headers、Cookie、query、请求体、响应体、DTO类型、对象地址与回调MethodInfo。

## G. 跨游戏重启历史文件

- `/storage/files`
- `/storage/download`

`files`用稳定cursor列出 `observation_files` 的全部登记项；`download`按file_id返回完整文件字节、MIME、长度和SHA-256，不以preview、摘要或metadata替代正文。旧session在游戏重启后仍可查询。

## 全局实现契约

1. 全部路由同步进入主路由、`/health`、404 available列表、README、`hlpatch_endpoints.txt`和静态契约测试。
2. 所有记录使用session_id、单调observation/file ID及可用的exchange_id关联。
3. 禁止固定空数组、固定null或 `ok=true` 占位冒充实现；当前状态没有对象或样本时返回结构化 `no_current_state/no_samples/server_only_not_observed/unresolved`，并保留查询来源和错误。
4. 已有 `/inherit/pair_compat`、`selected_parent_runtime`、`selected_parent_records` 保留兼容入口，但输出升级到统一版本化契约。
5. 所有文件与协议内容完整保存；不得加入字段过滤、固定正文切片、preview替代或只保留metadata的路径。
6. 累计补丁按依赖拆成可审计脚本，但必须在同一构建工作流顺序应用两遍并验证幂等；不得发布只含其中一域的半成品。

## 实施顺序（同一更新，不是删减范围）

```text
K1 storage files/download + exchange关联骨架
K2 单目标 IL2CPP call_targets/callers/type_detail/object_dump
K3 六节点家系、父祖比赛记录、因子树与完整相性
K4 autoplay/IdleSingleMode/GenerateSuccession运行时和协议适配
K5 factor finish/candidates/roll/history/stats/model/advice
K6 精确Hook控制、统一路由/文档/回归与Android arm64构建
```

## 验收

- 累计A–K全部脚本连续执行两遍，最终源码哈希一致；
- Android arm64 Release构建通过；
- 本文件所列路由均有真实处理函数和主路由接线；
- 静态检查拒绝固定占位成功、静默数据丢失和未登记路由；
- MethodIndex超时后单目标类型与调用目标查询仍可工作；
- 当前选中的父辈及祖辈可输出各自因子、胜鞍和比赛历史；
- 强制结束并重开游戏后，旧会话文件可列出并完整下载；
- 无因子或绿卡样本时准确报告无样本/服务端不可观察，不输出虚构概率。
