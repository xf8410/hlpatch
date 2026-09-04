#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Screen mirror A-stage (v3.28.0): 画面映射第一阶段——帧采集 + 触点接收。

- hook eglSwapBuffers（dlsym 全局符号，interceptor + trampoline）
- 渲染线程限频 150ms glReadPixels 抓 RGBA → mpsc → 编码线程降采样 1/2
  转 BMP（无压缩、无新依赖）
- GET  /api/frame         → 最新 BMP（X-Frame-Seq / X-Frame-Ts / X-Frame-W / X-Frame-H 头）
- POST /api/touch         → 记录归一化坐标（B 阶段做 IL2CPP 注入，本阶段只收）
- GET  /api/frame_toggle  → 0/1 开关采集（省渲染开销）
BOOT_SAFE：三个端点不触游戏托管内存，boot 期可用。
锚点适配 v3.27.9 原始源与 cumulative 后源（UNITY_SEND_ADDR 两态均唯一）。
"""
from pathlib import Path

SOURCE = Path("hachimi_ura_plugin/src/lib.rs")
s = SOURCE.read_text(encoding="utf-8")
MARKER = "// ===== Screen mirror frame A-stage (v3.28.0) ====="
if MARKER in s:
    print("screen_mirror_frame_a=already_applied")
    raise SystemExit(0)


def replace_once(old: str, new: str, label: str) -> None:
    global s
    count = s.count(old)
    assert count == 1, f"{label} anchor count={count}"
    s = s.replace(old, new, 1)


# ── 1. 核心区块：插在 UNITY_SEND_ADDR 观察器之前（原始源与 cumulative 后源均唯一）──
replace_once(
"""static mut UNITY_SEND_ADDR: usize = 0;
""",
"""// ===== Screen mirror frame A-stage (v3.28.0) =====
// 画面映射：hook eglSwapBuffers 抓帧 → BMP 缓存 → /api/frame。
// 点击坐标 → /api/touch（B 阶段注入）。全部本机回环，与 VPN 无关。
static MIRROR_ENABLED: AtomicBool = AtomicBool::new(true);
static mut MIRROR_LAST_CAPTURE_MS: u64 = 0;
static MIRROR_FRAME: Mutex<Option<Vec<u8>>> = Mutex::new(None); // 完整 BMP 字节
static MIRROR_FRAME_SEQ: AtomicU64 = AtomicU64::new(0);
static MIRROR_FRAME_TS: AtomicU64 = AtomicU64::new(0);
static MIRROR_GAME_W: AtomicU64 = AtomicU64::new(0);
static MIRROR_GAME_H: AtomicU64 = AtomicU64::new(0);
static mut MIRROR_ORIG_SWAP: usize = 0; // eglSwapBuffers trampoline
static mut MIRROR_GL_READPIXELS: usize = 0;
static mut MIRROR_GL_GETINTEGERV: usize = 0;
static mut MIRROR_GL_PIXELSTOREI: usize = 0;
static mut MIRROR_TX: Option<std::sync::mpsc::Sender<(Vec<u8>, usize, usize)>> = None;
static MIRROR_TOUCHES: Mutex<Vec<(u64, f32, f32)>> = Mutex::new(Vec::new());

#[allow(dead_code)]
fn mirror_now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// eglSwapBuffers(dpy, surface) → EGLBoolean(u32)。
/// 渲染线程回调：限频抓帧（读 backbuffer），随后调 trampoline 完成真实交换。
unsafe extern "C" fn mirror_egl_swap_handler(dpy: *mut c_void, surf: *mut c_void) -> u32 {
    if MIRROR_ENABLED.load(Ordering::Relaxed) && MIRROR_ORIG_SWAP != 0 {
        let now = mirror_now_ms();
        let last = MIRROR_LAST_CAPTURE_MS;
        if now.wrapping_sub(last) > 150 {
            MIRROR_LAST_CAPTURE_MS = now;
            mirror_capture();
        }
    }
    if MIRROR_ORIG_SWAP == 0 {
        return 0; // 未拿到 trampoline（防御分支，正常不会走到）
    }
    let orig: unsafe extern "C" fn(*mut c_void, *mut c_void) -> u32 =
        std::mem::transmute(MIRROR_ORIG_SWAP);
    orig(dpy, surf)
}

