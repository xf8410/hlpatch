//! Single-sampler observation bridge. HTTP and push never invoke game getters.
//! Transport receipt confirms delivery only, not valid state or completed AI computation.
use std::{
    io::{Read, Write}, net::{SocketAddr, TcpStream}, panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Condvar, Mutex}, thread::{self, JoinHandle}, time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use ramen_observation::Publisher;
use serde_json::{Value, json};

const SAMPLE_INTERVAL: Duration = Duration::from_secs(2);
const MIN_WAKE_INTERVAL: Duration = Duration::from_millis(250);
const MAX_SNAPSHOT_BYTES: usize = 4 * 1024 * 1024;
const MAX_HEADER_BYTES: usize = 8192;
const MAX_RECEIPT_BYTES: usize = 16384;

/// Host-visible capture lifecycle. None of these states infer unobserved game fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureState { Booting, WaitingForRun, Capturing, Incomplete, Ready, Error }

impl CaptureState {
    /// Stable diagnostic wire spelling.
    pub fn as_str(self) -> &'static str {
        match self { Self::Booting => "booting", Self::WaitingForRun => "waiting_for_run", Self::Capturing => "capturing",
            Self::Incomplete => "incomplete", Self::Ready => "ready", Self::Error => "error" }
    }
}

/// Identity-bound transport receipt. `run_id=None` explicitly represents a partial observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub receipt_id: String,
    pub schema_version: u64,
    pub collector_instance_id: String,
    pub run_id: Option<u64>,
    pub snapshot_id: u64,
}

impl Receipt {
    /// Accept only the Android transport-received contract, never arbitrary HTTP success.
    pub fn parse(value: &Value, snapshot: &Value) -> Result<Self, String> {
        if value["status"] != "received" || value["transport_received"] != true
            || value["accepted_for_validation"] != true || value["computed"] != false {
            return Err("invalid_transport_receipt_status".into());
        }
        let run_id = nullable_id(value, "run_id")?;
        let receipt = Self {
            receipt_id: value["receipt_id"].as_str().filter(|s| !s.is_empty() && s.len() <= 256)
                .ok_or("receipt_id missing")?.into(),
            schema_version: value["schema_version"].as_u64().ok_or("receipt schema_version missing")?,
            collector_instance_id: value["collector_instance_id"].as_str().ok_or("receipt instance missing")?.into(),
            run_id,
            snapshot_id: value["snapshot_id"].as_u64().ok_or("receipt snapshot_id missing")?,
        };
        receipt.matches(snapshot)?;
        Ok(receipt)
    }
    /// Guard again when an injected sender returns a receipt without using the HTTP helper.
    fn matches(&self, snapshot: &Value) -> Result<(), String> {
        if snapshot["schema_version"].as_u64() != Some(self.schema_version)
            || snapshot["collector_instance_id"].as_str() != Some(self.collector_instance_id.as_str())
            || nullable_id(snapshot, "run_id")? != self.run_id
            || snapshot["snapshot_id"].as_u64() != Some(self.snapshot_id) {
            return Err("transport_receipt_identity_mismatch".into());
        }
        Ok(())
    }
    /// Structured diagnostics for the last acknowledged transport identity.
    fn value(&self) -> Value {
        json!({"receipt_id":self.receipt_id,"schema_version":self.schema_version,
            "collector_instance_id":self.collector_instance_id,"run_id":self.run_id,"snapshot_id":self.snapshot_id})
    }
}

/// Missing identity is not interchangeable with the explicit null used by partial captures.
fn nullable_id(value: &Value, name: &str) -> Result<Option<u64>, String> {
    match value.get(name) {
        Some(Value::Null) => Ok(None),
        Some(value) => value.as_u64().filter(|v| *v > 0).map(Some).ok_or_else(|| format!("invalid {name}")),
        None => Err(format!("{name} missing")),
    }
}

/// A push attempt either has no work, is already acknowledged, or produces a bound receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PushOutcome { NoSnapshot, AlreadyAcknowledged, Acknowledged(Receipt) }

#[derive(Clone)]
struct State {
    current: Option<Arc<Value>>,
    raw_summary: Option<Arc<String>>,
    current_from_attempt: bool,
    capture_state: CaptureState,
    last_capture_ms: Option<u64>,
    last_successful_capture_ms: Option<u64>,
    last_send_ms: Option<u64>,
    last_ack_ms: Option<u64>,
    last_ack: Option<Arc<Receipt>>,
    capture_error: Option<Arc<String>>,
    send_error: Option<Arc<String>>,
    wake: bool,
    stopping: bool,
}

