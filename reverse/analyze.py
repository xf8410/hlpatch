#!/usr/bin/env python3
"""赛马娘 v2.28.5 全量逆向分析"""
import json, re, os, struct
from collections import defaultdict, OrderedDict

BASE = "/home/z/my-project/repos/hlpatch"
OUT = f"{BASE}/reverse"
SO_PATH = "/tmp/apk_extract/lib_arm64-v8a_libil2cpp.so"

print("Loading IL2CPP dump...")
with open(f"{BASE}/data/il2cpp_dump/dump_all_methods_ALL.json") as f:
    dump = json.load(f)
ALL_CLASSES = dump['classes']
CLASS_INDEX = {f"{c['ns']}.{c['class']}": c for c in ALL_CLASSES}
print(f"  {len(ALL_CLASSES)} classes, {dump['total_methods']} methods")

print("Loading lib.rs...")
with open(f"{BASE}/src/lib.rs") as f:
    LIBRS = f.read()

def mname(m):
    return m['name'] if isinstance(m, dict) else str(m)

def find_classes(pattern):
    return [c for c in ALL_CLASSES if re.search(pattern, f"{c['ns']}.{c['class']}", re.IGNORECASE)]

def methods_of(cls):
    return [mname(m) for m in cls.get('methods', [])]

def getters_of(cls):
    return [mname(m) for m in cls.get('methods', []) if 'get_' in mname(m).lower()]