/// glReadPixels 抓当前帧（RGBA），投递编码线程。
unsafe fn mirror_capture() {
    if MIRROR_GL_READPIXELS == 0 {
        let global = libc::dlopen(ptr::null(), libc::RTLD_NOW);
        if global.is_null() {
            return;
        }
        let names = ["glReadPixels", "glGetIntegerv", "glPixelStorei"];
        let mut addrs: [usize; 3] = [0; 3];
        for (i, name) in names.iter().enumerate() {
            if let Ok(c) = CString::new(*name) {
                addrs[i] = libc::dlsym(global, c.as_ptr()) as usize;
            }
        }
        libc::dlclose(global);
        MIRROR_GL_READPIXELS = addrs[0];
        MIRROR_GL_GETINTEGERV = addrs[1];
        MIRROR_GL_PIXELSTOREI = addrs[2];
        if MIRROR_GL_READPIXELS == 0 {
            return; // 无 GL 符号（Vulkan 后备：A 阶段暂不支持）
        }
    }
    type GetIntegervFn = unsafe extern "C" fn(i32, *mut i32);
    type PixelStoreiFn = unsafe extern "C" fn(i32, i32);
    type ReadPixelsFn = unsafe extern "C" fn(i32, i32, i32, i32, u32, u32, *mut c_void);
    let getint: GetIntegervFn = std::mem::transmute(MIRROR_GL_GETINTEGERV);
    let pixelstore: PixelStoreiFn = std::mem::transmute(MIRROR_GL_PIXELSTOREI);
    let readpx: ReadPixelsFn = std::mem::transmute(MIRROR_GL_READPIXELS);
    const GL_VIEWPORT: i32 = 0x0BA2;
    const GL_PACK_ALIGNMENT: i32 = 0x0D05;
    const GL_RGBA: u32 = 0x1908;
    const GL_UNSIGNED_BYTE: u32 = 0x1401;
    let mut vp = [0i32; 4];
    getint(GL_VIEWPORT, vp.as_mut_ptr());
    let (w, h) = (vp[2] as usize, vp[3] as usize);
    if w == 0 || h == 0 || w > 4096 || h > 4096 {
        return;
    }
    pixelstore(GL_PACK_ALIGNMENT, 1);
    let mut px = vec![0u8; w * h * 4];
    readpx(0, 0, w as i32, h as i32, GL_RGBA, GL_UNSIGNED_BYTE, px.as_mut_ptr() as *mut c_void);
    MIRROR_GAME_W.store(w as u64, Ordering::Relaxed);
    MIRROR_GAME_H.store(h as u64, Ordering::Relaxed);
    if let Some(tx) = &MIRROR_TX {
        let _ = tx.send((px, w, h));
    }
}

/// 编码线程：RGBA → 1/2 降采样 → 无压缩 BMP → 帧缓存。
fn mirror_encode_worker(rx: std::sync::mpsc::Receiver<(Vec<u8>, usize, usize)>) {
    for (px, w, h) in rx {
        let sw = w / 2;
        let sh = h / 2;
        if sw == 0 || sh == 0 {
            continue;
        }
        let row = sw * 3;
        let mut bmp = vec![0u8; 54 + row * sh];
        // BITMAPFILEHEADER
        bmp[0] = b'B';
        bmp[1] = b'M';
        let fsize = (54 + row * sh) as u32;
        bmp[2..6].copy_from_slice(&fsize.to_le_bytes());
        let off = 54u32;
        bmp[10..14].copy_from_slice(&off.to_le_bytes());
        // BITMAPINFOHEADER（40B，BI_RGB 无压缩 24bpp）
        let dib = 40u32;
        bmp[14..18].copy_from_slice(&dib.to_le_bytes());
        bmp[18..22].copy_from_slice(&(sw as i32).to_le_bytes());
        bmp[22..26].copy_from_slice(&(sh as i32).to_le_bytes());
        bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
        bmp[28..30].copy_from_slice(&24u16.to_le_bytes());
        // 像素：BGR + bottom-up + 步长 2 降采样
        for y in 0..sh {
            let sy = (sh - 1 - y) * 2;
            for x in 0..sw {
                let sx = x * 2;
                let si = (sy * w + sx) * 4;
                let di = 54 + y * row + x * 3;
                bmp[di] = px[si + 2];
                bmp[di + 1] = px[si + 1];
                bmp[di + 2] = px[si];
            }
        }
        if let Ok(mut g) = MIRROR_FRAME.lock() {
            *g = Some(bmp);
        }
        MIRROR_FRAME_SEQ.fetch_add(1, Ordering::Relaxed);
        MIRROR_FRAME_TS.store(mirror_now_ms(), Ordering::Relaxed);
    }
}

