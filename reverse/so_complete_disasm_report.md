# libil2cpp.so 完整反汇编报告

**分析日期**: 2026-07-12
**SO 大小**: 209MB (ARM64, stripped)
**反汇编范围**: il2cpp 代码段 93MB (0x406f960 - 0x99bce3c)

---

## 一、反汇编统计

| 指标 | 数量 |
|------|------|
| 识别的函数 | 177,385 |
| 调用边 (bl) | 2,917,744 |
| 浮点操作 | 142,147 |
| 整数立即数 | 9,318 |
| 反汇编分块 | 90 (每块 1MB) |
| 总输出大小 | 886MB |

### 浮点操作分布

| 指令 | 数量 | 说明 |
|------|------|------|
| fmov | 42,289 | 浮点移动/加载立即数 |
| fmul | 30,275 | 浮点乘法 |
| fadd | 21,441 | 浮点加法 |
| fcmp | 19,377 | 浮点比较 |
| fsub | 13,669 | 浮点减法 |
| fdiv | 5,941 | 浮点除法 |
| scvtf | 5,671 | 整数→浮点转换 |
| fcvt | 1,859 | 浮点精度转换 |
| ucvtf | 1,580 | 无符号整数→浮点转换 |

### fmov 立即数值 (前 20)

| 值 | 次数 | 可能用途 |
|---|------|---------|
| 1.0 | 9,546 | 基础乘数 |
| 0.5 | 1,876 | 半值计算 |
| 10.0 | 1,127 | 百分比转换 |
| -1.0 | 1,101 | 负数 |
| -0.5 | 621 | 负半值 |
| 2.0 | 299 | 倍率 |
| 8.0 | 224 | — |
| 3.0 | 148 | — |
| 30.0 | 135 | 百分比 |
| 5.0 | 109 | — |
| 4.0 | 103 | — |
| 20.0 | 94 | 百分比 |

**注意**: 0.1, 0.05, 0.6, 0.8, 0.9, 1.1, 1.2 均**不在** fmov 立即数中。ARM64 fmov 只能编码特定浮点值，这些值需要 literal pool 加载，但在代码段中也未找到。

---

## 二、训练相关函数

### 2.1 浮点密集的训练函数

| 函数 | 浮点数 | fmul | fadd | dump 地址 |
|------|--------|------|------|----------|
| TrainingParamChangeUI::ScenarioFreeEventShopItemAddParam | 294 | 133 | 69 | 0x7339554254 |
| TrainingParamChangeUI::SetParameterInfoByTrainingPartnerPosition | 165 | 52 | 45 | 0x7339554f40 |
| TrainingParamChangeUI::SetScenarioLegendAcquiredBuff | 163 | 58 | 35 | 0x7339555c7c |
| SingleModeOnsenAPI::SendFactorLottery | 157 | 0 | 0 | 0x733914d034 |
| MasterChampionsEvaluationRate::CalcScore | 126 | — | — | — |

### 2.2 SetParameterInfoByTrainingPartnerPosition 分析

函数地址: 0x866177c, 大小: 2928 字节

该函数从训练对象的不同偏移读取 short 值（`ldrsh w9, [x22, #offset]`），是 UI 显示层函数，不做计算。

偏移读取模式:
- 0xf4, 0xf6, 0xf8, 0xfa, 0xfc, 0xfe → 连续 2-byte short 数组（5项属性）
- 0x100, 0x102, 0x154, 0x156 → 额外参数

---

## 三、心情倍率确认

### 3.1 搜索结论

| 搜索位置 | 0.1f | 0.05f | 0.6f | 0.8f | 0.9f | 1.1f | 1.2f |
|---------|------|-------|------|------|------|------|------|
| SO 代码段 (float32) | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| SO 代码段 (float64) | — | — | 0 | 0 | 0 | 0 | 0 |
| SO fmov 立即数 | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| metadata 字段默认值 | — | — | 有 | 有 | 有 | 有 | 有 |
| MDB 表 | — | — | — | — | — | — | — |

**结论**: 训练心情倍率在游戏文件中不以可直接读取的常量形式存在。

### 3.2 UmaAi 公式验证

UmaAi 使用公式: `1 + 0.1 × (motivation - 3) × (1 + 0.01 × ganJing)`

该公式的系数 0.1 和 0.05 在 SO 代码段中不存在 float32 表示。可能原因:
1. 游戏使用**整数运算**实现等效计算（如 `(motivation - 3) * 10 / 100`）
2. 游戏使用**不同的公式**（UmaAi 是逆推近似）
3. 系数通过运行时对象链传递，不是硬编码

---

## 四、调用图分析

### 4.1 训练执行调用链

```
SingleModeTrainingCommandService::ExecTraining (0x7339d4ac48)
  → 6 个 bl 调用（调度函数，不做计算）
    → ApplyExecCommand (0x7339dc9214)
      → 1 个 bl 调用（进一步调度）
```

训练增益计算被分散到多个子服务中，通过虚函数表间接调用。静态分析无法追踪完整调用链。

### 4.2 彩圈判定调用链

```
IsTagTrainingPartner (0x6d4f0f0)
  → 多层 blr x8 间接调用（虚函数分发）
    → IsTagTrainingSupportCardTypeFriend
    → IsTagTrainingSupportCardTypeGroup
    → HasGroupSupportCardCharaEffectId
    → IsTagTrainingScenarioEffect (各剧本专属)
```

---

## 五、反汇编文件清单

| 文件 | 大小 | 内容 |
|------|------|------|
| chunk_000.asm ~ chunk_089.asm | 各~8.3MB | 分块反汇编 |
| index.json | 143MB | 完整索引（函数/调用图/浮点/立即数） |
| so_disasm_index_summary.json | 28KB | 摘要索引（适合 git） |

反汇编文件在 `artifacts/disasm/` 目录（未加入 git，因体积过大）。

---

## 六、下一步方向

### 6.1 静态分析天花板

SO 是 stripped + IL2CPP 内联 + 虚函数分发。核心计算逻辑通过函数指针表间接调用，静态反汇编无法追踪完整调用链。已到达静态分析天花板。

### 6.2 建议的运行时方案

1. **Frida hook**: hook `GetMotivationCoef` 和训练计算函数，打印运行时返回值
2. **对比测试**: 不同心情值做同一训练，记录 gains 差异
3. **IL2CPP 运行时 dump**: 用 Zygisk-Il2CppDumper 获取完整字段偏移和方法地址
4. **frida-il2cpp-bridge**: 直接调用 IL2CPP 方法并拦截返回值

*分析: 2026-07-12, 基于 libil2cpp.so 93MB il2cpp 代码段完整反汇编*
