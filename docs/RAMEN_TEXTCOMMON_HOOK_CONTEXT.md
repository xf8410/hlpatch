# Ramen/Hachimi TextCommon exact hook context

## Generated Rust source anchors

### `fn read_il2cpp_string`

matches=2

#### match 1 bytes 12459..18459

```rust
sg.to_string());
    }
}

// ★ v3.24.40: per-hook install status for /debug/hookdiag
static HOOK_STATUS: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
fn set_hook_status(name: &str, status: &str) {
    hook_log(&format!("hook[{}] = {}", name, status));
    if let Ok(mut g) = HOOK_STATUS.lock() {
        if let Some(e) = g.iter_mut().find(|(n, _)| n == name) {
            e.1 = status.to_string();
        } else {
            g.push((name.to_string(), status.to_string()));
        }
    }
}
static mut EVENT_CHOICES: Vec<EventChoice> = Vec::new();
static mut EVENT_SELECTED_IDX: i32 = -1;
static mut EVENT_STORY_ID: i32 = 0;
static mut EVENT_CHARA_ID: i32 = 0;

// Incremented whenever a new story_id takes over (or state is cleared).
// Guarded by EVENT_STATE_MUTEX; never read/write outside the lock.
static mut EVENT_GENERATION: u64 = 0;

// Cap against runaway AddChoiceButton repeats in abnormal UI rebuilds.
const EVENT_CHOICES_MAX: usize = 32;

#[derive(Clone)]
struct EventChoice {
    label: String,
    gain_id: i32,
    next_block_idx: i32,
    loop_exit_gain_id: i32,
}

// v3.24.73: bounded cache-only pairing. This is temporal co-occurrence,
// never a success/failure classification or a causality claim.
#[derive(Clone)]
struct PendingEventSelection {
    captured_at: u64,
    generation: u64,
    story_id: i32,
    chara_id: i32,
    selected_idx_raw: i32,
    choice: Option<EventChoice>,
}
static EVENT_PENDING_RESULT: Mutex<Option<PendingEventSelection>> = Mutex::new(None);
static EVENT_OBSERVATIONS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static EVENT_OBSERVATION_ID: AtomicU64 = AtomicU64::new(1);
const EVENT_OBSERVATIONS_MAX: usize = 16;
const EVENT_RESPONSE_PREVIEW_MAX: usize = 16 * 1024;

// ★ v3.24.2: Read C# string from IL2CPP String object
unsafe fn read_il2cpp_string(s: *const c_void) -> String {
    if s.is_null() {
        return String::new();
    }
    let len = std::ptr::read::<i32>((s as *const u8).offset(16) as *const i32);
    if len <= 0 || len > 4096 {
        return String::new();
    }
    let chars_ptr = (s as *const u8).offset(20);
    let chars_slice = std::slice::from_raw_parts(chars_ptr as *const u16, len as usize);
    String::from_utf16_lossy(chars_slice)
}

// ★ Push-to-app state (v3.10.0): auto-push /summary to uma-juece when data changes
static mut LAST_PUSH_HASH: u64 = 0;
static PUSH_INTERVAL_SECS: u64 = 1;

// ★ Config (v3.11.0): runtime config updated via POST /config from App
// No file editing needed — App settings page sends config to plugin HTTP endpoint
#[derive(Clone)]
struct PluginConfig {
    push_host: String,       // default: "127.0.0.1"
    push_port: u16,          // default: 18766
    http_port: u16,          // default: 18765
    push_interval_secs: u64, // default: 1
    push_enabled: bool,      // default: true
    http_enabled: bool,      // default: true
}

impl PluginConfig {
    fn defaults() -> Self {
        Self {
            push_host: "127.0.0.1".to_string(),
            push_port: 18766,
            http_port: 18765,
            push_interval_secs: 5,
            push_enabled: true,
            http_enabled: true,
        }
    }

    fn push_addr(&self) -> String {
        format!("{}:{}", self.push_host, self.push_port)
    }

    // Parse JSON config from POST /config body (simple manual parse, no serde)
    fn from_json(data: &str) -> Option<Self> {
        let mut cfg = Self::defaults();
        let mut changed = false;
        // Extract key-value pairs from JSON
        for line in data.lines() {
            let l = line.trim().trim_end_matches(',');
            if l.is_empty() || l == "{" || l == "}" {
                continue;
            }
            if let Some((k, v)) = l.split_once(':') {
                let k = k.trim().trim_matches('"');
                let v = v.trim().trim_matches('"');
                match k {
                    "push_host" => {
                        cfg.push_host = v.to_string();
                        changed = true;
                    }
                    "push_port" => {
                        if let Ok(n) = v.parse::<u16>() {
                            cfg.push_port = n;
                            changed = true;
                        }
                    }
                    "http_port" => {
                        if let Ok(n) = v.parse::<u16>() {
                            cfg.http_port = n;
                            changed = true;
                        }
                    }
                    "push_interval_secs" => {
                        if let Ok(n) = v.parse::<u64>() {
                            cfg.push_interval_secs = n.max(1);
                            changed = true;
                        }
                    }
                    "push_enabled" => {
                        cfg.push_enabled = v == "true";
                        changed = true;
                    }
                    "http_enabled" => {
                        cfg.http_enabled = v == "true";
                        changed = true;
                    }
                    _ => {}
                }
            }
        }
        if changed {
            Some(cfg)
        } else {
            None
        }
    }

    fn to_json(&self) -> String {
        format!(
            r#"{{"push_host":"{}","push_port":{},"http_port":{},"push_interval_secs":{},"push_enabled":{},"http_enabled":{}}}"#,
            self.push_host,
            self.push_port,
            self.http_port,
            self.push_interval_secs,
            self.push_enabled,
            self.http_enabled
        )
    }
}

static mut PLUGIN_CONFIG: Option<PluginConfig> = None;

// ★ Text edit buffers for GUI config (v3.12.0): persist across frames for egui immediate mode
static mut GUI_HOST_BUF: [u8; 64] = [0u8; 64]; // push_host input buffer
static mut GUI_HOST_BUF_LEN: i32 = 0;
static mut GUI_PORT_BUF: [u8; 8] = [0u8; 8]; // push_port input buffer
static mut GUI_PORT_BUF_LEN: i32 = 0;

unsafe fn 
```

#### match 2 bytes 506568..512568

