# SO 一次性观测端点升级计划（2026-08-06）

## 目标

一次构建补齐当前逆向任务反复卡住的通用能力，覆盖：请求/响应、IL2CPP 精确调用链、育成训练公式、综合评价、技能、支援卡、育成角色、继承种马相性、比赛相性、因子抽选概率与跨游戏重启持久记录。该计划优先增加可复用的底层能力，不为每个页面重复写一次性端点。

## P0：精确 IL2CPP 身份与调用关系

### 1. `/il2cpp/method_by_addr`

输入单个当前进程地址，返回：

```json
{
  "query_addr": "0x...",
  "match_kind": "exact_pointer|range_owner|none|ambiguous",
  "method_pointer": "0x...",
  "method_info": "0x...",
  "namespace": "Gallop",
  "declaring_type": "Outer/Nested",
  "method_name": "Method",
  "return_type": "System.Void",
  "parameters": [],
  "is_static": true,
  "generic_context": null,
  "function_start": "0x...",
  "function_end": "0x..."
}
```

必须区分 exact、函数范围归属、无匹配和多匹配；不得把最近地址冒充目标。

### 2. `/il2cpp/method_detail`

输入 `namespace + declaring_type + method + overload_index/signature`，返回完整 MethodInfo、参数名/类型、泛型实参、方法指针、invoker、函数边界、嵌套 declaring type。解决同名重载与短类名冲突。

### 3. `/il2cpp/call_targets`

仅解析一个明确方法的真实函数边界，列出直接 `BL/BLR` 目标，并逐个调用 `method_by_addr` 做身份映射。输出指令偏移、目标地址、托管身份、未解析原因。

### 4. `/il2cpp/callers`

输入一个明确 MethodInfo，仅在已建立的方法指针索引中查找直接调用者；支持 `limit/cursor`，返回调用者方法和调用偏移。禁止扫描任意内存区或自动扩大范围。

### 5. `/il2cpp/nested_types`

输入精确 declaring type，只列该类直接嵌套类型及字段/自有方法摘要。用于定位 `<>c__DisplayClass*`、状态机与 lambda，不再依赖短类名猜测。

### 6. `/il2cpp/type_detail` 与 `/il2cpp/enum_values`

前者返回精确类型的 namespace、完整嵌套名、父类、接口、字段、属性和自有方法；后者读取枚举常量名、显式整数值和底层类型。解决 `FinalTrainingRank` 等“顺序支持但数值未确认”的问题。

### 7. `/il2cpp/object_dump`

输入对象地址与精确类型，递归读取指定深度的字段，保留数组顺序、空值、混淆整数的原始结构与解码值；必须有深度、节点数和字节数上限，并明确返回 `truncated=true` 与未读路径，不能静默省略。

## P0：完整发包与收包

### 8. 请求—响应统一观测记录

把当前分散的 compress/decompress/post/Unity hook 合并为同一 `exchange_id`：

```text
request_created → serialized → compressed/encrypted → sent
→ response_received → decrypted/decompressed → deserialized → callback
```

每阶段保存时间、线程、URL、HTTP方法、完整headers/cookies/query、原始body、MessagePack/JSON解析、请求/响应DTO类型、对象地址和回调 MethodInfo。

### 9. `/api/sniff/exchanges`

支持按 `after_id`、path精确值、MethodInfo、DTO类型筛选和分页；每条记录同时返回完整请求与响应及各阶段原始字节。禁止环形缓冲覆盖而不报告：增加 `first_id/last_id/dropped_count/capacity_bytes`。

### 10. `/api/sniff/exchange?id=` 与持久文件

按ID读取单条完整交换；大正文写入持久会话目录并返回可完整下载的文件端点、长度、SHA-256。内存缓冲重启丢失时可继续读取历史记录。

### 11. DTO序列化/反序列化 Hook

记录：