/// One process bridge, one Publisher and one sampling worker. Closures are called only by that worker.
pub struct Bridge {
    version: String,
    collector_instance_id: String,
    state: Mutex<State>,
    changed: Condvar,
    sending: Mutex<()>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl Bridge {
    /// Start bounded periodic sampling. `ready` must perform only a safe readiness/probe operation.
    /// `capture` must read fresh legacy state; it must not call an HTTP/cache accessor.
    pub fn start<R, C>(version: &str, ready: R, capture: C) -> Result<Arc<Self>, String>
    where R: Fn() -> bool + Send + 'static, C: Fn() -> Result<String, String> + Send + 'static {
        Self::start_timed(version, ready, capture, SAMPLE_INTERVAL, MIN_WAKE_INTERVAL)
    }

    /// Shared worker constructor; tests shorten scheduling intervals without changing publication logic.
    fn start_timed<R, C>(version: &str, ready: R, capture: C, interval: Duration, minimum: Duration) -> Result<Arc<Self>, String>
    where R: Fn() -> bool + Send + 'static, C: Fn() -> Result<String, String> + Send + 'static {
        let publisher = Publisher::default();
        let bridge = Arc::new(Self {
            version: version.into(), collector_instance_id: publisher.collector_instance_id().into(),
            state: Mutex::new(State { current: None, raw_summary: None, current_from_attempt: false,
                capture_state: CaptureState::Booting, last_capture_ms: None, last_successful_capture_ms: None,
                last_send_ms: None, last_ack_ms: None, last_ack: None, capture_error: None, send_error: None,
                wake: true, stopping: false }), changed: Condvar::new(), sending: Mutex::new(()), worker: Mutex::new(None),
        });
        let worker_bridge = Arc::clone(&bridge);
        let worker = thread::Builder::new().name("ramen-observation".into()).spawn(move || {
            worker_bridge.sample_loop(publisher, ready, capture, interval, minimum);
        }).map_err(|e| e.to_string())?;
        *bridge.worker.lock().map_err(|_| "bridge worker lock poisoned")? = Some(worker);
        Ok(bridge)
    }

    /// Request an earlier probe. Repeated wakeups are coalesced and cannot cause a busy capture loop.
    pub fn wake(&self) {
        if let Ok(mut state) = self.state.lock() { state.wake = true; self.changed.notify_one(); }
    }

    /// Last publication for diagnostics, including history invalidated by boot/error.
    /// Decision/transport callers must use http_snapshot, summary or push_once instead.
    pub fn current(&self) -> Option<Value> {
        let current = { self.state.lock().ok()?.current.clone() };
        current.map(|value| value.as_ref().clone())
    }

    /// Legacy display bytes from the same sampling attempt as the published envelope.
    pub fn summary(&self) -> Option<String> {
        let raw = {
            let state = self.state.lock().ok()?;
            if state.capture_state == CaptureState::Booting || !state.current_from_attempt { return None; }
            state.raw_summary.clone()
        };
        raw.map(|value| value.as_ref().clone())
    }

    /// Capability identity is stable for this bridge's lifetime. Legacy conversion is still partial.
    pub fn capabilities(&self) -> Value {
        let capture_state = match self.state.lock() {
            Ok(state) => state.capture_state,
            Err(_) => return json!({"error":"bridge_lock_poisoned","ready":false}),
        };
        json!({"capability_schema_version":1,"snapshot_schema_versions":[2],
            "collector_instance_id":self.collector_instance_id,"collector_version":self.version,
            "capture_state":capture_state.as_str(),"complete_snapshot_supported":false})
    }

    /// HTTP snapshot response without starting a sample or re-numbering cached data.
    pub fn http_snapshot(&self) -> Value {
        let view = match self.state.lock() {
            Ok(state) => state.clone(),
            Err(_) => return json!({"ready":false,"error":"bridge_lock_poisoned"}),
        };
        if view.capture_state != CaptureState::Booting && view.current_from_attempt {
            if let Some(current) = view.current.as_ref() { return current.as_ref().clone(); }
        }
        json!({"schema_version":2,"collector_instance_id":self.collector_instance_id,
            "collector_version":self.version,"status":view.capture_state.as_str(),"ready":false,
            "missing_fields":["current_observation"],"error":view.capture_error.as_deref()})
    }

    /// Bounded diagnostic state; timestamps distinguish attempt, capture and acknowledged delivery.
    pub fn status(&self) -> Value {
        let view = match self.state.lock() {
            Ok(state) => state.clone(),
            Err(_) => return json!({"error":"bridge_lock_poisoned"}),
        };
        let current = view.current.as_deref();
        json!({"schema_version":2,"collector_instance_id":self.collector_instance_id,
            "capture_state":view.capture_state.as_str(),"ready":view.capture_state == CaptureState::Ready,
            "last_capture_ms":view.last_capture_ms,"last_successful_capture_ms":view.last_successful_capture_ms,
            "last_send_ms":view.last_send_ms,"last_ack_ms":view.last_ack_ms,
            "last_ack":view.last_ack.as_ref().map(|receipt| receipt.value()),"capture_error":view.capture_error.as_deref(),
            "send_error":view.send_error.as_deref(),"missing_fields":current.map(|v| &v["missing_fields"]),
            "snapshot_id":current.and_then(|v| v["snapshot_id"].as_u64()),
            "run_id":current.map(|v| &v["run_id"]),"sampling_interval_ms":SAMPLE_INTERVAL.as_millis()})
    }

    /// Send the cached identity once; failed delivery leaves it pending for the next attempt.
    /// State is unlocked for deep clones, JSON, retired-value destruction and network I/O.
    pub fn push_once<F>(&self, sender: F) -> Result<PushOutcome, String>
    where F: FnOnce(&Value) -> Result<Receipt, String> {
        let _sending = self.sending.lock().map_err(|_| "bridge send lock poisoned")?;
        let (snapshot, last_ack) = {
            let state = self.state.lock().map_err(|_| "bridge lock poisoned")?;
            if state.capture_state == CaptureState::Booting || !state.current_from_attempt { return Ok(PushOutcome::NoSnapshot); }
            let Some(snapshot) = state.current.clone() else { return Ok(PushOutcome::NoSnapshot); };
            (snapshot, state.last_ack.clone())
        };
        if last_ack.as_ref().is_some_and(|receipt| receipt.matches(&snapshot).is_ok()) {
            return Ok(PushOutcome::AlreadyAcknowledged);
        }
        let sent_at = now_ms();
        { self.state.lock().map_err(|_| "bridge lock poisoned")?.last_send_ms = Some(sent_at); }
        let receipt = sender(&snapshot).and_then(|receipt| { receipt.matches(&snapshot)?; Ok(receipt) });
        match receipt {
            Ok(receipt) => {
                let stored = Arc::new(receipt.clone()); let acknowledged_at = now_ms();
                let retired = {
                    let mut state = self.state.lock().map_err(|_| "bridge lock poisoned")?;
                    state.last_ack_ms = Some(acknowledged_at);
                    (state.last_ack.replace(stored), state.send_error.take())
                };
                drop(retired);
                Ok(PushOutcome::Acknowledged(receipt))
            },
            Err(error) => {
                let stored = Arc::new(bounded_error(&error));
                let retired = { self.state.lock().map_err(|_| "bridge lock poisoned")?.send_error.replace(stored) };
                drop(retired);
                Err(error)
            },
        }
    }

    /// Stop an owned bridge during controlled host shutdown/tests; wait for an in-flight capture.
    pub fn shutdown(&self) -> Result<(), String> {
        { let mut state = self.state.lock().map_err(|_| "bridge lock poisoned")?; state.stopping = true; self.changed.notify_one(); }
        let worker = self.worker.lock().map_err(|_| "bridge worker lock poisoned")?.take();
        if let Some(worker) = worker { worker.join().map_err(|_| "bridge worker panicked")?; }
        Ok(())
    }

    /// At most one capture is in flight. The publication lock is released around all native getters.
    fn sample_loop<R, C>(&self, mut publisher: Publisher, ready: R, capture: C, interval: Duration, minimum: Duration)
    where R: Fn() -> bool, C: Fn() -> Result<String, String> {
        let mut next = Instant::now();
        let mut earliest = next;
        loop {
            let Ok(mut state) = self.state.lock() else { return; };
            loop {
                if state.stopping { return; }
                let now = Instant::now();
                if now >= next || (state.wake && now >= earliest) { break; }
                let due = if state.wake { earliest.min(next) } else { next };
                match self.changed.wait_timeout(state, due.saturating_duration_since(now)) {
                    Ok((guard, _)) => state = guard,
                    Err(_) => return,
                }
            }
            state.wake = false;
            drop(state);
            if !catch_unwind(AssertUnwindSafe(&ready)).unwrap_or(false) {
                if let Ok(mut state) = self.state.lock() {
                    state.capture_state = CaptureState::Booting; state.current_from_attempt = false;
                }
            } else {
                self.capture_once(&mut publisher, &capture);
            }
            let now = Instant::now();
            next = now + interval;
            earliest = now + minimum;
        }
    }

    /// Exactly one ticket and one publication per native read. Publisher is worker-owned:
    /// all conversion, comparison, validation and retired Publisher allocations stay outside State.
    fn capture_once<C: Fn() -> Result<String, String>>(&self, publisher: &mut Publisher, capture: &C) {
        {
            let Ok(mut state) = self.state.lock() else { return; };
            state.capture_state = CaptureState::Capturing;
        }
        let ticket = match publisher.begin_capture() {
            Ok(ticket) => ticket,
            Err(error) => { self.capture_failed(None, error); return; },
        };
        let captured = catch_unwind(AssertUnwindSafe(capture)).unwrap_or_else(|_| Err("capture_panicked".into()));
        let raw = match captured {
            Ok(raw) if raw.len() <= MAX_SNAPSHOT_BYTES => raw,
            Ok(_) => json!({"error":"capture_exceeds_byte_budget"}).to_string(),
            Err(error) => json!({"error":bounded_error(&error)}).to_string(),
        };
        let parsed = serde_json::from_str::<Value>(&raw);
        let (raw, error) = match parsed {
            Ok(value) if value.is_object() => (raw, value["error"].as_str().map(bounded_error)),
            _ => (json!({"error":"invalid_capture_json"}).to_string(), Some("invalid_capture_json".into())),
        };
        let at = now_ms();
        let ready = match publisher.finish_summary(ticket, &raw, &self.version, at) {
            Ok(value) => value["ready"] == true,
            Err(error) => { self.capture_failed(Some(at), error); return; },
        };
        let current = publisher.shared_current();
        let raw = Arc::new(raw);
        let capture_state = match error.as_deref() {
            Some("no_sm" | "no_chara" | "no_wdm_inst" | "no_single_mode") => CaptureState::WaitingForRun,
            Some(_) => CaptureState::Error,
            None if ready => CaptureState::Ready,
            None => CaptureState::Incomplete,
        };
        let error = error.map(Arc::new);
        let retired = {
            let Ok(mut state) = self.state.lock() else { return; };
            state.last_capture_ms = Some(at);
            if error.is_none() { state.last_successful_capture_ms = Some(at); }
            state.current_from_attempt = true;
            state.capture_state = capture_state;
            (std::mem::replace(&mut state.current, current), state.raw_summary.replace(raw),
                std::mem::replace(&mut state.capture_error, error))
        };
        // State can hold the last reference to the previous large raw/value allocation.
        drop(retired);
    }

    fn capture_failed(&self, at: Option<u64>, error: String) {
        let error = Arc::new(bounded_error(&error));
        let retired = {
            let Ok(mut state) = self.state.lock() else { return; };
            if let Some(at) = at { state.last_capture_ms = Some(at); }
            state.capture_state = CaptureState::Error; state.current_from_attempt = false;
            state.capture_error.replace(error)
        };
        drop(retired);
    }

}

/// Wall-clock metadata only; sequencing and sampler deadlines use independent counters/monotonic time.
fn now_ms() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().min(u64::MAX as u128) as u64 }