```rust
             label,
                gain_id,
                next_block_idx: normalized_next,
                loop_exit_gain_id: normalized_loop_exit,
            });
        }

        drop(_lock);

        if !EVENT_CHOICE_HOOK_INSTALLED || EVENT_ADD_BTN_ADDR == 0 {
            return;
        }

        // ★ v3.24.9: Use trampoline — no unhook/rehook
        let trampoline = interceptor_get_trampoline(event_add_choice_hook_handler as usize);
        if trampoline == 0 {
            ura_log(1, "add_choice_hook: trampoline not found");
            return;
        }
        type FnAddBtn = unsafe extern "C" fn(*mut c_void, *mut c_void);
        let original: FnAddBtn = std::mem::transmute(trampoline);
        original(this, param);
    }
}

// Helper: call getter on an IL2CPP object (returns i32)
unsafe fn call_getter_int_raw(obj: *const c_void, method_name: &str) -> i32 {
    if obj.is_null() || API.is_null() {
        return 0;
    }
    // We need to find the class and method. For simplicity, we use the method pointer approach.
    // Since we don't know the class, we try to read directly from known offsets.
    // StoryChoiceParam is a simple struct, we can try field offsets.
    // Actually, let's use the proper IL2CPP approach.
    let api = &*API;
    let get_class_fn: Option<unsafe extern "C" fn(*const c_void) -> *mut c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_object_get_class");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    if get_class_fn.is_none() {
        return 0;
    }
    let class = get_class_fn.unwrap()(obj);
    if class.is_null() {
        return 0;
    }
    call_getter_int(class, obj, method_name)
}

// Helper: read IL2CPP string from object via getter
unsafe fn read_il2cpp_string_from_obj(obj: *const c_void, method_name: &str) -> String {
    if obj.is_null() || API.is_null() {
        return String::new();
    }
    let api = &*API;
    let get_class_fn: Option<unsafe extern "C" fn(*const c_void) -> *mut c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_object_get_class");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    if get_class_fn.is_none() {
        return String::new();
    }
    let class = get_class_fn.unwrap()(obj);
    if class.is_null() {
        return String::new();
    }
    let s = call_getter_string(class, obj, method_name);
    let result = read_il2cpp_string(s);
    result
}

// Helper: call a getter that returns a string (IL2CPP String*)
unsafe fn call_getter_string(
    class: *mut c_void,
    obj: *const c_void,
    method_name: &str,
) -> *const c_void {
    if class.is_null() || obj.is_null() || API.is_null() {
        return std::ptr::null();
    }
    let get_method_fn: Option<
        unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *const c_void,
    > = {
        let p = resolve_il2cpp_symbol("il2cpp_class_get_method_from_name");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    if get_method_fn.is_none() {
        return std::ptr::null();
    }
    let method = get_method_fn.unwrap()(class, to_cstr(method_name).as_ptr(), 0);
    if method.is_null() {
        return std::ptr::null();
    }

    let get_ptr_fn: Option<unsafe extern "C" fn(*const c_void) -> *const c_void> = {
        let p = resolve_il2cpp_symbol("il2cpp_method_get_pointer");
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute(p))
        }
    };
    if get_ptr_fn.is_none() {
        return std::ptr::null();
    }
    let ptr = get_ptr_fn.unwrap()(method);
    if ptr.is_null() {
        return std::ptr::null();
    }

    type FnGet = unsafe extern "C" fn(*const c_void) -> *const c_void;
    let getter: FnGet = std::mem::transmute(ptr);
    getter(obj)
}

// ★ v3.24.2: StoryManager.SetStory hook — capture story_id for event type identification
// StoryManager.SetStory(this, story_id, ???, ???, ???)
// ARM64: X0=this, X1=story_id, X2-X4=other params
// ★ v3.24.2 FIX: Don't call getters or ura_log in hook context — these run on the game's
// main thread without SIGSEGV recovery. If the IL2CPP object is in a transitional state,
// calling getters can crash the game process. We only store story_id (passed as parameter).
// chara_id is read from summary data via get_CardId instead.
extern "C" fn story_set_hook_handler(this: *mut c_void, story_id: i32, p2: i64, p3: i64, p4: i64) {
    unsafe {
        if !this.is_null() {
            let _lock = EVENT_STATE_MUTEX.lock();

            if EVENT_STORY_ID != story_id {
                // New event batch: drop stale choices from the previous one.
                // Same story_id re-entry does NOT clear.
                EVENT_CHOICES.clear();
                EVENT_SELECTED_IDX = -1;
                EVENT_CHARA_ID = 0;
                EVENT_GENERATION = EVENT_GENERATION.wrapping_add(1);
            }

            EVENT_STORY_ID = story_id;
            drop(_lock);
            ura_log(3, &format!("story_set: story_id={}", story_id));
        }

        if !STORY_SET_HOOK_INSTALLED || STORY_SET_ADDR == 0 {
            return;
        }

        // ★ v3.24.9: Use trampoline — no unhook/rehook
        let trampoline = interceptor_get_trampoline(story_set_hook_handler as usize);
        if trampoline == 0 {
            ura_log(1, "story_set_hook: trampoline not found");
            return;
        }
        type FnSetStory = unsafe extern "C" fn(*mut c_void, i32, i64, i64, i64);
        let original: FnSetStory = std::mem::transmute(trampoline);
        original(this, story_id, p2, p3, p4);
    }
}

unsafe fn install_event_choice_hook() {
    if EVENT_CHOICE_HOOK_INSTALLED {
        return;
    }
    if API.is_null() {
        return;
    }

    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return,
    };

    le
```

### `fn il2cpp_string`

matches=0

### `String::from_utf16`

matches=1

#### match 1 bytes 12858..18858

```rust
string();
        } else {
            g.push((name.to_string(), status.to_string()));
        }
    }
}
static mut EVENT_CHOICES: Vec<EventChoice> = Vec::new();
static mut EVENT_SELECTED_IDX: i32 = -1;
static mut EVENT_STORY_ID: i32 = 0;
static mut EVENT_CHARA_ID: i32 = 0;

// Incremented whenever a new story_id takes over (or state is cleared).
// Guarded by EVENT_STATE_MUTEX; never read/write outside the lock.
static mut EVENT_GENERATION: u64 = 0;

// Cap against runaway AddChoiceButton repeats in abnormal UI rebuilds.
const EVENT_CHOICES_MAX: usize = 32;

#[derive(Clone)]
struct EventChoice {
    label: String,
    gain_id: i32,
    next_block_idx: i32,
    loop_exit_gain_id: i32,
}

// v3.24.73: bounded cache-only pairing. This is temporal co-occurrence,
// never a success/failure classification or a causality claim.
#[derive(Clone)]
struct PendingEventSelection {
    captured_at: u64,
    generation: u64,
    story_id: i32,
    chara_id: i32,
    selected_idx_raw: i32,
    choice: Option<EventChoice>,
}
static EVENT_PENDING_RESULT: Mutex<Option<PendingEventSelection>> = Mutex::new(None);
static EVENT_OBSERVATIONS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static EVENT_OBSERVATION_ID: AtomicU64 = AtomicU64::new(1);
const EVENT_OBSERVATIONS_MAX: usize = 16;
const EVENT_RESPONSE_PREVIEW_MAX: usize = 16 * 1024;

// ★ v3.24.2: Read C# string from IL2CPP String object
unsafe fn read_il2cpp_string(s: *const c_void) -> String {
    if s.is_null() {
        return String::new();
    }
    let len = std::ptr::read::<i32>((s as *const u8).offset(16) as *const i32);
    if len <= 0 || len > 4096 {
        return String::new();
    }
    let chars_ptr = (s as *const u8).offset(20);
    let chars_slice = std::slice::from_raw_parts(chars_ptr as *const u16, len as usize);
    String::from_utf16_lossy(chars_slice)
}

// ★ Push-to-app state (v3.10.0): auto-push /summary to uma-juece when data changes
static mut LAST_PUSH_HASH: u64 = 0;
static PUSH_INTERVAL_SECS: u64 = 1;

// ★ Config (v3.11.0): runtime config updated via POST /config from App
// No file editing needed — App settings page sends config to plugin HTTP endpoint
#[derive(Clone)]
struct PluginConfig {
    push_host: String,       // default: "127.0.0.1"
    push_port: u16,          // default: 18766
    http_port: u16,          // default: 18765
    push_interval_secs: u64, // default: 1
    push_enabled: bool,      // default: true
    http_enabled: bool,      // default: true
}

impl PluginConfig {
    fn defaults() -> Self {
        Self {
            push_host: "127.0.0.1".to_string(),
            push_port: 18766,
            http_port: 18765,
            push_interval_secs: 5,
            push_enabled: true,
            http_enabled: true,
        }
    }

    fn push_addr(&self) -> String {
        format!("{}:{}", self.push_host, self.push_port)
    }

    // Parse JSON config from POST /config body (simple manual parse, no serde)
    fn from_json(data: &str) -> Option<Self> {
        let mut cfg = Self::defaults();
        let mut changed = false;
        // Extract key-value pairs from JSON
        for line in data.lines() {
            let l = line.trim().trim_end_matches(',');
            if l.is_empty() || l == "{" || l == "}" {
                continue;
            }
            if let Some((k, v)) = l.split_once(':') {
                let k = k.trim().trim_matches('"');
                let v = v.trim().trim_matches('"');
                match k {
                    "push_host" => {
                        cfg.push_host = v.to_string();
                        changed = true;
                    }
                    "push_port" => {
                        if let Ok(n) = v.parse::<u16>() {
                            cfg.push_port = n;
                            changed = true;
                        }
                    }
                    "http_port" => {
                        if let Ok(n) = v.parse::<u16>() {
                            cfg.http_port = n;
                            changed = true;
                        }
                    }
                    "push_interval_secs" => {
                        if let Ok(n) = v.parse::<u64>() {
                            cfg.push_interval_secs = n.max(1);
                            changed = true;
                        }
                    }
                    "push_enabled" => {
                        cfg.push_enabled = v == "true";
                        changed = true;
                    }
                    "http_enabled" => {
                        cfg.http_enabled = v == "true";
                        changed = true;
                    }
                    _ => {}
                }
            }
        }
        if changed {
            Some(cfg)
        } else {
            None
        }
    }

    fn to_json(&self) -> String {
        format!(
            r#"{{"push_host":"{}","push_port":{},"http_port":{},"push_interval_secs":{},"push_enabled":{},"http_enabled":{}}}"#,
            self.push_host,
            self.push_port,
            self.http_port,
            self.push_interval_secs,
            self.push_enabled,
            self.http_enabled
        )
    }
}

static mut PLUGIN_CONFIG: Option<PluginConfig> = None;

// ★ Text edit buffers for GUI config (v3.12.0): persist across frames for egui immediate mode
static mut GUI_HOST_BUF: [u8; 64] = [0u8; 64]; // push_host input buffer
static mut GUI_HOST_BUF_LEN: i32 = 0;
static mut GUI_PORT_BUF: [u8; 8] = [0u8; 8]; // push_port input buffer
static mut GUI_PORT_BUF_LEN: i32 = 0;

unsafe fn get_config() -> &'static PluginConfig {
    if PLUGIN_CONFIG.is_none() {
        PLUGIN_CONFIG = Some(PluginConfig::defaults());
    }
    PLUGIN_CONFIG.as_ref().unwrap()
}

unsafe fn update_config(new_cfg: PluginConfig) {
    PLUGIN_CONFIG = Some(new_cfg);
}

// ★ Training log (v3.7.9): auto-record snapshots from /data and /scenario
const MAX_LOG_ENTRIES: usize = 30;
static mut TRAINING_LOG: Vec
```

