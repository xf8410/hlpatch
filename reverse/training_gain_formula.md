# 训练增益完整公式

**来源**: UmaAi Game.cpp `calculateTrainingValueSingle()` + 用户确认（5等级实测逆推）
**确认日期**: 2026-07-12

---

## 核心公式

```
trainValue = basicValue × cardMultiplier × umaBonus × scenarioMultiplier
```

### cardMultiplier（支援卡倍率）

```
cardMultiplier = (1 + 0.05 × heads)
              × (1 + 0.01 × totalXunLian)
              × (1 + 0.1 × (motivation - 3) × (1 + 0.01 × totalGanjing))
              × totalYouqingMultiplier
```

| 因子 | 含义 | 来源 |
|------|------|------|
| heads | 出现人数 | TrainingPartnerArray |
| totalXunLian | 训练加成总和 | 各卡 type 8 (TrainingEffectUp) 之和 |
| motivation | 心情 1-5 | WorkSingleModeCharaData |
| totalGanjing | 干劲加成总和 | 各卡 type 2 (MotivationUp) 之和 |
| totalYouqingMultiplier | 友情倍率(累乘) | 彩圈卡 type 1 (SpecialTagEffectUp)，每张 `(1+0.01×youQing)` |

### 心情倍率（公式计算，不查表）

```
motivationFactor = 1 + 0.1 × (motivation - 3) × (1 + 0.01 × totalGanjing)
```

**base 值（ganJing=0）:**

| motivation | 心情 | base | 社区公认值 | 差异说明 |
|:---------:|------|:----:|:---------:|---------|
| 1 | 絶不調 | 0.8 | 0.6 | 社区值可能含干劲放大 |
| 2 | 不調 | 0.9 | 0.8 | 同上 |
| 3 | 普通 | 1.0 | 1.0 | ✅ 一致 |
| 4 | 好調 | 1.1 | 1.1 | ✅ 一致 |
| 5 | 絶好調 | 1.2 | 1.2 | ✅ 一致 |

**干劲放大效果:**

ganJing 越高，心情的加减幅度越大：

| ganJing | 绝不调 | 不调 | 普通 | 好调 | 绝好调 |
|:-------:|:-----:|:----:|:----:|:----:|:------:|
| 0 | 0.800 | 0.900 | 1.000 | 1.100 | 1.200 |
| 20 | 0.760 | 0.880 | 1.000 | 1.120 | 1.240 |
| 40 | 0.720 | 0.860 | 1.000 | 1.140 | 1.280 |
| 60 | 0.680 | 0.840 | 1.000 | 1.160 | 1.320 |

**社区值 0.6 对应 ganJing≈50 时的绝不调倍率。**

### basicValue（训练基础值）

来源: `single_mode_training_effect` 表

```
basicValue[stat] = single_mode_training_effect.effect_value
WHERE command_id = 当前训练 AND sub_id = 训练等级 AND result_state = 2 (成功)
```

### umaBonus（马娘加成）

```
umaBonus[stat] = 1 + 0.01 × fiveStatusBonus[stat]
```

fiveStatusBonus 来自角色固有的属性加成（如某些角色 Speed+10% 等）。

### scenarioMultiplier（剧本加成）

各剧本不同，例如机甲剧本：
- 研究等级加成: `(1 + 0.01 × (6 + 0.06 × rivalLv))`
- 齿轮加成: 3%~30% (随回合增长)
- 友情时额外: `(1 + 0.01 × 2 × upgrade[1][2])`
- Overdrive: ×1.25

---

## 逛街卡效果

逛街卡（非彩圈）贡献：
- ✅ type 2 (干劲): `totalGanjing += value` → 放大心情倍率
- ✅ type 3-7 (副属性): `basicValue[stat] += bonus[stat]`
- ✅ type 8 (训练加成): `totalXunlian += value`
- ✅ type 27 (失败率下降): `failRateMultiplier *= (1-0.01×value)`
- ✅ type 28 (体力消耗下降): `vitalCostMultiplier *= (1-0.01×value)`
- ❌ type 1 (友情加成): **不触发**，只有彩圈卡才乘

## 彩圈额外效果

彩圈卡在上述基础上额外：
- ✅ type 1 (友情加成): `totalYouqingMultiplier *= (1+0.01×youQing)`
- ✅ 智力彩圈: `vitalCostBasic -= vitalBonus`
- ✅ 剧本特殊加成（如机甲齿轮）

---

## 完整计算流程

```
1. 读 basicValue = single_mode_training_effect 表
2. 遍历出现的每张卡:
   a. getCardEffect(card, isShining, training, bond, ...)
   b. bonus[stat] += eff.bonus[stat]        // 副属性(逛街也加)
   c. if (isShining):
      - totalYouqingMultiplier *= (1+0.01*eff.youQing)  // 友情(彩圈才乘)
      - if (Wiz训练): vitalCostBasic -= eff.vitalBonus
   d. totalXunlian += eff.xunLian           // 训练加成
   e. totalGanjing += eff.ganJing           // 干劲加成
   f. vitalCostMultiplier *= (1-0.01*eff.vitalCostDrop)
   g. failRateMultiplier *= (1-0.01*eff.failRateDrop)

3. cardMultiplier = (1+0.05*heads) * (1+0.01*xunLian) 
                   * (1+0.1*(motiv-3)*(1+0.01*ganJing)) 
                   * youQing

4. trainValue = basicValue * cardMultiplier * umaBonus * scenarioMultiplier
```

---

## 与游戏 gains 的关系

`/summary` 中的 `gains` 是游戏引擎算好的最终值，已包含上述所有步骤。
因此 hlpatch 不需要自己计算 trainValue——直接读 gains 即可。

**AI 评估时** 不应再额外加 `shining_bonus`，因为 gains 已含彩圈加成。

*确认: 2026-07-12, UmaAi 源码 + 用户实测验证*
