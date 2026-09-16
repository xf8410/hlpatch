#!/usr/bin/env python3
# v3.28.3 - Remove the SO plugin's built-in AI evaluation (runtime layer).
# Decision ownership: jueceramen (uma-juece-ramen, aligned with the umaai-rs MCTS engine)
# owns all decisions. The SO returns to a pure data pipeline.
# Changes:
#   1) /summary's `let ai_json = {...}` evaluation block is fixed to output null
#      (anchor + brace-matching approach proven in the v3.27.20 release).
#   2) If the /event/recommend route string exists, rename it to a dead path (falls to 404).
#      Only the string literal is touched; no function bodies are modified.
#      Data endpoints (/api/event/choices, /api/event/observations) are untouched.
# Idempotent: repeated runs do not change the file. Missing anchors raise loudly.
from pathlib import Path

source = Path('hachimi_ura_plugin/src/lib.rs')
text = source.read_text(encoding='utf-8')

# ---- 1) ai_json evaluation block -> fixed null ----
if 'v3.28.3 ai removed' in text:
    print('ai_json=already_removed')
else:
    marker = 'let ai_json = {'
    pos = text.find(marker)
    if pos < 0:
        raise RuntimeError('ai_json block not found: anchor drift, refusing silent pass')
    i = pos + len(marker)
    depth = 1
    while i < len(text) and depth > 0:
        c = text[i]
        if c == '{':
            depth += 1
        elif c == '}':
            depth -= 1
        i += 1
    if depth != 0:
        raise RuntimeError('ai_json brace balance failed: refusing blind replacement')
    end = i + 1 if (i < len(text) and text[i] == ';') else i
    new_block = ('let ai_json = String::from("null"); '
                 '// v3.28.3 ai removed: SO is a pure data pipeline, jueceramen owns all decisions')
    text = text[:pos] + new_block + text[end:]
    print('ai_json=removed')

# ---- 2) /event/recommend route string renamed to a dead path ----
lit = '"/event/recommend"'
n = text.count(lit)
if n == 0:
    print('event_recommend=route_absent')
else:
    text = text.replace(lit, '"/event/__recommend_disabled_v3283__"')
    print('event_recommend=disabled occurrences=%d' % n)

if 'v3.28.3 ai removed' not in text:
    raise RuntimeError('ai marker missing after strip')
source.write_text(text, encoding='utf-8')
print('strip_ai_output_v3283=ok')