### `fn get_method_addr`

matches=0

### `fn interceptor_hook`

matches=1

#### match 1 bytes 441043..447043

```rust
addr;

    // ★ v3.24.9: Use interceptor API instead of write_hook_bytes
    if interceptor_hook(method_addr, training_hook_handler as usize) {
        TRAINING_HOOK_INSTALLED = true;
        ura_log(
            3,
            &format!(
                "Training hook installed at 0x{:x} (interceptor)",
                method_addr
            ),
        );
    } else {
        ura_log(
            1,
            "Training hook: interceptor_hook failed, falling back to write_hook_bytes",
        );
        // Fallback: old write_hook_bytes method (less safe but works without interceptor)
        std::ptr::copy_nonoverlapping(
            method_addr as *const u8,
            ORIG_ON_SUCCESS_PROLOGUE.as_mut_ptr(),
            16,
        );
        write_hook_bytes(method_addr, training_hook_handler as usize);
        TRAINING_HOOK_INSTALLED = true;
    }
}

// ★ v3.23.3: API sniffing — read IL2CPP byte array
// IL2CPP array layout: klass(8) + monitor(8) + bounds(8) + max_length(8) + data
unsafe fn read_il2cpp_byte_array(arr: *const c_void) -> Vec<u8> {
    if arr.is_null() {
        return vec![];
    }
    let len = std::ptr::read::<u64>((arr as *const u8).offset(24) as *const u64) as usize;
    if len == 0 || len > 2 * 1024 * 1024 {
        return vec![];
    }
    let cap = len.min(65536);
    let data_ptr = (arr as *const u8).offset(32);
    std::slice::from_raw_parts(data_ptr, cap).to_vec()
}

fn sniff_timestamp() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn hex_encode(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() * 2);
    for b in data {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

// ★ v3.23.3: Interceptor helpers — use Hachimi-Edge V3 interceptor API
unsafe fn interceptor_hook(orig_addr: usize, hook_addr: usize) -> bool {
    if API.is_null() || orig_addr == 0 || hook_addr == 0 {
        return false;
    }
    let api = &*API;
    if api.interceptor == 0 {
        return false;
    }
    if let Some(f) = api.interceptor_hook_fn {
        !f(
            api.interceptor,
            orig_addr as *mut c_void,
            hook_addr as *mut c_void,
        )
        .is_null()
    } else {
        false
    }
}

unsafe fn interceptor_get_trampoline(hook_addr: usize) -> usize {
    if API.is_null() || hook_addr == 0 {
        return 0;
    }
    let api = &*API;
    if api.interceptor == 0 {
        return 0;
    }
    if let Some(f) = api.interceptor_get_trampoline_addr_fn {
        f(api.interceptor, hook_addr as *mut c_void) as usize
    } else {
        0
    }
}

/// ★ v3.24.9: Unified hook installer — tries interceptor first, falls back to write_hook_bytes
unsafe fn install_hook_safe(
    name: &str,
    method_addr: usize,
    handler_addr: usize,
    orig_prologue: &mut [u8; 16],
) -> bool {
    if method_addr == 0 {
        return false;
    }
    if interceptor_hook(method_addr, handler_addr) {
        ura_log(
            3,
            &format!("{}: hooked at 0x{:x} (interceptor)", name, method_addr),
        );
        true
    } else {
        ura_log(
            2,
            &format!("{}: interceptor failed, fallback to write_hook_bytes", name),
        );
        std::ptr::copy_nonoverlapping(method_addr as *const u8, orig_prologue.as_mut_ptr(), 16);
        write_hook_bytes(method_addr, handler_addr);
        true
    }
}

fn unity_observer_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn unity_observer_path(url: &str) -> String {
    let no_query = url.split('?').next().unwrap_or(url);
    if let Some(scheme) = no_query.find("://") {
        let rest = &no_query[scheme + 3..];
        return rest
            .find('/')
            .map(|i| rest[i..].to_string())
            .unwrap_or_else(|| "/".to_string());
    }
    no_query.to_string()
}

unsafe fn unity_get_string(obj: *const c_void, getter: &str) -> String {
    if obj.is_null() {
        return String::new();
    }
    let class = get_class_from_object(obj);
    if class.is_null() {
        return String::new();
    }
    read_il2cpp_string(call_getter_on_instance(class, obj, getter))
}

unsafe fn observe_unity_web_request(request: *mut c_void) {
    if request.is_null() || !SNIFF_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    let method = unity_get_string(request, "get_method");
    let url = unity_get_string(request, "get_url");
    let request_class = get_class_from_object(request);
    let upload = if request_class.is_null() {
        ptr::null_mut()
    } else {
        call_getter_ref(request_class, request, "get_uploadHandler")
    };
    let (body_size, body_hex, content_type) = if upload.is_null() {
        (0, String::new(), String::new())
    } else {
        let upload_class = get_class_from_object(upload);
        let data = if upload_class.is_null() {
            ptr::null_mut()
        } else {
            call_getter_on_instance(upload_class, upload, "get_data")
        };
        let body_bytes = read_il2cpp_byte_array(data);
        let body_hex = body_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        (
            body_bytes.len(),
            body_hex,
            unity_get_string(upload, "get_contentType"),
        )
    };
    let item = UnityRequestObservation {
        id: UNITY_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed),
        timestamp_ms: unity_observer_timestamp_ms(),
        method,
        path: unity_observer_path(&url),
        body_size,
        body_hex,
        content_type,
    };
    if let Ok(mut entries) = UNITY_OBSERVATIONS.lock() {
        if entries.len() >= UNITY_OBSERVATIONS_MAX {
            entries.remove(0);
        }
        entries.push(item);
    }
}

// UnityWebRequest.SendWebRequest() is asynchronous; this observes request entry only.
extern "C" fn unity_send_hook_
```

### `fn interceptor_get_trampoline`

matches=1

#### match 1 bytes 441511..447511

