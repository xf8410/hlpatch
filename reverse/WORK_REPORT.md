# 赛马娘 v2.28.5 全量逆向工程工作报告

**项目**: hlpatch（赛马娘育成辅助 URA Plugin）
**仓库**: https://github.com/xf8410/hlpatch
**任务**: 对赛马娘 v2.28.5 进行全量逆向分析，生成14个剧本的机制报告
**执行日期**: 2026-07-11
**执行者**: Nova ⚡（AI 助手）
**Commit**: 459a0e9

---

## 一、任务背景

用户此前委托另一个 AI 执行同样的逆向分析任务，该 AI 声称"推送成功"并结束了 session。用户次日查看仓库发现文件不存在——那个 AI 大概率口头说了"推送"但实际没执行 git push（或根本没 commit），session 结束后临时环境销毁，文件全部丢失。

用户上传了3个该 AI 遗留的纯文本文件（从聊天记录里抢救出来的）：
1. `global-metadata.dat 全量解析报告`（TypeDef 53,684个，MethodDef 336,729个）
2. `il2cpp 全量逆向完整数据`（11,499个类名列表）
3. `游戏内部机制逆向工程分析报告`（35个子项，含类名/方法名/偏移量/逻辑描述）

用户要求重跑，并给出了极其详细的需求清单——30+个具体机制：
- 赛马娘角色ID、支援卡ID、换皮ID、同名不同卡ID
- 卡加成、技能、卡hit、友情训练标志、普通友情训标志
- 支援卡效果结果、剧本场景效果结果、心情效果
- 5个等级训练效果、训练属性具体ID映射
- 支援卡槽位局内如何分辨用户带了什么卡
- 羁绊槽增加逻辑、拉面杯剧本机制逻辑、拉面杯伙伴逻辑
- 无彩圈训练逻辑、育成训练中支援卡概率5种训练都不出现逻辑
- 支援卡80羁绊概率不闪逻辑、特殊支援卡友人卡逻辑
- 满羁绊不在本训练触发支援卡固有效果技能逻辑
- 目标比赛逻辑、赛后获取技能pt和属性逻辑、马娘加成逻辑
- 每个3到5星的初始属性
- 拉面杯吃道具加成、选择地区加成、拉面拉人头、做面条道具、万能菜
- 拉面杯拉人头以后闪彩识别到彩圈逻辑
- 拉面杯试食会逻辑（类似种田杯）
- 非支援卡羁绊的理事长羁绊和记者的羁绊条识别逻辑
- 育成中连续事件概率、乱入事件概率
- 其他文件里有的逻辑也要补充

---

## 二、数据源评估

仓库里有4个关键文件：

| 文件 | 状态 | 问题 |
|---|---|---|
| `base.apk` | LFS指针(134字节) | git-lfs不可用，需手动下载 |
| `split_config.arm64_v8a.apk` | LFS指针(133字节) | 同上 |
| `dump.cs` | 0字节空文件 | 从未被填充 |
| `data/il2cpp_dump/dump_all_methods_ALL.json` | 19MB ✓ | 可用 |
| `src/lib.rs` | 1,856行 ✓ | 可用，含已知偏移量和业务逻辑 |

另有 `libhachimi_ura/libhachimi_ura.so`（编译产物）和 `docs/rng-hunt/`（RNG调试记录）。

---

## 三、遇到的问题及解决过程

### 问题1：LFS文件下载

**现象**: `base.apk` 和 `split_config.arm64_v8a.apk` 都是 LFS 指针文件（134/133字节），实际内容 107MB/77MB。环境没有安装 `git-lfs` 命令。

**尝试1**: `pip3 install git-lfs` — 失败，pip 包不提供 git lfs 命令。
**尝试2**: `apt-get install -y p7zip-full` — 失败，无 root 权限。

**解决方案**: 通过 GitHub LFS batch API 直接下载：

```python
url = f'https://github.com/{repo}.git/info/lfs/objects/batch'
req_data = json.dumps({
    'operation': 'download',
    'transfers': ['basic'],
    'objects': [{'oid': oid, 'size': size}]
}).encode()
```

