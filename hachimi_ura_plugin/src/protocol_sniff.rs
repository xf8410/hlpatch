use rmpv::Value;
use serde_json::{json, Map, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Cursor;

const MAX_DECODE_BYTES: usize = 4 * 1024 * 1024;
const MAX_PREFIX_PROBE: usize = 32;
const MAX_SCHEMA_PATHS: usize = 4096;

fn json_key(v: &Value) -> String {
    match v {
        Value::String(s) => s.as_str().unwrap_or("<invalid-utf8>").to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Nil => "null".to_string(),
        other => format!("{other:?}"),
    }
}

fn msgpack_to_json(v: &Value) -> JsonValue {
    match v {
        Value::Nil => JsonValue::Null,
        Value::Boolean(x) => json!(x),
        Value::Integer(x) => x
            .as_i64()
            .map(|n| json!(n))
            .or_else(|| x.as_u64().map(|n| json!(n)))
            .unwrap_or_else(|| json!(x.to_string())),
        Value::F32(x) => json!(x),
        Value::F64(x) => json!(x),
        Value::String(x) => json!(x.as_str().unwrap_or("<invalid-utf8>")),
        Value::Binary(x) => json!({"$type":"binary","length":x.len()}),
        Value::Array(xs) => JsonValue::Array(xs.iter().map(msgpack_to_json).collect()),
        Value::Map(xs) => {
            let mut out = Map::new();
            for (k, value) in xs {
                out.insert(json_key(k), msgpack_to_json(value));
            }
            JsonValue::Object(out)
        }
        Value::Ext(kind, bytes) => json!({"$type":"extension","kind":kind,"length":bytes.len()}),
    }
}

fn decode(data: &[u8]) -> Result<(usize, JsonValue), String> {
    if data.is_empty() {
        return Err("empty payload".to_string());
    }
    if data.len() > MAX_DECODE_BYTES {
        return Err(format!("payload exceeds {} bytes", MAX_DECODE_BYTES));
    }
    let max_offset = MAX_PREFIX_PROBE.min(data.len().saturating_sub(1));
    for offset in 0..=max_offset {
        let remaining = &data[offset..];
        let mut cursor = Cursor::new(remaining);
        if let Ok(value) = rmpv::decode::read_value(&mut cursor) {
            if cursor.position() as usize == remaining.len() {
                return Ok((offset, msgpack_to_json(&value)));
            }
        }
    }
    Err("not one complete MessagePack value (prefix probe 0..32)".to_string())
}

fn schema_type(v: &JsonValue) -> &'static str {
    match v {
        JsonValue::Null => "null",
        JsonValue::Bool(_) => "bool",
        JsonValue::Number(n) if n.is_i64() => "i64",
        JsonValue::Number(n) if n.is_u64() => "u64",
        JsonValue::Number(_) => "float",
        JsonValue::String(_) => "string",
        JsonValue::Array(_) => "array",
        JsonValue::Object(o) if o.get("$type") == Some(&json!("binary")) => "binary",
        JsonValue::Object(o) if o.get("$type") == Some(&json!("extension")) => "extension",
        JsonValue::Object(_) => "object",
    }
}

fn collect_schema(v: &JsonValue, path: &str, out: &mut BTreeSet<String>) {
    if out.len() >= MAX_SCHEMA_PATHS {
        return;
    }
    out.insert(format!("{}:{}", path, schema_type(v)));
    match v {
        JsonValue::Object(map) => {
            for (key, child) in map {
                if key == "$type" || key == "length" || key == "kind" {
                    continue;
                }
                let escaped = key.replace('~', "~0").replace('/', "~1");
                collect_schema(child, &format!("{path}/{escaped}"), out);
            }
        }
        JsonValue::Array(items) => {
            for child in items.iter().take(32) {
                collect_schema(child, &format!("{path}/*"), out);
            }
        }
        _ => {}
    }
}

fn decoded_side(data: Option<&[u8]>) -> JsonValue {
    match data {
        None => json!({"present":false}),
        Some(bytes) => match decode(bytes) {
            Ok((prefix_bytes, value)) => json!({
                "present": true,
                "size": bytes.len(),
                "decoded": true,
                "prefix_bytes": prefix_bytes,
                "value": value
            }),
            Err(error) => json!({
                "present": true,
                "size": bytes.len(),
                "decoded": false,
                "error": error
            }),
        },
    }
}

pub fn render_decoded(
    id: u64,
    requests: &[(u64, String, String, Vec<u8>)],
    responses: &[(u64, Vec<u8>)],
) -> String {
    let req = requests.iter().rev().find(|x| x.0 == id);
    let resp = responses.iter().rev().find(|x| x.0 == id);
    if req.is_none() && resp.is_none() {
        return json!({"error":"not_found","id":id}).to_string();
    }
    json!({
        "id": id,
        "url": req.map(|x| x.1.as_str()).unwrap_or(""),
        "request": decoded_side(req.map(|x| x.3.as_slice())),
        "response": decoded_side(resp.map(|x| x.1.as_slice()))
    })
    .to_string()
}

pub fn render_schema(
    id: u64,
    requests: &[(u64, String, String, Vec<u8>)],
    responses: &[(u64, Vec<u8>)],
) -> String {
    let req = requests.iter().rev().find(|x| x.0 == id);
    let resp = responses.iter().rev().find(|x| x.0 == id);
    let side = |data: Option<&[u8]>| -> JsonValue {
        match data.and_then(|b| decode(b).ok()) {
            Some((prefix, value)) => {
                let mut paths = BTreeSet::new();
                collect_schema(&value, "$", &mut paths);
                json!({"decoded":true,"prefix_bytes":prefix,"paths":paths})
            }
            None => json!({"decoded":false,"paths":[]}),
        }
    };
    json!({
        "id": id,
        "request": side(req.map(|x| x.3.as_slice())),
        "response": side(resp.map(|x| x.1.as_slice()))
    })
    .to_string()
}

pub fn render_routes(requests: &[(u64, String, String, Vec<u8>)]) -> String {
    let mut routes: BTreeMap<String, (usize, usize, u64)> = BTreeMap::new();
    for (id, url, _, body) in requests {
        let no_query = url.split('?').next().unwrap_or(url);
        let path = if let Some(i) = no_query.find("://") {
            let rest = &no_query[i + 3..];
            rest.find('/').map(|j| &rest[j..]).unwrap_or("/")
        } else {
            no_query
        };
        let entry = routes.entry(path.to_string()).or_insert((0, 0, 0));
        entry.0 += 1;
        entry.1 += body.len();
        entry.2 = entry.2.max(*id);
    }
    let values: Vec<JsonValue> = routes
        .into_iter()
        .map(|(path, (count, request_bytes, last_request_id))| {
            json!({"path":path,"count":count,"request_bytes":request_bytes,"last_request_id":last_request_id})
        })
        .collect();
    json!({"count":values.len(),"routes":values}).to_string()
}