/// Prevent native errors from becoming an unbounded diagnostic response.
fn bounded_error(error: &str) -> String { error.chars().take(512).collect() }

/// Send one bounded HTTP request with a single absolute deadline, including slow-drip responses.
/// Only HTTP 202 with the exact identity-bound transport receipt acknowledges the snapshot.
pub fn post_snapshot(address: SocketAddr, path: &str, snapshot: &Value, timeout: Duration) -> Result<Receipt, String> {
    if timeout.is_zero() || timeout > Duration::from_secs(5) { return Err("invalid_push_timeout".into()); }
    if !path.starts_with('/') || path.bytes().any(|c| c <= b' ' || c >= 127) { return Err("invalid_push_path".into()); }
    let body = serde_json::to_vec(snapshot).map_err(|e| e.to_string())?;
    if body.len() > MAX_SNAPSHOT_BYTES { return Err("snapshot_exceeds_push_budget".into()); }
    let deadline = Instant::now() + timeout;
    let mut stream = TcpStream::connect_timeout(&address, timeout).map_err(|e| format!("push_connect: {e}"))?;
    let header = format!("POST {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
    write_deadline(&mut stream, header.as_bytes(), deadline)?;
    write_deadline(&mut stream, &body, deadline)?;
    let mut bytes = Vec::new();
    let mut header_end = None;
    let mut body_len = None;
    loop {
        if let (Some(start), Some(length)) = (header_end, body_len) {
            if bytes.len() >= start + length { break; }
        }
        stream.set_read_timeout(Some(remaining(deadline)?)).map_err(|e| e.to_string())?;
        let mut chunk = [0u8; 2048];
        let size = stream.read(&mut chunk).map_err(|e| format!("push_read: {e}"))?;
        if size == 0 { break; }
        bytes.extend_from_slice(&chunk[..size]);
        if bytes.len() > MAX_HEADER_BYTES + MAX_RECEIPT_BYTES { return Err("receipt_exceeds_byte_budget".into()); }
        if header_end.is_none() {
            if let Some(index) = bytes.windows(4).position(|v| v == b"\r\n\r\n") {
                if index > MAX_HEADER_BYTES { return Err("receipt_header_exceeds_budget".into()); }
                let header = std::str::from_utf8(&bytes[..index]).map_err(|_| "invalid_receipt_header")?;
                let mut lines = header.split("\r\n");
                let mut status_line = lines.next().unwrap_or_default().split_whitespace();
                if !matches!(status_line.next(), Some("HTTP/1.0" | "HTTP/1.1")) { return Err("invalid_receipt_http_version".into()); }
                let status = status_line.next();
                if status != Some("202") { return Err(format!("receipt_requires_http_202: {status:?}")); }
                for line in lines {
                    let Some((key, value)) = line.split_once(':') else { return Err("invalid_receipt_header".into()); };
                    if key.eq_ignore_ascii_case("transfer-encoding") { return Err("chunked_receipt_not_supported".into()); }
                    if key.eq_ignore_ascii_case("content-length") {
                        let length = value.trim().parse::<usize>().map_err(|_| "invalid_receipt_length")?;
                        if body_len.is_some() || length > MAX_RECEIPT_BYTES { return Err("invalid_receipt_length".into()); }
                        body_len = Some(length);
                    }
                }
                header_end = Some(index + 4);
            } else if bytes.len() > MAX_HEADER_BYTES { return Err("receipt_header_exceeds_budget".into()); }
        }
    }
    let start = header_end.ok_or("receipt_header_incomplete")?;
    let body = &bytes[start..];
    if body.len() > MAX_RECEIPT_BYTES || body_len.is_some_and(|length| body.len() < length) { return Err("receipt_body_incomplete_or_too_large".into()); }
    let body = &body[..body_len.unwrap_or(body.len())];
    let value = serde_json::from_slice(body).map_err(|e| format!("invalid_receipt_json: {e}"))?;
    Receipt::parse(&value, snapshot)
}

/// A per-read timeout alone is insufficient when the peer sends one byte before each timeout.
fn remaining(deadline: Instant) -> Result<Duration, String> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() { Err("push_deadline_exceeded".into()) } else { Ok(remaining) }
}

