# 赛马娘 v2.28.5 完整逆向分析总结

**分析日期**: 2026-07-12
**分析者**: Nova ⚡

---

## 文件清单

| 文件 | 大小 | 状态 | 位置 |
|------|------|------|------|
| libil2cpp.so | 209MB | ✅ 完整反汇编 | artifacts/libil2cpp.so |
| global-metadata.dat | 44MB | ✅ 解析 | artifacts/global-metadata.dat |
| master.mdb | 42MB | ✅ 查询 | uma-data/master.mdb |
| IL2CPP method dump | 19MB | ✅ 索引 | data/il2cpp_dump/ |

## 反汇编覆盖

| 代码段 | 大小 | 函数数 | 浮点操作 | 状态 |
|--------|------|--------|---------|------|
| .text (Unity引擎) | 4.8MB | 1,881 | 2,678 | ✅ |
| il2cpp (游戏逻辑) | 93MB | 177,385 | 142,147 | ✅ |
| .plt (跳转表) | 9KB | — | — | ✅ |
| **总计** | **98MB** | **179,266** | **144,825** | ✅ |

反汇编输出: 926MB (95个asm文件), 在 artifacts/disasm/

## 已确认的完整映射

### 1. 支援卡效果类型 (SupportCardEffectType)
- type 1-32 完整枚举 (从 metadata 确认)
- type 1 = SpecialTagEffectUp = 友情加成(彩圈才乘)
- type 2 = MotivationUp = 干劲加成(逛街也加)
- type 3-7 = 副属性加成(逛街也加)
- type 8 = TrainingEffectUp = 训练加成(逛街也加)
- type 19 = GoodTrainingRateUp = 得意率
- type 27 = TrainingFailureRateDown
- type 28 = TrainingHPConsumptionDown

### 2. 特殊固有效果 (type 102-122)
- 44张卡, 21种触发条件
- type 101 = bond阈值 (60/80/100)
- type 120 = Orfevre "只、君臨す"
- type 112 = Nakayama Festa
- type 30 = Haru Urara
- 具体触发条件引擎硬编码,不在metadata枚举中

### 3. 训练增益公式
```
cardMultiplier = (1+0.05×heads) × (1+0.01×xunLian)
              × (1+0.1×(motiv-3)×(1+0.01×ganJing))
              × totalYouqingMultiplier
```
- 心情倍率是公式计算,不查表
- base: 0.8/0.9/1.0/1.1/1.2 (ganJing=0)
- 干劲加成放大心情效果

### 4. 彩圈判定逻辑
- 普通卡: bond≥threshold && 训练匹配
- 友人卡: 永远不彩圈
- 团体卡: TipsEventPartnerArray包含该partner → 彩圈
- 逛街卡: 出现在TrainingPartnerArray但训练不匹配 → 不彩圈但效果生效

### 5. 比赛心情倍率
- race_motivation_rate表: 0.96/0.98/1.0/1.02/1.04

### 6. SO基址和地址转换
- load_base = 0x7330ef37c4
- vaddr = dump_addr - load_base
- file_offset = vaddr - 0x406f960 + 0x406b960

## 未确认的

1. 训练心情倍率: 公式已知但游戏原生实现可能是整数运算
2. 特殊固有type 102-122触发条件: 需运行时hook
3. 彩圈虚函数完整调用链: 需运行时追踪
4. metadata字段偏移精确映射: fieldStart编码方式未完全确认

## 仓库文档索引

| 文件 | 内容 |
|------|------|
| reverse/effect_type_mapping.md | 效果类型完整映射 |
| reverse/mechanism_analysis.md | 心情/逛街/伙伴/彩圈机制 |
| reverse/unique_effect_analysis.md | 固有效果三层体系 |
| reverse/all_special_unique_effects.md | 44张特殊固有卡清单 |
| reverse/training_gain_formula.md | 训练增益完整公式 |
| reverse/so_complete_disasm_report.md | SO反汇编完整报告 |
| reverse/so_disasm_index_summary.json | 反汇编索引摘要 |
| reverse/il2cpp_key_classes_methods.txt | 16个核心类方法地址 |
| reverse/so_deep_analysis.md | SO深度分析 |
| reverse/WORK_REPORT.md | 工作报告 |