/// 安装：dlsym eglSwapBuffers → interceptor hook + trampoline；启动编码线程。
unsafe fn install_screen_mirror_hook() {
    let cs = match CString::new("eglSwapBuffers") {
        Ok(v) => v,
        Err(_) => return,
    };
    let global = libc::dlopen(ptr::null(), libc::RTLD_NOW);
    if global.is_null() {
        set_hook_status("mirror.egl", "dlopen_global_failed");
        return;
    }
    let sym = libc::dlsym(global, cs.as_ptr());
    libc::dlclose(global);
    if sym.is_null() {
        set_hook_status("mirror.egl", "symbol_not_found_vulkan_possible");
        return;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    MIRROR_TX = Some(tx);
    std::thread::spawn(move || mirror_encode_worker(rx));
    if interceptor_hook(sym as usize, mirror_egl_swap_handler as usize) {
        MIRROR_ORIG_SWAP = interceptor_get_trampoline(mirror_egl_swap_handler as usize);
        set_hook_status(
            "mirror.egl",
            &format!("hooked@0x{:x} tramp=0x{:x}", sym as usize, MIRROR_ORIG_SWAP),
        );
    } else {
        set_hook_status("mirror.egl", "interceptor_hook_failed");
    }
}

static mut UNITY_SEND_ADDR: usize = 0;
""", "core_block")

# ── 2. BOOT_SAFE 白名单：三端点不触游戏托管内存 ──
replace_once(
'''            "/api/md5log/clear",
''',
'''            "/api/md5log/clear",
            "/api/frame",
            "/api/frame_toggle",
            "/api/touch",
''', "boot_safe")

# ── 3. HTTP 路由 ──
replace_once(
'''    } else if path == "/config" {
''',
'''    } else if path == "/api/frame" {
        // 画面映射帧：二进制 BMP 直写 socket（BMP 含非 UTF-8 字节，不能走统一 String 路径）
        {
            let (seq, ts, gw, gh) = (
                MIRROR_FRAME_SEQ.load(Ordering::Relaxed),
                MIRROR_FRAME_TS.load(Ordering::Relaxed),
                MIRROR_GAME_W.load(Ordering::Relaxed),
                MIRROR_GAME_H.load(Ordering::Relaxed),
            );
            let maybe_bmp = MIRROR_FRAME.lock().ok().and_then(|g| g.clone());
            match maybe_bmp {
                Some(bmp) => {
                    let head = format!(
                        "HTTP/1.1 200 OK\\r\\nContent-Type: image/bmp\\r\\nContent-Length: {}\\r\\nX-Frame-Seq: {}\\r\\nX-Frame-Ts: {}\\r\\nX-Frame-W: {}\\r\\nX-Frame-H: {}\\r\\nConnection: close\\r\\n\\r\\n",
                        bmp.len(), seq, ts, gw, gh
                    );
                    let _ = stream.write_all(head.as_bytes());
                    let _ = stream.write_all(&bmp);
                    return;
                }
                None => {
                    let b = r#"{"ok":false,"error":"no_frame_yet"}"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\\r\\nContent-Type: application/json\\r\\nContent-Length: {}\\r\\nConnection: close\\r\\n\\r\\n{}",
                        b.len(), b
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    return;
                }
            }
        }
    } else if path == "/api/frame_toggle" {
        let flag = parse_query(&full_uri, "enabled");
        let on = flag != "0" && !flag.is_empty();
        MIRROR_ENABLED.store(on, Ordering::Relaxed);
        format!(
            r#"{{"ok":true,"collect":{}}}"#,
            if on { "true" } else { "false" }
        )
    } else if path == "/api/touch" {
        let is_post = req.starts_with("POST");
        if is_post {
            // body: {"x":0.5,"y":0.5}（归一化 0..1）
            let body_start = req.find("\\r\\n\\r\\n").map(|i| i + 4).unwrap_or(req.len());
            let post_body = &req[body_start..];
            let fx = extract_json_f32(post_body, "x");
            let fy = extract_json_f32(post_body, "y");
            match (fx, fy) {
                (Some(x), Some(y)) if (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y) => {
                    let ts = mirror_now_ms();
                    if let Ok(mut g) = MIRROR_TOUCHES.lock() {
                        g.push((ts, x, y));
                        let n = g.len();
                        if n > 256 {
                            g.drain(0..n - 256);
                        }
                    }
                    unsafe { ura_log(3, &format!("mirror touch: {:.3},{:.3}", x, y)); }
                    r#"{"ok":true,"received":true,"phase":"a_logged_only","injection":"B-stage"}"#.to_string()
                }
                _ => r#"{"ok":false,"error":"x_y_0_to_1_required"}"#.to_string(),
            }
        } else {
            // GET: 最近触点观测
            let touches = MIRROR_TOUCHES.lock().map(|g| g.clone()).unwrap_or_default();
            let items: Vec<String> = touches
                .iter()
                .rev()
                .take(50)
                .map(|(ts, x, y)| format!("[{},{:.3},{:.3}]", ts, x, y))
                .collect();
            format!(r#"{{"touches":[{}]}}"#, items.join(","))
        }
    } else if path == "/config" {
''', "http_routes")

# ── 4. 安装点：http 启动前 ──
replace_once(
'''    boot_trace("before_http");
    start_http_server();
''',
'''    boot_trace("before_http");
    unsafe { install_screen_mirror_hook() };
    start_http_server();
''', "install_call")

# ── 5. JSON 数值提取（独立小函数，插在 parse_query 附近不可行——放区块内已含；
#        此处补一个文件级 helper）──
replace_once(
"""fn to_cstr(s: &str) -> CString {
""",
"""/// 粗粒度 JSON 数字提取：{"x":0.5,...} → x=0.5（无 json 依赖）。
fn extract_json_f32(body: &str, key: &str) -> Option<f32> {
    let pat = format!("\\"{}\\":", key);
    let i = body.find(&pat)? + pat.len();
    let rest = &body[i..];
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(rest.len());
    rest[..end].parse::<f32>().ok()
}

fn to_cstr(s: &str) -> CString {
""", "json_helper")

SOURCE.write_text(s, encoding="utf-8")

# 版本 bump：动态读当前版本（兼容 3.27.11 / 3.27.23 前置链）→ 3.28.0
cargo_toml = Path("hachimi_ura_plugin/Cargo.toml")
t = cargo_toml.read_text(encoding="utf-8")
cur = None
for line in t.splitlines():
    if line.strip().startswith("version ="):
        cur = line.strip().split('"')[1]
        break
assert cur is not None and cur.startswith("3.27."), f"unexpected baseline version {cur}"
t = t.replace(f'version = "{cur}"', 'version = "3.28.0"', 1)
cargo_toml.write_text(t, encoding="utf-8")
cargo_lock = Path("hachimi_ura_plugin/Cargo.lock")
lk = cargo_lock.read_text(encoding="utf-8")
lk = lk.replace(
    'name = "hachimi_ura"\nversion = "' + cur + '"',
    'name = "hachimi_ura"\nversion = "3.28.0"',
    1,
)
cargo_lock.write_text(lk, encoding="utf-8")
print(f"screen_mirror_frame_a=applied_v3.28.0_stage_a(from_{cur})")
