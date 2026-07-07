# RNG Seed Hunt - Session 2026-07-08

## 目标
找到赛马娘训练中使用的随机数生成器，实现"训练前透视"。

## 关键发现

### 1. RaceRandom 结构
- NS: `StandaloneSimulator`
- 字段: `_random` @ offset 0x10 (16), type 18（值类型）
- 方法: `GetRandom(i4x2, i4x1, r4x2, r4x1)`
- 接口: `IRaceRandomGenerator`

### 2. RaceManager（比赛专用，训练时不可用）
- NS: `Gallop`
- `_randomGenerator` @ offset 0x90 (144) → RaceRandom 实例
- `_instance` @ offset 0x8（静态字段）
- **训练时 `_instance` = 0（null）**，RaceManager 仅比赛场景存在

### 3. WorkSingleModeData — 训练数据主入口 ⭐
- 通过 `WorkDataManager.SingleMode` 访问
- `WorkDataManager` @ `0x72625f1960`（单例，地址会变）
- `WorkSingleModeData` @ `0x7479c02000`
- **`_fixedTurnCharaSeed` @ offset 408** — 训练回合种子！

### 4. 训练执行入口
- `SingleModeTrainingCommandService.ExecTraining`（静态方法，2参数）
- `SingleModeScenarioOnsenTrainingCommandService.ExecTraining`
- `SingleModeScenarioPioneerTrainingCommandService.ExecTraining`

### 5. ObscuredIdleSingleModeEndInfo
- 46 个方法
- `RaceRandomProgramArray` (fnptr) — 预计算的比赛随机序列
- `RaceConditionArray` (fnptr)
- `UncheckedEventArray` (fnptr)
- `EffectedFactorArray` (fnptr)

### 6. 其他随机源
- `MasterCharacterSystemLottery.GetRandom` — 静态方法，可能共用
- `TrainingRandom` 搜索 → 0结果

## 地址汇总（本次会话，游戏重启会变）
| 对象 | 地址 |
|------|------|
| WorkDataManager | 0x72625f1960 |
| WorkSingleModeData | 0x7479c02000 |
| GameSystem | 0x7562101af0 |

## 下一步
1. 读取 `WorkSingleModeData` 内存，提取 `_fixedTurnCharaSeed`（offset 408）
2. 确认 `_fixedTurnCharaSeed` 是否为训练随机种子
3. 分析种子算法，实现本地预测
4. 确认 `ExecTraining` 的调用链是否使用此种子

## 文件清单
共 20 个 Transfer Dock 文件，包含本会话所有 IL2CPP 查询结果。