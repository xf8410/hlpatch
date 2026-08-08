# 因子类别、遗传因子与不可继承技能语义

本文件是统一端点计划中因子模块的强制实现约束。

## 四类因子

```text
blue        = 五维属性因子
red_or_pink = 场地、距离、跑法适性因子
white       = 通用可继承因子（技能、比赛、剧本、遗传等）
green       = 育成马娘本体固有技能继承因子
```

本体固有技能只能进入绿色来源链，不能加入白因子候选，也不能设计“固有转白”。

## 遗传因子

白因子包含独立 `hereditary` 子类型，涉及10类适性：短、英、中、长、草、泥、逃、先、差、追。用户规则为相关父母星数达到6星开始概率出现，达到“12红”必出；是否计祖辈、是否要求同一种适性及精确合并方式必须由游戏消费者确认。

端点保留：

```text
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

## 不可继承技能

带不可继承符号的技能在候选资格阶段排除，不能统计为“入候选未抽中”：

```text
skill_id
inheritance_prohibited_raw
inheritance_prohibited_source
white_factor_mapping
candidate_eligible=false
exclusion_reason=non_inheritable_skill
```

必须定位Master或运行时标志、UI消费者、白因子候选过滤分支及skill_id到white factor_id映射。只有UI符号而没有候选过滤证据时，不提升为运行时公式确认。

## 统一输出

因子端点必须输出：

```text
factor_category = blue | red_or_pink | white | green
factor_type_raw
factor_id
star
source_kind
```

白技能因子保存source_skill_id、lower_skill_id、rarity、group_id、group_rate、不可继承标志、white_factor_id及candidate/weight/result。绿色因子保存trained_chara_id、trained_card_id、chara_id、unique_skill_id/level、green_factor_id、star及继承后技能。

分类优先读取游戏原始类型枚举、Master或运行时消费者，不能只按factor_id范围或UI颜色猜测。

## 验收失败条件

1. 本体固有技能出现在白候选。
2. 带不可继承符号的技能进入白候选。
3. 遗传因子未按10类适性输出红因子树、门槛、概率/必出状态和结果。
4. 四大类及白色子类型被合并成单一“总因子概率”。
