# 恢复候选的作者验收步骤

本候选针对游戏文字更新被吞、大正文导出风险和游戏重启后的采集身份恢复。它是**诊断候选**，尚未通过真机验收。新增 ABI 校验要求目标游戏和 Unity 的原生回调契约；目前没有经过验证的配置，因此涉及托管方法的 Hook 会拒绝安装并报告 `unsupported_native_abi`。这不代表嗅探功能已经恢复。旧摘要仍缺少完整盘面，`ready=false` 是完整性保护；不能以出现 AI 推荐作为本批唯一验收条件。

## 先固定安装组合

记录手机型号、Android 版本、内存、游戏版本、Hachimi 宿主版本及安装包名。保存当前 APK/SO 和 SHA256；不卸载旧 App，不删除局记录。不同接收 App 会竞争本机 18766，测试期间只启动一个接收服务。

候选 SO、符号、Build ID 和 `BUILD-MANIFEST.json` 必须来自同一个产物目录。旧 v3.28.2 的发布文件与本仓库用现代工具链重新构建的基线不是相同二进制。诊断以实际装载文件 SHA256/Build ID 为准。

## 本批接口变化

- `/health` 和 `/api/diagnostics/recovery` 返回有效 JSON、构建指纹、Hook 状态、采样和推送状态、后台队列预算及错误。它们不读取游戏盘面。
- `/summary` 只返回单采样 worker 的缓存，用于兼容展示。`3.28.2-recovery.2` 中拉面的 `turn/year` 为 `null`；未经校准的原始回合值及来源单独保留，旧 AI 为不可用。V2 的 `display_summary.turn_observation` 保留这些证据，不表示正式回合已校准或完整盘面已采齐。
- `/api/ai/ramen/capabilities` 声明 V2 和本次游戏进程的 `collector_instance_id`。`/api/ai/ramen/v2/snapshot` 与推送复用同一快照编号。同一盘面重复轮询不会产生新编号。
- `3.28.2-recovery.3` 的解析、深复制及旧快照释放移到共享锁外；进入 `booting` 就撤销旧快照可发送资格。恢复为 `capturing` 但新观测尚未完成时，`/summary`、快照和推送不能重新返回 boot 前的盘面；诊断历史不表示当前已恢复。采样身份和相同内容的快照编号保持原规则。
- `/api/sniff/toggle` 必须显式传 `enabled=1` 或 `enabled=0`。默认不采集；本候选不安装插件 TextCommon Hook，默认不安装屏幕镜像 Hook。关闭采集不等于卸载已验证的原调用转发 Hook。
- `/api/hooks/abi` 返回单采样 worker 缓存的只读方法签名诊断。运行时尚不可用时明确等待；初始化回调缺失时，仍可通过采样线程的有限探针收集元数据，`callback_received` 单独显示。方法名、托管参数个数和反射类型不足以单独证明原生隐藏参数 ABI；作者还需提供游戏/Unity/宿主版本及对应导出证据。
- `/api/sniff/metadata` 只返回预览，默认最多 64 条、256 KiB。原始正文保存到文件，字段不能当作完整正文。
- `/storage/download?file_id=...` 返回文件引用和校验信息；正文用 `/storage/read_range?file_id=...&offset=...&length=...` 分段读取，单段最多 1 MiB。
- `/storage/turn_event_jsons` 要求 `cursor_version=2`，用 `after_file_id` 续页。旧 `after_sequence`、`cursor`、`max_json_chars` 参数会明确报迁移错误；不能沿用旧匹配序号。
- `/storage/flush` 暂停新正文入队并等待已有任务。超时、写入故障或历史缺口不会报告完整成功，也不会自动恢复采集。确认故障后显式调用 `/storage/capture_resume`；原有缺口计数不会被清零。
- 恢复候选关闭旧的任意内存调用、行为修改、在线自更新等实验路由。未列入恢复接口的路由返回 `410`；不通过 HTTP 启动第二套游戏 getter。
- 启动入口也不执行自更新、自动上传／删除旧崩溃文件或旧 SQLCipher Hook 开关。手机保留的历史标记文件不会重新启用这些功能；文件本身不删除。这样不会在测试中途悄悄更换 SO 或发送历史日志。

后台正文队列总预算为 16 MiB，预算包含正文和有界附带数据。超限或队列故障会记录缺口，保留游戏原调用；不会把截断正文标成完整采集。已有的大文件仍可按段完整导出。请求正文与 Post 仅在压缩内容 SHA256 唯一匹配时关联；无法确认的响应保持“关联未知”，不把 FIFO 顺序当作因果证明。

