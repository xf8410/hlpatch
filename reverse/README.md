# hlpatch 逆向工程完整索引

**项目**: 赛马娘 v2.28.5 全量逆向分析
**仓库**: https://github.com/xf8410/hlpatch
**更新日期**: 2026-07-12

---

## 一、逆向数据文件 (JSON)

### 支援卡
| 文件 | 大小 | 内容 |
|------|------|------|
| `all_support_cards_full.json` | 1.8MB | 541张卡全属性(基本+effect_table全等级+unique_effect全等级) |
| `all_special_unique_effects.md` | — | 44张特殊固有卡清单(type 30,102-122) |
| `effect_type_mapping.md` | — | SupportCardEffectType枚举完整映射(type 1-32) |
| `unique_effect_analysis.md` | — | 三层效果体系(面板/标准固有/特殊条件固有) |

### 马娘
| 文件 | 内容 |
|------|------|
| `all_chara_cards_complete.json` | 169角色/258换皮卡/805星级属性(★3-5)完整 |
| `chara_relation_ranking.txt` | 高相性组合排列(8点28组/7点10组/2点150组) |

### 技能
| 文件 | 内容 |
|------|------|
| `all_skill_and_factor_data.json` | 2099技能+技能升级条件+特殊条件+继承技能 |

### 因子/继承
| 文件 | 内容 |
|------|------|
| `succession_complete.json` | 因子(2517)+效果(6508)+相性(3037+7249成员)+种马(7450)+等级阈值+初始因子条件 |

### 事件
| 文件 | 内容 |
|------|------|
| `all_event_data.json` | 支援卡事件(1178)+触发条件(478)+优先级(81)+加成(945)+启发(4919) |
| `all_story_and_ramen_events.json` | 故事事件(19943)+角色固定(917)+拉面杯完整 |

### MDB 全量
| 文件 | 大小 | 内容 |
|------|------|------|
| `mdb_table_catalog.json` | 165KB | 619张表完整目录(名称/行数/列名) |
| `mdb_key_tables_export.json` | 4.4MB | 14张关键表(支援卡/训练/NPC/心情/拉面杯) |
| `mdb_all_scenario_tables.json` | 583KB | 74张剧本专属表(6个剧本完整) |
| `mdb_game_mode_tables.json` | 3.1MB | 80+张游戏模式表(champions/team/ultimate/collect/crane) |
| `mdb_all_remaining_tables.json` | 6.9MB | 29张补充表(动画/语音/因子/技能/比赛) |
| `mdb_export_a_g.json` | 60MB | A-G 开头的表全量(134张) |
| `mdb_export_h_r.json` | 29MB | H-R 开头的表全量(162张) |
| `mdb_export_s_z.json` | 67MB | S-Z 开头的表全量(323张) |

**MDB 合计: 619张表 / 524,170行 / 100%覆盖**

---

## 二、逆向分析报告 (MD)

### 核心机制
| 文件 | 内容 |
|------|------|
| `training_gain_formula.md` | 训练增益完整公式(心情×干劲×友情×训练×人头) |
| `mechanism_analysis.md` | 心情/逛街/伙伴/彩圈完整机制 |
| `complete_call_chain.md` | 训练执行调用链(服务器端计算确认) |
| `complete_analysis_summary.md` | 全部逆向成果总结 |

### SO 反汇编
| 文件 | 内容 |
|------|------|
| `so_complete_disasm_report.md` | 98MB代码全反汇编(179K函数/144K浮点) |
| `so_deep_analysis.md` | SO深度分析(基址/函数地址/心情倍率搜索) |
| `so_disasm_index_summary.json` | 反汇编索引摘要(179K函数/291万调用) |
| `so_float_constants.txt` | 316个唯一浮点常量(adrp+ldr追踪) |
| `so_disasm_analysis.md` | 初步反汇编分析 |
| `so_string_search.md` | SO字符串搜索结果 |

### APK 分析
| 文件 | 内容 |
|------|------|
| `apk_complete_analysis.md` | APK完整分析(3801文件+native库+classes.dex) |
| `injection_and_anticheat_analysis.md` | 反作弊+插件注入(241个IL2CPP API+_Cyan.dll) |
| `packet_interception_analysis.md` | 服务器请求拦截(三个Hook点+SSL不需绕过) |