SCENARIO_MAP = OrderedDict([
    (1, "URA"), (2, "TeamRace"), (3, "Live"), (4, "Free"), (5, "Venus"),
    (6, "Arc"), (7, "Sport"), (8, "Cook"), (9, "Mecha"), (10, "Legend"),
    (11, "Pioneer"), (12, "Onsen"), (13, "Breeders"), (14, "Ramen"),
])
SCENARIO_FULL = {
    1:"URA (育成シナリオ)", 2:"TeamRace (チームレース)", 3:"Live (ライブ)",
    4:"Free (フリー)", 5:"Venus (ヴィーナス)", 6:"Arc (アーク)",
    7:"Sport (スポーツ)", 8:"Cook (クック)", 9:"Mecha (メカ)",
    10:"Legend (レジェンド)", 11:"Pioneer (パイオニア/青春杯)",
    12:"Onsen (温泉)", 13:"Breeders (ブリーダーズ/種田杯)",
    14:"Ramen (ラーメン/トゥインクル・ラーメン杯)",
}
CMD_MAP = {101:"Speed", 102:"Stamina", 103:"Guts", 105:"Power", 106:"Wisdom"}
TT_MAP = {1:"Speed",2:"Stamina",3:"Guts",4:"Power",5:"Wiz",10:"HP",20:"Motivation",30:"SkillPt"}
MOT_MAP = {5:"Best",4:"Good",3:"Normal",2:"Bad",1:"Worst"}

    
# ============================================================
# Report 1: Master Analysis
# ============================================================
def gen_master():
    r = []
    r.append("# 赛马娘 v2.28.5 全量逆向工程分析报告")
    r.append(f"\n**游戏版本**: v2.28.5 (日服)  \n**libil2cpp.so**: 209MB (ARM64)  \n**IL2CPP版本**: v31")
    r.append(f"**数据源**: IL2CPP方法转储 ({len(ALL_CLASSES)}类, {dump['total_methods']}方法) + hlpatch插件源码 ({len(LIBRS.splitlines())}行)")
    r.append(f"**生成时间**: 2026-07-11\n\n---\n")

    # === Part 1: Scenario System ===
    r.append("## 一、剧本系统总览\n")
    r.append(f"共 **14个剧本ID**：\n")
    r.append("| ID | 剧本名 | 专用类数 | Obscured类数 |")
    r.append("|---|---|---|---|")
    for sid, sname in SCENARIO_MAP.items():
        sc = [c for c in ALL_CLASSES if f"SingleMode{sid}" in c['class'] and c['ns'] == 'Gallop']
        obs = [c for c in ALL_CLASSES if 'Obscured' in c['class'] and sname in c['class'] and c['ns'] == 'Gallop']
        r.append(f"| {sid} | {SCENARIO_FULL[sid]} | {len(sc)} | {len(obs)} |")

    # === Part 2: Core Data Path ===
    r.append("\n## 二、核心数据访问路径\n")
    r.append("```")
    r.append("WorkSingleModeData (育成根节点)")
    r.append("  └→ WorkSingleModeCharaData (角色育成数据)")
    r.append("       ├→ support_card_array → SupportCardEntry[] (支援卡槽位)")
    r.append("       ├→ evaluation_info_array → EvaluationInfo[] (羁绊信息)")
    r.append("       ├→ training_level_info_array → TrainingLevelInfo[] (训练等级)")
    r.append("       └→ get_HomeInfoData() → WorkSingleModeHomeInfoData")
    r.append("            └→ CommandInfoArray → SingleModeCommandInfoData[] (训练命令)")
    r.append("                 ├→ get_CommandId() → ObscuredInt (101-106)")
    r.append("                 ├→ get_IsEnable() → ObscuredInt (0/1)")
    r.append("                 ├→ get_FailureRate() → ObscuredInt (%)")
    r.append("                 ├→ get_TrainingPartnerArray() → 伙伴列表")
    r.append("                 ├→ get_TipsEventPartnerArray() → 彩圈伙伴列表")
    r.append("                 └→ get_ParamsIncDecInfoArray() → 属性增减列表")
    r.append("                      ├→ get_TargetType() → ObscuredInt (1-30)")
    r.append("                      └→ get_Value() → ObscuredInt (增减值)")
    r.append("```")

    # WorkSingleModeCharaData getters
    wsmcd = CLASS_INDEX.get('Gallop.WorkSingleModeCharaData', {})
    if wsmcd:
        gtrs = getters_of(wsmcd)
        r.append(f"\n### WorkSingleModeCharaData — {wsmcd.get('method_count',0)} methods, {len(gtrs)} getters\n")
        for m in gtrs[:80]:
            r.append(f"  - `{m}`")

    # === Part 3: Command/Target/Motivation mappings ===
    r.append("\n## 三、ID映射系统\n")
    r.append("### 3.1 CommandId → 训练类型")
    r.append("| CommandId | 训练 |")
    r.append("|---|---|")
    for k in sorted(CMD_MAP): r.append(f"| {k} | {CMD_MAP[k]} |")
    r.append("\n> ⚠️ CommandId 非对称：Guts=103, Power=105（跳过104）")
    
    r.append("\n### 3.2 TargetType → 属性类型")
    r.append("| TargetType | 属性 |")
    r.append("|---|---|")
    for k in sorted(TT_MAP): r.append(f"| {k} | {TT_MAP[k]} |")
    r.append("\n> ⚠️ TargetType 与 CommandId 完全独立：Guts=3, Power=4")
    
    r.append("\n### 3.3 Motivation (心情)")
    r.append("| 等级 | 心情 |")
    r.append("|---|---|")
    for k in sorted(MOT_MAP, reverse=True): r.append(f"| {k} | {MOT_MAP[k]} |")

    # === Part 4: Support Card System ===
    r.append("\n## 四、支援卡系统\n")
    r.append("### 4.1 内存布局 (SupportCardEntry)")
    r.append("```")
    r.append("+0x10  position               i32   槽位 (1-8)")
    r.append("+0x14  support_card_id        i32   支援卡ID")
    r.append("+0x18  limit_break_count      i32   凸数 (0-4)")
    r.append("+0x20  training_partner_state i32   训练伙伴状态")
    r.append("``")
    r.append("- IL2CPP Array: length@+0x18, elements@+0x20 (8-byte ptrs)")
    r.append("- 访问: `WorkSingleModeCharaData.get_SupportCardArray()`")
    
    sc_classes = find_classes(r'SupportCard')
    r.append(f"\n### 4.2 支援卡相关类 ({len(sc_classes)}个)\n")
    for c in sc_classes[:40]:
        r.append(f"- `{c['ns']}.{c['class']}` ({c.get('method_count',0)}m)")

    # SupportCardType
    sct = find_classes(r'SupportCardType|CardType')
    r.append(f"\n### 4.3 支援卡类型枚举 ({len(sct)}个)")
    for c in sct[:10]:
        r.append(f"- `{c['ns']}.{c['class']}` (enum={c.get('is_enum',False)})")

    # === Part 5: Bond/Evaluation ===
    r.append("\n## 五、羁绊系统\n")
    r.append("### 5.1 内存布局 (EvaluationInfo)")
    r.append("```")
    r.append("+0x10  target_id    i32  目标ID (支援卡ID / 特殊NPC ID)")
    r.append("+0x14  evaluation   i32  羁绊值 (0-100)")
    r.append("+0x20  is_appear    i32  是否出现 (0/1)")
    r.append("``")
    r.append("- 访问: `WorkSingleModeCharaData.get_EvaluationInfoArray()`")
    r.append("- target_id 对应 support_card_id；理事长/记者等特殊NPC使用固定ID")
    
    ev_classes = find_classes(r'Evaluation')
    r.append(f"\n### 5.2 羁绊相关类 ({len(ev_classes)}个)\n")
    for c in ev_classes[:30]:
        r.append(f"- `{c['ns']}.{c['class']}` ({c.get('method_count',0)}m)")
        for m in methods_of(c)[:8]:
            r.append(f"  - `{m}`")

    # === Part 6: Training System ===
    r.append("\n## 六、训练系统\n")
    cmd_cls = CLASS_INDEX.get('Gallop.SingleModeCommandInfoData', {})
    if cmd_cls:
        gtrs = getters_of(cmd_cls)
        r.append(f"### 6.1 SingleModeCommandInfoData ({cmd_cls.get('method_count',0)}m, {len(gtrs)} getters)\n")
        for m in gtrs:
            r.append(f"  - `{m}`")

    r.append("\n### 6.2 训练数据结构")
    r.append("```")
    r.append("SingleModeCommandInfoData:")
    r.append("  get_CommandId()             → ObscuredInt (101-106)")
    r.append("  get_IsEnable()              → ObscuredInt (0/1)")
    r.append("  get_FailureRate()           → ObscuredInt (失败率%)")
    r.append("  get_TrainingPartnerArray()  → Array (训练伙伴)")
    r.append("  get_TipsEventPartnerArray() → Array (彩圈伙伴)")
    r.append("  get_ParamsIncDecInfoArray() → Array<SingleModeParamsIncDecInfoData>")
    r.append("")
    r.append("SingleModeParamsIncDecInfoData:")
    r.append("  get_TargetType() → ObscuredInt")
    r.append("  get_Value()      → ObscuredInt")
    r.append("```")

    r.append("\n### 6.3 训练等级 (TrainingLevelInfo)")
    r.append("```")
    r.append("+0x10  command_id  i32  训练类型 (101-106)")
    r.append("+0x14  level       i32  等级 (1-5)")
    r.append("``")
    r.append("- 访问: `WorkSingleModeCharaData.get_TrainingLevelInfoArray()`")

    tr_classes = find_classes(r'ExecTraining|TrainingResult|TrainingCommand')
    r.append(f"\n### 6.4 训练执行相关类 ({len(tr_classes)}个)\n")
    for c in tr_classes[:30]:
        r.append(f"- `{c['ns']}.{c['class']}` ({c.get('method_count',0)}m)")
        for m in methods_of(c)[:5]:
            r.append(f"  - `{m}`")

    # === Part 7: Shining ===
    r.append("\n## 七、彩圈(Shining)系统\n")
    r.append("```")
    r.append("彩圈 = TipsEventPartnerArray.Length > 0")
    r.append("")
    r.append("判定链:")
    r.append("  1. 支援卡出现在训练 → TrainingPartnerArray 包含该卡")
    r.append("  2. 支援卡闪彩 → TipsEventPartnerArray 包含该卡")
    r.append("  3. 闪彩条件 (推断):")
    r.append("     a. bond ≥ 80")
    r.append("     b. support_card_type 匹配 CommandId")
    r.append("     c. specialty 匹配训练类型")
    r.append("```")
    tips = find_classes(r'TipsEvent|Shining')
    r.append(f"\n### 7.1 彩圈相关类 ({len(tips)}个)")
    for c in tips[:20]:
        r.append(f"- `{c['ns']}.{c['class']}` ({c.get('method_count',0)}m)")

    # === Part 8: Character ===
    r.append("\n## 八、角色系统\n")
    chara = find_classes(r'CharacterData|CharaData|CardData')
    r.append(f"### 8.1 角色数据类 ({len(chara)}个)\n")
    for c in chara[:30]:
        r.append(f"- `{c['ns']}.{c['class']}` ({c.get('method_count',0)}m)")
        for m in methods_of(c)[:5]:
            r.append(f"  - `{m}`")

    # === Part 9: Skill ===
    r.append("\n## 九、技能系统\n")
    skills = find_classes(r'Gallop\.Skill')
    r.append(f"### 9.1 技能类 ({len(skills)}个)\n")
    for c in skills[:30]:
        r.append(f"- `{c['ns']}.{c['class']}` ({c.get('method_count',0)}m)")

    # === Part 10: Race ===
    r.append("\n## 十、比赛系统\n")
    races = find_classes(r'Gallop\.Race|RaceData|RaceResult|TargetRace')
    r.append(f"### 10.1 比赛类 ({len(races)}个)\n")
    for c in races[:30]:
        r.append(f"- `{c['ns']}.{c['class']}` ({c.get('method_count',0)}m)")

    # === Part 11: Events ===
    r.append("\n## 十一、事件系统\n")
    events = find_classes(r'SingleModeEvent|MiniEvent|RandomEvent|OutingEvent|ContinuousEvent')
    r.append(f"### 11.1 事件类 ({len(events)}个)\n")
    for c in events[:30]:
        r.append(f"- `{c['ns']}.{c['class']}` ({c.get('method_count',0)}m)")

    # === Part 12: ObscuredInt ===
    r.append("\n## 十二、ObscuredInt加密\n")
    r.append("```")
    r.append("ObscuredInt (20 bytes inline):")
    r.append("  +0x00: key        (i32) XOR密钥")
    r.append("  +0x04: hidden     (i32) 加密值 (actual = key ^ hidden)")
    r.append("  +0x08: inited     (i32) 初始化标志")
    r.append("  +0x0c: fake       (i32) 伪装值")
    r.append("  +0x10: fakeActive (i32) 伪装激活")
    r.append("")
    r.append("读取: call_getter_obscured_int(class, obj, \"get_XXX\") 或 value = key ^ hidden")
    r.append("```")
    obs = find_classes(r'Obscured')
    obs_types = sorted(set(c['class'] for c in obs))
    r.append(f"\n### 12.1 Obscured类型 ({len(obs_types)}种)\n")
    for t in obs_types[:60]:
        r.append(f"- `{t}`")

    # === Part 13: ChangeParameterInfo ===
    r.append("\n## 十三、属性变化系统\n")
    cpi = CLASS_INDEX.get('Gallop.WorkSingleModeChangeParameterInfo', {})
    if cpi:
        gtrs = getters_of(cpi)
        r.append(f"### 13.1 WorkSingleModeChangeParameterInfo ({cpi.get('method_count',0)}m, {len(gtrs)} getters)\n")
        for m in gtrs[:60]:
            r.append(f"  - `{m}`")

    r.append("\n### 13.2 各剧本独立属性变化类")
    for sid, sname in SCENARIO_MAP.items():
        cpi_cls = CLASS_INDEX.get(f'Gallop.WorkSingleModeChangeParameterInfoScenario{sname}', {})
        if cpi_cls:
            gtrs = getters_of(cpi_cls)
            r.append(f"\n**Scenario {sid} - {sname}** ({cpi_cls.get('method_count',0)}m, {len(gtrs)} getters):")
            for m in gtrs[:20]:
                r.append(f"  - `{m}`")

    # === Part 14: AI Evaluation ===
    r.append("\n## 十四、AI评价系统\n")
    r.append("### 14.1 评价分数表")
    score_match = re.search(r'const FIVE_STATUS_FINAL_SCORE:\s*\[i32;\s*\d+\]\s*=\s*\[([^\]]+)\]', LIBRS)
    if score_match:
        scores = [int(s.strip()) for s in score_match.group(1).split(',')]
        r.append(f"- 总共 {len(scores)} 个值 (索引=总修正属性值, 值=评价分数)")
        r.append(f"- 范围: {min(scores)} ~ {max(scores)}")
    
    r.append("\n### 14.2 基础五维上限")
    r.append("```")
    r.append("BASIC_FIVE_STATUS_LIMIT = [2300, 2200, 1800, 1400, 1400]")
    r.append("// [Speed, Stamina, Power, Guts, Wisdom]")
    r.append("```")

    r.append("\n### 14.3 各剧本总回合数")
    turn_match = re.search(r'let total_turn.*?match scenario_id \{(.*?)\}', LIBRS, re.DOTALL)
    if turn_match:
        r.append("```rust")
        for line in turn_match.group(1).strip().split('\n'):
            if line.strip():
                r.append(f"  {line.strip()}")
        r.append("```")

    # === Part 15: Special NPCs ===
    r.append("\n## 十五、特殊NPC (理事长/记者)\n")
    npc = find_classes(r'President|Reporter|Director|Chairman|SpecialNPC|StoryChara')
    r.append(f"相关类 ({len(npc)}个):\n")
    for c in npc[:20]:
        r.append(f"- `{c['ns']}.{c['class']}` ({c.get('method_count',0)}m)")

    return '\n'.join(r)