```rust
k to write_hook_bytes",
        );
        // Fallback: old write_hook_bytes method (less safe but works without interceptor)
        std::ptr::copy_nonoverlapping(
            method_addr as *const u8,
            ORIG_ON_SUCCESS_PROLOGUE.as_mut_ptr(),
            16,
        );
        write_hook_bytes(method_addr, training_hook_handler as usize);
        TRAINING_HOOK_INSTALLED = true;
    }
}

// ★ v3.23.3: API sniffing — read IL2CPP byte array
// IL2CPP array layout: klass(8) + monitor(8) + bounds(8) + max_length(8) + data
unsafe fn read_il2cpp_byte_array(arr: *const c_void) -> Vec<u8> {
    if arr.is_null() {
        return vec![];
    }
    let len = std::ptr::read::<u64>((arr as *const u8).offset(24) as *const u64) as usize;
    if len == 0 || len > 2 * 1024 * 1024 {
        return vec![];
    }
    let cap = len.min(65536);
    let data_ptr = (arr as *const u8).offset(32);
    std::slice::from_raw_parts(data_ptr, cap).to_vec()
}

fn sniff_timestamp() -> u64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn hex_encode(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() * 2);
    for b in data {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

// ★ v3.23.3: Interceptor helpers — use Hachimi-Edge V3 interceptor API
unsafe fn interceptor_hook(orig_addr: usize, hook_addr: usize) -> bool {
    if API.is_null() || orig_addr == 0 || hook_addr == 0 {
        return false;
    }
    let api = &*API;
    if api.interceptor == 0 {
        return false;
    }
    if let Some(f) = api.interceptor_hook_fn {
        !f(
            api.interceptor,
            orig_addr as *mut c_void,
            hook_addr as *mut c_void,
        )
        .is_null()
    } else {
        false
    }
}

unsafe fn interceptor_get_trampoline(hook_addr: usize) -> usize {
    if API.is_null() || hook_addr == 0 {
        return 0;
    }
    let api = &*API;
    if api.interceptor == 0 {
        return 0;
    }
    if let Some(f) = api.interceptor_get_trampoline_addr_fn {
        f(api.interceptor, hook_addr as *mut c_void) as usize
    } else {
        0
    }
}

/// ★ v3.24.9: Unified hook installer — tries interceptor first, falls back to write_hook_bytes
unsafe fn install_hook_safe(
    name: &str,
    method_addr: usize,
    handler_addr: usize,
    orig_prologue: &mut [u8; 16],
) -> bool {
    if method_addr == 0 {
        return false;
    }
    if interceptor_hook(method_addr, handler_addr) {
        ura_log(
            3,
            &format!("{}: hooked at 0x{:x} (interceptor)", name, method_addr),
        );
        true
    } else {
        ura_log(
            2,
            &format!("{}: interceptor failed, fallback to write_hook_bytes", name),
        );
        std::ptr::copy_nonoverlapping(method_addr as *const u8, orig_prologue.as_mut_ptr(), 16);
        write_hook_bytes(method_addr, handler_addr);
        true
    }
}

fn unity_observer_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn unity_observer_path(url: &str) -> String {
    let no_query = url.split('?').next().unwrap_or(url);
    if let Some(scheme) = no_query.find("://") {
        let rest = &no_query[scheme + 3..];
        return rest
            .find('/')
            .map(|i| rest[i..].to_string())
            .unwrap_or_else(|| "/".to_string());
    }
    no_query.to_string()
}

unsafe fn unity_get_string(obj: *const c_void, getter: &str) -> String {
    if obj.is_null() {
        return String::new();
    }
    let class = get_class_from_object(obj);
    if class.is_null() {
        return String::new();
    }
    read_il2cpp_string(call_getter_on_instance(class, obj, getter))
}

unsafe fn observe_unity_web_request(request: *mut c_void) {
    if request.is_null() || !SNIFF_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    let method = unity_get_string(request, "get_method");
    let url = unity_get_string(request, "get_url");
    let request_class = get_class_from_object(request);
    let upload = if request_class.is_null() {
        ptr::null_mut()
    } else {
        call_getter_ref(request_class, request, "get_uploadHandler")
    };
    let (body_size, body_hex, content_type) = if upload.is_null() {
        (0, String::new(), String::new())
    } else {
        let upload_class = get_class_from_object(upload);
        let data = if upload_class.is_null() {
            ptr::null_mut()
        } else {
            call_getter_on_instance(upload_class, upload, "get_data")
        };
        let body_bytes = read_il2cpp_byte_array(data);
        let body_hex = body_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        (
            body_bytes.len(),
            body_hex,
            unity_get_string(upload, "get_contentType"),
        )
    };
    let item = UnityRequestObservation {
        id: UNITY_OBSERVATION_ID.fetch_add(1, Ordering::Relaxed),
        timestamp_ms: unity_observer_timestamp_ms(),
        method,
        path: unity_observer_path(&url),
        body_size,
        body_hex,
        content_type,
    };
    if let Ok(mut entries) = UNITY_OBSERVATIONS.lock() {
        if entries.len() >= UNITY_OBSERVATIONS_MAX {
            entries.remove(0);
        }
        entries.push(item);
    }
}

// UnityWebRequest.SendWebRequest() is asynchronous; this observes request entry only.
extern "C" fn unity_send_hook_handler(this: *mut c_void) -> *mut c_void {
    unsafe {
        let trampoline = interceptor_get_trampoline(unity_send_hook_handler as usize);
        if trampoline == 0 {
            return ptr::null_mut();
        }
        type FnType = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let original: FnType = std::mem::transmute(trampoline);
        // Observation failures must never block or replace the game's request.
        let _ = std::panic::catch
```

### `fn install_api_sniff_hooks`

matches=1

#### match 1 bytes 484138..490138

