# 因子类别、遗传因子与本体固有技能语义纠正

本文件是 `UNIFIED_OBSERVATION_ENDPOINT_UPGRADE_PLAN_20260806.md` 因子模块的强制语义补充；实现时以本文件分类为准。

## 四类必须分开

```text
blue        = 五维属性因子
red_or_pink = 场地、距离、跑法适性因子
white       = 通用可继承因子（符合规则的技能、比赛、剧本、遗传因子等）
green       = 育成马娘本体固有技能的继承因子
```

育成马娘本体固有技能不是白色技能因子。它在继承系统中对应独立绿色因子；不得把角色本体固有技能加入白因子候选池，也不得设计“固有技能转白因子”的概率问题。

## 遗传因子

白色因子中另有“遗传因子”。用户给出的业务规则是：

```text
来源适性共10类：
短距离 / 英里 / 中距离 / 长距离
草地 / 泥地
逃 / 先 / 差 / 追

父母相关星数达到6星：遗传因子开始按概率出现
达到“12红”：遗传因子必出
```

SO不能只把它归入普通白技能因子，必须建立独立子类型：

```text
white_factor_kind = skill | race | scenario | hereditary | other

hereditary_factor:
  target_proper_type
  parent1_matching_red_factors[]
  parent2_matching_red_factors[]
  ancestor_matching_red_factors[]
  counted_star_total
  threshold_6_reached
  threshold_12_reached
  candidate_eligible
  guaranteed
  probability_or_weight
  result_factor_id
  result_star
```

“6星”和“12红”的精确计数范围必须从游戏运行时消费者确认：是否只计两名亲本、是否计祖辈、是否要求同一种适性、以及星数如何跨继承树合并。在闭合前，端点同时保留用户规则、原始红因子树和游戏返回，不能自行选择口径。

## 不可继承技能过滤

带不可继承符号的技能不会产生白因子。候选端点必须在抽选前记录资格过滤：

```text
skill_id
inheritance_prohibited_raw
inheritance_prohibited_source
white_factor_mapping
candidate_eligible=false
exclusion_reason=non_inheritable_skill
```

这类技能属于“资格阶段被排除”，不能记为“进入候选但未抽中”，否则会污染白技能因子概率。

需要定位并Hook：

1. 技能Master或运行时对象中的不可继承标志；
2. UI不可继承符号的实际消费者；
3. 白技能因子候选生产器的过滤分支；
4. skill_id到white factor_id的映射。

只有UI图标而没有候选过滤方法时，结论仍不能提升为运行时公式确认。

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
inheritance_prohibited_raw
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
4. 是否仅通过综合评价间接相关；
5. 若带不可继承符号，必须直接排除于白候选。

不得把金技能研究与绿色本体固有因子混在同一候选类别。

## 验收

1. 任一育成马娘本体固有技能只能出现在绿色因子来源链。
2. 白因子候选中若出现本体固有技能，测试失败并报告具体 `trained_card_id/chara_id/unique_skill_id/factor_id`。
3. 带不可继承符号的技能若进入白候选，测试失败并报告过滤标志、消费者和候选记录。
4. 遗传因子按10类适性逐项输出红因子树、计数、6星门槛、12红门槛、概率/必出状态和最终结果。
5. 同名或派生技能必须依据游戏Master关系和运行时类型判定，不能凭文本名归类。
6. 概率统计按四大类及白色子类型分别计算数量与1/2/3星分布，禁止合并后给出“总因子概率”。