master = gen_master()
with open(f"{OUT}/master_analysis.md", 'w') as f:
    f.write(master)
print(f"  ✓ master_analysis.md ({len(master)} bytes)")

# ============================================================
# Report 2: Per-scenario reports
# ============================================================
def gen_scenario(sid, sname):
    r = []
    r.append(f"# 剧本 {sid}: {SCENARIO_FULL[sid]}\n")
    r.append(f"**WorkScenario类**: `WorkSingleModeScenario{sname}`")
    r.append(f"**ObscuredDataSet**: `ObscuredSingleMode{sname}DataSet`\n---\n")

    # All classes for this scenario
    sc = [c for c in ALL_CLASSES if (f"SingleMode{sid}" in c['class'] or sname in c['class']) and c['ns'] == 'Gallop']
    r.append(f"## 相关类 ({len(sc)}个)\n")
    for c in sc[:60]:
        full = f"{c['ns']}.{c['class']}"
        r.append(f"### `{full}` ({c.get('method_count',0)}m)")
        for m in methods_of(c)[:20]:
            r.append(f"  - `{m}`")
        r.append("")

    # Obscured classes
    obs = [c for c in ALL_CLASSES if 'Obscured' in c['class'] and sname in c['class'] and c['ns'] == 'Gallop']
    if obs:
        r.append(f"\n## Obscured加密数据类 ({len(obs)}个)\n")
        for c in obs:
            r.append(f"### `{c['ns']}.{c['class']}` ({c.get('method_count',0)}m)")
            for m in methods_of(c)[:15]:
                r.append(f"  - `{m}`")
            r.append("")

    # Master classes
    mc = [c for c in ALL_CLASSES if c['class'].startswith('Master') and (f'SingleMode{sid}' in c['class'] or sname in c['class']) and c['ns'] == 'Gallop']
    if mc:
        r.append(f"\n## Master数据库表 ({len(mc)}个)\n")
        r.append("| 表名 | 方法数 |")
        r.append("|---|---|")
        for c in mc:
            r.append(f"| `{c['class']}` | {c.get('method_count',0)} |")

    # WorkScenario class
    wcls = CLASS_INDEX.get(f'Gallop.WorkSingleModeScenario{sname}', {})
    if wcls:
        r.append(f"\n## WorkSingleModeScenario{sname}\n")
        r.append(f"方法数: {wcls.get('method_count',0)}\n")
        for m in methods_of(wcls)[:40]:
            r.append(f"  - `{m}`")

    # DataSet class
    dcls = CLASS_INDEX.get(f'Gallop.WorkSingleModeScenario{sname}DataSet', {})
    if dcls:
        gtrs = getters_of(dcls)
        r.append(f"\n## WorkSingleModeScenario{sname}DataSet\n")
        r.append(f"方法数: {dcls.get('method_count',0)}, {len(gtrs)} getters\n")
        for m in gtrs[:60]:
            r.append(f"  - `{m}`")

    # ChangeParameterInfo
    cpi = CLASS_INDEX.get(f'Gallop.WorkSingleModeChangeParameterInfoScenario{sname}', {})
    if cpi:
        gtrs = getters_of(cpi)
        r.append(f"\n## 剧本独立属性变化 ({cpi.get('method_count',0)}m, {len(gtrs)} getters)\n")
        for m in gtrs:
            r.append(f"  - `{m}`")

    # lib.rs references
    r.append(f"\n## lib.rs相关引用\n")
    r.append("```")
    for line in LIBRS.split('\n'):
        if sname.lower() in line.lower() or f'Scenario{sid}' in line or f'{sid}=>"' in line or f'sid == {sid}' in line:
            r.append(line.strip())
    r.append("```")

    return '\n'.join(r)