```text
runtime_type / object_address / serializer_method / field_or_key_order
raw_before / raw_after / parse_error_offset
```

这可直接闭合 `SingleModeFinishResponse → ResultDataContainer`、技能/育成响应和 Ramen Apply DTO。

### 12. 精确 Hook 控制端点

```text
/api/hook/install?namespace=&type=&method=&signature=
/api/hook/remove?id=
/api/hook/list
/api/hook/events?after_id=
```

只允许显式目标；事件保存入参对象、返回对象、异常、线程、时间与调用序号。Hook安装失败必须返回原始阶段与错误。

## P0：统一育成时序与公式证据

### 13. `/training/snapshot`

一次读取同一逻辑时点的：

- scenario/card_id/chara_id/turn/date/command state；
- 五维、上限、体力、干劲、技能点、粉丝；
- 五项训练的基础命令、等级、失败率、主体收益与Bonus收益；
- 每个 partner 的实体类型、partner_id、chara_id、support_card_id、deck_position、羁绊、位置、友情资格、固有激活、Tips；
- 支援卡等级、突破、普通 effect table 解析值、unique effect条件与当前判定；
- 剧本 ActiveEffect、参数变化容器和当前合法操作。

所有字段必须带 `snapshot_id`、逻辑回合、捕获阶段和对象版本，避免跨时点拼接。

### 14. `/training/transition`

Hook `ExecTraining` 及对应响应应用链，自动关联 `before_snapshot_id`、请求交换、响应交换、参数变化、羁绊变化、事件流与 `after_snapshot_id`。保存预览值、实际值和差值。

### 15. `/training/effect_breakdown`

对每项训练输出来源分解：MDB基础、角色成长率、心情、各支援卡普通效果、固有、友情人数、剧本效果、体力/失败率修正、上限/溢出及每层中间整数。若游戏只下发最终值而无本地中间值，字段必须标记 `not_observed`，不能反推填充。

### 16. `/training/action_timeline`

统一保存：训练前快照→预览生成→请求→响应→属性应用→羁绊写回→事件资格/红点→实际事件→记录UI追加。用于验证79→86羁绊边界、事件先后和随机状态。

## P1：综合评价与技能

### 17. `/evaluation/finish_trace`

专门关联育成结束请求：

```text
Finish Request DTO
→ Finish Response DTO
→ 响应转换方法
→ SingleModeResultDataContainer
→ TrainedChara
→ RankScore / FinalTrainingRank
```

同时保存五维、适性、AcquiredSkillArray、grade_value候选和结果字段，允许比较服务器返回值与任何客户端中间值。

### 18. `/skills/current`

每个已获/可获技能输出：skill_id、level、group_id、group_rate、rarity、grade_value、disable_singlemode、hint等级、折扣、需求点、正负/固有/特殊状态、同组高低阶关系和Master来源。

### 19. `/skills/transition`

Hook技能学习、技能移除/升级、事件Hint和结束结算，关联请求/响应、学习前后数组及评价分变化。这样才能验证 `grade_value` 是完整值、差值、最高阶值还是有过滤/倍率。

### 20. `/evaluation/rank_thresholds`

直接从当前MDB输出 `FinalTrainingRank` 显式枚举值与 `single_mode_rank` 的id/min/max联表，并报告缺失、重复和不连续区间。

## P1：支援卡、育成角色与事件归属

### 21. `/support/deck_runtime`

按 deck_position 输出 support_card_id、关联chara_id、卡等级/突破、effect_table_id、unique_effect_id、当前普通效果值、固有条件原始字段、训练前激活状态和运行时实体类型。严禁用chara_id替代support_card_id。

### 22. `/character/runtime_identity`

输出 trained card_id、chara_id、dress_id/ChangedModelDressId、才能等级、成长率、固有技能、角色效果与事件来源。保持角色本体、育成换皮、显示衣装三层分离。

### 23. `/events/source_trace`