**验证**: 下载后用 SHA256 校验：
- `base.apk`: 806a1eb0... → ✓ 匹配
- `split_config.arm64_v8a.apk`: 首次下载 SHA256 不匹配 ✗

**问题**: split APK 首次下载被 HTTP 截断（80764928 vs 期望 80877650 字节，少了约11KB）。

**解决**: 重新下载，这次加了进度监控和完整 SHA256 验证：
- 第二次下载: 4d8bd7dc... → ✓ 匹配

**教训**: LFS 下载必须验证 SHA256，HTTP 截断是真实存在的风险。

---

### 问题2：APK ZIP结构异常

**现象**: `base.apk`（107MB）无法用 Python `zipfile` 或 `unzip` 打开，报错 "End-of-central-directory signature not found"。

**排查过程**:
1. `file` 命令识别为 "Android package (APK)" — 文件格式没错
2. 读取前8字节: `504b0304` = `PK\x03\x04` — ZIP local file header 签名正确
3. 搜索文件尾部 EOCD 签名 `PK\x05\x06` — 未找到
4. 搜索 ZIP64 EOCD 签名 `PK\x06\x06` — 未找到
5. 扫描最后1MB所有字节 — 无任何 ZIP 结束标记

**结论**: base.apk 的 EOCD（End of Central Directory）缺失。这可能是 Cygames 的 APK 保护措施，或者是 XAPK/split 格式的特殊处理。

**解决方案**: 手动解析 local file header（`PK\x03\x04`），逐个提取 entry：
- 扫描到 3,801 个文件条目
- 每个条目读取: method(压缩方式) + comp_size + uncomp_size + filename + data
- 对 deflated 条目用 `zlib.decompress(data, -15)` 解压

**split_config.arm64_v8a.apk** 则是正常 ZIP，`zipfile` 可以打开，成功提取 `libil2cpp.so`（209MB）。

---

### 问题3：global-metadata.dat 未找到

**目标**: 从 base.apk 提取 `global-metadata.dat`（IL2CPP metadata，magic: 0xFAB11BAF）。

**尝试1**: 在 base.apk 原始字节中搜索 magic `AF 1B B1 FA` — 未找到。
**尝试2**: 逐个解压 base.apk 的 3,801 个条目，检查解压后数据前4字节是否为 magic — 未找到。
**尝试3**: 搜索文件名含 "metadata" 或 "global" 的条目 — 未找到。

**结论**: Cygames 对 global-metadata.dat 进行了加密，不以明文形式存储在 APK 中。这与之前已知的信息一致——需要运行时用 Zygisk-Il2CppDumper 绕过加密提取。

**替代方案**: 使用已有的 IL2CPP 方法转储 `dump_all_methods_ALL.json`（27,695类/160,909方法），这是运行时反射的结果，包含完整的类名和方法签名。

---

### 问题4：master.mdb 未找到

**现象**: base.apk 中 3,801 个条目里没有 `master.mdb` 或任何 `.mdb`/`.db` 文件。

**结论**: Master数据库运行时从 Cygames 服务器下载，不打包在 APK 中。具体数值表（角色ID、支援卡ID、技能数据等）无法从静态文件提取。

**替代方案**: 生成 SQL 查询模板（`master_db_sql_queries.md`），列出需要的表和查询语句，供运行时查询使用。

---

### 问题5：dump.cs 为空文件

**现象**: 仓库中的 `dump.cs` 文件大小为 0 字节。

**结论**: 该文件从未被填充。可能是占位符或从未执行过 il2cppdumper 静态 dump（因为 Cygames 加密了 metadata，静态 dump 无法工作）。

**替代方案**: 同问题3，使用运行时方法转储。

---

### 问题6：IL2CPP method dump 格式问题

**现象**: 分析脚本首次运行报错：`AttributeError: 'dict' object has no attribute 'lower'`。

**排查**: 检查 JSON 结构发现 method 不是字符串而是字典：
```json
{"name": "get_acquiredSkill", "addr": "0x7335039720", "params": 0, "return_type": "type_19", "static": false}
```

**修复**: 添加 `mname()` 函数提取方法名：
```python
def mname(m):
    return m['name'] if isinstance(m, dict) else str(m)
```

---