for sid, sname in SCENARIO_MAP.items():
    rep = gen_scenario(sid, sname)
    fname = f"scenario_{sid:02d}_{sname}.md"
    with open(f"{OUT}/{fname}", 'w') as f:
        f.write(rep)
    print(f"  ✓ {fname} ({len(rep)} bytes)")

# ============================================================
# Report 3: lib.rs offset analysis
# ============================================================
def gen_offsets():
    r = []
    r.append("# lib.rs 已知偏移量和常量分析\n")
    r.append(f"**源文件**: src/lib.rs ({len(LIBRS.splitlines())}行)\n---\n")

    r.append("## 一、剧本ID映射\n```rust\nlet scn_s = match sid {")
    r.append("    1=>\"URA\", 2=>\"TeamRace\", 3=>\"Live\", 4=>\"Free\", 5=>\"Venus\",")
    r.append("    6=>\"Arc\", 7=>\"Sport\", 8=>\"Cook\", 9=>\"Mecha\", 10=>\"Legend\",")
    r.append("    11=>\"Pioneer\", 12=>\"Onsen\", 13=>\"Breeders\", 14=>\"Ramen\", _=>\"Unknown\"")
    r.append("};\n```\n")

    r.append("## 二、Command ID映射\n```rust\nlet cname = match cid {")
    r.append("    101=>\"Speed\", 102=>\"Stamina\", 103=>\"Guts\",")
    r.append("    105=>\"Power\", 106=>\"Wiz\", _=>\"Unknown\"")
    r.append("};\n```\n")

    r.append("## 三、TargetType映射\n```rust\nlet tn = match tt {")
    r.append("    1=>\"Speed\", 2=>\"Stamina\", 3=>\"Guts\",")
    r.append("    4=>\"Power\", 5=>\"Wiz\", 10=>\"HP\",")
    r.append("    20=>\"Motivation\", 30=>\"SkillPt\", _=>\"Unknown\"")
    r.append("};\n```\n")

    r.append("## 四、内存偏移量\n")
    r.append("### 4.1 IL2CPP Array\n```\n+0x18  length (usize)\n+0x20  elements (ptr[])\n```")
    r.append("\n### 4.2 SupportCardEntry\n```\n+0x10  position               i32\n+0x14  support_card_id        i32\n+0x18  limit_break_count      i32\n+0x20  training_partner_state i32\n```")
    r.append("\n### 4.3 EvaluationInfo\n```\n+0x10  target_id    i32\n+0x14  evaluation   i32\n+0x20  is_appear    i32\n```")
    r.append("\n### 4.4 TrainingLevelInfo\n```\n+0x10  command_id  i32\n+0x14  level       i32\n```")

    r.append("\n## 五、评价分数表\n")
    score_match = re.search(r'const FIVE_STATUS_FINAL_SCORE:\s*\[i32;\s*(\d+)\]\s*=\s*\[([^\]]+)\]', LIBRS)
    if score_match:
        n = int(score_match.group(1))
        scores = [int(s.strip()) for s in score_match.group(2).split(',')]
        r.append(f"FIVE_STATUS_FINAL_SCORE: [{n}] 个值")
        r.append(f"- 范围: {min(scores)} ~ {max(scores)}")
        r.append(f"- 前20: {scores[:20]}")
        r.append(f"- 后20: {scores[-20:]}")
    
    r.append("\n```\nBASIC_FIVE_STATUS_LIMIT = [2300, 2200, 1800, 1400, 1400]\n// [Speed, Stamina, Power, Guts, Wisdom]\n```")

    r.append("\n## 六、各剧本总回合数\n```rust")
    turn_match = re.search(r'let total_turn.*?match scenario_id \{(.*?)\}', LIBRS, re.DOTALL)
    if turn_match:
        for line in turn_match.group(1).strip().split('\n'):
            if line.strip():
                r.append(f"  {line.strip()}")
    r.append("```")

    r.append("\n## 七、WorkSingleModeScenario类名映射\n```rust")
    for line in LIBRS.split('\n'):
        if 'WorkSingleModeScenario' in line and '=>' in line:
            r.append(f"  {line.strip()}")
    r.append("```")

    # Vital evaluation
    r.append("\n## 八、Vital评价函数\n```rust")
    vital_match = re.search(r'fn vital_evaluation.*?\n\}', LIBRS, re.DOTALL)
    if vital_match:
        r.append(vital_match.group(0))
    r.append("```")

    # Max vital
    r.append("\n## 九、Max Vital计算\n```rust")
    maxvital_match = re.search(r'fn calculate_max_vital.*?\n\}', LIBRS, re.DOTALL)
    if maxvital_match:
        r.append(maxvital_match.group(0))
    r.append("```")

    return '\n'.join(r)

