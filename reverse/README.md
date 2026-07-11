# 赛马娘 v2.28.5 逆向工程报告索引

**生成时间**: 2026-07-11  
**游戏版本**: v2.28.5 (日服)  
**libil2cpp.so**: 209MB (ARM64, IL2CPP v31)  
**数据源**: IL2CPP方法转储 (27,695类, 160,909方法) + hlpatch插件源码 (1,856行) + libil2cpp.so二进制

---

## 文件列表

### 主报告
| 文件 | 大小 | 说明 |
|---|---|---|
| `master_analysis.md` | 33KB | **总报告** — 14剧本总览、核心数据路径、ID映射、支援卡、羁绊、训练、彩圈、角色、技能、比赛、事件、ObscuredInt加密、属性变化、AI评价 |
| `master_db_sql_queries.md` | 27KB | Master数据库SQL查询模板 — 角色ID、支援卡ID、技能、训练效果、剧本机制等查询语句 |
| `librs_offset_analysis.md` | 3.4KB | lib.rs已知偏移量分析 — 字段偏移、常量、映射表 |
| `so_string_search.md` | 14KB | libil2cpp.so字符串搜索 — 8,901个字符串，按关键词分类 |
| `all_classes_dump.md` | 49KB | 全量类名转储 — 按命名空间分类统计 |

### 剧本报告 (14个)
| 文件 | 剧本 | ID | 说明 |
|---|---|---|---|
| `scenario_01_URA.md` | URA育成シナリオ | 1 | 66个专用类，原始育成剧本 |
| `scenario_02_TeamRace.md` | チームレース | 2 | 团队竞赛剧本 |
| `scenario_03_Live.md` | ライブ | 3 | Live表演剧本 |
| `scenario_04_Free.md` | フリー | 4 | 自由剧本 |
| `scenario_05_Venus.md` | ヴィーナス | 5 | 维纳斯剧本 |
| `scenario_06_Arc.md` | アーク | 6 | Arc剧本 |
| `scenario_07_Sport.md` | スポーツ | 7 | 运动剧本 |
| `scenario_08_Cook.md` | クック | 8 | 料理剧本 |
| `scenario_09_Mecha.md` | メカ | 9 | 16个专用类，机甲剧本 |
| `scenario_10_Legend.md` | レジェンド | 10 | 14个专用类，传说剧本 |
| `scenario_11_Pioneer.md` | パイオニア/青春杯 | 11 | 11个专用类，青春/种田杯 |
| `scenario_12_Onsen.md` | 温泉 | 12 | 15个专用类，温泉剧本 |
| `scenario_13_Breeders.md` | ブリーダーズ/種田杯 | 13 | 10个专用类，育成者杯 |
| `scenario_14_Ramen.md` | ラーメン/トゥインクル・ラーメン杯 | 14 | 16个专用类，拉面杯 |
| `scenario_14_ramen_deep.md` | 拉面杯深度分析 | 14 | 97个类详细分析：Feeling、CheckPoint、Uraf、Tasting、Region、Command系统 |

### 参考文件 (前次分析)
| 文件 | 说明 |
|---|---|
| `uploaded_global_metadata_report.txt` | global-metadata.dat解析摘要 — 53,684类型定义, 336,729方法 |
| `uploaded_il2cpp_analysis_report.txt` | IL2CPP全量分析 — 35个子项，含偏移和逻辑描述 |
| `uploaded_il2cpp_class_dump.txt` | IL2CPP类名转储 — 11,499个类名 |

---

## 关键发现摘要

### ID映射系统
- **CommandId**: 101=Speed, 102=Stamina, **103=Guts(非Power!)**, **105=Power(非104!)**, 106=Wisdom
- **TargetType**: 1=Speed, 2=Stamina, 3=Guts, 4=Power, 5=Wiz, 10=HP, 20=Motivation, 30=SkillPt
- **Motivation**: 5=Best, 4=Good, 3=Normal, 2=Bad, 1=Worst
- **剧本ID**: 1=URA, 2=TeamRace, 3=Live, 4=Free, 5=Venus, 6=Arc, 7=Sport, 8=Cook, 9=Mecha, 10=Legend, 11=Pioneer, 12=Onsen, 13=Breeders, 14=Ramen