### 问题7：sed 全局替换破坏脚本

**现象**: 用 `sed -i` 批量替换 `for m in ... if 'get_' in m.lower()` 模式时，部分替换不完整或破坏了代码结构。

**修复**: 重写整个分析脚本，避免使用 sed 批量替换。这与之前血泪教训一致——**sed 全局替换会搞坏代码，以后只用 Edit 工具精确替换**。

---

### 问题8：OrderedDict 未导入

**现象**: 脚本运行报 `NameError: name 'OrderedDict' is not defined`。

**修复**: 在 import 行添加 `from collections import defaultdict, OrderedDict`。

---

### 问题9：libil2cpp.so 字符串搜索

**现象**: 209MB 的 SO 文件，字符串提取需要遍历整个文件。直接在 Python 中单线程扫描，约需3-4分钟。

**过程**:
- 首次尝试在前台运行，超过 bash 默认 2 分钟超时
- 改用 `nohup` 后台运行 + 日志文件
- 最终完成，提取出 8,901 个字符串（>=8字符）

**结果**: SO 中的字符串全是 Unity 引擎层符号（mono_*, UnityEngine.* 等），游戏类名被 IL2CPP metadata 加密，运行时才解密。这与预期一致。

**验证**: 搜索 "Ramen"、"SingleMode"、"Training" 等关键词，均未在游戏代码区域找到匹配——确认 metadata 加密的事实。

---

### 问题10：后台进程管理

**现象**: 多次尝试用 `nohup ... &` 后台运行脚本时，bash 工具报告 "执行完成" 但没有输出。

**排查**: 
- `ps aux` 发现进程确实在运行
- 但 bash 工具的 session 管理会杀死子进程
- 部分情况下需要用 `setsid` 或 `disown`

**解决**: 对于长时间运行的任务，改用前台运行 + 设置足够的 timeout（如 600000ms = 10分钟）。

---

## 四、成功完成的部分

### 最终生成的文件（24个）

| 文件 | 大小 | 说明 |
|---|---|---|
| `README.md` | 索引 | 总目录+关键发现摘要 |
| `master_analysis.md` | 33KB | **总报告** |
| `master_db_sql_queries.md` | 27KB | SQL查询模板 |
| `librs_offset_analysis.md` | 3.4KB | 已知偏移量 |
| `so_string_search.md` | 14KB | SO字符串搜索 |
| `all_classes_dump.md` | 49KB | 27695类分类统计 |
| `scenario_01_URA.md` | 24KB | URA剧本 |
| `scenario_02_TeamRace.md` | 19KB | 团队竞赛 |
| `scenario_03_Live.md` | 19KB | Live剧本 |
| `scenario_04_Free.md` | 18KB | 自由剧本 |
| `scenario_05_Venus.md` | 18KB | 维纳斯 |
| `scenario_06_Arc.md` | 18KB | Arc |
| `scenario_07_Sport.md` | 19KB | 体育 |
| `scenario_08_Cook.md` | 23KB | 烹饪 |
| `scenario_09_Mecha.md` | 17KB | 机体 |
| `scenario_10_Legend.md` | 22KB | 传说 |
| `scenario_11_Pioneer.md` | 16KB | 青春杯 |
| `scenario_12_Onsen.md` | 20KB | 温泉 |
| `scenario_13_Breeders.md` | 25KB | 种田杯 |
| `scenario_14_Ramen.md` | 22KB | 拉面杯 |
| `scenario_14_ramen_deep.md` | 30KB | 拉面杯深度分析（97类） |
| `uploaded_global_metadata_report.txt` | — | 前次AI遗留参考 |
| `uploaded_il2cpp_analysis_report.txt` | — | 前次AI遗留参考 |
| `uploaded_il2cpp_class_dump.txt` | — | 前次AI遗留参考 |

### 关键技术发现

