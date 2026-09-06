from pathlib import Path

P=Path('scripts/apply_unified_endpoint_k_complete.py')
s=P.read_text(encoding='utf-8')
old_call='''        k_domain_endpoint(path, &full_uri)
'''
new_call='''        k_domain_endpoint(&path, &full_uri)
'''
count=s.count(old_call)
if count!=1:
    if s.count(new_call)==1:
        print('k_rust_type_fix already applied, skip call')
    else:
        raise AssertionError(f'k_domain_template_call_count={count}')
else:
    s=s.replace(old_call,new_call,1)
old_replace="""    let token=domain.replace('/','_');let like=format!(\"%{}%\",token);
"""
new_replace="""    let token=domain.replace('/',\"_\");let like=format!(\"%{}%\",token);
"""
if s.count(old_replace)!=1:
    if s.count(new_replace)==1:
        print('k_rust_type_fix already applied, skip replace')
    else:
        raise AssertionError(f'domain_replace_count={s.count(old_replace)}')
else:
    s=s.replace(old_replace,new_replace,1)
P.write_text(s,encoding='utf-8')
print('k_rust_type_fix=applied')