```rust
to_string_lossy();
        if name.contains(substr) {
            if let Some(get_ptr) = method_get_ptr_fn {
                let ptr = get_ptr(mi);
                if !ptr.is_null() {
                    ura_log(
                        3,
                        &format!(
                            "find_method_fuzzy: {}~{} -> 0x{:x}",
                            substr, name, ptr as usize
                        ),
                    );
                    return ptr as usize;
                }
            }
        }
    }
    0
}

/// ★ v3.24.40: fuzzy variant — first class whose name CONTAINS `substr`.
unsafe fn find_class_fuzzy(image: *const c_void, substr: &str) -> *mut c_void {
    let get_count_fn = resolve_il2cpp_symbol("il2cpp_image_get_class_count");
    let get_class_fn = resolve_il2cpp_symbol("il2cpp_image_get_class");
    let get_name_fn = resolve_il2cpp_symbol("il2cpp_class_get_name");
    if get_count_fn.is_null() || get_class_fn.is_null() || get_name_fn.is_null() {
        return ptr::null_mut();
    }
    let get_count: FnImageGetClassCount = std::mem::transmute(get_count_fn);
    let get_class: FnImageGetClass = std::mem::transmute(get_class_fn);
    let get_name: unsafe extern "C" fn(*const c_void) -> *const c_char =
        std::mem::transmute(get_name_fn);
    let count = get_count(image);
    for i in 0..count {
        let cls = get_class(image, i);
        if cls.is_null() {
            continue;
        }
        let np = get_name(cls);
        if np.is_null() {
            continue;
        }
        let name = CStr::from_ptr(np).to_string_lossy();
        if name.contains(substr) {
            ura_log(3, &format!("find_class_fuzzy: {}~{}", substr, name));
            return cls as *mut c_void;
        }
    }
    ptr::null_mut()
}

unsafe fn install_api_sniff_hooks() {
    let all_hooked = COMPRESS_REQUEST_ADDR != 0
        && DECOMPRESS_RESPONSE_ADDR != 0
        && POST_ADDR != 0
        && UNITY_SEND_ADDR != 0
        && UNITY_COMPLETE_ADDR != 0;
    if all_hooked {
        return;
    }
    if API.is_null() {
        ura_log(3, "API sniff: API is null");
        set_hook_status("sniff", "failed: api_null");
        return;
    }
    let api = &*API;
    if api.interceptor == 0 {
        ura_log(3, "API sniff: interceptor not available");
        set_hook_status("sniff", "failed: interceptor_unavailable");
        return;
    }

    // Get umamusume.dll assembly image
    let get_asm = match api.il2cpp_get_assembly_image_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_assembly_image not available");
            return;
        }
    };
    let get_class = match api.il2cpp_get_class_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_class not available");
            return;
        }
    };
    let get_method_addr = match api.il2cpp_get_method_addr_fn {
        Some(f) => f,
        None => {
            ura_log(3, "API sniff: get_method_addr not available");
            return;
        }
    };

    // Observe the lower UnityWebRequest request-entry path used by boot/auth traffic.
    if UNITY_SEND_ADDR == 0 {
        let unity_image = get_asm(to_cstr("UnityEngine.UnityWebRequestModule.dll").as_ptr());
        if unity_image.is_null() {
            set_hook_status("sniff.unity_send", "failed: image_not_found");
        } else {
            let unity_request = get_class(
                unity_image,
                to_cstr("UnityEngine.Networking").as_ptr(),
                to_cstr("UnityWebRequest").as_ptr(),
            );
            if unity_request.is_null() {
                set_hook_status("sniff.unity_send", "failed: class_not_found");
            } else {
                let addr = get_method_addr(
                    unity_request as usize,
                    to_cstr("SendWebRequest").as_ptr(),
                    0,
                );
                if addr == 0 {
                    set_hook_status("sniff.unity_send", "failed: method_not_found");
                } else if interceptor_hook(addr, unity_send_hook_handler as usize) {
                    UNITY_SEND_ADDR = addr;
                    set_hook_status("sniff.unity_send", &format!("hooked@0x{:x}", addr));
                    ura_log(
                        3,
                        &format!(
                            "API sniff: UnityWebRequest.SendWebRequest hooked at 0x{:x}",
                            addr
                        ),
                    );
                } else {
                    set_hook_status("sniff.unity_send", "failed: interceptor_hook");
                }
            }
        }
    }

    if UNITY_COMPLETE_ADDR == 0 {
        let core_image = get_asm(to_cstr("UnityEngine.CoreModule.dll").as_ptr());
        if core_image.is_null() {
            set_hook_status("sniff.unity_complete", "failed: image_not_found");
        } else {
            let async_operation = get_class(
                core_image,
                to_cstr("UnityEngine").as_ptr(),
                to_cstr("AsyncOperation").as_ptr(),
            );
            if async_operation.is_null() {
                set_hook_status("sniff.unity_complete", "failed: class_not_found");
            } else {
                let addr = get_method_addr(async_operation as usize, to_cstr("InvokeCompletionEvent").as_ptr(), 0);
                if addr == 0 {
                    set_hook_status("sniff.unity_complete", "failed: method_not_found");
                } else if interceptor_hook(addr, unity_complete_hook_handler as usize) {
                    UNITY_COMPLETE_ADDR = addr;
                    set_hook_status("sniff.unity_complete", &format!("hooked@0x{:x}", addr));
                } else {
                    set_hook_status("sniff.unity_complete", "failed: interceptor_hook");
                }
            }
        }
    }

    // Hook Cryptographer.MakeMd5 to capture salt
    if MAKEMD5_ADDR == 0 {
        let umamusume_img = get_
```

### `install_api_sniff_hooks();`

matches=3

#### match 1 bytes 290988..296988

```rust
      Ok(s) => s,
            Err(_) => return, // App not running, that's fine
        };
    let body = json.as_bytes();
    let req = format!(
        "POST /data HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        addr_str, body.len()
    );
    let _ = stream.write_all(req.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
    let mut buf = [0u8; 256];
    let _ = stream.read(&mut buf);
}

fn push_loop() {
    let interval =
        std::time::Duration::from_secs(unsafe { get_config() }.push_interval_secs.max(2));
    let mut consecutive_errors: u32 = 0;

    // ★ Initial push: try pushing current data on startup
    // Don't rely solely on GAME_INITIALIZED callback — it may never fire
    // if the game was already initialized before the plugin loaded.
    // Instead, try reading data; if it succeeds, the game is ready.
    for wait_round in 0..60 {
        if GAME_INITIALIZED.load(Ordering::Relaxed) {
            break;
        }
        boot_trace("push_probe_begin");
        // Try a probe read — if it doesn't error, game is ready
        let probe = read_summary();
        if !probe.contains("\"error\"") {
            GAME_INITIALIZED.store(true, Ordering::Relaxed);
            unsafe {
                ura_log(3, "Push: game detected via probe (no callback)");
                // v3.22.98: Install hooks in fallback (on_game_initialized may never fire)
                install_training_hook();
                install_exec_training_hook();
                install_failure_rate_hook();
                install_event_choice_hook();
                // ★ v3.24.40: sniff hooks were missing here — fallback mode
                // left /api/sniff permanently unhooked.
                install_api_sniff_hooks();
            }
            break;
        }
        if wait_round % 10 == 0 {
            unsafe {
                ura_log(
                    3,
                    &format!("Push: waiting for game... round={}", wait_round),
                );
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    let init_summary = read_summary();
    if !init_summary.contains("\"error\"") {
        unsafe {
            LAST_PUSH_HASH = simple_hash(&init_summary);
        }
        push_to_app(&init_summary);
        unsafe {
            ura_log(3, "Push: initial data pushed");
        }
    }

    loop {
        std::thread::sleep(interval);
        // Don't gate on GAME_INITIALIZED — just try reading;
        // if the game isn't ready, read_summary returns error and we skip.
        let summary = read_summary();
        if summary.contains("\"error\"") {
            consecutive_errors += 1;
            // ★ v3.22.89: Extra cooldown for SIGSEGV recovery — game state transition
            if summary.contains("sigsegv") {
                let cool = std::time::Duration::from_secs(60);
                unsafe {
                    ura_log(
                        2,
                        "Push: SIGSEGV recovered, cooling 60s for game state transition",
                    );
                }
                std::thread::sleep(cool);
            }
            // ★ v3.14.2: backoff on consecutive errors to avoid crash loop
            if consecutive_errors >= 1 {
                let backoff =
                    std::time::Duration::from_secs((consecutive_errors as u64 * 5).min(60));
                unsafe {
                    ura_log(
                        3,
                        &format!(
                            "Push: {} consecutive errors, backing off {}s",
                            consecutive_errors,
                            backoff.as_secs()
                        ),
                    );
                }
                std::thread::sleep(backoff);
            }
            continue;
        }
        consecutive_errors = 0;
        // If we got here, game is definitely ready
        if !GAME_INITIALIZED.load(Ordering::Relaxed) {
            GAME_INITIALIZED.store(true, Ordering::Relaxed);
        }
        let hash = simple_hash(&summary);
        let should_push = unsafe {
            if hash != LAST_PUSH_HASH {
                LAST_PUSH_HASH = hash;
                true
            } else {
                false
            }
        };
        if should_push {
            unsafe {
                ura_log(3, "Push: data changed, pushing to app");
            }
            push_to_app(&summary);
        }
    }
}

fn start_http_server() {
    if HTTP_RUNNING.load(Ordering::Relaxed) {
        return;
    }
    HTTP_RUNNING.store(true, Ordering::Relaxed);
    std::thread::spawn(|| {
        unsafe {
            // ★ v3.24.32: bind loopback only. The floating-window App talks to
            // the plugin on the same device, and desktop/LAN debugging works
            // via `adb forward tcp:18765 tcp:18765`. Binding 0.0.0.0 exposed
            // /il2cpp/call, /il2cpp/read_mem, /update etc. to the whole LAN
            // without authentication.
            ura_log(3, "HTTP starting on 127.0.0.1:18765");
        }
        let listener = match std::net::TcpListener::bind("127.0.0.1:18765") {
            Ok(l) => l,
            Err(e) => {
                unsafe {
                    ura_log(1, &format!("HTTP bind failed: {}", e));
                }
                HTTP_RUNNING.store(false, Ordering::Relaxed);
                return;
            }
        };
        unsafe {
            ura_log(3, "HTTP listening on :18765");
        }
        unsafe {
            ura_notify("URA HTTP :18765 ON");
        }

        // ★ Start push-to-app loop (v3.10.0)
        std::thread::spawn(|| {
            push_loop();
        });

        for stream in listener.incoming() {
            if !HTTP_RUNNING.load(Ordering::Relaxed) {
                break;
            }
            match stream {
                Ok(stream) => {
                    // ★ v3.18.8: s
```