offsets = gen_offsets()
with open(f"{OUT}/librs_offset_analysis.md", 'w') as f:
    f.write(offsets)
print(f"  ✓ librs_offset_analysis.md ({len(offsets)} bytes)")

# ============================================================
# Report 4: SO string search
# ============================================================
print("\nSearching libil2cpp.so for strings...")
def search_so(patterns, max_per=30):
    results = {}
    with open(SO_PATH, 'rb') as f:
        data = f.read()
    for pat in patterns:
        pb = pat.encode('utf-8')
        matches = []
        idx = 0
        while len(matches) < max_per:
            idx = data.find(pb, idx)
            if idx == -1: break
            s = idx
            while s > 0 and data[s-1] != 0: s -= 1
            e = idx + len(pb)
            while e < len(data) and data[e] != 0: e += 1
            try:
                string = data[s:e].decode('utf-8', errors='replace')
                if len(string) < 500 and string not in [m[1] for m in matches]:
                    matches.append((idx, string))
            except: pass
            idx = e
        results[pat] = matches
    return results

patterns = [
    "SingleModeScenario", "RamenFeeling", "RamenCheckPoint", "RamenSozai",
    "RamenRecipe", "RamenNoodle", "RamenTopping", "RamenShop", "RamenGuest",
    "RamenUraf", "RamenTasting", "RamenCommand", "RamenSelect",
    "BreedersEnhance", "PioneerDream", "CookRecipe", "OnsenFacility",
    "MechaBoard", "LiveAudience", "LegendCheckPoint", "ArcBoost",
    "VenusHeart", "SportTraining", "TeamRace", "URAFinal",
    "TrainingCommand", "ExecTraining", "SupportCardEffect",
    "EvaluationInfo", "TipsEventPartner", "ShiningTraining",
    "PartnerEffect", "BondEffect", "chara_data", "card_data",
    "support_card_data", "scenario_id", "CommandId", "TargetType",
    "TrainingLevel", "ObscuredInt", "ObscuredSingleMode",
    "TrainingPartner", "TrainingPartnerState", "SupportCardType",
    "RamenDataSet", "BreedersDataSet", "PioneerDataSet",
    "ChangeParameterInfo", "OutingEvent", "ContinuousEvent",
    "RandomEvent", "MiniEvent", "RaceReward", "SkillPtReward",
    "TargetRace", "AfterRace", "InitialStatus", "LimitBreakCount",
    "FriendCard", "GroupCard", "SpecialCard",
]