每个候选与实际事件保存 story_id/event_id/chara_id/trained_card_id/support_card_id/source_scope、资格判定、红点、触发序号、奖励与前后状态。角色共通日历、衣装专属、支援事件和全角色通用事件不得混为同一来源。

## P1：继承种马、准确相性与比赛相性

### 24. `/inherit/tree`

读取当前选择的两名亲本及四名祖辈：trained_chara_id、owner/viewer、card_id、chara_id、因子数组、战绩、剧本、适性、固有技能和继承槽位。必须按具体育成马ID保存，不能只保留角色ID。

### 25. `/inherit/pair_compat`

输入两个 `chara_id` 或两个继承树节点，输出：

- 共有 `succession_relation` 群组；
- 每组 relation_type、relation_point、双方member命中证据；
- 两者基础相性数字及逐项和；
- 游戏运行时若存在计算方法，同时返回方法结果和差异。

这能读取角色组合的基础相性数字，但不能单靠角色ID代表某一匹具体种马的完整继承相性。

### 26. `/inherit/full_compat`

对当前完整六段继承树逐段计算：子↔亲1、子↔亲2、亲1↔祖1-1、亲1↔祖1-2、亲2↔祖2-1、亲2↔祖2-2；另外追踪游戏实际消费者，输出最终内部数字、界面圈级和阈值。若实际公式还包含同名重复惩罚、战绩或其他修正，必须列为独立分项，不能只套社区公式。

### 27. `/inherit/race_compat`

对继承树每个节点读取完整比赛历史，并与比赛关系Master逐项连接，输出：

```text
race/program/grade/turn
命中的共同比赛或赛事组
每一项增加值
重复/同年/同赛事处理
作用于哪一段pair
最终race_bonus
```

同时 Hook 游戏的比赛相性计算方法，记录输入列表、中间累计和返回值，才能把社区表与当前版本实际实现区分开。

### 28. `/inherit/selected_parent_runtime`

读取当前种马选择页面或育成开始请求中的具体种马对象，返回服务器已经提供的相性总数字、圈级、各分项（若DTO有）、亲本/祖辈ID和本地重算差异。DTO有数字时标记 `server_field`；只有圈标时用Master+战绩重算并标记 `computed_local`。

### 29. `/inherit/compat_trace`

Hook种马列表加载、筛选、选择与育成开始请求，关联：

```text
候选种马DTO → 显示相性 → 选中种马 → start request → start response
```

## P1：因子抽选与种马生产

### 30. `/factor/finish_trace`

关联 `factorLotteryId`、育成结束请求/响应、完成前育成马状态与最终因子数组。保存：

- 五维及每项所处区间；
- `RankScore`、`FinalTrainingRank`、粉丝、胜场、完整赛历与剧本；
- 已学技能的 skill_id、等级、稀有度、group、grade_value；
- 金技能、下位技能、固有技能、剧本技能分别独立标记；
- 所有适性、因子研究/重抽/活动状态；
- 抽选前候选（若运行时存在）、每次随机判定（若可观测）、最终 factor_id/star/value；
- 原始请求/响应DTO、服务器字段和客户端应用方法。

### 31. `/factor/candidates`

Hook 因子候选集合生产器，逐个输出候选来源与资格门控：属性、适性、普通技能、金技能、固有、比赛、剧本、称号及其他类型。记录“未进入候选”“进入候选但未抽中”“抽中及星级”三种状态，避免仅用最终结果估计资格。

### 32. `/factor/roll_trace`

Hook 候选抽取、星级决定、数量决定和重抽链，保存每层输入权重、总权重、随机值、阈值、命中项和返回结果。若抽选完全在服务端且客户端只收到结果，明确标记 `server_only_not_observed`，改用响应字段与样本统计，不伪造权重。

### 33. `/factor/probability_model`

输出按当前版本证据得到的概率表，并为每一项标明来源：

