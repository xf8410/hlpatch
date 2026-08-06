# K批次：因子、IL2CPP调用关系与历史文件端点同步实现范围（2026-08-07）

## 基线

本分支从累计A–J实现分支 `workbench/unified-observation-endpoint-implementation-20260806` 创建。K批次不是单一网络请求特判；以下13个端点必须在同一累计补丁、同一构建与同一验收矩阵中接线。

## 强制端点

### 因子链

- `/factor/finish_trace`
- `/factor/candidates`
- `/factor/roll_trace`
- `/factor/probability_model`
- `/factor/history`
- `/factor/stats`
- `/factor/breeding_advice`

### IL2CPP通用能力

- `/il2cpp/call_targets`
- `/il2cpp/callers`
- `/il2cpp/type_detail`
- `/il2cpp/object_dump`

### 跨重启历史文件

- `/storage/files`
- `/storage/download`

## 实现契约

1. `/factor/finish_trace` 关联结束请求/响应、完成前状态、最终因子和持久exchange/file ID。
2. `/factor/candidates` 区分未进入候选、进入未命中、命中及星级；游戏未暴露候选时返回明确 `server_only_not_observed`，不得伪造数组。
3. `/factor/roll_trace` 只记录实际观察到的数量、候选、星级判定输入和随机值；服务端步骤不可见时保留原始响应并标不可观察。
4. `/factor/probability_model` 每项必须标 `confirmed_runtime_formula`、`confirmed_server_field`、`confirmed_mdb`、`estimated_from_samples` 或 `unknown`。
5. `/factor/history` 从SQLite会话及已登记原始文件读取跨重启样本；`/factor/stats` 保留样本ID、分层条件、样本数、命中数和区间。
6. `/factor/breeding_advice` 只能引用 probability_model 中已有证据，不得把相关性或未知量写成确定公式。
7. `/il2cpp/call_targets` 对一个精确方法解析真实函数边界内的直接调用目标；`/il2cpp/callers` 有界分页查直接调用者。二者不得依赖永久成功的全量MethodIndex，需支持精确类+签名单目标解析。
8. `/il2cpp/type_detail` 输出父类、接口、字段、属性和自有方法；`/il2cpp/object_dump` 保留字段原值、数组顺序、空值、混淆结构及解码值，并显式报告深度/节点/字节技术上限与未读路径。
9. `/storage/files` 使用稳定cursor列出 `observation_files` 全部登记项；`/storage/download` 按file_id返回完整文件字节、MIME、长度和SHA-256，不返回preview或metadata替代正文。
10. 13个路由必须同步加入主路由、`/health`、404 available列表、README/端点清单和静态JSON契约测试。
11. 所有新增记录使用session_id、observation/file ID及可用的exchange_id关联；游戏重启后旧session仍可查询。
12. 禁止以固定空数组、固定null或ok=true占位冒充端点实现；能力未观察必须返回结构化状态和原始证据引用。

## 与继承批次的关系

K批次提供父辈/祖辈、胜鞍、因子树和完整相性端点所需的通用调用关系、对象读取和历史文件基础。继承端点仍在同一统一升级计划中推进，不得因K批次而删除、缩窄或改成一次性GenerateSuccession请求方案。

## 验收

- 累计A–K补丁连续执行两遍保持幂等；
- Android arm64 Release构建通过；
- 13个端点均有真实路由和非占位实现；
- 数据损失词与固定切片审计通过；
- MethodIndex超时后单目标 `call_targets/type_detail` 仍可工作；
- 强制结束并重开游戏后，旧会话因子文件可由files列出并由download完整读取；
- 当前无因子抽选样本时，factor端点准确报告无样本或服务端不可观察，不输出虚构概率。