so_res = search_so(patterns)

r = []
r.append("# libil2cpp.so 字符串搜索报告")
r.append(f"\n**文件**: libil2cpp.so (209MB, ARM64)")
r.append(f"**搜索模式**: {len(patterns)}个\n---\n")
for pat, matches in so_res.items():
    if matches:
        r.append(f"## \"{pat}\" ({len(matches)}匹配)\n")
        for off, s in matches[:20]:
            r.append(f"- `@0x{off:08x}`: `{s}`")
        r.append("")
    else:
        r.append(f"## \"{pat}\" — 无匹配\n")

so_report = '\n'.join(r)
with open(f"{OUT}/so_string_search.md", 'w') as f:
    f.write(so_report)
print(f"  ✓ so_string_search.md ({len(so_report)} bytes)")

# ============================================================
# Report 5: Master DB SQL queries
# ============================================================
def gen_sql():
    r = []
    r.append("# Master Database SQL 查询集\n")
    r.append("**说明**: master.mdb 运行时从服务器下载，不在APK中。")
    r.append("以下SQL基于IL2CPP Master类名推断表结构。\n---\n")

    mc = sorted([c for c in ALL_CLASSES if c['class'].startswith('Master') and c['ns'] == 'Gallop'], key=lambda x: x['class'])
    r.append(f"## Master表清单 ({len(mc)}个)\n")
    r.append("| 表名(推断) | 方法数 | 用途 |")
    r.append("|---|---|---|")
    for c in mc:
        tn = c['class'].replace('Master', '').lower()
        purpose = ""
        cl = c['class'].lower()
        if 'chara' in cl: purpose = "角色"
        elif 'card' in cl: purpose = "卡牌"
        elif 'support' in cl: purpose = "支援卡"
        elif 'skill' in cl: purpose = "技能"
        elif 'training' in cl: purpose = "训练"
        elif 'race' in cl: purpose = "比赛"
        elif 'scenario' in cl: purpose = "剧本"
        elif 'event' in cl: purpose = "事件"
        elif 'text' in cl: purpose = "文本"
        elif 'item' in cl: purpose = "道具"
        elif 'ramen' in cl: purpose = "拉面杯"
        elif 'cook' in cl: purpose = "料理杯"
        elif 'breeders' in cl: purpose = "种田杯"
        elif 'pioneer' in cl: purpose = "青春杯"
        elif 'onsen' in cl: purpose = "温泉"
        elif 'mecha' in cl: purpose = "机甲"
        elif 'live' in cl: purpose = "Live"
        elif 'arc' in cl: purpose = "Arc"
        elif 'venus' in cl: purpose = "维纳斯"
        elif 'legend' in cl: purpose = "传奇"
        elif 'sport' in cl: purpose = "运动"
        elif 'team' in cl: purpose = "团队"
        elif 'ura' in cl: purpose = "URA"
        r.append(f"| `{tn}` | {c.get('method_count',0)} | {purpose} |")

    r.append("\n## 关键SQL查询\n")
    queries = [
        ("角色ID映射", "SELECT id, name, rarity FROM chara_data ORDER BY id;"),
        ("支援卡ID映射", "SELECT id, name, rarity, support_card_type FROM support_card_data ORDER BY id;"),
        ("同名不同卡", "SELECT name, COUNT(*) cnt FROM chara_data GROUP BY name HAVING cnt > 1;"),
        ("3-5星初始属性", "SELECT chara_id, rarity, speed, stamina, power, guts, wisdom FROM chara_initial_status WHERE rarity >= 3;"),
        ("支援卡加成", "SELECT id, support_card_type, effect_type, effect_value FROM support_card_effect ORDER BY id;"),
        ("技能数据", "SELECT id, name, rarity, skill_type FROM skill_data ORDER BY id;"),
        ("训练等级效果", "SELECT command_id, level, gain_rate FROM single_mode_training_level ORDER BY command_id, level;"),
        ("拉面杯地区", "SELECT id, area_name FROM single_mode_14_area ORDER BY id;"),
        ("拉面杯道具", "SELECT id, item_name, effect_type, effect_value FROM single_mode_14_item ORDER BY id;"),
        ("拉面杯食材", "SELECT id, sozai_name, category FROM single_mode_14_sozai ORDER BY id;"),
        ("拉面杯配方", "SELECT id, recipe_name, required_sozai FROM single_mode_14_recipe ORDER BY id;"),
        ("拉面杯试食会", "SELECT id, tasting_name, condition FROM single_mode_14_tasting ORDER BY id;"),
        ("拉面杯检查点", "SELECT id, check_point_type, condition FROM single_mode_14_check_point ORDER BY id;"),
        ("种田杯Enhance", "SELECT id, group_type, level, effect FROM single_mode_13_enhance ORDER BY id;"),
        ("青春杯Dream", "SELECT id, dream_point, effect FROM single_mode_11_dream ORDER BY id;"),
        ("目标比赛", "SELECT id, race_name, race_grade, required_month FROM single_mode_target_race ORDER BY id;"),
        ("赛后技能PT", "SELECT race_id, skill_pt_reward FROM single_mode_race_reward ORDER BY race_id;"),
        ("连续事件概率", "SELECT event_id, probability FROM single_mode_continuous_event ORDER BY event_id;"),
        ("乱入事件", "SELECT event_id, probability, condition FROM single_mode_random_event ORDER BY event_id;"),
        ("马娘加成", "SELECT chara_id, bonus_type, bonus_value FROM chara_bonus ORDER BY chara_id;"),
    ]
    for name, sql in queries:
        r.append(f"### {name}\n```sql\n{sql}\n```\n")
    return '\n'.join(r)