#### match 2 bytes 336979..342979

```rust
    SNIFF_ENABLED.load(Ordering::Relaxed),
                SNIFF_REQUESTS.len(),
                SNIFF_RESPONSES.len(),
                SNIFF_METADATA.len(),
                request_count,
                response_count,
                last_id,
                SNIFF_RAW_MAX,
                SNIFF_METADATA_MAX
            )
        }
    } else if path == "/api/sniff/metadata" {
        let after_id = parse_query(&full_uri, "after_id")
            .parse::<u64>()
            .unwrap_or(0);
        let _lock = SNIFF_MUTEX.lock();
        unsafe {
            let entries: Vec<String> = SNIFF_METADATA.iter()
                .filter(|m| m.id > after_id)
                .map(|m| {
                    let headers_json: String = m.headers.iter()
                        .map(|(k, v)| format!(r#"{{"key":"{}","value":"{}"}}"#, json_escape(k), json_escape(v)))
                        .collect::<Vec<String>>()
                        .join(",");
                    format!(r#"{{"id":{},"request_id":{},"timestamp_ms":{},"direction":"{}","path":"{}","size":{},"body_hex":"{}","headers":[{}]}}"#,
                        m.id, m.request_id, m.timestamp_ms, m.direction, json_escape(&m.path), m.size, m.body_hex, headers_json)
                })
                .collect();
            let last_id = SNIFF_METADATA.last().map(|m| m.id).unwrap_or(after_id);
            format!(
                r#"{{"enabled":{},"after_id":{},"last_id":{},"count":{},"entries":[{}]}}"#,
                SNIFF_ENABLED.load(Ordering::Relaxed),
                after_id,
                last_id,
                entries.len(),
                entries.join(",")
            )
        }
    } else if path == "/api/sniff/toggle" {
        // ★ v3.24.40: lazy retry for fallback-mode installs.
        unsafe {
            install_api_sniff_hooks();
        }
        // ★ If hooks installed successfully, game is ready — set GAME_INITIALIZED
        let any_hooked = unsafe {
            COMPRESS_REQUEST_ADDR != 0
                || DECOMPRESS_RESPONSE_ADDR != 0
                || POST_ADDR != 0
                || MAKEMD5_ADDR != 0
                || COMPUTEHASH_ADDR != 0
        };
        if any_hooked && !GAME_INITIALIZED.load(Ordering::Relaxed) {
            GAME_INITIALIZED.store(true, Ordering::Relaxed);
            unsafe {
                ura_log(3, "sniff/toggle: GAME_INITIALIZED set (hooks installed via toggle)");
            }
        }
        let requested = parse_query(&full_uri, "enabled");
        let new_val = match requested.as_str() {
            "1" | "true" => true,
            "0" | "false" => false,
            _ => !SNIFF_ENABLED.load(Ordering::Relaxed),
        };
        SNIFF_ENABLED.store(new_val, Ordering::Relaxed);
        let req_hooked = unsafe { COMPRESS_REQUEST_ADDR != 0 };
        let resp_hooked = unsafe { DECOMPRESS_RESPONSE_ADDR != 0 };
        let post_hooked = unsafe { POST_ADDR != 0 };
        format!(
            r#"{{"sniff_enabled":{},"compress_hooked":{},"decompress_hooked":{},"post_hooked":{}}}"#,
            new_val, req_hooked, resp_hooked, post_hooked
        )
    } else if path == "/api/sniff/clear" {
        let _lock = SNIFF_MUTEX.lock();
        unsafe {
            SNIFF_REQUESTS.clear();
            SNIFF_RESPONSES.clear();
            if let Ok(mut entries) = UNITY_OBSERVATIONS.lock() {
                entries.clear();
            }
            if let Ok(mut completed) = UNITY_COMPLETED_RESPONSE_HEADERS.lock() {
                completed.clear();
            }
            SNIFF_METADATA.clear();
            SNIFF_RESPONSE_QUEUE.clear();
            PENDING_REQ_BODY = None;
        }
        r#"{"ok":true}"#.to_string()
    } else if path.starts_with("/debug/hooklog") {
        // ★ v3.24.40/42: last HOOK_LOG_MAX lines, optional ?filter=substr
        let filter = parse_query(&full_uri, "filter");
        let entries: Vec<String> = match HOOK_LOG.lock() {
            Ok(g) => g
                .iter()
                .filter(|l| filter.is_empty() || l.contains(&filter))
                .map(|l| json_escape(l))
                .collect(),
            Err(_) => Vec::new(),
        };
        format!(
            r#"{{"count":{},"entries":[{}]}}"#,
            entries.len(),
            entries.join(",")
        )
    } else if path == "/debug/resource_reads" {
        // ★ v3.24.58: meta/dat file-read trace. Lazy-starts the /proc watcher
        // on first request (never at init — thread spawn in init context).
        start_res_fd_watcher();
        let entries: Vec<String> = match RES_READ_LOG.lock() {
            Ok(g) => g
                .iter()
                .map(|l| format!("\"{}\"", json_escape(l)))
                .collect(),
            Err(_) => Vec::new(),
        };
        format!(
            r#"{{"count":{},"entries":[{}]}}"#,
            entries.len(),
            entries.join(",")
        )
    } else if path.starts_with("/debug/mem_scan_sqlite") {
        // ★ v3.24.58: hunt plaintext "SQLite format 3" pages in process memory
        // — any custom decryption MUST materialize this in RAM.
        let max_hits: usize = parse_query(&full_uri, "max").parse().unwrap_or(8);
        let mut hits: Vec<String> = Vec::new();
        let needle = b"SQLite format 3 ";
        if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
            let mem = std::fs::File::open("/proc/self/mem");
            use std::os::unix::fs::FileExt;
            if let Ok(mem) = mem {
                'outer: for line in maps.lines() {
                    let cols: Vec<&str> = line.split_whitespace().collect();
                    if cols.len() < 6 {
                        continue;
                    }
                    if !cols[1].contains("rw") {
                        continue;
                    }
                    let range: Vec<&str> = cols[0].split('-').collect();
                    if range.len() != 2 {
                        continue;
                    }
             
```

#### match 3 bytes 514651..520651

