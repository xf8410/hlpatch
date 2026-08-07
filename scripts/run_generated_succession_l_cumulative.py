import subprocess
from pathlib import Path
import hashlib

common=[
 'scripts/fix_unified_endpoint_f_patch_anchors.py',
 'scripts/apply_unified_endpoint_a_patch.py','scripts/apply_unified_endpoint_a_compile_fix.py',
 'scripts/apply_unified_endpoint_b_storage_patch.py','scripts/apply_unified_endpoint_b_storage_compile_fix.py',
 'scripts/apply_unified_endpoint_c_inherit_pair_patch.py','scripts/apply_unified_endpoint_d_selected_parent_patch.py',
 'scripts/apply_unified_endpoint_e_runtime_correction.py','scripts/apply_unified_endpoint_f_pre_release_fix.py',
 'scripts/apply_unified_endpoint_g_release_gate_fix.py','scripts/apply_unified_endpoint_h_response_headers.py',
 'scripts/apply_unified_endpoint_i_parent_multisource.py','scripts/apply_unified_endpoint_j_parent_runtime_semantics.py',
 'scripts/apply_unified_endpoint_k_complete.py']
# 两项修正修改的是K补丁生成器，必须在K第一次生成Rust源码之前执行。
subprocess.run(['python3','scripts/fix_unified_endpoint_k_complete_anchors.py'],check=True)
subprocess.run(['python3','scripts/fix_unified_endpoint_k_rust_types.py'],check=True)
for pass_no in (1,2):
    for script in common: subprocess.run(['python3',script],check=True)
    subprocess.run(['python3','scripts/apply_generated_succession_runtime_l.py'],check=True)
    subprocess.run(['python3','scripts/apply_generated_succession_runtime_l_support_fix.py'],check=True)
    subprocess.run(['python3','scripts/apply_ramen_global_durable_observation.py'],check=True)
    subprocess.run(['python3','scripts/apply_hachimi_textcommon_observer.py'],check=True)
    source=Path('hachimi_ura_plugin/src/lib.rs').read_bytes()
    Path(f'source-{pass_no}.sha').write_text(hashlib.sha256(source).hexdigest()+'\n')
assert Path('source-1.sha').read_text()==Path('source-2.sha').read_text()
print('generated_succession_l_cumulative=idempotent_with_ramen_text_observation')