```text
confirmed_runtime_formula
confirmed_server_field
confirmed_mdb
estimated_from_samples
unknown
```

必须能回答并区分：

- 属性值对蓝因子候选与1/2/3星概率的影响；
- 综合评价等级/RankScore是否独立影响因子数量、类型或星级；
- 普通技能与金技能是否使用不同资格或权重；
- 学会金技能是产生金技能自身因子、对应下位白因子，还是只改变候选权重；
- 适性等级对红因子类型与星级的影响；
- 比赛胜利、赛事组、剧本和固有技能对因子的影响；
- 因子研究、重抽和活动加成如何修改原始分布。

现阶段这些问题不得按社区印象预填答案。

### 34. `/factor/history` 与 `/factor/stats`

跨局保存每次育成结束的完整输入与结果；统计时按游戏版本、剧本、因子研究状态、属性区间、评价等级、技能类型和比赛条件分层。输出样本数、命中数、经验概率和区间，同时保留每条原始样本ID。不得把相关性直接写成游戏公式。

### 35. `/factor/breeding_advice`

在概率公式闭合后，按目标因子给出可解释建议：目标属性阈值、是否值得学习指定普通/金技能、目标适性、必跑比赛、综合评价边际收益与机会成本。每条建议必须引用 `/factor/probability_model` 的证据项；公式未知时只列待验证变量，不输出虚假最优策略。

## P1：比赛观测

### 36. `/race/pre_snapshot` 与 `/race/result_trace`

保存参赛育成马完整状态、技能、适性、赛道、距离、天气、马场、枠番、作战、对手、RaceRandomProgramArray，以及请求/响应、技能触发、分段位置、最终名次和奖励。相性研究中的比赛历史与实际比赛模拟必须分开字段。

## P2：跨游戏重启持久保存

### 37. 会话目录与索引

训练快照、transition、timeline、完整协议交换、Hook事件、比赛、相性和因子抽选不能只放内存。SO启动时确定一个可写持久根目录，并由端点返回实际绝对路径；在未验证Hachimi/Android运行时路径前，不把设计文档中的示例写成固定真实路径。

建议布局：

```text
<runtime_resolved_persistent_root>/hlpatch-observations/
  index.sqlite
  sessions/<session_id>/session.json
  sessions/<session_id>/timeline.ndjson
  sessions/<session_id>/snapshots/*.json
  sessions/<session_id>/exchanges/<exchange_id>/request.*
  sessions/<session_id>/exchanges/<exchange_id>/response.*
  sessions/<session_id>/factors/*.json
  sessions/<session_id>/races/*.json
  blobs/<sha256>
```

SQLite只保存索引与关联；NDJSON/原始文件保存完整内容和字节。写入使用临时文件、flush、fsync、原子rename；启动时恢复未完成会话并标记 `recovered_after_restart=true`。

### 38. 持久记录端点

```text
/storage/status
/storage/sessions
/storage/session?id=
/storage/files?session_id=&cursor=
/storage/download?file_id=
/storage/flush
/storage/recover
```

`/storage/status` 返回实际根目录、是否可写、可用空间、当前会话、上次成功flush、索引版本与恢复错误。游戏退出或进程异常前未flush的最后少量内存事件可能丢失，因此关键事件（请求/响应、训练结算、育成结束、因子结果）必须同步提交；普通高频事件可批量提交。

### 39. 生命周期

```text
SO加载 → 打开/校验index.sqlite → 恢复历史会话索引
→ 创建新session_id → 持续追加
→ 游戏正常退出时flush并关闭
→ 下次启动仍可查询全部历史session
```

游戏重开后旧对象地址和方法地址不能复用；持久记录保留业务ID、原始数据、版本和相对偏移，新进程重新建立MethodInfo索引。

## P2：运行稳定性、版本与数据质量

### 40. 版本化签名清单

