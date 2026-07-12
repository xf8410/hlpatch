# 训练执行完整调用链 — 层层剥皮

**分析日期**: 2026-07-12
**方法**: IL2CPP dump 地址映射 + SO 反汇编 + 调用图分析

---

## 完整数据流

```
玩家点击训练
    │
    ▼
1. SingleModeMainTrainingDecideConfirmScenarioRamen::CreateExecCommandAlertConfirmActionQueue
    │   (UI层：确认弹窗)
    ▼
2. SingleModeRamenAPI::SendExecCommand (0x73391f1ca4)
    │   (发送训练请求到服务器)
    │   14个剧本各有自己的SendExecCommand
    ▼
3. 服务器返回 SingleModeCheckEventResponse
    │   (包含训练结果：属性变化、伙伴出现、彩圈判定等)
    │   ★ 训练增益是服务器端计算的！★
    ▼
4. MsgPack 反序列化
    │   SingleModeChangeParameterInfoFormatter::Deserialize
    │   SingleModeCommandInfoFormatter::Deserialize
    │   (将二进制响应转为C#对象)
    ▼
5. WorkSingleModeData::ApplyExecCommand (0x7339dc9214)
    │   (将服务器返回的结果应用到本地状态)
    │   323个bl调用 — 纯分发函数，无计算
    │   0个浮点操作
    ▼
6. WorkSingleModeChangeParameterInfo (120个getter/setter)
    │   set_Speed / set_Stamina / set_Power / set_Guts / set_Wiz
    │   set_Motivation / set_SkillPoint / set_Hp / set_FanNum
    │   set_EvaluationChangeList / set_AddSkillList / ...
    │   (纯数据容器，不做任何计算)
    ▼
7. SingleModeTrainingCommandEntity (15个getter/setter)
    │   get_TrainingGainParameterList — 训练增益值列表
    │   get_TrainingGainBonusParameterList — 加成增益列表
    │   get_TrainingFailureRate — 失败率
    │   (也是纯数据容器)
    ▼
8. /summary 端点读取这些已存储的值并输出JSON
```

---

## 关键发现：训练增益是服务器端计算的

### 证据

1. **ApplyExecCommand 无浮点操作**: 323个bl调用，0个浮点指令 → 纯分发函数
2. **WorkSingleModeChangeParameterInfo 全是getter/setter**: 120个方法，全是属性访问 → 纯数据容器
3. **SingleModeTrainingCommandEntity 全是getter/setter**: TrainingGainParameterList有set方法 → 值从外部设置
4. **SendExecCommand发送到服务器**: 14个剧本各有自己的API::SendExecCommand

### 结论

```
训练增益计算流程：
客户端发送训练请求 → 服务器计算所有结果 → 返回MessagePack二进制
→ 客户端反序列化 → 存储到WorkSingleModeChangeParameterInfo → 显示

客户端不做训练增益计算！
```

这意味着：
- `/summary` 中的 `gains` = 服务器算好的值，直接存储
- 心情倍率 = 在服务器端应用，客户端收到的已经是最终值
- 彩圈判定 = 服务器决定，客户端收到的 TipsEventPartnerArray 已含结果
- 训练基础值 + 支援卡效果 + 心情倍率 + 友情加成 = 全部服务器端计算

### 为什么SO里找不到计算逻辑

因为计算逻辑在**Cygames的服务器**上，不在客户端SO里！

客户端SO只负责：
1. 发送请求（SendExecCommand）
2. 接收响应（MsgPack反序列化）
3. 存储结果（ApplyExecCommand → set方法）
4. 显示结果（UI层）
5. 读取存储的值（/summary端点）

---

## 14个剧本的API::SendExecCommand

| 剧本 | 类名 | 地址 |
|------|------|------|
| URA | SingleModeURAAPI | 0x7339293c6c |
| TeamRace | SingleModeTeamRaceAPI | 0x733928f8b0 |
| Live | SingleModeLiveAPI | 0x73390a19c4 |
| Free | SingleModeFreeAPI | 0x7339029e90 |
| Venus | SingleModeVenusAPI | 0x73392c9fd8 |
| Arc | SingleModeCookAPI | 0x7338fe23d8 |
| Sport | SingleModeSportAPI | 0x7339229180 |
| Cook | SingleModeCookAPI | 0x7338fe23d8 |
| Mecha | SingleModeMechaAPI | 0x73390fb920 |
| Legend | SingleModeLegendAPI | 0x7339085d44 |
| Pioneer | SingleModePioneerAPI | 0x73391c49cc |
| Onsen | SingleModeOnsenAPI | 0x733914c7ac |
| Breeders | SingleModeBreedersAPI | 0x7338f5f844 |
| Ramen | SingleModeRamenAPI | 0x73391f1ca4 |
| 通用 | SingleModeAPI | 0x73392cf48c |

---

## 客户端能做的和不能做的

### 能做（已实现）
- ✅ 读取服务器返回的最终值（gains, evaluation, kizuna等）
- ✅ 读取训练伙伴列表（TrainingPartnerArray）
- ✅ 读取启示事件伙伴（TipsEventPartnerArray）
- ✅ 读取支援卡信息（EquipSupportCardArray）
- ✅ 从MasterDB读取静态数据（command_id, bond_threshold, effect_table）
- ✅ 判断彩圈（基于已返回的数据：bond + training match + TipsEvent）

### 不能做（服务器端逻辑）
- ❌ 计算训练增益值
- ❌ 应用心情倍率
- ❌ 计算友情加成
- ❌ 决定支援卡出现在哪个训练
- ❌ 决定启示事件触发
- ❌ 计算失败率

### 透视原理
URA Plugin的"透视"能力来自：
1. 客户端发送训练请求 → 服务器返回 `SingleModeCheckEventResponse`
2. 响应中包含训练结果（包含所有属性变化）
3. URA Plugin截获这个响应，提前读取结果
4. 显示给用户看 → "透视"

**但是**：当失败率变服务器运算后，响应中可能不包含完整结果 → 透视失效

---

## 客户端唯一做计算的地方

客户端在训练**之前**做的是：
1. 从MasterDB读取训练基础值（single_mode_training_effect表）
2. 从支援卡effect_table读取加成值
3. 从unique_effect读取固有效果
4. **估算**可能的训练结果（用于AI推荐）

这个估算逻辑在 UmaAi 的 `calculateTrainingValueSingle` 里实现了，
但游戏客户端本身**不做估算**——它等服务器返回结果。

---

*分析: 2026-07-12, 基于完整调用链反汇编*
