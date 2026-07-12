# SO 深度反汇编分析报告

**来源**: libil2cpp.so (209MB, ARM64, stripped) + global-metadata.dat (44MB, v31) + master.mdb (42MB)
**分析日期**: 2026-07-12
**SO 基址**: 0x7330ef37c4 (通过 IL2CPP dump 最小地址反算)

---

## 一、ELF 结构

| 段 | vaddr | file_offset | size | 属性 |
|----|-------|-------------|------|------|
| .text | 0x3b9ff4c | 0x3b9bf4c | 0x4cfa14 | 可执行 |
| il2cpp | 0x406f960 | 0x406b960 | 0x594d4dc | 可执行 |
| .plt | 0x99bce40 | 0x99b8e40 | 0x2440 | 可执行 |
| 数据段1 | 0x99c3280 | 0x99bb280 | 0x66a680 | 可写 |
| 数据段2 | 0xa031900 | 0xa025900 | 0x306f238 | 可写 |

il2cpp 代码段包含绝大部分游戏逻辑（~93MB）。

## 二、心情倍率搜索结果

### 2.1 搜索范围
- ✅ master.mdb: race_motivation_rate 表（比赛用，0.96-1.04）
- ❌ master.mdb: 所有 single_mode_* 表（无训练心情倍率）
- ❌ global-metadata.dat: field_default_values（无心情倍率常量）
- ❌ libil2cpp.so 代码段: float32 0.6/0.8/0.9/1.1/1.2 — 零命中
- ❌ libil2cpp.so 代码段: float64 同上 — 零命中
- ❌ libil2cpp.so 代码段: int32 6000/8000/9000/10000/11000/12000 — 无聚集
- ❌ libil2cpp.so 代码段: ARM64 fmov 可编码值（0.59375/0.625/0.875等） — 零命中
- ❌ metadata + SO: int32 数组 [60,80,90,100,110,120] — 不存在

### 2.2 结论

**训练心情倍率不以任何常量形式存在于游戏文件中。**

可能的原因：
1. **运行时公式计算**: 倍率通过 `(base + (motivation - offset) * step)` 计算
2. **服务器下发**: 训练配置从服务器获取
3. **社区值可能不准确**: 0.6/0.8/1.0/1.1/1.2 可能只是近似值
4. **使用比赛倍率**: 训练也可能使用 race_motivation_rate (0.96-1.04)，社区值是错的

### 2.3 唯一确认的倍率

比赛心情倍率（race_motivation_rate）:

| 心情 | 倍率 | 万分比 |
|------|------|--------|
| 1 絶不調 | 0.96 | 9600 |
| 2 不調 | 0.98 | 9800 |
| 3 普通 | 1.00 | 10000 |
| 4 好調 | 1.02 | 10200 |
| 5 絶好調 | 1.04 | 10400 |

## 三、关键函数定位

dump 地址 → vaddr → file_offset 转换:
- load_base = 0x7330ef37c4
- vaddr = dump_addr - load_base
- file_offset = vaddr - 0x406f960 + 0x406b960

### 3.1 已定位函数

| 函数 | dump 地址 | 验证结果 |
|------|----------|---------|
| ExecTraining | 0x7339d4ac48 | 调度函数，6个bl调用，无浮点操作 |
| ApplyExecCommand | 0x7339dc9214 | 调度函数，1个bl调用，无浮点操作 |
| IsTagTrainingPartner | 0x7339d4f254 | 复杂调度函数，blr间接调用 |
| GetMotivationCoef | 0x73377e5264 | 比赛用，大量对象空检查 |
| GetBestTraining | 0x733893dd54 | 返回 command_id 的 getter |
| IsHaveUniqueEffect | 0x7339eab3f4 | — |

### 3.2 dump 地址偏差

IL2CPP dump 中的地址指向函数**内部**某条指令，不是函数入口。多个相邻方法的地址可能只差 4-16 字节，说明它们被内联到一个编译单元中。

## 四、SO 结构特征

- **stripped**: 无符号表，无函数名字符串
- **IL2CPP 内联**: 小方法（getter/setter）被内联到大函数中，无法单独定位
- **虚函数分发**: 彩圈判定等核心逻辑通过函数指针表（vtable）间接调用
- **三层调度**: ExecTraining → ApplyExecCommand → 子服务，计算逻辑分散

## 五、结论

### 确认的
- ✅ 比赛心情倍率: 0.96/0.98/1.0/1.02/1.04
- ✅ 训练基础值: single_mode_training_effect 表
- ✅ 效果类型映射: type 1-32 完整枚举
- ✅ 特殊固有分类: 44张卡，21种条件
- ✅ SO 基址和地址转换公式

### 未确认的
- ❌ 训练心情倍率: 不在游戏文件中以常量存在
- ❌ 特殊固有 type 102-122 触发条件: 需更深层逆向
- ❌ 彩圈完整逻辑: 虚函数分发无法静态追踪
- ❌ 得意率→概率转换: 同上

### 下一步方向

1. **运行时 hook**: 在设备上 hook `GetMotivationCoef` 或训练计算函数，直接打印返回值
2. **Frida 脚本**: 用 frida-il2cpp-bridge hook IL2CPP 方法，获取运行时参数和返回值
3. **对比测试**: 用不同心情值做同一训练，记录 gains 差异，反推倍率

*分析: 2026-07-12, 基于 libil2cpp.so + global-metadata.dat + master.mdb 完整搜索*
