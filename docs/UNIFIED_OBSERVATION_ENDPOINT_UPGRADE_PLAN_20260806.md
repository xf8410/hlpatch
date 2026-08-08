# SO 一次性观测端点升级计划（2026-08-06）

## 目标

一次构建补齐当前逆向任务反复卡住的通用能力，覆盖请求/响应、IL2CPP 精确调用链、训练公式、综合评价、技能、支援卡、育成角色、继承相性、比赛相性、因子抽选与跨重启持久记录。

## 实现顺序

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

## P0：IL2CPP身份

- `/il2cpp/method_by_addr`：区分精确入口、MethodInfo、候选范围、无匹配与多匹配。
- `/il2cpp/method_detail`：按完整 declaring type 和参数签名消歧。
- `/il2cpp/call_targets`、`/il2cpp/callers`：只解析明确方法，不能任意地址或自动扩大扫描。
- `/il2cpp/nested_types`、`/il2cpp/type_detail`、`/il2cpp/enum_values`：读取嵌套类型、字段、方法与显式枚举值。
- `/il2cpp/object_dump`：保留字段、数组顺序、空值、原始结构及解码值；任何未读范围必须显式报告。

## P0：持久会话与协议交换

运行时解析可写根目录，使用 SQLite 索引及原始文件：

```text
<runtime_root>/hlpatch-observations/
  index.sqlite
  sessions/<session_id>/session.json
  sessions/<session_id>/timeline.ndjson
  sessions/<session_id>/snapshots/
  sessions/<session_id>/exchanges/<exchange_id>/request.*
  sessions/<session_id>/exchanges/<exchange_id>/response.*
  sessions/<session_id>/factors/
  sessions/<session_id>/races/
  blobs/<sha256>
```

端点：

```text
/storage/status
/storage/sessions
/storage/session?id=
/storage/files?session_id=&cursor=
/storage/download?file_id=
/storage/flush
/storage/recover
```

SO启动时恢复索引并创建新session。关键事件同步提交；游戏重启后旧session仍可读取。对象和方法绝对地址只属于原进程，新进程重新建立索引。

协议按 `exchange_id` 统一关联：

```text
request_created → serialized → compressed/encrypted → sent
→ response_received → decrypted/decompressed → deserialized → callback
```

保存完整 URL、headers、cookies、query、原始请求响应、解析结果、DTO类型、对象地址和回调MethodInfo。所有缓冲输出 capacity/count/bytes/dropped_count/first_id/last_id。

## P0：训练时序

- `/training/snapshot`：同一逻辑时点保存角色、训练盘面、partner身份、支援效果、剧本效果和合法操作。
- `/training/transition`：关联前快照、请求响应、参数与羁绊变化、事件流和后快照。
- `/training/effect_breakdown`：输出MDB基础、成长率、心情、支援卡普通/固有/友情、剧本效果和逐层整数；未观测字段标 `not_observed`。
- `/training/action_timeline`：训练前预览→执行→属性应用→羁绊写回→事件门控→实际事件→UI记录。

## P1：评价、技能与身份

- `/evaluation/finish_trace`：结束请求/响应→转换方法→ResultDataContainer→TrainedChara→RankScore/FinalTrainingRank。
- `/evaluation/rank_thresholds`：显式枚举值与 `single_mode_rank` id/min/max 联表。
- `/skills/current`、`/skills/transition`：技能Master、同组关系、学习变化及评价影响。
- `/support/deck_runtime`：严格按 support_card_id 和 deck_position 输出。
- `/character/runtime_identity`：trained card_id、chara_id、dress_id分离。
- `/events/source_trace`：候选资格、红点、来源ID、触发顺序、奖励与前后状态。

## P1：继承、因子与比赛

- `/inherit/tree`、`pair_compat`、`full_compat`、`race_compat`、`selected_parent_runtime`、`compat_trace`：保存具体种马、六段树、比赛历史、游戏内部数字与本地重算差异。
- `/factor/finish_trace`、`candidates`、`roll_trace`、`probability_model`、`history`、`stats`、`breeding_advice`：区分未入候选、入候选未抽中、抽中与星级；服务端不可见步骤标 `server_only_not_observed`。
- `/race/pre_snapshot`、`/race/result_trace`：保存完整赛前状态、请求响应、触发、分段位置、结果与奖励。

## 事务、版本和稳定性

所有记录使用单调ID、单调时钟、线程ID、逻辑turn和correlation_id。Hook保存游戏版本、插件commit、完整签名、方法指针、模块基址、安装与命中状态。游戏更新后必须重新解析，不能沿用绝对地址。

默认只读观测。调用端点必须要求精确签名、显式参数类型、对象生命周期、场景门控和dry-run解析。

## 验收重点

1. 结束API可定位嵌套闭包、网络Response DTO、转换方法和RankScore/TrainingRank写入。
2. Ramen Apply可从回调地址反查托管身份并取得完整请求响应。
3. 一次训练自动生成前后快照、预览、请求响应、羁绊和事件序列。
4. 支援卡、NPC和剧本伙伴身份不会混用。
5. 当前六段继承树数字、比赛加成与游戏内部返回可逐项对照。
6. 游戏强制结束并重开后，旧session仍可读取。
7. 单目标查询不依赖 `/il2cpp/classes`、`/il2cpp/disassemble_addr*` 或全量方法dump。

## 当前证据边界

A阶段源码已通过Android aarch64 Artifact-only编译，但尚未部署验证运行时API和端点样本。后续阶段不能把源码存在等同于运行时闭合，也不能按社区印象预填评价、相性或因子概率。
