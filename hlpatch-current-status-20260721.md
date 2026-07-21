# hlpatch current status — 2026-07-21

## Remote commits

- uma-data `8b89409`: feat: 全541张卡事件链目录 + 14个目标Story详情 + 证据级别Schema
- uma-data: pending push — scenario_14_ramen_model/training_formula_evidence

## Scenario 14 Ramen training formula

### Evidence status

| Method | Evidence Level | Key Finding |
|--------|---------------|-------------|
| Apply(int) | direct_extract | sdiv truncating division; divisor {1:2, 2:1, other:4}; 9 effect types |
| GetWithCheckPointPt | partial_direct_extract | Mid-function; calls Add/Contains on 3 lists at 0xf0/0xf8/0x100 |
| GetTrainingMatchingObtain | direct_extract | 8-entry table; 0.1f constant; codegen=start |
| IsBonusEffectTraining | direct_extract | Checks training type 0xa7 (167); returns boolean |
| CreateRegionEffect | direct_extract | Creates instance; calls setup; stores to +0xb8 |

### Tool results

- Il2CppDumper v6.7.46: FAILED (binary protected)
- Cpp2IL 2022.0.7: FAILED (metadata v31 unsupported)
- capstone Python: SUCCESS

### Remaining UNKNOWN

- apply_full_arithmetic: function ~0x400 bytes, only ~0x200 disassembled
- getwithcheckpointpt_start: codegen mid-function, start not found
- rodata_double_constants: adrp targets unresolved
- region_effect_into_training: call chain 0x96b5190/0x96b5094 not resolved
- checkpointpt_to_region_effect: full GetWithCheckPointPt not visible
- rounding_positions: only Apply's sdiv confirmed

### Tests

- 35/35 pytest passing (test_training_formula_evidence.py)

## CI

- No CI triggered (uma-data has no Actions workflow)
- hlpatch CI: not triggered (changes in uma-data only)