```rust
ull() {
        let set_story_addr = find_method_addr(story_mgr_class, "SetStory", 4);
        if set_story_addr != 0 {
            STORY_SET_ADDR = set_story_addr;
            STORY_SET_HOOK_INSTALLED = true;
            install_hook_safe(
                "StorySet",
                set_story_addr,
                story_set_hook_handler as usize,
                &mut ORIG_STORY_SET_PROLOGUE,
            );
            set_hook_status(
                "event.story_set",
                &format!("resolved@0x{:x}", set_story_addr),
            );
            ura_log(
                3,
                &format!(
                    "Event hook: StoryManager.SetStory hooked at 0x{:x}",
                    set_story_addr
                ),
            );
        } else {
            ura_log(3, "Event hook: StoryManager.SetStory NOT FOUND");
            set_hook_status("event.story_set", "failed: method_not_found");
        }
    } else {
        ura_log(3, "Event hook: StoryManager class NOT FOUND");
        set_hook_status("event.story_set", "failed: class_not_found");
    }

    // ★ v3.24.40: only mark installed when at least one hook landed, so the
    // lazy retry in /api/event/choices can re-attempt after early-boot misses.
    EVENT_CHOICE_HOOK_INSTALLED =
        EVENT_ADD_BTN_ADDR != 0 || EVENT_CHOICE_ADDR != 0 || STORY_SET_HOOK_INSTALLED;
}

extern "C" fn on_game_initialized(_userdata: *mut c_void) {
    GAME_INITIALIZED.store(true, Ordering::Relaxed);
    boot_trace("game_init_cb");
    unsafe {
        ura_log(3, "Game initialized");
        ura_notify("URA: Game ready!");
        // v3.22.98: Install hooks FIRST (before precache, which may panic)
        install_training_hook();
        install_exec_training_hook();
        install_failure_rate_hook();
        install_api_sniff_hooks();
        install_event_choice_hook();
        // v3.22.51: Pre-cache all IL2CPP metadata on game thread
        precache_metadata();
        boot_trace("game_init_done");
    }
}

extern "C" fn on_menu_section(ui: *mut c_void, _userdata: *mut c_void) {
    unsafe {
        if API.is_null() || ui.is_null() {
            return;
        }
        let api = &*API;

        if let Some(f) = api.gui_ui_heading_fn {
            f(
                ui,
                to_cstr(&format!("URA Assistant v{}", PLUGIN_VERSION)).as_ptr(),
            );
        }
        if let Some(f) = api.gui_ui_separator_fn {
            f(ui);
        }

        if let Some(f) = api.gui_ui_colored_label_fn {
            if GAME_INITIALIZED.load(Ordering::Relaxed) {
                f(ui, 0, 255, 136, 255, to_cstr("Game: Connected").as_ptr());
            } else {
                f(ui, 255, 200, 0, 255, to_cstr("Game: Waiting...").as_ptr());
            }
        }

        if let Some(f) = api.gui_ui_colored_label_fn {
            if HTTP_RUNNING.load(Ordering::Relaxed) {
                f(
                    ui,
                    0,
                    255,
                    136,
                    255,
                    to_cstr(&format!(
                        "HTTP: Running :{}",
                        unsafe { get_config() }.http_port
                    ))
                    .as_ptr(),
                );
            } else {
                f(ui, 255, 80, 80, 255, to_cstr("HTTP: Failed").as_ptr());
            }
        }

        if let Some(f) = api.gui_ui_label_fn {
            f(
                ui,
                to_cstr("Data: WDM->SingleMode->Chara (getters)").as_ptr(),
            );
        }

        let c = CHARA;
        if c.valid {
            if let Some(f) = api.gui_ui_separator_fn {
                f(ui);
            }

            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    0,
                    200,
                    255,
                    255,
                    to_cstr(&format!(
                        "Month {} | Half {} | PS:{}",
                        c.month, c.half, c.playing_state
                    ))
                    .as_ptr(),
                );
            }

            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    255,
                    100,
                    100,
                    255,
                    to_cstr(&format!("SPD: {}", c.speed)).as_ptr(),
                );
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    100,
                    255,
                    100,
                    255,
                    to_cstr(&format!("STA: {}", c.stamina)).as_ptr(),
                );
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    255,
                    200,
                    50,
                    255,
                    to_cstr(&format!("POW: {}", c.power)).as_ptr(),
                );
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    255,
                    130,
                    50,
                    255,
                    to_cstr(&format!("GUT: {}", c.guts)).as_ptr(),
                );
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                f(
                    ui,
                    100,
                    180,
                    255,
                    255,
                    to_cstr(&format!("WIZ: {}", c.wiz)).as_ptr(),
                );
            }

            if let Some(f) = api.gui_ui_label_fn {
                f(
                    ui,
                    to_cstr(&format!("Vital: {}/{}", c.vital, c.max_vital)).as_ptr(),
                );
            }
            if let Some(f) = api.gui_ui_colored_label_fn {
                let mot_text = match c.motivation {
                    5 => "Motivation: Best
```

### `fn append_global_observation`

matches=1

#### match 1 bytes 946282..952282

```rust
ted as usize));
    }
    format!(r#"{{"ok":true,"requested":"{}","direct_only":true,"count":{},"nested_types":[{}]}}"#, json_escape(&requested), items.len(), items.join(","))
}

unsafe fn il2cpp_enum_values_capability(uri: &str) -> String {
    let pairs = match parse_query_pairs(uri) { Ok(value) => value, Err(error) => return format!(r#"{{"ok":false,"error":"{}"}}"#, json_escape(&error)) };
    let requested = query_pair(&pairs, "type");
    if requested.is_empty() { return r#"{\"ok\":false,\"error\":\"missing_type\"}"#.to_string(); }
    let required = ["il2cpp_class_get_fields", "il2cpp_field_get_flags", "il2cpp_field_static_get_value"];
    let available: Vec<bool> = required.iter().map(|name| !resolve_il2cpp_symbol(name).is_null()).collect();
    format!(r#"{{"ok":true,"requested":"{}","value_status":"unresolved","integer_values":null,"declaration_order_inference":false,"runtime_api":{{"il2cpp_class_get_fields":{},"il2cpp_field_get_flags":{},"il2cpp_field_static_get_value":{}}}}}"#,
        json_escape(&requested), available[0], available[1], available[2])
}

// ===== Unified observation persistent storage B-stage =====
static STORAGE_SESSION_ID: Mutex<Option<String>> = Mutex::new(None);
static STORAGE_LAST_FLUSH_MS: AtomicU64 = AtomicU64::new(0);
static STORAGE_LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn storage_set_error(error: &str) {
    if let Ok(mut value) = STORAGE_LAST_ERROR.lock() {
        *value = Some(error.to_string());
    }
}

fn storage_clear_error() {
    if let Ok(mut value) = STORAGE_LAST_ERROR.lock() { *value = None; }
}


// 全局观测记录先追加到当前会话的NDJSON，再允许调用方更新内存索引。
// 每行以换行符作为完整提交边界；读取方不得把无换行的尾部当作完整记录。
static GLOBAL_OBSERVATION_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static GLOBAL_OBSERVATION_WRITE_LOCK: Mutex<()> = Mutex::new(());

fn append_global_observation(
    observation_type: &str,
    completeness: &str,
    payload_json: &str,
    critical: bool,
) -> Result<(String, u64, u64), String> {
    let _write_guard = GLOBAL_OBSERVATION_WRITE_LOCK
        .lock().map_err(|_| "global_observation_write_lock_poisoned".to_string())?;
    let session_id = ensure_observation_session()?;
    let sequence = GLOBAL_OBSERVATION_SEQUENCE.fetch_add(1, Ordering::SeqCst).saturating_add(1);
    let timestamp_ms = sniff_timestamp_ms();
    let session_directory = observation_storage_root().join("sessions").join(&session_id);
    std::fs::create_dir_all(&session_directory)
        .map_err(|error| format!("create_global_observation_dir:{}", error))?;
    let journal_path = session_directory.join("timeline.ndjson");
    let line = format!(
        r#"{{"session_id":"{}","sequence":{},"timestamp_ms":{},"type":"{}","completeness":"{}","payload":{}}}\n"#,
        json_escape(&session_id), sequence, timestamp_ms, json_escape(observation_type),
        json_escape(completeness), payload_json
    );
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&journal_path)
        .map_err(|error| format!("open_global_observation_journal:{}", error))?;
    std::io::Write::write_all(&mut file, line.as_bytes())
        .map_err(|error| format!("append_global_observation:{}", error))?;
    if critical {
        file.sync_data().map_err(|error| format!("sync_global_observation:{}", error))?;
    }
    let byte_length = file.metadata().map_err(|error| format!("stat_global_observation:{}", error))?.len();
    drop(file);
    let connection = open_observation_storage()?;
    connection.execute(
        "INSERT OR REPLACE INTO observation_files(
             session_id, relative_path, content_type, byte_length, sha256, created_at_ms
         ) VALUES(?1, 'timeline.ndjson', 'application/x-ndjson', ?2, NULL, ?3)",
        rusqlite::params![session_id, byte_length as i64, timestamp_ms as i64],
    ).map_err(|error| format!("index_global_observation:{}", error))?;
    STORAGE_LAST_FLUSH_MS.store(timestamp_ms, Ordering::Release);
    storage_clear_error();
    Ok((session_id, sequence, timestamp_ms))
}

fn persist_protocol_observation_boundary(
    direction: &str,
    request_id: u64,
    url: &str,
    relative_base: &str,
    headers_length: usize,
    payload_length: usize,
) -> Result<(), String> {
    let payload = format!(
        r#"{{"direction":"{}","request_id":{},"url":"{}","relative_base":"{}","headers_length":{},"payload_length":{}}}"#,
        json_escape(direction), request_id, json_escape(url), json_escape(relative_base),
        headers_length, payload_length
    );
    append_global_observation("protocol_exchange_part", "complete", &payload, true).map(|_| ())
}

fn persist_protocol_capture(direction: &str, request_id: u64, url: &str, headers: &[u8], payload: &[u8]) -> Result<(), String> {
    let session_id = ensure_observation_session()?;
    let now = sniff_timestamp_ms();
    let suffix = if direction == "response" { format!("{}-{}", request_id, now) } else { request_id.to_string() };
    let relative_base = format!("protocol/{}/{}", direction, suffix);
    let session_dir = observation_storage_root().join("sessions").join(&session_id);
    let target_dir = session_dir.join(&relative_base);
    std::fs::create_dir_all(&target_dir).map_err(|error| format!("create_protocol_dir:{}", error))?;
    let files: [(&str, &[u8], &str); 3] = [
        ("url.txt", url.as_bytes(), "text/plain; charset=utf-8"),
        ("headers.raw", headers, "application/octet-stream"),
        ("payload.bin", payload, "application/octet-stream"),
    ];
    for (name, bytes, _) in &files {
        let temporary = target_dir.join(format!("{}.tmp", name));
        let mut file = std::fs::File::create(&temporary)
            .map_err(|error| format!("create_protocol_file:{}:{}", name, error))?;
        std::io::Write::write_all(&mut file, bytes)
            .map_err(|error| format!("write_protocol_file:{}:{}", name, error))?;
        file.sync_data().map_err(|error| format!("sync_protocol_file:{}:{}", name, error))?;
        drop(file);
        std::fs::rename(&temporary, targ
```

