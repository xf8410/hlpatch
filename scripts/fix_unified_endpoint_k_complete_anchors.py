from pathlib import Path

P=Path('scripts/apply_unified_endpoint_k_complete.py')
s=P.read_text(encoding='utf-8')
old='''health='r#"{{\\\\"status\\\\":\\\\"ok\\\\",\\\\"version\\\\":\\\\"{}\\\\",\\\\"endpoints\\\\":['
assert s.count(health)==1
s=s.replace(health,health+prefix,1)
available='r#"{{\\\\"error\\\\":\\\\"not_found\\\\",\\\\"path\\\\":\\\\"{}\\\\",\\\\"available\\\\":['
assert s.count(available)==1
s=s.replace(available,available+prefix,1)
'''
new='''health='r#"{{\\"status\\":\\"ok\\",\\"version\\":\\"{}\\",\\"endpoints\\":['
assert s.count(health)==1, f"health_advertisement_anchor_count={s.count(health)}"
s=s.replace(health,health+prefix,1)
available='r#"{{\\"error\\":\\"not_found\\",\\"path\\":\\"{}\\",\\"available\\":['
assert s.count(available)==1, f"available_advertisement_anchor_count={s.count(available)}"
s=s.replace(available,available+prefix,1)
'''
assert s.count(old)==1, s.count(old)
s=s.replace(old,new,1)
P.write_text(s,encoding='utf-8')
print('k_complete_anchor_fix=applied')