每个Hook保存：游戏版本、插件commit、类型/方法签名、方法指针、模块基址、安装状态、首次/末次命中、调用次数和最后错误。游戏更新后指针变化必须重新解析，禁止沿用绝对地址。

### 41. 统一诊断

`/debug/hookdiag` 增加每个Hook的 resolution/install/invocation/capture/serialization 五阶段状态；对象为空、场景不符、签名不符与Hook未命中分开报告。

### 42. 事务一致性

所有 snapshot/exchange/hook_event 使用单调ID、单调时钟、线程ID、逻辑turn和correlation_id。提供 `/timeline?after_id=` 按真实顺序合并协议、方法、状态和UI事件。

### 43. 容量与完整性

所有缓冲报告 `capacity/count/bytes/dropped_count/first_id/last_id`；分页使用稳定cursor。大对象和原始字节允许完整文件下载；任何工具上限造成的未读取都显式列出。

### 44. 只读优先与调用门控

默认只安装观测Hook和读取端点。运行时调用端点必须要求精确签名、显式参数类型、对象生命周期检查、场景门控和dry-run解析；禁止旧式“只支持Int32却调用Action参数”的不匹配路径。

## 推荐实现顺序

```text
A. MethodInfo索引 + method_by_addr/method_detail/nested_types/enum_values
B. 持久会话存储 + storage status/recovery
C. call_targets/callers + 精确Hook管理
D. exchange_id协议全链与持久交换记录
E. snapshot/transition/timeline通用关联框架
F. evaluation/skills/training专用适配
G. support/character/event身份层
H. inherit tree/pair/full/race/selected_parent trace
I. factor finish/candidates/roll/history/probability/advice
J. race trace与稳定性回归
```

A-E完成后，后续大多数机制不再需要新增底层端点，只需增加类型适配器。

## 验收矩阵

1. `SendSingleModeFinishRequest`：能从包装器定位嵌套闭包、网络Response DTO、转换方法和容器字段写入。
2. Ramen `SendUrafEffectApply`：能从回调调用地址反查托管身份并取得完整请求—响应。
3. 训练：一次操作自动产出前快照、预览、请求/响应、效果分解、羁绊后快照与事件序列。
4. 技能：一次学习自动显示同组替换、技能点变化及结束评价影响。
5. 支援卡：同一chara的NPC与携带支援卡不会混用效果。
6. 育成角色：card_id/chara_id/dress_id保持独立。
7. 相性：当前六段继承树逐段数字之和、比赛加成、圈级与游戏内部返回一致；不一致时输出具体差异项。
8. 因子：一次育成结束能关联完成前状态、候选集合、抽选/星级链和最终因子；无法观察的服务端步骤明确标记。
9. 持久化：强制结束并重开游戏后，旧session仍可列出并完整读取已提交的训练、协议、相性和因子记录。
10. 协议：重启前捕获的大交换仍可按ID完整读取，且dropped_count可核对。
11. 版本：游戏更新后旧Hook显示signature_mismatch或unresolved，不静默使用旧地址。
12. 稳定性：所有单目标查询均不依赖 `/il2cpp/classes`、`/il2cpp/disassemble_addr*` 或全量方法dump。

## 当前事实边界

- 当前已部署SO在本次规划时不可连接，故未读取实时 `/inherit/compat` 或因子端点输出；不可把已有端点名视为精确相性或因子概率已经闭合。
- 仓库 `reverse/relation_formula_exact.md` 保存社区相性公式与MDB群组解释，但比赛相性、当前版本完整树修正和服务器是否下发精确数字仍未由运行时Hook确认。
- 现阶段只确认结束API存在 `factorLotteryId` 参数和最终因子数据结构线索；评价、属性、技能、比赛对因子概率的精确影响仍是待验证问题。
- `hlpatch_endpoints.txt` 已列出142个端点；本计划先补精确MethodInfo、持久会话、统一交换记录和时序关联，再实现因子概率适配器。
