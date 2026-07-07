# RNG 种子搜索记录

## 目标
寻找赛马娘手游中训练结果的预计算机制，实现"训练前透视"——在点击训练之前就知道训练结果。

## 核心理论
用户认为赛马娘是"假的概率游戏"，训练结果在开局时由 RNG 种子预计算好，不会因为点击时机改变。

## 搜索时间线

### 2026-07-08 06:00-06:25

#### 1. 搜索 `Random` 类
端点: `/classes/search/keyword?keyword=Random`
结果: 64个匹配
- **关键发现**: `StandaloneSimulator.RaceRandom` (1个字段)
- `StandaloneSimulator.IRaceRandomGenerator` (接口)
- `Gallop.Live.RandomTable`, `Gallop.Live.RandomTable'1`
- 文件: `01_search_Random.txt`

#### 2. 搜索 `Lot` 类
端点: `/classes/search/keyword?keyword=Lot`
结果: 72个匹配
- `CharacterSystemLotteryTrigger` (230个字段!)
- `SingleMode10LotteryTurn`, `FactorLotteryType`
- 文件: `02_search_Lot.txt`

#### 3. 搜索 `Seed`
端点: `/classes/search/keyword?keyword=Seed`
结果: 仅2个匹配 (TimelineKeyWordData, 无关)

#### 4. RaceRandom 方法查询
端点: `/il2cpp/methods?name=RaceRandom`
结果: 4个 `GetRandom` 重载
- `GetRandom(2 params) -> i4`
- `GetRandom(1 param) -> i4`
- `GetRandom(2 params) -> r4`
- `GetRandom(1 param) -> r4`

#### 5. IRaceRandomGenerator 方法查询
端点: `/il2cpp/methods?name=IRaceRandomGenerator`
结果: 4个 `GetRandom` 方法，与 RaceRandom 一致

#### 6. 搜索 RaceRandom 引用
端点: `/il2cpp/search_methods?keyword=RaceRandom`
结果: 发现 `ObscuredIdleSingleModeEndInfo` 类
- `get_RaceRandomProgramArray` -> `fnptr`
- `set_RaceRandomProgramArray` -> `fnptr`

#### 7. ObscuredIdleSingleModeEndInfo 方法查询
端点: `/il2cpp/methods?name=ObscuredIdleSingleModeEndInfo`
结果: 46个方法
- 关键: `RaceRandomProgramArray` (fnptr), `RaceConditionArray` (fnptr)
- 文件: `03_ObscuredIdleSingleModeEndInfo_methods.txt`

#### 8. 搜索 RaceRandomProgram 引用
端点: `/il2cpp/search_methods?keyword=RaceRandomProgram`
结果: 仅 `ObscuredIdleSingleModeEndInfo` 使用

#### 9. 搜索 ExecTraining
端点: `/il2cpp/search_methods?keyword=ExecTraining`
结果: 48个方法匹配
- 文件: `04_search_ExecTraining.txt`

#### 10. 搜索 TrainingCommand
端点: `/il2cpp/search_methods?keyword=TrainingCommand`
结果: 152个方法匹配
- 文件: `05_search_TrainingCommand.txt`

## 关键发现总结

### RaceRandom - 随机数生成器
- NS: `StandaloneSimulator`
- 字段: 1个 (未成功dump)
- 方法: 4个 GetRandom 重载
- 接口: `IRaceRandomGenerator`

### ObscuredIdleSingleModeEndInfo - 预计算容器
- NS: `Gallop`
- 46个方法
- 包含 `RaceRandomProgramArray` (fnptr 类型)
- 可能是开局时预先生成的随机程序数组

### 待解决
- `/il2cpp/field` 端点参数格式不明，无法获取字段偏移
- `CharacterSystemLotteryTrigger` (230字段) dump 闪退
- `ObscuredIdleSingleModeEndInfo` classes dump 闪退
- RaceRandom 的 1 个字段尚未获取到

### 下一步
1. 找到正确的 `/il2cpp/field` 参数格式
2. 获取 RaceRandom 的字段偏移
3. 在 singleton 中找 RaceRandom 实例地址
4. 通过 read_mem 读取种子值
5. 分析 RaceRandomProgramArray 的结构