1. **14个剧本ID完整映射**: 1=URA, 2=TeamRace, ..., 14=Ramen
2. **CommandId 映射非对称**: 根性=103、力量=105（非直觉的103/104/105顺序）
3. **support_card_type → training_id 映射也不按数值顺序**: 1→Speed(101), 2→Stamina(102), 3→Power(105), 4→Guts(103), 5→Wisdom(106)
4. **ObscuredInt 加密**: 20字节/key+hidden，key⊕hidden 解密
5. **彩圈判定**: bond≥80 + support_card_type 匹配当前训练 CommandId
6. **核心数据路径**: WorkDataManager → WorkSingleModeData → WorkSingleModeCharaData
7. **拉面杯有97个专用类**: 包含 Feeling/CheckPoint/Uraf/Tasting/Region 等子系统
8. **libil2cpp.so 仅8901个字符串**: 全是 Unity 引擎层，游戏类名被 metadata 加密

### 推送结果

```
commit 459a0e9
27 files changed, 20276 insertions(+)
git push origin main → 成功
```

---

## 五、成功后仍然存在的问题

### 5.1 静态文件无法提取的数据

| 缺失数据 | 原因 | 解决方案 |
|---|---|---|
| global-metadata.dat | Cygames加密，不在APK中 | 运行时Zygisk-Il2CppDumper提取 |
| master.mdb | 运行时从服务器下载 | 运行时从游戏内存读取或网络抓包 |
| dump.cs | 0字节空文件 | 不可用，使用运行时dump替代 |

### 5.2 报告中无法覆盖的需求

以下用户需求需要运行时数据才能完整回答：

1. **角色ID完整映射表** — 需要 master.mdb 的 `chara_data` 表
2. **支援卡ID完整映射表** — 需要 master.mdb 的 `support_card_data` 表
3. **换皮ID/同名不同卡ID** — 需要 master.mdb 跨表查询
4. **每个3到5星的初始属性** — 需要 master.mdb 的 `chara_data` 表
5. **卡有什么加成/技能** — 需要 master.mdb 的 `support_card_data` + `skill_data` 表
6. **具体概率值**（连续事件、乱入事件、支援卡不出现等） — 需要 master.mdb 或 ARM64 反汇编
7. **RNG算法** — `_fixedTurnCharaSeed` (offset 408) 的精确算法需 ARM64 反汇编
8. **拉面杯道具系统具体数值** — Sozai/Ingredient/Recipe 等需要运行时内存扫描或 master.mdb

### 5.3 报告深度不足的领域

- **训练增益精确公式** — 需对 `ExecTraining` 等方法做 ARM64 反汇编
- **心情效果具体倍率** — 知道5档(Best/Good/Normal/Bad/Worst)但缺倍率值
- **友情训练标志具体判定** — 知道数据路径但缺判定逻辑细节
- **支援卡80羁绊不闪概率** — 知道判定条件(bond≥80)但缺不闪的触发概率
- **友人卡/团队卡特殊逻辑** — 知道 type 不在1-5但缺完整逻辑

### 5.4 SO字符串搜索局限性

libil2cpp.so 中的 8,901 个字符串全部是 Unity 引擎层符号。搜索 "Ramen"、"SingleMode"、"Training" 等关键词均无游戏代码匹配——确认 Cygames 对 IL2CPP metadata 做了加密，类名和方法名只在运行时解密到内存中。

这意味着：纯静态分析 libil2cpp.so 无法获取游戏逻辑信息，必须依赖运行时 dump（即 `dump_all_methods_ALL.json`）或运行时内存扫描。

---

## 六、过程时间线

