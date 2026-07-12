# SO 反汇编分析报告

**来源**: libil2cpp.so (ARM64, stripped) + global-metadata.dat + master.mdb
**分析日期**: 2026-07-12

---

## 一、心情倍率

### 1.1 比赛心情倍率 — 已确认

来源: `race_motivation_rate` 表 (master.mdb)

| id | 心情 | motivation_rate | 倍率 |
|:--:|------|:--------------:|:----:|
| 1 | 絶不調 | 9600 | 0.96 |
| 2 | 不調 | 9800 | 0.98 |
| 3 | 普通 | 10000 | 1.00 |
| 4 | 好調 | 10200 | 1.02 |
| 5 | 絶好調 | 10400 | 1.04 |

**注意**: 这是**比赛**用倍率，不是训练倍率。

### 1.2 训练心情倍率 — 未确认

训练心情倍率**不在 master.mdb 的任何表中**。已检查所有 `single_mode_*` 表，没有包含 motivation/rate/coef 字段的。

社区公认训练倍率（0.6/0.8/1.0/1.1/1.2）在 libil2cpp.so 机器码中硬编码。

**SO 搜索结果**: float32 常量 0.6(15次), 0.8(20次), 0.9(9次), 1.0(20次), 1.1(3次), 1.2(9次) 分散在代码段中，但没有找到它们作为数组的聚集。

`GetMotivationCoef` 函数 (vaddr 0x47e5154) 反汇编显示它做大量对象空检查和属性加载，最终 tail-call 到 0x519f434——心情系数可能是通过对象链间接计算的，不是简单的查表。

**结论**: 训练心情倍率需要更深入的反汇编或运行时 hook 确认。

---

## 二、训练基础值

### 2.1 single_mode_training_effect 表

来源: master.mdb

| 字段 | 说明 |
|------|------|
| command_id | 训练类型 (101=Speed, 102=Stamina, 103=Guts, 105=Power, 106=Wiz) |
| sub_id | 子ID（可能对应训练等级） |
| result_state | 结果状态 (2=成功) |
| target_type | 目标属性 (1=Speed, 2=Stamina, 3=Guts, 4=Power, 5=Wiz, 30=SkillPt, 10=HP) |
| effect_value | 效果值（基础训练量） |
| scenario_id | 剧本ID |

### 2.2 示例数据

```
command_id=101 (Speed), sub_id=1, result_state=2:
  target_type=1  (Speed),   effect_value=11
  target_type=3  (Guts),    effect_value=6
  target_type=30 (SkillPt), effect_value=4
  target_type=10 (HP),      effect_value=-21
```

Speed 训练 LV1 成功时: Speed+11, Guts+6, SkillPt+4, HP-21

---

## 三、NPC 数据

### 3.1 single_mode_npc 表

NPC 有完整属性:
- speed/stamina/pow/guts/wiz
- proper_distance/running_style/ground
- motivation_min/max
- skill_set_id

NPC 的心情范围由 `motivation_min` 和 `motivation_max` 控制。

---

## 四、IL2CPP 函数地址映射

### 4.1 ELF 布局

```
il2cpp 代码段: vaddr=0x406f960, file_offset=0x406b960, size=0x594d4dc
运行时 load_base ≈ 0x7333000000
```

### 4.2 关键函数地址

| 函数 | dump 地址 | vaddr | file_offset |
|------|----------|-------|-------------|
| GetMotivationCoef | 0x73377e5264 | 0x47e5264 | 0x47e1264 |
| AdjustByMotivation | 0x73377e5368 | 0x47e5368 | 0x47e1368 |
| IsTagTrainingPartner | 0x7339d4f254 | 0x6d4f254 | 0x6d4b254 |
| ExecTraining | 0x7339d4ac48 | 0x6d4ac48 | 0x6d46c48 |
| GetBestTraining | 0x733893dd54 | 0x593dd54 | 0x5939d54 |
| IsHaveUniqueEffect | 0x7339eab3f4 | 0x6eab3f4 | 0x6ea73f4 |
| GetMasterUniqueEffect | 0x7339ea9838 | 0x6ea9838 | 0x6ea5838 |

### 4.3 dump 地址偏差

dump 中的地址是函数尾部（ret 之前），不是函数入口。函数入口需要向前搜索 prologue (`str x30, [sp, #-N]!` 或 `stp x29, x30`)。

---

## 五、待完成

1. ✅ 比赛心情倍率 — 从 MDB 确认 (0.96/0.98/1.0/1.02/1.04)
2. ❌ 训练心情倍率 — 不在 MDB 中，需要反汇编或 hook
3. ✅ 训练基础值 — single_mode_training_effect 表
4. ❌ 特殊固有 type 102-122 触发条件 — 需要反汇编 IsHaveUniqueEffect
5. ❌ 彩圈判定完整逻辑 — 需要反汇编 IsTagTrainingPartner
6. ❌ 得意率→概率分布转换 — 需要反汇编 distribution 构建函数

*分析: 2026-07-12, 基于 libil2cpp.so + global-metadata.dat + master.mdb*