### `fn set_hook_status`

matches=1

#### match 1 bytes 10819..16819

```rust
s, effects, branches)
static mut EVENT_CHOICE_HOOK_INSTALLED: bool = false;
static mut EVENT_CHOICE_ADDR: usize = 0; // StoryChoiceController.Choice
static mut EVENT_ADD_BTN_ADDR: usize = 0; // StoryChoiceController.AddChoiceButton
static mut ORIG_EVENT_CHOICE_PROLOGUE: [u8; 16] = [0; 16];
static mut ORIG_EVENT_ADD_BTN_PROLOGUE: [u8; 16] = [0; 16];
// ★ v3.24.2: StoryManager.SetStory hook — capture story_id and chara_id for event type identification
static mut STORY_SET_HOOK_INSTALLED: bool = false;
static mut STORY_SET_ADDR: usize = 0;
static mut ORIG_STORY_SET_PROLOGUE: [u8; 16] = [0; 16];
// Event state: accumulated choices for current event
static EVENT_STATE_MUTEX: Mutex<()> = Mutex::new(());

// ★ v3.24.40: mirror every ura_log line into a queryable ring buffer
// (Hachimi logcat was the only outlet before; /debug/hooklog exposes it).
static HOOK_LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
const HOOK_LOG_MAX: usize = 256;

// ★ v3.24.42: high-frequency read_summary/push spam is excluded from the
// ring (still goes to logcat) so event/sniff diagnostics survive.
const HOOK_LOG_NOISE: &[&str] = &[
    "★ read_summary",
    "ramen scalar",
    "ramen arrays",
    "evaluation_list",
    "sc: ",
    "skill_eval=",
    "v3.22.51 ramen",
    "★ Scenario 14",
    "Push:",
    "call_getter: 'get_Skill",
    "call_getter: 'get_PossessSkill",
    "find_field_offset: 'RemainTurn'",
];
fn hook_log(msg: &str) {
    if HOOK_LOG_NOISE.iter().any(|n| msg.contains(n)) {
        return;
    }
    if let Ok(mut g) = HOOK_LOG.lock() {
        if g.len() >= HOOK_LOG_MAX {
            g.remove(0);
        }
        g.push(msg.to_string());
    }
}

// ★ v3.24.40: per-hook install status for /debug/hookdiag
static HOOK_STATUS: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
fn set_hook_status(name: &str, status: &str) {
    hook_log(&format!("hook[{}] = {}", name, status));
    if let Ok(mut g) = HOOK_STATUS.lock() {
        if let Some(e) = g.iter_mut().find(|(n, _)| n == name) {
            e.1 = status.to_string();
        } else {
            g.push((name.to_string(), status.to_string()));
        }
    }
}
static mut EVENT_CHOICES: Vec<EventChoice> = Vec::new();
static mut EVENT_SELECTED_IDX: i32 = -1;
static mut EVENT_STORY_ID: i32 = 0;
static mut EVENT_CHARA_ID: i32 = 0;

// Incremented whenever a new story_id takes over (or state is cleared).
// Guarded by EVENT_STATE_MUTEX; never read/write outside the lock.
static mut EVENT_GENERATION: u64 = 0;

// Cap against runaway AddChoiceButton repeats in abnormal UI rebuilds.
const EVENT_CHOICES_MAX: usize = 32;

#[derive(Clone)]
struct EventChoice {
    label: String,
    gain_id: i32,
    next_block_idx: i32,
    loop_exit_gain_id: i32,
}

// v3.24.73: bounded cache-only pairing. This is temporal co-occurrence,
// never a success/failure classification or a causality claim.
#[derive(Clone)]
struct PendingEventSelection {
    captured_at: u64,
    generation: u64,
    story_id: i32,
    chara_id: i32,
    selected_idx_raw: i32,
    choice: Option<EventChoice>,
}
static EVENT_PENDING_RESULT: Mutex<Option<PendingEventSelection>> = Mutex::new(None);
static EVENT_OBSERVATIONS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static EVENT_OBSERVATION_ID: AtomicU64 = AtomicU64::new(1);
const EVENT_OBSERVATIONS_MAX: usize = 16;
const EVENT_RESPONSE_PREVIEW_MAX: usize = 16 * 1024;

// ★ v3.24.2: Read C# string from IL2CPP String object
unsafe fn read_il2cpp_string(s: *const c_void) -> String {
    if s.is_null() {
        return String::new();
    }
    let len = std::ptr::read::<i32>((s as *const u8).offset(16) as *const i32);
    if len <= 0 || len > 4096 {
        return String::new();
    }
    let chars_ptr = (s as *const u8).offset(20);
    let chars_slice = std::slice::from_raw_parts(chars_ptr as *const u16, len as usize);
    String::from_utf16_lossy(chars_slice)
}

// ★ Push-to-app state (v3.10.0): auto-push /summary to uma-juece when data changes
static mut LAST_PUSH_HASH: u64 = 0;
static PUSH_INTERVAL_SECS: u64 = 1;

// ★ Config (v3.11.0): runtime config updated via POST /config from App
// No file editing needed — App settings page sends config to plugin HTTP endpoint
#[derive(Clone)]
struct PluginConfig {
    push_host: String,       // default: "127.0.0.1"
    push_port: u16,          // default: 18766
    http_port: u16,          // default: 18765
    push_interval_secs: u64, // default: 1
    push_enabled: bool,      // default: true
    http_enabled: bool,      // default: true
}

impl PluginConfig {
    fn defaults() -> Self {
        Self {
            push_host: "127.0.0.1".to_string(),
            push_port: 18766,
            http_port: 18765,
            push_interval_secs: 5,
            push_enabled: true,
            http_enabled: true,
        }
    }

    fn push_addr(&self) -> String {
        format!("{}:{}", self.push_host, self.push_port)
    }

    // Parse JSON config from POST /config body (simple manual parse, no serde)
    fn from_json(data: &str) -> Option<Self> {
        let mut cfg = Self::defaults();
        let mut changed = false;
        // Extract key-value pairs from JSON
        for line in data.lines() {
            let l = line.trim().trim_end_matches(',');
            if l.is_empty() || l == "{" || l == "}" {
                continue;
            }
            if let Some((k, v)) = l.split_once(':') {
                let k = k.trim().trim_matches('"');
                let v = v.trim().trim_matches('"');
                match k {
                    "push_host" => {
                        cfg.push_host = v.to_string();
                        changed = true;
                    }
                    "push_port" => {
                        if let Ok(n) = v.parse::<u16>() {
                            cfg.push_port = n;
                            changed = true;
                        }
                    }
                    "http_port" => {
                        if let Ok(n) = v.parse::<u16>() {
 
```

## Exact IL2CPP metadata scalar records

hit_count=0