/// Write bounded chunks while honoring the same absolute request deadline.
fn write_deadline(stream: &mut TcpStream, mut bytes: &[u8], deadline: Instant) -> Result<(), String> {
    while !bytes.is_empty() {
        stream.set_write_timeout(Some(remaining(deadline)?)).map_err(|e| e.to_string())?;
        let count = stream.write(&bytes[..bytes.len().min(16384)]).map_err(|e| format!("push_write: {e}"))?;
        if count == 0 { return Err("push_write_zero".into()); }
        bytes = &bytes[count..];
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::TcpListener, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc}};

    // Thread-local instrumentation observes real serde/Value/String allocation and destruction.
    // It never sleeps or allocates in the allocator callback. Only the tested thread is armed.
    struct LockProbe {
        bridge: std::sync::Weak<Bridge>,
        blocked_allocations: AtomicUsize,
        blocked_deallocations: AtomicUsize,
        large_allocations: AtomicUsize,
        large_deallocations: AtomicUsize,
        old_raw_pointer: AtomicUsize,
        old_snapshot_pointer: AtomicUsize,
        old_raw_drops: AtomicUsize,
        old_snapshot_drops: AtomicUsize,
    }
    impl LockProbe {
        fn new(bridge: &Arc<Bridge>) -> Arc<Self> {
            Arc::new(Self { bridge: Arc::downgrade(bridge), blocked_allocations: AtomicUsize::new(0),
                blocked_deallocations: AtomicUsize::new(0), large_allocations: AtomicUsize::new(0),
                large_deallocations: AtomicUsize::new(0), old_raw_pointer: AtomicUsize::new(0),
                old_snapshot_pointer: AtomicUsize::new(0), old_raw_drops: AtomicUsize::new(0),
                old_snapshot_drops: AtomicUsize::new(0) })
        }
        fn assert_unlocked(&self) {
            assert_eq!(self.blocked_allocations.load(Ordering::SeqCst), 0, "allocation while state mutex unavailable");
            assert_eq!(self.blocked_deallocations.load(Ordering::SeqCst), 0, "destruction while state mutex unavailable");
        }
    }
    thread_local! {
        static ALLOCATION_PROBE: std::cell::RefCell<Option<Arc<LockProbe>>> = const { std::cell::RefCell::new(None) };
    }
    fn arm_probe(probe: Option<Arc<LockProbe>>) { ALLOCATION_PROBE.with(|slot| *slot.borrow_mut() = probe); }
    fn observe_allocation(size: usize, deallocating: bool, pointer: usize) {
        let _ = ALLOCATION_PROBE.try_with(|slot| {
            let Ok(probe) = slot.try_borrow() else { return; };
            let Some(probe) = probe.as_ref() else { return; };
            let Some(bridge) = probe.bridge.upgrade() else { return; };
            if deallocating && pointer != 0 {
                if probe.old_raw_pointer.compare_exchange(pointer, 0, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                    probe.old_raw_drops.fetch_add(1, Ordering::SeqCst);
                }
                if probe.old_snapshot_pointer.compare_exchange(pointer, 0, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                    probe.old_snapshot_drops.fetch_add(1, Ordering::SeqCst);
                }
            }
            if size >= 64 * 1024 {
                let count = if deallocating { &probe.large_deallocations } else { &probe.large_allocations };
                count.fetch_add(1, Ordering::SeqCst);
            }
            if bridge.state.try_lock().is_err() {
                let count = if deallocating { &probe.blocked_deallocations } else { &probe.blocked_allocations };
                count.fetch_add(1, Ordering::SeqCst);
            }
        });
    }
    struct ObservedAllocator;
    #[global_allocator]
    static TEST_ALLOCATOR: ObservedAllocator = ObservedAllocator;
    unsafe impl std::alloc::GlobalAlloc for ObservedAllocator {
        unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
            observe_allocation(layout.size(), false, 0);
            unsafe { std::alloc::GlobalAlloc::alloc(&std::alloc::System, layout) }
        }
        unsafe fn alloc_zeroed(&self, layout: std::alloc::Layout) -> *mut u8 {
            observe_allocation(layout.size(), false, 0);
            unsafe { std::alloc::GlobalAlloc::alloc_zeroed(&std::alloc::System, layout) }
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
            observe_allocation(layout.size(), true, ptr as usize);
            unsafe { std::alloc::GlobalAlloc::dealloc(&std::alloc::System, ptr, layout) }
        }
        unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, size: usize) -> *mut u8 {
            observe_allocation(size, false, 0);
            unsafe { std::alloc::GlobalAlloc::realloc(&std::alloc::System, ptr, layout, size) }
        }
    }

    #[test]
    fn publication_conversion_and_old_value_drop_do_not_hold_state_lock() {
        let (probe_tx, probe_rx) = mpsc::channel::<Arc<LockProbe>>();
        let (done_tx, done_rx) = mpsc::channel();
        let rounds = AtomicUsize::new(0);
        let ready_probe = Arc::new(Mutex::new(None::<Arc<LockProbe>>));
        let evidence = ready_probe.clone();
        let capture_probe = std::cell::RefCell::new(None::<Arc<LockProbe>>);
        let captured = AtomicUsize::new(0);
        let bridge = Bridge::start_timed("test", move || {
            let round = rounds.fetch_add(1, Ordering::SeqCst);
            if round == 0 { return true; }
            arm_probe(None);
            if round == 1 {
                let probe = evidence.lock().unwrap().as_ref().unwrap().clone();
                let bridge = probe.bridge.upgrade().unwrap();
                let state = bridge.state.lock().unwrap();
                probe.old_raw_pointer.store(state.raw_summary.as_ref().unwrap().as_ptr() as usize, Ordering::SeqCst);
                probe.old_snapshot_pointer.store(state.current.as_deref().unwrap()["display_summary"]["trainings"]
                    .as_str().unwrap().as_ptr() as usize, Ordering::SeqCst);
                return true;
            }
            arm_probe(None); done_tx.send(()).unwrap(); false
        }, move || {
            arm_probe(None);
            if capture_probe.borrow().is_none() {
                *capture_probe.borrow_mut() = Some(probe_rx.recv_timeout(Duration::from_secs(2)).unwrap());
            }
            let raw = json!({"scenario":"Ramen","stats":{"vital":captured.fetch_add(1,Ordering::SeqCst)},
                "trainings":"x".repeat(256*1024)}).to_string();
            arm_probe(capture_probe.borrow().clone());
            Ok(raw)
        }, Duration::from_millis(1), Duration::from_millis(1)).unwrap();
        let probe = LockProbe::new(&bridge);
        *ready_probe.lock().unwrap() = Some(probe.clone()); probe_tx.send(probe.clone()).unwrap();
        let finished = done_rx.recv_timeout(Duration::from_secs(3));
        bridge.shutdown().unwrap(); finished.unwrap();
        assert_eq!(bridge.current().unwrap()["snapshot_id"], 2);
        assert!(probe.large_allocations.load(Ordering::SeqCst) > 0, "conversion was not exercised");
        assert!(probe.large_deallocations.load(Ordering::SeqCst) > 0, "old values were not destroyed");
        assert_eq!(probe.old_raw_drops.load(Ordering::SeqCst), 1, "old raw buffer was not destroyed");
        assert_eq!(probe.old_snapshot_drops.load(Ordering::SeqCst), 1, "old published buffer was not destroyed");
        probe.assert_unlocked();
    }

    #[test]
    fn boot_resume_does_not_reexpose_previous_attempt_during_capture() {
        let ready = Arc::new(AtomicBool::new(true)); let readiness = ready.clone();
        let captures = AtomicUsize::new(0);
        let (entered_tx, entered_rx) = mpsc::channel(); let (release_tx, release_rx) = mpsc::channel();
        let bridge = Bridge::start_timed("test", move || readiness.load(Ordering::SeqCst), move || {
            let capture = captures.fetch_add(1, Ordering::SeqCst);
            if capture == 1 { entered_tx.send(()).unwrap(); release_rx.recv_timeout(Duration::from_secs(3)).unwrap(); }
            Ok(json!({"stats":{"vital":70+capture}}).to_string())
        }, Duration::from_secs(60), Duration::from_millis(1)).unwrap();
        wait_for(|| bridge.current().is_some());
        let identity = bridge.capabilities()["collector_instance_id"].clone();
        ready.store(false, Ordering::SeqCst); bridge.wake();
        wait_for(|| bridge.status()["capture_state"] == "booting");
        ready.store(true, Ordering::SeqCst); bridge.wake();
        entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        // Capture all observations before releasing B; assertions cannot strand its worker.
        let during = bridge.http_snapshot(); let summary = bridge.summary();
        let sent = AtomicBool::new(false);
        let push = bridge.push_once(|snapshot| { sent.store(true,Ordering::SeqCst); Receipt::parse(&receipt_value(snapshot),snapshot) });
        release_tx.send(()).unwrap();
        wait_for(|| bridge.current().unwrap()["snapshot_id"] == 2); bridge.shutdown().unwrap();
        assert!(during.get("snapshot_id").is_none(), "boot-before A became externally visible before fresh capture B");
        assert!(summary.is_none()); assert_eq!(push.unwrap(),PushOutcome::NoSnapshot);
        assert!(!sent.load(Ordering::SeqCst));
        assert_eq!(bridge.capabilities()["collector_instance_id"],identity);
    }

    #[test]
    fn getter_clones_json_and_push_receipts_do_not_hold_state_lock() {
        let bridge = Bridge::start_timed("test", || true, || Ok(json!({"scenario":"Ramen",
            "trainings":"x".repeat(256*1024)}).to_string()), Duration::from_secs(60), Duration::from_millis(1)).unwrap();
        wait_for(|| bridge.current().is_some()); bridge.shutdown().unwrap();
        let probe = LockProbe::new(&bridge); arm_probe(Some(probe.clone()));
        let current = bridge.current().unwrap();
        assert_eq!(bridge.http_snapshot(), current);
        assert!(bridge.summary().unwrap().len() > 256*1024);
        assert_eq!(bridge.capabilities()["snapshot_schema_versions"], json!([2]));
        assert_eq!(bridge.status()["capture_state"], "incomplete");
        assert!(bridge.push_once(|_| Err("first transport failure".into())).is_err());
        bridge.push_once(|snapshot| Receipt::parse(&receipt_value(snapshot), snapshot)).unwrap();
        assert_eq!(bridge.push_once(|_| panic!("already acknowledged")).unwrap(), PushOutcome::AlreadyAcknowledged);
        arm_probe(None);
        assert!(probe.large_allocations.load(Ordering::SeqCst) > 0, "large clones were not exercised");
        probe.assert_unlocked();
    }

    /// Wait for a concrete state transition, failing instead of hanging the test process.
    fn wait_for(mut predicate: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while !predicate() {
            assert!(Instant::now() < deadline, "bridge condition timed out");
            thread::sleep(Duration::from_millis(2));
        }
    }

    /// Standard receipt, including explicit null run identity for incomplete real observations.
    fn receipt_value(snapshot: &Value) -> Value {
        json!({"status":"received","transport_received":true,"accepted_for_validation":true,"computed":false,
            "receipt_id":"test-receipt","schema_version":snapshot["schema_version"],
            "collector_instance_id":snapshot["collector_instance_id"],"run_id":snapshot["run_id"],"snapshot_id":snapshot["snapshot_id"]})
    }

    /// Create a deterministic partial publisher whose normal sampling interval does not race tests.
    fn partial_bridge() -> Arc<Bridge> {
        let bridge = Bridge::start_timed("test", || true, || Ok(r#"{"scenario":"Ramen","stats":{"vital":70}}"#.into()),
            Duration::from_secs(60), Duration::from_millis(1)).unwrap();
        wait_for(|| bridge.current().is_some());
        bridge
    }

    /// Read only the request needed by a loopback fake Android receiver.
    fn read_request(stream: &mut TcpStream) -> Value {
        stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut bytes = Vec::new();
        let (start, length) = loop {
            let mut chunk = [0u8; 2048];
            let count = stream.read(&mut chunk).unwrap();
            assert!(count > 0); bytes.extend_from_slice(&chunk[..count]);
            if let Some(end) = bytes.windows(4).position(|x| x == b"\r\n\r\n") {
                let header = std::str::from_utf8(&bytes[..end]).unwrap();
                let length: usize = header.lines().filter_map(|line| line.split_once(':'))
                    .find(|(name, _)| name.eq_ignore_ascii_case("content-length")).unwrap().1.trim().parse().unwrap();
                break (end + 4, length);
            }
        };
        while bytes.len() < start + length {
            let mut chunk = [0u8; 2048];
            let count = stream.read(&mut chunk).unwrap(); assert!(count > 0); bytes.extend_from_slice(&chunk[..count]);
        }
        serde_json::from_slice(&bytes[start..start + length]).unwrap()
    }

    #[test]
    fn getters_never_capture_and_native_capture_releases_publication_lock() {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let bridge = Bridge::start_timed("test", || true, move || {
            counter.fetch_add(1, Ordering::SeqCst); entered_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            Ok("{}".into())
        }, Duration::from_secs(60), Duration::from_millis(1)).unwrap();
        entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        for _ in 0..50 {
            assert!(bridge.current().is_none()); assert!(bridge.summary().is_none());
            assert_eq!(bridge.http_snapshot()["ready"], false);
            assert_eq!(bridge.status()["capture_state"], "capturing");
            assert_eq!(bridge.capabilities()["snapshot_schema_versions"], json!([2]));
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        release_tx.send(()).unwrap(); wait_for(|| bridge.current().is_some());
        let original = bridge.current().unwrap();
        for _ in 0..50 { assert_eq!(bridge.http_snapshot(), original); }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        bridge.shutdown().unwrap();
    }

    #[test]
    fn boot_then_resume_probes_without_sixty_second_backoff() {
        let ready = Arc::new(AtomicBool::new(false));
        let readiness = ready.clone();
        let calls = Arc::new(AtomicUsize::new(0)); let counter = calls.clone();
        let bridge = Bridge::start_timed("test", move || readiness.load(Ordering::SeqCst), move || {
            counter.fetch_add(1, Ordering::SeqCst); Ok(r#"{"error":"no_sm"}"#.into())
        }, Duration::from_millis(20), Duration::from_millis(1)).unwrap();
        thread::sleep(Duration::from_millis(30));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let status = bridge.http_snapshot();
        assert_eq!(status["schema_version"], 2); assert!(status.get("snapshot_id").is_none());
        assert!(status["collector_instance_id"].is_string());
        ready.store(true, Ordering::SeqCst); bridge.wake();
        wait_for(|| bridge.status()["capture_state"] == "waiting_for_run");
        assert_eq!(bridge.current().unwrap()["ready"], false);
        ready.store(false, Ordering::SeqCst); bridge.wake();
        wait_for(|| bridge.status()["capture_state"] == "booting");
        assert!(bridge.http_snapshot().get("snapshot_id").is_none());
        assert!(bridge.summary().is_none());
        assert_eq!(bridge.push_once(|_| panic!("booting must not send stale state")).unwrap(), PushOutcome::NoSnapshot);
        bridge.shutdown().unwrap();
    }

    #[test]
    fn app_starts_late_same_snapshot_retries_until_bound_receipt() {
        let bridge = partial_bridge();
        let original = bridge.current().unwrap();
        let reserved = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = reserved.local_addr().unwrap(); drop(reserved);
        assert!(bridge.push_once(|snapshot| post_snapshot(address, "/notify", snapshot, Duration::from_millis(150))).is_err());
        assert!(bridge.status()["last_ack_ms"].is_null());
        assert!(!bridge.status()["last_send_ms"].is_null());
        let receiver = TcpListener::bind(address).unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = receiver.accept().unwrap();
            let snapshot = read_request(&mut stream);
            let receipt = receipt_value(&snapshot).to_string();
            write!(stream, "HTTP/1.1 202 Accepted\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", receipt.len(), receipt).unwrap();
            snapshot
        });
        let result = bridge.push_once(|snapshot| {
            // This read would deadlock if push held the publication lock over transport I/O.
            assert_eq!(bridge.current().as_ref(), Some(snapshot));
            post_snapshot(address, "/notify", snapshot, Duration::from_secs(1))
        }).unwrap();
        assert!(matches!(result, PushOutcome::Acknowledged(Receipt { run_id: None, .. })));
        assert_eq!(server.join().unwrap(), original);
        assert_eq!(bridge.current().unwrap(), original);
        assert!(!bridge.status()["last_ack_ms"].is_null());
        assert_eq!(bridge.push_once(|_| panic!("acknowledged snapshot must not resend")).unwrap(), PushOutcome::AlreadyAcknowledged);
        bridge.shutdown().unwrap();
    }

    #[test]
    fn arbitrary_success_and_wrong_identity_do_not_acknowledge() {
        let bridge = partial_bridge();
        let snapshot = bridge.current().unwrap();
        assert!(Receipt::parse(&json!({"ok":true}), &snapshot).is_err());
        let mut wrong = receipt_value(&snapshot); wrong["collector_instance_id"] = json!("retired-process");
        assert!(Receipt::parse(&wrong, &snapshot).is_err());
        let mut missing = receipt_value(&snapshot); missing.as_object_mut().unwrap().remove("run_id");
        assert!(Receipt::parse(&missing, &snapshot).is_err());
        for code in [200, 204] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap(); let address = listener.local_addr().unwrap();
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap(); let snapshot = read_request(&mut stream);
                let body = receipt_value(&snapshot).to_string();
                let _ = write!(stream, "HTTP/1.1 {code} OK\r\nContent-Length: {}\r\n\r\n{}", body.len(), body);
            });
            assert!(bridge.push_once(|snapshot| post_snapshot(address, "/notify", snapshot, Duration::from_secs(1))).is_err());
            server.join().unwrap();
        }
        assert!(bridge.status()["last_ack_ms"].is_null()); bridge.shutdown().unwrap();
    }

    #[test]
    fn slow_drip_receipt_respects_absolute_deadline() {
        let bridge = partial_bridge(); let snapshot = bridge.current().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap(); let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap(); let _ = read_request(&mut stream);
            stream.write_all(b"HTTP/1.1 202 Accepted\r\nContent-Length: 1000\r\n\r\n").unwrap();
            for _ in 0..50 { if stream.write_all(b" ").is_err() { break; } thread::sleep(Duration::from_millis(10)); }
        });
        let started = Instant::now();
        assert!(post_snapshot(address, "/notify", &snapshot, Duration::from_millis(80)).is_err());
        assert!(started.elapsed() < Duration::from_millis(500));
        server.join().unwrap(); bridge.shutdown().unwrap();
    }

    #[test]
    fn failed_or_oversized_capture_remains_explicitly_incomplete() {
        let bridge = Bridge::start_timed("test", || true, || Err("guarded_read_failed".into()),
            Duration::from_secs(60), Duration::from_millis(1)).unwrap();
        wait_for(|| bridge.current().is_some());
        assert_eq!(bridge.status()["capture_state"], "error");
        assert_eq!(bridge.http_snapshot()["ready"], false);
        assert!(bridge.http_snapshot()["run_id"].is_null());
        assert!(bridge.status()["last_successful_capture_ms"].is_null());
        bridge.shutdown().unwrap();
        let oversized = Bridge::start_timed("test", || true, || Ok("x".repeat(MAX_SNAPSHOT_BYTES + 1)),
            Duration::from_secs(60), Duration::from_millis(1)).unwrap();
        wait_for(|| oversized.current().is_some());
        assert_eq!(oversized.status()["capture_error"], "capture_exceeds_byte_budget");
        assert_eq!(oversized.http_snapshot()["ready"], false);
        oversized.shutdown().unwrap();
    }

    #[test]
    fn older_ack_does_not_acknowledge_a_new_capture_published_during_send() {
        let counter = Arc::new(AtomicUsize::new(70)); let captures = counter.clone();
        let bridge = Bridge::start_timed("test", || true, move || {
            Ok(json!({"stats":{"vital":captures.fetch_add(1, Ordering::SeqCst)}}).to_string())
        }, Duration::from_secs(60), Duration::from_millis(1)).unwrap();
        wait_for(|| bridge.current().is_some());
        let original = bridge.current().unwrap();
        bridge.push_once(|snapshot| {
            bridge.wake(); wait_for(|| bridge.current().unwrap()["snapshot_id"] != snapshot["snapshot_id"]);
            Receipt::parse(&receipt_value(snapshot), snapshot)
        }).unwrap();
        assert_eq!(bridge.status()["last_ack"]["snapshot_id"], original["snapshot_id"]);
        let second = bridge.push_once(|snapshot| {
            assert_ne!(snapshot["snapshot_id"], original["snapshot_id"]);
            Receipt::parse(&receipt_value(snapshot), snapshot)
        }).unwrap();
        assert!(matches!(second, PushOutcome::Acknowledged(_)));
        bridge.shutdown().unwrap();
    }

    #[test]
    fn oversized_receipt_is_rejected_before_reading_declared_body() {
        let bridge = partial_bridge(); let snapshot = bridge.current().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap(); let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap(); let _ = read_request(&mut stream);
            let _ = stream.write_all(b"HTTP/1.1 202 Accepted\r\nContent-Length: 1000000\r\n\r\n");
        });
        let result = post_snapshot(address, "/notify", &snapshot, Duration::from_secs(1));
        assert!(result.err().is_some_and(|error| error == "invalid_receipt_length"));
        server.join().unwrap(); bridge.shutdown().unwrap();
    }

    #[test]
    fn late_ticket_cannot_renumber_or_replace_current_envelope() {
        let mut publisher = Publisher::new("sampler-test").unwrap();
        let old = publisher.begin_capture().unwrap(); let new = publisher.begin_capture().unwrap();
        let current = publisher.finish_summary(new, r#"{"stats":{"vital":80}}"#, "test", 100).unwrap().clone();
        assert!(publisher.finish_summary(old, r#"{"stats":{"vital":70}}"#, "test", 200).is_err());
        assert_eq!(publisher.current(), Some(&current));
        let fresh = publisher.begin_capture().unwrap();
        assert!(publisher.finish_capture(fresh, current.clone(), 300).is_err());
        assert_eq!(publisher.current(), Some(&current));
    }

    #[test]
    fn periodic_identical_capture_keeps_identity_and_original_timestamp() {
        let calls = Arc::new(AtomicUsize::new(0)); let capture_calls = calls.clone();
        let bridge = Bridge::start_timed("test", || true, move || {
            capture_calls.fetch_add(1, Ordering::SeqCst);
            Ok(r#"{"scenario":"Ramen","stats":{"vital":70}}"#.into())
        }, Duration::from_millis(20), Duration::from_millis(1)).unwrap();
        wait_for(|| bridge.current().is_some());
        let first = bridge.current().unwrap();
        wait_for(|| calls.load(Ordering::SeqCst) >= 3
            && bridge.status()["last_capture_ms"].as_u64() > first["captured_at_ms"].as_u64());
        assert_eq!(bridge.current(), Some(first.clone()));
        assert_eq!(bridge.http_snapshot(), first);
        assert_eq!(first["snapshot_id"], 1);
        bridge.shutdown().unwrap();
    }
}
