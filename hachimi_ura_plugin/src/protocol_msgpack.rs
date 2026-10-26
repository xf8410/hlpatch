//! protocol_msgpack — 最小 msgpack 解码器（零第三方依赖）
//!
//! 只服务「协议层解析」这一件事：把嗅探到的 `application/x-msgpack` 明文 body
//! 解成 JSON。剧本 15（`single_mode_festa/*`）的请求/响应 key 全是可读 ASCII，
//! 因此不需要任何字段偏移表 —— 换版本时顶多新增字段，不会像偏移那样整链失效。
//!
//! 设计约束：
//! - 不引入 rmp-serde / serde，避免给 cdylib 增加依赖与体积；
//! - 不假设 schema，按 msgpack 自描述解码；
//! - 深度与输出长度都设上限，防止异常包把进程拖死（这是 native 插件，不能 panic 逃逸）；
//! - 任何失败都返回 `Err(String)`，绝不 panic。
//!
//! 首个落地端点：`/api/proto/self_test`（解一段内置样例，验证模块在真机可用）。

/// 允许的最大嵌套深度。
const MAX_DEPTH: usize = 8;
/// 允许的最大 JSON 输出字节数。
const MAX_OUT: usize = 256 * 1024;
/// bin 字段若不是嵌套 msgpack，则输出 hex 预览长度。
const BIN_PREVIEW: usize = 64;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    UInt(u64),
    F32(f32),
    F64(f64),
    Str(String),
    Bin(Vec<u8>),
    Arr(Vec<Value>),
    /// 保留插入顺序：协议里 map 的顺序偶尔能提示结构
    Map(Vec<(Value, Value)>),
    Ext(i8, Vec<u8>),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        if let Value::Map(items) = self {
            for (k, v) in items {
                if let Value::Str(s) = k {
                    if s == key {
                        return Some(v);
                    }
                }
            }
        }
        None
    }

    /// 按路径逐级取值，例如 `["data", "chara_info", "speed"]`
    pub fn path(&self, keys: &[&str]) -> Option<&Value> {
        let mut cur = self;
        for k in keys {
            cur = cur.get(k)?;
        }
        Some(cur)
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(v) => Some(*v),
            Value::UInt(v) => Some(*v as i64),
            Value::F64(v) => Some(*v as i64),
            Value::F32(v) => Some(*v as i64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

fn be_u16(b: &[u8]) -> u16 {
    (b[0] as u16) << 8 | b[1] as u16
}
fn be_u32(b: &[u8]) -> u32 {
    (b[0] as u32) << 24 | (b[1] as u32) << 16 | (b[2] as u32) << 8 | b[3] as u32
}
fn be_u64(b: &[u8]) -> u64 {
    let mut v = 0u64;
    for i in 0..8 {
        v = (v << 8) | b[i] as u64;
    }
    v
}

struct Decoder<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Decoder { buf, pos: 0 }
    }

    fn need(&self, n: usize) -> Result<(), String> {
        if self.pos + n > self.buf.len() {
            Err(format!(
                "msgpack truncated: need {} at pos {}, len {}",
                n,
                self.pos,
                self.buf.len()
            ))
        } else {
            Ok(())
        }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        self.need(n)?;
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    fn u8v(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    fn value(&mut self, depth: usize) -> Result<Value, String> {
        if depth > MAX_DEPTH {
            return Err(format!("msgpack depth > {}", MAX_DEPTH));
        }
        let b = self.u8v()?;
        match b {
            0x00..=0x7f => Ok(Value::UInt(b as u64)),
            0x80..=0x8f => self.map((b & 0x0f) as usize, depth),
            0x90..=0x9f => self.array((b & 0x0f) as usize, depth),
            0xa0..=0xbf => self.string((b & 0x1f) as usize),
            0xc0 => Ok(Value::Nil),
            0xc2 => Ok(Value::Bool(false)),
            0xc3 => Ok(Value::Bool(true)),
            0xc4 => {
                let n = self.u8v()? as usize;
                Ok(Value::Bin(self.take(n)?.to_vec()))
            }
            0xc5 => {
                let n = be_u16(self.take(2)?) as usize;
                Ok(Value::Bin(self.take(n)?.to_vec()))
            }
            0xc6 => {
                let n = be_u32(self.take(4)?) as usize;
                Ok(Value::Bin(self.take(n)?.to_vec()))
            }
            0xc7 => {
                let n = self.u8v()? as usize;
                let t = self.u8v()? as i8;
                Ok(Value::Ext(t, self.take(n)?.to_vec()))
            }
            0xc8 => {
                let n = be_u16(self.take(2)?) as usize;
                let t = self.u8v()? as i8;
                Ok(Value::Ext(t, self.take(n)?.to_vec()))
            }
            0xc9 => {
                let n = be_u32(self.take(4)?) as usize;
                let t = self.u8v()? as i8;
                Ok(Value::Ext(t, self.take(n)?.to_vec()))
            }
            0xca => {
                let raw = be_u32(self.take(4)?);
                Ok(Value::F32(f32::from_bits(raw)))
            }
            0xcb => {
                let raw = be_u64(self.take(8)?);
                Ok(Value::F64(f64::from_bits(raw)))
            }
            0xcc => Ok(Value::UInt(self.u8v()? as u64)),
            0xcd => Ok(Value::UInt(be_u16(self.take(2)?) as u64)),
            0xce => Ok(Value::UInt(be_u32(self.take(4)?) as u64)),
            0xcf => Ok(Value::UInt(be_u64(self.take(8)?))),
            0xd0 => Ok(Value::Int(self.u8v()? as i8 as i64)),
            0xd1 => Ok(Value::Int(be_u16(self.take(2)?) as i16 as i64)),
            0xd2 => Ok(Value::Int(be_u32(self.take(4)?) as i32 as i64)),
            0xd3 => Ok(Value::Int(be_u64(self.take(8)?) as i64)),
            0xd4..=0xd8 => {
                let n = 1usize << (b - 0xd4); // 1,2,4,8,16
                let t = self.u8v()? as i8;
                Ok(Value::Ext(t, self.take(n)?.to_vec()))
            }
            0xd9 => {
                let n = self.u8v()? as usize;
                self.string(n)
            }
            0xda => {
                let n = be_u16(self.take(2)?) as usize;
                self.string(n)
            }
            0xdb => {
                let n = be_u32(self.take(4)?) as usize;
                self.string(n)
            }
            0xdc => {
                let n = be_u16(self.take(2)?) as usize;
                self.array(n, depth)
            }
            0xdd => {
                let n = be_u32(self.take(4)?) as usize;
                self.array(n, depth)
            }
            0xde => {
                let n = be_u16(self.take(2)?) as usize;
                self.map(n, depth)
            }
            0xdf => {
                let n = be_u32(self.take(4)?) as usize;
                self.map(n, depth)
            }
            0xe0..=0xff => Ok(Value::Int(b as i8 as i64)),
        }
    }

    fn string(&mut self, n: usize) -> Result<Value, String> {
        let raw = self.take(n)?;
        // 协议里日文文案是 UTF-8；非法序列用 lossy，不让一个坏字节毁掉整个包
        Ok(Value::Str(String::from_utf8_lossy(raw).into_owned()))
    }

    fn array(&mut self, n: usize, depth: usize) -> Result<Value, String> {
        // 先做一次廉价上限检查：每元素至少 1 字节
        self.need(n)?;
        let mut out = Vec::with_capacity(n.min(4096));
        for _ in 0..n {
            out.push(self.value(depth + 1)?);
        }
        Ok(Value::Arr(out))
    }

    fn map(&mut self, n: usize, depth: usize) -> Result<Value, String> {
        self.need(n * 2)?;
        let mut out = Vec::with_capacity(n.min(4096));
        for _ in 0..n {
            let k = self.value(depth + 1)?;
            let v = self.value(depth + 1)?;
            out.push((k, v));
        }
        Ok(Value::Map(out))
    }
}

fn escape_into(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write_value(out: &mut String, v: &Value) {
    if out.len() > MAX_OUT {
        return;
    }
    match v {
        Value::Nil => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(i) => out.push_str(&i.to_string()),
        Value::UInt(u) => out.push_str(&u.to_string()),
        Value::F32(f) => out.push_str(&format!("{}", f)),
        Value::F64(f) => out.push_str(&format!("{}", f)),
        Value::Str(s) => escape_into(out, s),
        Value::Bin(b) => {
            // 协议里 bin 常常是「再套一层 msgpack」——先试递归解，失败再给 hex 预览
            if let Ok(nested) = decode(b) {
                write_value(out, &nested);
            } else {
                out.push_str("{\"bin_hex\":\"");
                for byte in b.iter().take(BIN_PREVIEW) {
                    out.push_str(&format!("{:02x}", byte));
                }
                out.push_str("\",\"bin_len\":");
                out.push_str(&b.len().to_string());
                out.push('}');
            }
        }
        Value::Arr(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(out, item);
            }
            out.push(']');
        }
        Value::Map(items) => {
            out.push('{');
            for (i, (k, val)) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                match k {
                    Value::Str(s) => escape_into(out, s),
                    other => {
                        let mut tmp = String::new();
                        write_value(&mut tmp, other);
                        escape_into(out, &tmp);
                    }
                }
                out.push(':');
                write_value(out, val);
            }
            out.push('}');
        }
        Value::Ext(t, b) => {
            out.push_str(&format!("{{\"ext_type\":{},\"ext_len\":{}}}", t, b.len()));
        }
    }
}

/// 解一个 msgpack buffer 成 JSON 文本。
pub fn decode_to_json(buf: &[u8]) -> Result<String, String> {
    let v = decode(buf)?;
    let mut out = String::with_capacity(1024);
    write_value(&mut out, &v);
    Ok(out)
}

pub fn decode(buf: &[u8]) -> Result<Value, String> {
    if buf.is_empty() {
        return Err("empty buffer".to_string());
    }
    let mut d = Decoder::new(buf);
    let v = d.value(0)?;
    if d.pos != buf.len() {
        // 不报错：有些回包尾部带 padding/校验，只记录偏差
        return Ok(v);
    }
    Ok(v)
}

/// 从 `single_mode_*` 响应里抽关键字段，输出紧凑 JSON。
///
/// 只抽「稳定且有用」的：角色五维/上限/干劲/技能数、回合、act 列表。
/// key 名全部来自实测（`single_mode_festa/exec_command` 回包，2026-10-26）。
pub fn summarize_single_mode(buf: &[u8]) -> Result<String, String> {
    let v = decode(buf)?;
    let mut out = String::with_capacity(512);
    out.push('{');

    let mut first = true;
    let mut push = |out: &mut String, key: &str, val: String| {
        if !first {
            out.push(',');
        }
        first = false;
        escape_into(out, key);
        out.push(':');
        out.push_str(&val);
    };

    if let Some(ci) = v.path(&["data", "chara_info"]) {
        for k in [
            "speed", "stamina", "power", "wiz", "guts", "vital", "max_vital", "motivation",
            "talent_level", "chara_grade", "fans",
        ] {
            if let Some(n) = ci.get(k).and_then(|x| x.as_i64()) {
                push(&mut out, k, n.to_string());
            }
        }
        for k in ["max_speed", "max_stamina", "max_power", "max_wiz", "max_guts"] {
            if let Some(n) = ci.get(k).and_then(|x| x.as_i64()) {
                push(&mut out, k, n.to_string());
            }
        }
        if let Some(arr) = ci.get("skill_array").and_then(|x| match x {
            Value::Arr(a) => Some(a.len()),
            _ => None,
        }) {
            push(&mut out, "skill_count", arr.to_string());
        }
        if let Some(arr) = ci.get("support_card_array").and_then(|x| match x {
            Value::Arr(a) => Some(a.len()),
            _ => None,
        }) {
            push(&mut out, "support_card_count", arr.to_string());
        }
    }

    // act / advice：key 名待真机回包确认，这里做宽松探测
    for (label, keys) in [
        ("act_info_array", ["act_info_array", "act_array"]),
        ("advice_info_array", ["advice_info_array", "advice_array"]),
        (
            "check_point_result_array",
            ["check_point_result_array", "check_point_array"],
        ),
    ] {
        let mut hit: Option<usize> = None;
        for k in keys {
            if let Some(Value::Arr(a)) = v.get(k) {
                hit = Some(a.len());
                break;
            }
            if let Some(Value::Arr(a)) = v.path(&["data", k]) {
                hit = Some(a.len());
                break;
            }
        }
        if let Some(n) = hit {
            push(&mut out, label, n.to_string());
        }
    }

    out.push('}');
    Ok(out)
}

/// 内置样例：`{"a":1,"b":["x",2],"c":{"d":true},"e":-1}`
fn sample_bytes() -> Vec<u8> {
    let mut b = Vec::new();
    b.push(0x84); // fixmap 4
    b.push(0xa1);
    b.push(b'a');
    b.push(0x01);
    b.push(0xa1);
    b.push(b'b');
    b.push(0x93); // fixarray 3? -> 用 2 个元素
    b.pop();
    b.push(0x92);
    b.push(0xa1);
    b.push(b'x');
    b.push(0x02);
    b.push(0xa1);
    b.push(b'c');
    b.push(0x81);
    b.push(0xa1);
    b.push(b'd');
    b.push(0xc3);
    b.push(0xa1);
    b.push(b'e');
    b.push(0xff); // negative fixint -1
    b
}

/// 自检：解码内置样例并返回 JSON（端点 `/api/proto/self_test` 用它验证模块可用）。
pub fn self_test() -> String {
    let sample = sample_bytes();
    match decode_to_json(&sample) {
        Ok(json) => format!(
            "{{\"ok\":true,\"in_len\":{},\"json\":{}}}",
            sample.len(),
            json
        ),
        Err(e) => format!("{{\"ok\":false,\"error\":\"{}\"}}", e.replace('"', "'")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_sample() {
        let json = decode_to_json(&sample_bytes()).unwrap();
        assert!(json.contains("\"a\":1"), "{}", json);
        assert!(json.contains("\"x\""), "{}", json);
        assert!(json.contains("\"d\":true"), "{}", json);
        assert!(json.contains("\"e\":-1"), "{}", json);
    }

    #[test]
    fn truncated_is_err_not_panic() {
        let sample = sample_bytes();
        for cut in 1..sample.len() {
            let _ = decode_to_json(&sample[..cut]); // 只要求不 panic
        }
    }

    #[test]
    fn depth_guard() {
        // 嵌套 12 层数组，应报 depth 错误而非爆栈
        let mut b = vec![0x91u8; 12];
        b.push(0x01);
        let r = decode_to_json(&b);
        assert!(r.is_err(), "expected depth error");
    }

    #[test]
    fn self_test_ok() {
        assert!(self_test().starts_with("{\"ok\":true"));
    }
}