| 时间 | 事件 |
|---|---|
| 10:00 | 用户下达任务，上传3个参考文件 |
| 10:01 | 读取参考文件，评估仓库数据源 |
| 10:01 | 发现 APK 是 LFS 指针，git-lfs 不可用 |
| 10:01-10:02 | 通过 GitHub LFS API 下载 base.apk (107MB) |
| 10:02-10:04 | 下载 split_config.arm64_v8a.apk，首次下载不完整 |
| 10:04 | SHA256 验证发现 split APK 不完整，重新下载 |
| 10:04-10:05 | 第二次下载成功，SHA256 验证通过 |
| 10:05 | 尝试 zipfile 打开 base.apk — 失败(EOCD缺失) |
| 10:05-10:06 | 手动解析 local file header，扫描3801个条目 |
| 10:06-10:07 | 搜索 global-metadata.dat magic — 未找到 |
| 10:07-10:08 | 搜索 master.mdb — 未找到 |
| 10:08 | 确认 dump.cs 为0字节 |
| 10:08-10:10 | 编写分析脚本 |
| 10:10-10:11 | 首次运行报错(method是dict不是str) |
| 10:11 | 修复方法名提取函数 |
| 10:11-10:12 | sed 批量替换破坏脚本 |
| 10:12 | 重写分析脚本 |
| 10:12-10:13 | 运行报错(OrderedDict未导入) |
| 10:13 | 修复 import |
| 10:13-10:15 | 成功生成20个报告文件 |
| 10:15-10:17 | 后台运行 SO 字符串搜索(209MB) |
| 10:17 | SO 搜索完成，8901个字符串 |
| 10:17-10:18 | 复制上传参考文件到 reverse/ |
| 10:18 | 生成 README.md 总索引 |
| 10:18-10:19 | git add + commit + push |
| 10:19 | 推送成功 (commit 459a0e9) |

---

## 七、经验总结

### 7.1 工具链经验

1. **LFS 下载不靠 git-lfs**: 环境无 git-lfs 时，通过 GitHub LFS batch API 下载是可行方案。必须验证 SHA256，HTTP 截断是真实风险。
2. **APK EOCD 缺失**: Cygames 的 base.apk 可能做了特殊处理，Python zipfile/unzip 均无法打开。手动解析 local file header 是可靠的 fallback。
3. **SO 字符串搜索**: 209MB 文件需要约3分钟，必须用 nohup 或设置足够 timeout。结果只有 Unity 引擎层字符串，游戏类名被 metadata 加密。
4. **IL2CPP method dump 格式**: JSON 中的 method 是 dict（含 name/addr/params/return_type/static），不是字符串。处理时必须用 `m['name']` 而非 `str(m)`。

### 7.2 流程经验

1. **不能轻信其他 AI 的"推送成功"声明**: 那个 AI 大概率口头说了"推送"但实际没执行 git push，session 结束后文件全丢。验证方式：查 `git log` 看 commit 是否存在。
2. **大任务先理清数据源再写脚本**: 30+个需求不可能手动分析，必须先确定数据源（IL2CPP dump + lib.rs + SO），再写批量分析脚本。
3. **sed 批量替换是陷阱**: 与之前血泪教训一致。sed 会搞坏代码结构，尤其是包含正则特殊字符的模式。只用 Edit 工具精确替换。
4. **长时间运行的任务用 nohup**: SO 搜索需要3分钟，超过 bash 默认2分钟超时。用 nohup + 日志文件 + 后续检查日志是可靠方案。

### 7.3 数据局限性认知

1. **静态分析的天花板**: Cygames 加密了 metadata 和 master.db，纯静态分析 APK 能获取的只有类结构和方法签名。具体数值（概率、倍率、ID映射）需要运行时数据。
2. **IL2CPP method dump 的价值**: 虽然拿不到具体数值，但 27,695 个类的 160,909 个方法签名足以构建完整的类层次结构和数据访问路径。
3. **lib.rs 的偏移量是金标准**: 这些是运行时实测验证的值，比静态分析更可靠。报告中所有偏移量均来自 lib.rs。

---

## 八、后续建议

1. **运行时提取 global-metadata.dat**: 用 Zygisk-Il2CppDumper 在游戏运行时 dump metadata，补充完整的类字段信息。
2. **运行时提取 master.mdb**: 通过网络抓包或内存扫描获取 Master 数据库，填充角色ID、支援卡ID、技能数据等具体数值。
3. **ARM64 反汇编关键方法**: 对 `ExecTraining`、`CalcTrainingGain`、`DetermineTrainingPartner` 等方法做反汇编，确定训练增益公式和 RNG 算法。
4. **拉面杯道具系统**: 进一步搜索 Sozai/Ingredient/Recipe 等关键词，或通过运行时内存扫描定位食材计数。
5. **补充之前的遗留任务**: skills count=0（用错 getter 名）、bond_gain 需前后快照对比、static mut → Atomic 等。

---

*报告生成: 2026-07-12*
*由 Nova ⚡ 基于 hlpatch 仓库逆向数据编写*
