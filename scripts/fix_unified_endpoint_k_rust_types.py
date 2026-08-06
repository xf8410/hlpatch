from pathlib import Path

P=Path('scripts/apply_unified_endpoint_k_complete.py')
s=P.read_text(encoding='utf-8')
old_call='''        k_domain_endpoint(path, &full_uri)
'''
new_call='''        k_domain_endpoint(&path, &full_uri)
'''
count=s.count(old_call)
assert count==1, f'k_domain_template_call_count={count}'
s=s.replace(old_call,new_call,1)
old_replace="""    let token=domain.replace('/','_');let like=format!(\"%{}%\",token);
"""
new_replace="""    let token=domain.replace('/',\"_\");let like=format!(\"%{}%\",token);
"""
assert s.count(old_replace)==1, f'domain_replace_count={s.count(old_replace)}'
s=s.replace(old_replace,new_replace,1)
P.write_text(s,encoding='utf-8')
print('k_rust_type_fix=applied')
