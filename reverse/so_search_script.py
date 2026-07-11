#!/usr/bin/env python3
"""Search libil2cpp.so for strings - runs as standalone script"""
import sys

SO_PATH = "/tmp/apk_extract/lib_arm64-v8a_libil2cpp.so"
OUT = "/home/z/my-project/repos/hlpatch/reverse/so_string_search.md"

print("Extracting strings from libil2cpp.so (209MB)...", flush=True)
with open(SO_PATH, 'rb') as f:
    data = f.read()

# Extract all null-terminated ASCII strings >= 8 chars
strings = []
i = 0
n = len(data)
while i < n:
    if 32 <= data[i] < 127:
        start = i
        while i < n and 32 <= data[i] < 127:
            i += 1
        if i - start >= 8 and i < n and data[i] == 0:
            strings.append((start, data[start:i].decode('ascii')))
    else:
        i += 1
    if i % 10000000 == 0:
        print(f"  {i*100//n}%...", flush=True)

print(f"Total strings >= 8 chars: {len(strings)}", flush=True)

# Search for key patterns
patterns = [
    'SingleMode', 'Ramen', 'Breeders', 'Pioneer', 'Cook', 'Onsen', 'Mecha',
    'Legend', 'Venus', 'Arc', 'Live', 'Sport', 'TeamRace', 'URA',
    'Training', 'SupportCard', 'Evaluation', 'TipsEvent', 'Shining',
    'CommandId', 'TargetType', 'TrainingLevel', 'ObscuredInt',
    'ChangeParameter', 'OutingEvent', 'MiniEvent', 'RandomEvent',
    'RaceReward', 'TargetRace', 'SkillPt', 'BondEffect',
    'PartnerEffect', 'FriendCard', 'GroupCard',
    'chara_data', 'card_data', 'support_card',
    'Feeling', 'CheckPoint', 'Sozai', 'Recipe', 'Tasting',
    'Enhance', 'Dream', 'Facility', 'Board', 'Audience',
    'Boost', 'Heart', 'FixedTurn', 'RNG', 'Random',
    'SuccessRate', 'FailureRate', 'Vital', 'Motivation',
    'SkillData', 'CharaData', 'CardData', 'TextData',
    'MasterData', 'ScenarioData', 'TrainingData',
    'Outing', 'Shopping', 'Rest', 'Race',
    'AreaSelect', 'Region', 'Noodle', 'Ingredient',
    'Trial', 'Exhibition', 'Store', 'Shop',
    'Reporter', 'Chairman', 'President', 'Director',
]

with open(OUT, 'w') as f:
    f.write("# libil2cpp.so 字符串搜索报告\n")
    f.write(f"\n**文件**: libil2cpp.so ({len(data)//1048576}MB, ARM64)\n")
    f.write(f"**总字符串数(>=8字符)**: {len(strings)}\n---\n\n")
    
    for pat in patterns:
        matches = []
        seen = set()
        for off, s in strings:
            if pat.lower() in s.lower() and s not in seen and len(s) < 500:
                seen.add(s)
                matches.append((off, s))
        
        if matches:
            f.write(f"## \"{pat}\" ({len(matches)}匹配)\n\n")
            for off, s in matches[:50]:
                f.write(f"- `@0x{off:08x}`: `{s}`\n")
            f.write("\n")

print(f"Report written to {OUT}", flush=True)