## 顺序验证

1. **干净冷启动、未开启嗅探。** 重启游戏进程后验证主页和育成内的所有动态文字、TP/RP、货币、五维、体力。保存截图及诊断 JSON。确认 `text_observer_installed=false`；不能只关闭旧插件开关后继续使用已被 Hook 的进程。
2. **先验证未确认 ABI 会被拒绝。** 请求 `GET /api/sniff/toggle?enabled=1`，保存 `/api/hooks/abi` 和诊断 JSON。当前候选预期 Hook 不安装；游戏显示保持正常只能证明隔离有效，不能证明已解决启用嗅探后的全部故障。补齐 ABI 配置并构建后，再执行普通主页刷新和一回合操作，验证文字、数值和请求结果。
3. **MD5 安装顺序。** 当前版本验证两种顺序都明确拒绝未知 ABI。匹配配置经审查后，分别在独立冷启动中验证“先 MD5 后嗅探”和“先嗅探后 MD5”；重复安装须幂等，失败不改写目标函数作为回退。
4. **大正文与断开下载。** 保存触发问题的准确端点、原始文件大小、HTTP 参数。对 1/16/64 MiB 已存文件逐段导出并核对 SHA256，中途断开和并行请求后再查队列、缺口、PSS 和日志。超预算明确拒绝属于预期；无证据不能标“完整捕获”。
5. **App 后启动。** 先游戏、后 App；推送失败必须保留待发送快照，App 启动后收到同一个待发送身份。只有 HTTP 202 且身份相符的收件 ACK 才记成功。收件 ACK 不代表可以决策。
6. **同局三次游戏重启。** App 保持运行，重启游戏并继续同一育成三次。每次采集实例变化；实例内快照编号可重新开始，旧回调不能覆盖新状态。记录中保留各次进程身份和独立 `archive_seq`，无覆盖或串局。
   同时观察 `booting → capturing → incomplete/ready` 的边界：新的采集完成前，对外入口保持等待/缺当前观测，不重新外发上一次有效盘面。本地阻塞采样测试已经覆盖此窗口；设备实际出现的阶段和时序仍需留存。
7. **App 重启与复盘。** 保留局目录，重启 App 后重新握手；验证同快照取消、失败、同配置重算、换配置重算的记录。跨引擎/数据版本不得被错误地当作当前版本统一复盘。

当前第 5–7 项可以验证传输、实例及记录边界，不能据此验收真实整局恢复。当前采集仍缺真实局号、阶段和完整 continuation；完整决策对照必须在这些实现完成、取得真实样本后补做。核对回合时同时保存游戏可见日期、育成阶段及原始字段，覆盖开局、跨年和续养，不按单张截图推断转换公式。

## 取证命令

以下命令在连接作者手机的 PC 上执行。不要清除已有 logcat，先记录复现起止时间。

```powershell
adb forward tcp:18765 tcp:18765
curl.exe --fail http://127.0.0.1:18765/health -o health.json
curl.exe --fail http://127.0.0.1:18765/api/ai/ramen/capabilities -o capabilities.json
curl.exe --fail http://127.0.0.1:18765/api/ai/ramen/v2/snapshot -o snapshot.json
curl.exe --fail http://127.0.0.1:18765/api/diagnostics/recovery -o recovery.json
curl.exe --fail http://127.0.0.1:18765/api/hooks/abi -o hook-abi.json
adb logcat -b all -d -v threadtime > logcat.txt
adb shell dumpsys meminfo GAME_PACKAGE > game-meminfo.txt
adb shell dumpsys activity exit-info GAME_PACKAGE > game-exit-info.txt
```

将 `GAME_PACKAGE` 替换为实际包名。若系统允许，另附 tombstone 与崩溃库偏移，用候选的未剥离符号解析。截图中的占位数字不能证明服务器数据被修改；区分显示异常、原生崩溃、ANR 和系统低内存终止。

## 回退与结论

出现新游戏异常时停用候选并退出游戏进程，恢复事先保存的 SO 后重新启动；保留候选的日志、局包和产物清单。不要删除用户记录或以卸载应用绕过迁移问题。

本批通过条件是文字/数值更新无干扰、预算和缺口可解释、重启身份恢复正确。完整真实盘面、整局决策、10 局/两小时稳定性及旗舰性能仍按原开发方案单独验收。
