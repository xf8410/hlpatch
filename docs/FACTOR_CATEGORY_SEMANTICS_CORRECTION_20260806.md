# 因子类别与育成马娘本体固有技能语义纠正

本文件是 `UNIFIED_OBSERVATION_ENDPOINT_UPGRADE_PLAN_20260806.md` 因子模块的强制语义补充；实现时以本文件分类为准。

## 四类必须分开

```text
blue        = 五维属性因子
red_or_pink = 场地、距离、跑法适性因子
white       = 通用可继承因子（符合规则的技能、比赛、剧本等）
green       = 育成马娘本体固有技能的继承因子
```

育成马娘本体固有技能不是白色技能因子。它在继承系统中对应独立绿色因子；不得把角色本体固有技能加入白因子候选池，也不得设计“固有技能转白因子”的概率问题。

## 端点模型纠正

`/factor/finish_trace`、`/factor/candidates`、`/factor/roll_trace`、`/factor/probability_model`、`/factor/history`、`/factor/stats` 必须输出：

```text
factor_category = blue | red_or_pink | white | green
factor_type_raw
factor_id
star
source_kind
```

白色通用技能因子保存：

```text
source_skill_id
lower_skill_id
rarity
group_id
group_rate
white_factor_id
candidate/weight/result
```

绿色本体固有因子保存：

```text
trained_chara_id
trained_card_id
chara_id
unique_skill_id
unique_skill_level
green_factor_id
star
继承后实际技能ID与等级
```

分类必须优先读取游戏原始因子类型枚举、Master或运行时消费者；不能仅按factor_id数值范围或UI颜色猜测。

## 金技能研究边界

金技能任务只验证：

1. 是否形成金技能自身对应的白因子；
2. 是否转为对应下位通用技能的白因子；
3. 是否只改变白因子候选资格或权重；
4. 是否仅通过综合评价间接相关。

不得把金技能研究与绿色本体固有因子混在同一候选类别。

## 验收

1. 任一育成马娘本体固有技能只能出现在绿色因子来源链。
2. 白因子候选中若出现本体固有技能，测试必须失败并报告具体 `trained_card_id/chara_id/unique_skill_id/factor_id`。
3. 同名或派生技能必须依据游戏Master关系和运行时类型判定，不能凭文本名归类。
4. 概率统计按四类分别计算数量与1/2/3星分布，禁止合并后给出“总因子概率”。
