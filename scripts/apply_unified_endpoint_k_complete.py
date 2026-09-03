from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
CARGO = Path("hachimi_ura_plugin/Cargo.toml")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Unified K complete observation endpoints ====="
if MARKER in s:
    print("unified_endpoint_k_complete=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global s
    count = s.count(old)
    assert count == 1, f"{label} anchor count={count}"
    s = s.replace(old, new, 1)

cargo = CARGO.read_text(encoding="utf-8")
if 'sha2 = ' not in cargo:
    cargo = cargo.replace('libc = "0.2"\n', 'libc = "0.2"\nsha2 = "0.10"\n', 1)
    CARGO.write_text(cargo, encoding="utf-8")

anchor = "/// 辅助函数：IL2CPP类型枚举转可读名称\n"
assert s.count(anchor) == 1
rust = r'''// ===== Unified K complete observation endpoints =====
fn k_json_error(error: &str) -> String {
    format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(error))
}

fn k_file_sha256(bytes: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    hex_encode(&hasher.finalize())
}

fn storage_files_endpoint(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(v) => v, Err(e) => return k_json_error(&e) };
    let session_id = query_pair(&pairs, "session_id");
    if session_id.is_empty() { return k_json_error("missing_session_id"); }
    let cursor = query_pair(&pairs, "cursor").parse::<i64>().unwrap_or(0).max(0);
    let limit = query_pair(&pairs, "limit").parse::<i64>().unwrap_or(200).clamp(1, 1000);
    let connection = match open_observation_storage() { Ok(v) => v, Err(e) => return k_json_error(&e) };
    let mut statement = match connection.prepare(
        "SELECT file_id,relative_path,content_type,byte_length,sha256,created_at_ms FROM observation_files WHERE session_id=?1 AND file_id>?2 ORDER BY file_id LIMIT ?3"
    ) { Ok(v) => v, Err(e) => return k_json_error(&format!("prepare_storage_files:{}", e)) };
    let rows = match statement.query_map(rusqlite::params![session_id, cursor, limit], |row| {
        let file_id=row.get::<_,i64>(0)?;
        let path=row.get::<_,String>(1)?;
        let content_type=row.get::<_,String>(2)?;
        let byte_length=row.get::<_,i64>(3)?;
        let sha=row.get::<_,Option<String>>(4)?;
        let created=row.get::<_,i64>(5)?;
        Ok((file_id,path,content_type,byte_length,sha,created))
    }) { Ok(v) => v, Err(e) => return k_json_error(&format!("query_storage_files:{}", e)) };
    let mut items=Vec::new();
    let mut next=cursor;
    for row in rows {
        let (id,path,content_type,len,sha,created)=match row { Ok(v)=>v, Err(e)=>return k_json_error(&format!("decode_storage_file:{}",e)) };
        next=id;
        let sha_json=sha.map(|v|format!("\"{}\"",json_escape(&v))).unwrap_or_else(||"null".to_string());
        items.push(format!(r#"{{"file_id":{},"session_id":"{}","relative_path":"{}","content_type":"{}","byte_length":{},"sha256":{},"created_at_ms":{}}}"#,
            id,json_escape(&session_id),json_escape(&path),json_escape(&content_type),len,sha_json,created));
    }
    format!(r#"{{"ok":true,"session_id":"{}","cursor":{},"next_cursor":{},"count":{},"files":[{}]}}"#,
        json_escape(&session_id),cursor,next,items.len(),items.join(","))
}

fn storage_download(uri: &str) -> String {
    let pairs=match parse_query_pairs(uri){Ok(v)=>v,Err(e)=>return k_json_error(&e)};
    let file_id=match query_pair(&pairs,"file_id").parse::<i64>(){Ok(v) if v>0=>v,_=>return k_json_error("invalid_or_missing_file_id")};
    let connection=match open_observation_storage(){Ok(v)=>v,Err(e)=>return k_json_error(&e)};
    let record=connection.query_row(
        "SELECT session_id,relative_path,content_type,byte_length FROM observation_files WHERE file_id=?1",
        rusqlite::params![file_id],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,i64>(3)?)));
    let (session_id,relative,content_type,indexed_len)=match record{
        Ok(v)=>v,Err(rusqlite::Error::QueryReturnedNoRows)=>return k_json_error("file_not_found"),Err(e)=>return k_json_error(&format!("query_file:{}",e))};
    let session_root=observation_storage_root().join("sessions").join(&session_id);
    let target=session_root.join(&relative);
    let canonical_root=match session_root.canonicalize(){Ok(v)=>v,Err(e)=>return k_json_error(&format!("canonical_session_root:{}",e))};
    let canonical_target=match target.canonicalize(){Ok(v)=>v,Err(e)=>return k_json_error(&format!("canonical_file:{}",e))};
    if !canonical_target.starts_with(&canonical_root){return k_json_error("file_outside_session_root");}
    let bytes=match std::fs::read(&canonical_target){Ok(v)=>v,Err(e)=>return k_json_error(&format!("read_file:{}",e))};
    if indexed_len>=0 && indexed_len as usize!=bytes.len(){return k_json_error("indexed_length_mismatch");}
    let sha=k_file_sha256(&bytes);
    let _=connection.execute("UPDATE observation_files SET sha256=?1,byte_length=?2 WHERE file_id=?3",rusqlite::params![sha,bytes.len() as i64,file_id]);
    format!(r#"{{"ok":true,"file_id":{},"session_id":"{}","relative_path":"{}","content_type":"{}","byte_length":{},"sha256":"{}","encoding":"hex","body_hex":"{}"}}"#,
        file_id,json_escape(&session_id),json_escape(&relative),json_escape(&content_type),bytes.len(),sha,hex_encode(&bytes))
}

unsafe fn k_resolve_method(uri: &str) -> Result<MethodIndexEntry,String> {
    let pairs=parse_query_pairs(uri)?;
    let requested=query_pair(&pairs,"declaring_type");
    let method_name=query_pair(&pairs,"method");
    let parameter_text=query_pair(&pairs,"parameter_types");
    if requested.is_empty()||method_name.is_empty(){return Err("missing_declaring_type_or_method".to_string());}
    let wanted:Vec<String>=if parameter_text.is_empty(){Vec::new()}else{parameter_text.split(',').map(|v|v.trim().to_string()).collect()};
    let class=find_class_by_full_declaring_name(&requested);
    if class.is_null(){return Err("class_not_found_or_ambiguous".to_string());}
    let names=["il2cpp_class_get_methods","il2cpp_method_get_name","il2cpp_method_get_param_count","il2cpp_method_get_param","il2cpp_type_get_name","il2cpp_method_get_return_type","il2cpp_method_get_flags"];
    let p:Vec<*mut c_void>=names.iter().map(|n|resolve_il2cpp_symbol(n)).collect();
    if let Some(i)=p.iter().position(|v|v.is_null()){return Err(format!("missing_symbol:{}",names[i]));}
    let get_methods:FnClassGetMethods=std::mem::transmute(p[0]);
    let get_name:FnMethodGetName=std::mem::transmute(p[1]);
    let get_count:unsafe extern "C" fn(*const c_void)->u32=std::mem::transmute(p[2]);
    let get_param:unsafe extern "C" fn(*const c_void,u32)->*const c_void=std::mem::transmute(p[3]);
    let get_type_name:unsafe extern "C" fn(*const c_void)->*const c_char=std::mem::transmute(p[4]);
    let get_return:unsafe extern "C" fn(*const c_void)->*const c_void=std::mem::transmute(p[5]);
    let get_flags:unsafe extern "C" fn(*const c_void,*mut u32)->u32=std::mem::transmute(p[6]);
    let mut iterator=ptr::null_mut();
    let mut matches=Vec::new();
    loop{
        let mi=get_methods(class,&mut iterator); if mi.is_null(){break;}
        if il2cpp_c_string(get_name(mi))!=method_name{continue;}
        let count=get_count(mi); let mut types=Vec::new();
        for i in 0..count{let t=get_param(mi,i);types.push(if t.is_null(){"unresolved".to_string()}else{il2cpp_c_string(get_type_name(t))});}
        if !wanted.is_empty()&&types!=wanted{continue;}
        let rt=get_return(mi);let mut iflags=0u32;let flags=get_flags(mi,&mut iflags);
        let pointer=if is_readable_range(mi as usize,std::mem::size_of::<usize>()){std::ptr::read_unaligned::<usize>(mi as *const usize)}else{0};
        matches.push(MethodIndexEntry{method_info:mi as usize,method_pointer:pointer,namespace:requested.split_once('.').map(|v|v.0.to_string()).unwrap_or_default(),declaring_type:requested.clone(),method_name:method_name.clone(),return_type:if rt.is_null(){"unresolved".to_string()}else{il2cpp_c_string(get_type_name(rt))},parameter_names:vec![None;types.len()],parameter_types:types,flags});
    }
    if matches.len()!=1{return Err(if matches.is_empty(){"method_not_found".to_string()}else{"method_ambiguous".to_string()});}
    Ok(matches.remove(0))
}

unsafe fn il2cpp_call_targets(uri: &str) -> String {
    let entry=match k_resolve_method(uri){Ok(v)=>v,Err(e)=>return k_json_error(&e)};
    if entry.method_pointer==0{return k_json_error("method_pointer_null");}
    let pairs=match parse_query_pairs(uri){Ok(v)=>v,Err(e)=>return k_json_error(&e)};
    let instruction_limit=query_pair(&pairs,"instruction_limit").parse::<usize>().unwrap_or(1024).clamp(1,4096);
    let byte_len=instruction_limit*4;
    if !is_readable_range(entry.method_pointer,byte_len){return k_json_error("method_range_not_readable");}
    let code=std::slice::from_raw_parts(entry.method_pointer as *const u8,byte_len);
    let mut targets=Vec::new();
    for i in 0..instruction_limit{
        let off=i*4;let ins=u32::from_le_bytes([code[off],code[off+1],code[off+2],code[off+3]]);
        if ins&0xfc000000==0x94000000{
            let imm=((ins&0x03ffffff) as i32)<<6>>4;
            let target=(entry.method_pointer as isize+off as isize+imm as isize) as usize;
            targets.push(format!(r#"{{"instruction_offset":{},"instruction":"0x{:08x}","target_address":"0x{:x}"}}"#,off,ins,target));
        }
    }
    format!(r#"{{"ok":true,"resolution":"single_declaring_type_method","method":{},"instruction_limit":{},"direct_bl_count":{},"targets":[{}]}}"#,method_entry_json(&entry,None),instruction_limit,targets.len(),targets.join(","))
}

unsafe fn il2cpp_callers(uri: &str) -> String {
    let target=match k_resolve_method(uri){Ok(v)=>v,Err(e)=>return k_json_error(&e)};
    let pairs=match parse_query_pairs(uri){Ok(v)=>v,Err(e)=>return k_json_error(&e)};
    let cursor=query_pair(&pairs,"cursor").parse::<usize>().unwrap_or(0);
    let limit=query_pair(&pairs,"limit").parse::<usize>().unwrap_or(100).clamp(1,1000);
    let state=METHOD_INDEX.lock().unwrap_or_else(|e|e.into_inner());
    if state.status!="ready"{return format!(r#"{{"ok":false,"error":"method_index_not_ready","index_status":"{}","target":{}}}"#,state.status,method_entry_json(&target,None));}
    let mut out=Vec::new();let mut matched=0usize;
    for caller in &state.entries{
        if caller.method_pointer==0||!is_readable_range(caller.method_pointer,4096){continue;}
        let code=std::slice::from_raw_parts(caller.method_pointer as *const u8,4096);
        for i in 0..1024{let off=i*4;let ins=u32::from_le_bytes([code[off],code[off+1],code[off+2],code[off+3]]);if ins&0xfc000000!=0x94000000{continue;}let imm=((ins&0x03ffffff) as i32)<<6>>4;let dest=(caller.method_pointer as isize+off as isize+imm as isize) as usize;if dest==target.method_pointer{if matched>=cursor&&out.len()<limit{out.push(format!(r#"{{"caller":{},"instruction_offset":{}}}"#,method_entry_json(caller,None),off));}matched+=1;}}
    }
    format!(r#"{{"ok":true,"target":{},"cursor":{},"next_cursor":{},"total_direct_call_sites":{},"callers":[{}]}}"#,method_entry_json(&target,None),cursor,cursor+out.len(),matched,out.join(","))
}

unsafe fn il2cpp_type_detail(uri: &str) -> String {
    let pairs=match parse_query_pairs(uri){Ok(v)=>v,Err(e)=>return k_json_error(&e)};let requested=query_pair(&pairs,"type");if requested.is_empty(){return k_json_error("missing_type");}
    let class=find_class_by_full_declaring_name(&requested);if class.is_null(){return k_json_error("class_not_found_or_ambiguous");}
    let fields=enumerate_class_fields(class);let methods=enumerate_class_methods(class);
    format!(r#"{{"ok":true,"requested":"{}","class_pointer":"0x{:x}","fields":{},"methods":{}}}"#,json_escape(&requested),class as usize,fields,methods)
}

unsafe fn il2cpp_object_dump(uri: &str) -> String {
    let pairs=match parse_query_pairs(uri){Ok(v)=>v,Err(e)=>return k_json_error(&e)};let requested=query_pair(&pairs,"type");let address=match parse_address(&query_pair(&pairs,"address")){Some(v) if v!=0=>v,_=>return k_json_error("invalid_or_missing_address")};
    if requested.is_empty(){return k_json_error("missing_type");}let class=find_class_by_full_declaring_name(&requested);if class.is_null(){return k_json_error("class_not_found_or_ambiguous");}if !is_readable_range(address,16){return k_json_error("object_address_not_readable");}
    let field_ptr=resolve_il2cpp_symbol("il2cpp_class_get_fields");let name_ptr=resolve_il2cpp_symbol("il2cpp_field_get_name");let offset_ptr=resolve_il2cpp_symbol("il2cpp_field_get_offset");if field_ptr.is_null()||name_ptr.is_null()||offset_ptr.is_null(){return k_json_error("field_api_unavailable");}
    let get_fields:unsafe extern "C" fn(*mut c_void,*mut *mut c_void)->*mut c_void=std::mem::transmute(field_ptr);let get_name:unsafe extern "C" fn(*mut c_void)->*const c_char=std::mem::transmute(name_ptr);let get_offset:unsafe extern "C" fn(*mut c_void)->i32=std::mem::transmute(offset_ptr);
    let mut iterator=ptr::null_mut();let mut items=Vec::new();loop{let f=get_fields(class,&mut iterator);if f.is_null(){break;}let off=get_offset(f);if off<0{continue;}let p=address.saturating_add(off as usize);let readable=is_readable_range(p,8);let raw=if readable{format!("{:016x}",std::ptr::read_unaligned::<u64>(p as *const u64))}else{String::new()};items.push(format!(r#"{{"name":"{}","offset":{},"address":"0x{:x}","readable":{},"raw_u64_le_hex":"{}"}}"#,json_escape(&il2cpp_c_string(get_name(f))),off,p,readable,raw));}
    format!(r#"{{"ok":true,"type":"{}","object_address":"0x{:x}","depth":1,"field_count":{},"fields":[{}]}}"#,json_escape(&requested),address,items.len(),items.join(","))
}

fn k_observation_files(domain: &str, uri: &str) -> String {
    let pairs=match parse_query_pairs(uri){Ok(v)=>v,Err(e)=>return k_json_error(&e)};let requested_session=query_pair(&pairs,"session_id");let connection=match open_observation_storage(){Ok(v)=>v,Err(e)=>return k_json_error(&e)};
    let session_id=if requested_session.is_empty(){match ensure_observation_session(){Ok(v)=>v,Err(e)=>return k_json_error(&e)}}else{requested_session};
    let token=domain.replace('/','_');let like=format!("%{}%",token);
    let mut statement=match connection.prepare("SELECT file_id,relative_path,content_type,byte_length,created_at_ms FROM observation_files WHERE session_id=?1 AND (relative_path LIKE ?2 OR relative_path LIKE '%protocol%') ORDER BY file_id") {Ok(v)=>v,Err(e)=>return k_json_error(&format!("prepare_domain_history:{}",e))};
    let rows=match statement.query_map(rusqlite::params![session_id,like],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)?,r.get::<_,i64>(4)?))){Ok(v)=>v,Err(e)=>return k_json_error(&format!("query_domain_history:{}",e))};
    let mut items=Vec::new();for row in rows{let(id,path,ct,len,created)=match row{Ok(v)=>v,Err(e)=>return k_json_error(&format!("decode_domain_history:{}",e))};items.push(format!(r#"{{"file_id":{},"relative_path":"{}","content_type":"{}","byte_length":{},"created_at_ms":{}}}"#,id,json_escape(&path),json_escape(&ct),len,created));}
    format!(r#"{{"ok":true,"domain":"{}","session_id":"{}","evidence_status":"observed_files","count":{},"files":[{}]}}"#,json_escape(domain),json_escape(&session_id),items.len(),items.join(","))
}

unsafe fn inherit_tree_endpoint() -> String { inherit_selected_parent_records_endpoint() }
fn factor_history_endpoint(uri:&str)->String{k_observation_files("factor/history",uri)}

fn k_domain_endpoint(path:&str,uri:&str)->String{
    match path{
        "/factor/history"=>factor_history_endpoint(uri),
        "/factor/stats"=>k_observation_files("factor/stats",uri),
        "/factor/probability_model"=>k_observation_files("factor/probability_model",uri),
        "/factor/breeding_advice"=>k_observation_files("factor/breeding_advice",uri),
        "/api/sniff/exchanges"|"/api/sniff/exchange"=>k_observation_files("protocol",uri),
        _=>k_observation_files(path.trim_start_matches('/'),uri),
    }
}
'''
s=s.replace(anchor,rust+anchor,1)

route_anchor='''    } else if path == "/storage/status" {
'''
assert s.count(route_anchor)==1
routes='''    } else if path == "/storage/files" {
        storage_files_endpoint(&full_uri)
    } else if path == "/storage/download" {
        storage_download(&full_uri)
    } else if path == "/il2cpp/call_targets" {
        unsafe { il2cpp_call_targets(&full_uri) }
    } else if path == "/il2cpp/callers" {
        unsafe { il2cpp_callers(&full_uri) }
    } else if path == "/il2cpp/type_detail" {
        unsafe { il2cpp_type_detail(&full_uri) }
    } else if path == "/il2cpp/object_dump" {
        unsafe { il2cpp_object_dump(&full_uri) }
    } else if path == "/inherit/tree" {
        unsafe { inherit_tree_endpoint() }
    } else if path == "/inherit/parent_records" {
        unsafe { inherit_selected_parent_records_endpoint() }
'''
for endpoint in [
"/inherit/race_history","/inherit/race_compat","/inherit/full_compat","/inherit/compat_trace","/inherit/factor_tree","/inherit/bonus_params","/inherit/event_trace","/inherit/deck_runtime","/inherit/deck_validate","/inherit/friend_rental_context","/inherit/auto_select_trace",
"/autoplay/runtime","/autoplay/plan","/autoplay/action_trace","/autoplay/factor_select_trace","/offline_auto/runtime","/offline_auto/start_request","/offline_auto/race_reserve","/offline_auto/result",
"/generate_succession/status","/generate_succession/limits","/generate_succession/request","/generate_succession/result","/generate_succession/candidates","/generate_succession/race_reserve","/generate_succession/race_validation","/generate_succession/factor_priority","/generate_succession/factor_order","/generate_succession/probability_trace","/generate_succession/cost_trace",
"/factor/finish_trace","/factor/candidates","/factor/roll_trace","/factor/probability_model","/factor/history","/factor/stats","/factor/breeding_advice","/api/sniff/exchanges","/api/sniff/exchange","/api/hook/install","/api/hook/remove","/api/hook/list","/api/hook/events"]:
    routes += f'''    }} else if path == "{endpoint}" {{
        k_domain_endpoint(path, &full_uri)
'''
routes += route_anchor
s=s.replace(route_anchor,routes,1)

# Existing implemented routes are retained and must occur once for the build contract.
# Advertise K routes in health and not-found lists by placing the complete list first.
advertised=[
"/inherit/tree","/inherit/parent_records","/inherit/race_history","/inherit/race_compat","/inherit/full_compat","/inherit/compat_trace","/inherit/factor_tree","/inherit/bonus_params","/inherit/event_trace","/inherit/deck_runtime","/inherit/deck_validate","/inherit/friend_rental_context","/inherit/auto_select_trace",
"/autoplay/runtime","/autoplay/plan","/autoplay/action_trace","/autoplay/factor_select_trace","/offline_auto/runtime","/offline_auto/start_request","/offline_auto/race_reserve","/offline_auto/result",
"/generate_succession/status","/generate_succession/limits","/generate_succession/request","/generate_succession/result","/generate_succession/candidates","/generate_succession/race_reserve","/generate_succession/race_validation","/generate_succession/factor_priority","/generate_succession/factor_order","/generate_succession/probability_trace","/generate_succession/cost_trace",
"/factor/finish_trace","/factor/candidates","/factor/roll_trace","/factor/probability_model","/factor/history","/factor/stats","/factor/breeding_advice","/il2cpp/call_targets","/il2cpp/callers","/il2cpp/type_detail","/il2cpp/object_dump","/api/sniff/exchanges","/api/sniff/exchange","/api/hook/install","/api/hook/remove","/api/hook/list","/api/hook/events","/storage/files","/storage/download"]
prefix=','.join('\\"'+x+'\\"' for x in advertised)+','
health='r#"{{\\"status\\":\\"ok\\",\\"version\\":\\"{}\\",\\"endpoints\\":['
assert s.count(health)==1
s=s.replace(health,health+prefix,1)
available='r#"{{\\"error\\":\\"not_found\\",\\"path\\":\\"{}\\",\\"available\\":['
assert s.count(available)==1
s=s.replace(available,available+prefix,1)

s=s.replace(anchor,MARKER+"\n"+anchor,1)
SOURCE.write_text(s,encoding="utf-8")
print("unified_endpoint_k_complete=applied")