### 三文件状态
| 文件 | 内容 |
|------|------|
| `three_files_complete_status.md` | libil2cpp.so + global-metadata.dat + master.mdb 最终扒光状态 |

### 剧本报告
| 文件 | 内容 |
|------|------|
| `scenario_01_URA.md` ~ `scenario_14_Ramen.md` | 14个剧本的IL2CPP类分析 |
| `scenario_14_ramen_deep.md` | 拉面杯深度分析 |

### IL2CPP
| 文件 | 内容 |
|------|------|
| `il2cpp_all_classes_by_category.json` | 27,695类按14分类导出(160,909方法) |
| `il2cpp_key_classes_methods.txt` | 16个核心类方法+地址索引 |
| `all_classes_dump.md` | 全量类名dump |

### 工作报告
| 文件 | 内容 |
|------|------|
| `WORK_REPORT.md` | 逆向工程工作报告 |
| `master_analysis.md` | MasterDB分析 |
| `master_db_sql_queries.md` | SQL查询模板 |
| `librs_offset_analysis.md` | lib.rs偏移量分析 |

---

## 三、工具脚本 (Python)

| 文件 | 用途 |
|------|------|
| `analyze.py` | IL2CPP dump分析脚本 |
| `extract_metadata.py` | global-metadata.dat解析器 |
| `so_search_script.py` | SO字符串搜索脚本 |

---

## 四、IL2CPP 剧本映射

| ID | 剧本 | 报告 |
|----|------|------|
| 1 | URA | scenario_01_URA.md |
| 2 | 青春杯 | scenario_02_TeamRace.md |
| 3 | LIVE | scenario_03_Live.md |
| 4 | 自由赛 | scenario_04_Free.md |
| 5 | 维纳斯 | scenario_05_Venus.md |
| 6 | 开发者杯 | scenario_06_Arc.md |
| 7 | 体育杯 | scenario_07_Sport.md |
| 8 | 厨艺杯 | scenario_08_Cook.md |
| 9 | 机甲杯 | scenario_09_Mecha.md |
| 10 | 传奇杯 | scenario_10_Legend.md |
| 11 | 开拓者杯 | scenario_11_Pioneer.md |
| 12 | 温泉杯 | scenario_12_Onsen.md |
| 13 | 育成者杯 | scenario_13_Breeders.md |
| 14 | 拉面杯 | scenario_14_Ramen.md + scenario_14_ramen_deep.md |

---

## 五、关键结论

1. **训练增益**: 服务器端计算，客户端只存储显示
2. **心情倍率**: 公式计算 `1+0.1×(motiv-3)×(1+0.01×ganJing)`，不查表
3. **彩圈判定**: 普通卡=bond+训练匹配 / 友人卡=不彩圈 / 团体卡=TipsEvent
4. **反作弊**: _Cyan.dll(未知) + Google.Play.Integrity + libnative.so(SQLCipher+mbedTLS)
5. **拦包**: 三个Hook点已实现(CompressRequest/Post/DecompressResponse)，SSL不需绕过
6. **透视**: 能提前读取当前回合结果，不能预测未来
7. **IL2CPP API**: 241个导出函数，可通过dlsym直接调用
8. **G1因子**: 获取概率在服务器端，MDB只有因子定义不含概率

---

## 六、反汇编数据(未入git)

| 路径 | 大小 | 内容 |
|------|------|------|
| `artifacts/disasm/` | 926MB | 95个ASM分块文件(90 il2cpp + 5 .text) |
| `artifacts/disasm/index.json` | 143MB | 完整索引(函数/调用图/浮点/立即数) |
| `artifacts/libil2cpp.so` | 209MB | 原始SO |
| `artifacts/global-metadata.dat` | 44MB | 原始metadata |
| `artifacts/libnative.so` | 2.5MB | Cygames native库(curl+sqlite+mbedTLS) |
| `artifacts/libunity.so` | 18MB | Unity引擎 |

*最后更新: 2026-07-12*