sql_rep = gen_sql()
with open(f"{OUT}/master_db_sql_queries.md", 'w') as f:
    f.write(sql_rep)
print(f"  ✓ master_db_sql_queries.md ({len(sql_rep)} bytes)")

# ============================================================
# Report 6: Full class dump by namespace
# ============================================================
def gen_classdump():
    r = []
    r.append("# IL2CPP 全量类转储 (按命名空间)\n")
    r.append(f"**总类数**: {len(ALL_CLASSES)}  \n**总方法数**: {dump['total_methods']}\n---\n")

    ns_map = defaultdict(list)
    for c in ALL_CLASSES:
        ns_map[c['ns']].append(c)
    ns_sorted = sorted(ns_map.items(), key=lambda x: sum(c.get('method_count',0) for c in x[1]), reverse=True)

    r.append("## 命名空间统计 (Top 50)\n")
    r.append("| 命名空间 | 类数 | 方法数 |")
    r.append("|---|---|---|")
    for ns, classes in ns_sorted[:50]:
        tm = sum(c.get('method_count',0) for c in classes)
        r.append(f"| `{ns}` | {len(classes)} | {tm} |")

    # Detail for Gallop namespace
    if 'Gallop' in ns_map:
        r.append(f"\n## Gallop 命名空间详细 ({len(ns_map['Gallop'])} classes)\n")
        for c in sorted(ns_map['Gallop'], key=lambda x: x.get('method_count',0), reverse=True)[:300]:
            r.append(f"### `{c['class']}` ({c.get('method_count',0)}m)")
            for m in methods_of(c)[:5]:
                r.append(f"  - `{m}`")
            r.append("")

    return '\n'.join(r)