### 核心数据路径
```
WorkSingleModeData → WorkSingleModeCharaData (211 methods, 81 getters)
  ├→ get_SupportCardArray() → SupportCardEntry[] (+0x10=position, +0x14=card_id, +0x18=limit_break, +0x20=partner_state)
  ├→ get_EvaluationInfoArray() → EvaluationInfo[] (+0x10=target_id, +0x14=evaluation, +0x20=is_appear)
  ├→ get_TrainingLevelInfoArray() → TrainingLevelInfo[] (+0x10=command_id, +0x14=level)
  └→ get_HomeInfoData() → WorkSingleModeHomeInfoData
       └→ CommandInfoArray → SingleModeCommandInfoData[]
            ├→ get_CommandId() → ObscuredInt (101-106)
            ├→ get_IsEnable() → ObscuredInt (0/1)
            ├→ get_FailureRate() → ObscuredInt (%)
            ├→ get_TrainingPartnerArray() → 训练伙伴列表
            ├→ get_TipsEventPartnerArray() → 彩圈伙伴列表
            └→ get_ParamsIncDecInfoArray() → SingleModeParamsIncDecInfoData[]
                 ├→ get_TargetType() → ObscuredInt (1-30)
                 └→ get_Value() → ObscuredInt
```

### ObscuredInt加密
```
20 bytes inline: key(4) + hidden(4) + inited(4) + fake(4) + fakeActive(4)
解密: actual = key ^ hidden
共有 233 种 Obscured 类型
```

### 彩圈(Shining)判定
```
彩圈 = TipsEventPartnerArray.Length > 0
条件(推断): bond ≥ 80 + support_card_type 匹配 CommandId + specialty 匹配
```

### 拉面杯(Scenario 14)核心机制
- **Feeling系统**: 3种Feeling + SpecialFeeling，每种有FeelingId和RemainTurn
- **CheckPoint**: 检查点类型 + 结果状态 + 进度点数 (CheckPointPt)
- **Uraf**: 裏面效果系统 (UrafEffectType + UrafEffectState)
- **Tasting(试食会)**: LastTastingInfo记录上次试食的Feeling数量和RegionId
- **Region选择**: Junior/Classic/Senior各阶段选地区，AutoSelect支持
- **CommandFeeling**: 每个训练指令关联FeelingId
- **TrainingExec**: 记录BaseCommandId和ExecCount
- **ReduceBaseTurn**: FeelingId关联的减少基础回合数
- **ActiveEffect**: EffectCategory + EffectId + EffectValue

### 各剧本属性变化类 (WorkSingleModeChangeParameterInfoScenarioN)
每个剧本有独立的属性变化子类，包含剧本特有的getter：
- Scenario 10 (Legend): 18个getter (BuffGauge, FriendGauge, Masterly等)
- Scenario 13 (Breeders): 12个getter (TeamMember, EnhancePoint, DreamPoint等)
- Scenario 14 (Ramen): 7个getter (Feeling, CheckPoint, Uraf等)
- Scenario 8 (Cook): 7个getter (Dish, Material, CarePoint等)

### 特殊NPC (理事长/记者)
- 通过 EvaluationInfo.target_id 识别
- 理事长 → MiniDirector类系 (52方法)
- 记者 → 相关Dialog类系
- 非支援卡羁绊使用固定target_id

### AI评价系统
- 2801个评价值 (索引=总修正属性值, 值=评价分数)
- 基础五维上限: [2300, 2200, 1800, 1400, 1400]
- URA剧本78回合，其他72回合

---

## 限制说明

1. **global-metadata.dat 未提取**: Cygames加密了metadata，APK中未找到明文。需要运行时Zygisk-Il2CppDumper提取
2. **master.mdb 未提取**: Master数据库运行时从服务器下载，不在APK中
3. **libil2cpp.so 字符串有限**: 游戏类名被IL2CPP metadata加密，SO中只有Unity引擎层字符串
4. **字段偏移来源**: 来自lib.rs中实测验证的值，非静态分析结果
5. **RNG系统**: _fixedTurnCharaSeed (offset 408) 的精确算法需ARM64反汇编确认

---

*由 Nova ⚡ 基于hlpatch仓库逆向数据生成*
