# v3.28.4 — meta 直查控制台（读取加密资源清单）+ 纯数据管道

基线沿革：**v3.28.2 同款链**（v3.28.0 提交 721c086 + cumulative 3.27.23 + SIGSEGV 防护 + 崩溃日志落盘修复）。
在你手机当前验证能玩的 v3.28.2 谱系上，叠加本轮功能与既有加固，不动 main、不动 28.3。

## 本版变更

1. **新增：`/debug/resource_meta_query`（只读 SQL 控制台）**
   用游戏自带 libnative.so 的 sqlite3mc + 已捕获的钥匙（`files/ura_meta_key.txt`，内存捕获 `META_KEY_HEX` 兜底）直开 164MB 加密资源清单 `/data/user/0/<pkg>/files/meta`：
   - 不带参数 → 列出全部表/视图名；
   - `?table=NAME&limit=50&offset=M` → 读某张表的行；
   - `?sql=SELECT...` → 单条只读语句（列名字、列类型全认，BLOB 给大小，文本 300 字封顶）。
   实现要点：只以 `SQLITE_OPEN_READONLY` 打开（绝不在活文件上建 journal）；三种喂钥匙姿势（key v1 / key_v2 空库名 / key_v2 "main"）**逐一用 `SELECT count(*) FROM sqlite_master` 验证**，哪把真能解密就用哪把（响应里报 `key_used`）；行数与总字节双封顶（默认 50 行、最多 500 行、响应约 900KB 上限），单次查询不可能挤爆 18765 通道。
   入口同时加进 `/health` 端点表、404 可用表与 `?dl=1` 下载白名单。

2. **摘除内置 AI（纯数据管道）**：`/summary` 的 `ai` 字段固定输出 `null`（源码层固化，不再依赖运行时）；`/event/recommend` 停用（404）。决策全部归 jueceramen（uma-juece-ramen，对齐 umaai-rs MCTS）。**数据端点一个不少**；嗅探/发收包落盘（`/api/sniff/*`、`hlpatch-observations` 会话存储）经发布链断言原样保留（见下）。

3. **动态版本号**：/health 及各端点版本字段改读 `PLUGIN_VERSION`（Cargo 版本单源），不再出现「装了新包还报 3.22.91」的旧现象。

4. **画面映射（截屏 hook）不装**：按机主 2026-09-17 决定（卡顿源）暂停；发布链带反向校验（产物里发现 `install_screen_mirror_hook` 即红灯拒发），代码保留在 `scripts/apply_screen_mirror_frame_a.py`，需要时一行回植。

保留项：SIGSEGV 防护（hl_ptr_mapped）、崩溃日志落盘修复（Android/media/<pkg>/hachimi/uma_predict.log）、md5log/嗅探钩子、全部 142 个数据端点。
不含：training_anim_skip（闪退头号嫌疑，按 A/B 纪律持续排除）、签名明文观测（与 v3.28.2 保持一致的最小改动）。

## 与摘 AI 无关的机械证明（发布链断言）

- `grep -c '/api/sniff'` ≥ 20；
- `fn install_api_sniff_hooks` 在场；
- `hlpatch-observations` 观察存储在场。
任一缺失 → CI 红灯，拒绝发布。摘 AI 只动 `/summary` 的 ai_json 块与 `/event/recommend` 路由字面量，与嗅探/发收包是两条独立管线。

## 用法示例

```
# 表清单
http://127.0.0.1:18765/debug/resource_meta_query
# 读前 50 行
http://127.0.0.1:18765/debug/resource_meta_query?table=a&limit=50
# 单条查询
http://127.0.0.1:18765/debug/resource_meta_query?sql=SELECT%20name%20FROM%20sqlite_master%20LIMIT%205
```

前提：`ura_sqlcipher_hooks.flag` 已在（钥匙已捕获并落盘）。若报 `no_key_captured` → 带 flag 重启一次游戏。

SHA256 见 SHA256SUMS / BUILD-MANIFEST.txt。
