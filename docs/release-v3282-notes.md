基线严格等于 **v3.28.0**（workflow 里 pin 到发布提交 `721c086`，完整重放 cumulative 3.27.23 → SIGSEGV guard → 画面映射 A 阶段），之后只多打一个补丁：**崩溃日志落盘路径**。

## 根因

插件所有崩溃落盘点写的是：

```
/data/data/jp.pokemon.pokeuma/files/uma_predict.log
```

但 `jp.pokemon.pokeuma` 这个包名在机上根本不存在（真实包名从 `/proc/self/cmdline` 读到，是 `jp.co.cygames.umamusume`）。`open()` 一直失败，于是：

- `crash_signal_handler` 的 `CRASH at step N sig=11` 记录 —— 丢
- panic hook 的 `PANIC: ...` 记录 —— 丢
- 每一条 `log_predict_step` —— 丢
- `/debug/crashlog` 读不到东西，GitHub 自动上传也没内容可传

这就是「育成第五回合闪退、之后一进就闪退」在设备上**一个字证据都没留下**的原因。

## 修复

崩溃日志改写到一个应用无需任何权限就能写的目录 —— 也就是 `ura_boot.log` 已经验证可写的同一个地方：

```
/sdcard/Android/media/<pkg>/hachimi/uma_predict.log
```

- 旧路径保留为兜底，行为不会比现在更差；
- 信号处理函数用的是启动时预热好的静态缓冲区，**处理崩溃时不分配内存**（async-signal-safe，避免在 malloc 里崩掉导致死锁）；
- `/debug/crashlog` 现在合并读取三个落点，哪个有内容看哪个；
- `ura_boot.log` 会新增一行 `crash_log=<实际路径>`，一眼确认补丁生效。

## 已知现象（不是回归）

本包**不含** `training_anim_skip`（v3.28.1 的头号嫌疑），也**不含** `version_single_source`，所以各端点返回的 `version` 字段仍会显示历史硬编码值 `3.22.91`。认这个包请看 `ura_boot.log` 里的 `crash_log=` 行，或比对 SHA256SUMS。

## 用途

这是 A/B 的 B 组：v3.28.0 行为 + 崩溃日志能落盘。

- 装上后如果育成不再闪退 → 与 v3.28.0 一致，说明问题在 v3.28.1 的 anim_skip；
- 如果仍然闪退 → 这次会在 `Android/media/<pkg>/hachimi/uma_predict.log` 留下 `CRASH at step N sig=11`，直接定位到具体步骤。