cd = gen_classdump()
with open(f"{OUT}/all_classes_dump.md", 'w') as f:
    f.write(cd)
print(f"  ✓ all_classes_dump.md ({len(cd)} bytes)")

# ============================================================
# Report 7: Ramen-specific deep dive (Scenario 14)
# ============================================================
def gen_ramen_deep():
    r = []
    r.append("# 剧本14 拉面杯(Twinkle Ramen)深度分析\n")
    r.append("**剧本ID**: 14  \n**WorkScenario**: `WorkSingleModeScenarioRamen`")
    r.append("**DataSet**: `ObscuredSingleModeRamenDataSet`\n---\n")

    # All Ramen classes
    ramen = [c for c in ALL_CLASSES if 'Ramen' in c['class'] and c['ns'] == 'Gallop']
    r.append(f"## 拉面杯相关类 ({len(ramen)}个)\n")
    for c in sorted(ramen, key=lambda x: x['class']):
        r.append(f"### `{c['class']}` ({c.get('method_count',0)}m)")
        gtrs = getters_of(c)
        if gtrs:
            r.append(f"**Getters** ({len(gtrs)}个):")
            for m in gtrs[:20]:
                r.append(f"  - `{m}`")
        else:
            for m in methods_of(c)[:10]:
                r.append(f"  - `{m}`")
        r.append("")

    # Ramen DataSet
    rds = CLASS_INDEX.get('Gallop.ObscuredSingleModeRamenDataSet', {})
    if rds:
        r.append(f"\n## ObscuredSingleModeRamenDataSet ({rds.get('method_count',0)}m)\n")
        for m in methods_of(rds):
            r.append(f"  - `{m}`")

    # Ramen Command Info
    rci = find_classes(r'RamenCommand')
    r.append(f"\n## 拉面杯指令系统\n")
    for c in rci:
        r.append(f"### `{c['class']}` ({c.get('method_count',0)}m)")
        for m in methods_of(c)[:15]:
            r.append(f"  - `{m}`")
        r.append("")

    # Ramen Feeling
    rf = find_classes(r'RamenFeeling')
    r.append(f"\n## 拉面杯Feeling(心情)系统\n")
    for c in rf:
        r.append(f"### `{c['class']}` ({c.get('method_count',0)}m)")
        for m in methods_of(c)[:15]:
            r.append(f"  - `{m}`")
        r.append("")

    # Ramen CheckPoint
    rcp = find_classes(r'RamenCheckPoint|RamenCheckEvent')
    r.append(f"\n## 拉面杯检查点系统\n")
    for c in rcp:
        r.append(f"### `{c['class']}` ({c.get('method_count',0)}m)")
        for m in methods_of(c)[:15]:
            r.append(f"  - `{m}`")
        r.append("")

    # Ramen Evaluation
    rev = find_classes(r'RamenEvaluation')
    r.append(f"\n## 拉面杯羁绊/评价系统\n")
    for c in rev:
        r.append(f"### `{c['class']}` ({c.get('method_count',0)}m)")
        for m in methods_of(c)[:15]:
            r.append(f"  - `{m}`")
        r.append("")

    # Ramen Start/Load
    rsl = find_classes(r'RamenDataSetStart|RamenDataSetLoad')
    r.append(f"\n## 拉面杯初始化系统\n")
    for c in rsl:
        r.append(f"### `{c['class']}` ({c.get('method_count',0)}m)")
        for m in methods_of(c)[:20]:
            r.append(f"  - `{m}`")
        r.append("")

    # Master Ramen tables
    mr = [c for c in ALL_CLASSES if c['class'].startswith('Master') and 'Ramen' in c['class'] and c['ns'] == 'Gallop']
    if mr:
        r.append(f"\n## Master数据库拉面杯表 ({len(mr)}个)\n")
        r.append("| 表名 | 方法数 |")
        r.append("|---|---|")
        for c in mr:
            r.append(f"| `{c['class']}` | {c.get('method_count',0)} |")

    # Ramen related strings from SO
    r.append(f"\n## libil2cpp.so 中的拉面杯字符串\n")
    with open(f"{OUT}/so_string_search.md") as f:
        so_content = f.read()
    for line in so_content.split('\n'):
        if 'Ramen' in line or 'ramen' in line:
            r.append(line)

    return '\n'.join(r)

ramen_rep = gen_ramen_deep()
with open(f"{OUT}/scenario_14_ramen_deep.md", 'w') as f:
    f.write(ramen_rep)
print(f"  ✓ scenario_14_ramen_deep.md ({len(ramen_rep)} bytes)")

print("\n=== 全部报告生成完成! ===")
print(f"输出目录: {OUT}/")
