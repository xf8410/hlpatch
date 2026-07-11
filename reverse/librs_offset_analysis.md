# lib.rs 已知偏移量和常量分析

**源文件**: src/lib.rs (1856行)
---

## 一、剧本ID映射
```rust
let scn_s = match sid {
    1=>"URA", 2=>"TeamRace", 3=>"Live", 4=>"Free", 5=>"Venus",
    6=>"Arc", 7=>"Sport", 8=>"Cook", 9=>"Mecha", 10=>"Legend",
    11=>"Pioneer", 12=>"Onsen", 13=>"Breeders", 14=>"Ramen", _=>"Unknown"
};
```

## 二、Command ID映射
```rust
let cname = match cid {
    101=>"Speed", 102=>"Stamina", 103=>"Guts",
    105=>"Power", 106=>"Wiz", _=>"Unknown"
};
```

## 三、TargetType映射
```rust
let tn = match tt {
    1=>"Speed", 2=>"Stamina", 3=>"Guts",
    4=>"Power", 5=>"Wiz", 10=>"HP",
    20=>"Motivation", 30=>"SkillPt", _=>"Unknown"
};
```

## 四、内存偏移量

### 4.1 IL2CPP Array
```
+0x18  length (usize)
+0x20  elements (ptr[])
```

### 4.2 SupportCardEntry
```
+0x10  position               i32
+0x14  support_card_id        i32
+0x18  limit_break_count      i32
+0x20  training_partner_state i32
```

### 4.3 EvaluationInfo
```
+0x10  target_id    i32
+0x14  evaluation   i32
+0x20  is_appear    i32
```

### 4.4 TrainingLevelInfo
```
+0x10  command_id  i32
+0x14  level       i32
```

## 五、评价分数表

FIVE_STATUS_FINAL_SCORE: [2801] 个值
- 范围: 0 ~ 14280
- 前20: [0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10]
- 后20: [14098, 14116, 14116, 14134, 14134, 14152, 14152, 14170, 14170, 14189, 14189, 14207, 14207, 14225, 14225, 14243, 14243, 14261, 14261, 14280]

```
BASIC_FIVE_STATUS_LIMIT = [2300, 2200, 1800, 1400, 1400]
// [Speed, Stamina, Power, Guts, Wisdom]
```

## 六、各剧本总回合数
```rust
  1 => 78,  // URA
  _ => 72,
```

## 七、WorkSingleModeScenario类名映射
```rust
  1=>"WorkSingleModeScenarioURA", 2=>"WorkSingleModeScenarioTeamRace",
  3=>"WorkSingleModeScenarioLive", 4=>"WorkSingleModeScenarioFree",
  5=>"WorkSingleModeScenarioVenus", 6=>"WorkSingleModeScenarioArc",
  7=>"WorkSingleModeScenarioSport", 8=>"WorkSingleModeScenarioCook",
  9=>"WorkSingleModeScenarioMecha", 10=>"WorkSingleModeScenarioLegend",
  11=>"WorkSingleModeScenarioPioneer", 12=>"WorkSingleModeScenarioOnsen",
  13=>"WorkSingleModeScenarioBreeders", 14=>"WorkSingleModeScenarioRamen",
  1 => "WorkSingleModeScenarioURA",
  2 => "WorkSingleModeScenarioTeamRace",
  3 => "WorkSingleModeScenarioLive",
  4 => "WorkSingleModeScenarioFree",
  5 => "WorkSingleModeScenarioVenus",
  6 => "WorkSingleModeScenarioArc",
  7 => "WorkSingleModeScenarioSport",
  8 => "WorkSingleModeScenarioCook",
  9 => "WorkSingleModeScenarioMecha",
  10 => "WorkSingleModeScenarioLegend",
  11 => "WorkSingleModeScenarioPioneer",
  12 => "WorkSingleModeScenarioOnsen",
  13 => "WorkSingleModeScenarioBreeders",
  14 => "WorkSingleModeScenarioRamen",
```

## 八、Vital评价函数
```rust
fn vital_evaluation(vital: i32, max_vital: i32) -> f64 {
    let v = if vital > max_vital { max_vital } else { vital };
    if v <= 50 {
        2.0 * v as f64
    } else if v <= 70 {
        1.5 * (v - 50) as f64 + 100.0  // 2.0 * 50 = 100
    } else {
        1.0 * (v - 70) as f64 + 130.0   // 100 + 1.5*20 = 130
    }
}
```

## 九、Max Vital计算
```rust
fn calculate_max_vital_eq(turn: i32, max_vital: i32) -> i32 {
    if turn >= 76 { return 0; }
    if turn > 71 { return 10; }
    if turn == 71 { return 30; }
    // Assume max 6 non-race turns before URA
    let non_race_turns = std::cmp::min(6, 71 - turn);
    let eq = 30 + 15 * non_race_turns;
    if eq > max_vital { max_vital } else { eq }
}
```