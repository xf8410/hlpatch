//! Bounded raw-observation writer. Reserve before copying game-owned bytes.
//! Submission never waits for channel capacity or disk I/O; short accounting
//! locks contain no writer callbacks. The caller must reserve the complete owned
//! payload size and the writer must release that payload before returning.
use std::collections::VecDeque;
use std::fmt;
use std::io;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{mpsc::{self, Receiver, SyncSender, TrySendError}, Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const MAX_DIAGNOSTICS: usize = 16;
pub const MAX_ERROR_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueError {
    EmptyCapture,
    OverBudget,
    QueueFull,
    Paused,
    Stopped,
    ForeignReservation,
    WriterError,
    WorkerPanicked,
    WorkerDisconnected,
    ReservationDropped,
    JobDropped,
}

impl QueueError {
    pub fn code(self) -> &'static str {
        match self {
            Self::EmptyCapture => "empty_capture",
            Self::OverBudget => "over_budget",
            Self::QueueFull => "queue_full",
            Self::Paused => "paused",
            Self::Stopped => "stopped",
            Self::ForeignReservation => "foreign_reservation",
            Self::WriterError => "writer_error",
            Self::WorkerPanicked => "worker_panicked",
            Self::WorkerDisconnected => "worker_disconnected",
            Self::ReservationDropped => "reservation_dropped",
            Self::JobDropped => "job_dropped",
        }
    }
}

impl fmt::Display for QueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.code()) }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: QueueError,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Jobs accepted by try_submit, excluding rejected submissions.
    pub admitted: u64,
    pub completed: u64,
    pub failed: u64,
    /// Accepted jobs not yet completed or failed; includes the active writer.
    pub pending: usize,
    /// All live reservation bytes, including jobs still being copied by callers.
    pub bytes: usize,
    pub peak_bytes: usize,
    pub reservations: usize,
    pub rejected: u64,
    pub abandoned_reservations: u64,
    pub paused: bool,
    pub stopped: bool,
    pub worker_alive: bool,
    /// Monotonic evidence of capture loss. Resume never clears it.
    pub gaps: u64,
    pub diagnostics: VecDeque<Diagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrainError {
    Timeout(Stats),
    /// No outstanding work remains, but at least one observation was lost.
    Incomplete(Stats),
}

struct Shared {
    max_bytes: usize,
    stats: Mutex<Stats>,
    changed: Condvar,
}

impl Shared {
    fn new(max_bytes: usize) -> Self {
        let mut stats = Stats::default();
        stats.worker_alive = true;
        stats.diagnostics = VecDeque::with_capacity(MAX_DIAGNOSTICS);
        Self { max_bytes, stats: Mutex::new(stats), changed: Condvar::new() }
    }
}

fn record_gap(stats: &mut Stats, kind: QueueError, message: &str) {
    stats.gaps = stats.gaps.saturating_add(1);
    let mut end = message.len().min(MAX_ERROR_BYTES);
    while !message.is_char_boundary(end) { end -= 1; }
    if stats.diagnostics.len() == MAX_DIAGNOSTICS { stats.diagnostics.pop_front(); }
    stats.diagnostics.push_back(Diagnostic { kind, message: message[..end].to_string() });
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage { Reserved, Submitted, Finalized }

/// Queue-bound and deliberately not Clone. Drop always returns the byte budget.
/// Dropping an unused reservation records a gap rather than claiming completeness.
pub struct Reservation {
    shared: Arc<Shared>,
    bytes: usize,
    stage: Stage,
}

impl Reservation {
    pub fn bytes(&self) -> usize { self.bytes }

    fn reject(&mut self, error: QueueError) {
        let mut stats = self.shared.stats.lock().unwrap_or_else(|e| e.into_inner());
        stats.rejected = stats.rejected.saturating_add(1);
        record_gap(&mut stats, error, error.code());
        self.stage = Stage::Finalized;
    }

    fn finish(&mut self, failure: Option<(QueueError, &str)>) {
        let mut stats = self.shared.stats.lock().unwrap_or_else(|e| e.into_inner());
        stats.pending -= 1;
        if let Some((kind, message)) = failure {
            stats.failed = stats.failed.saturating_add(1);
            stats.paused = true;
            if kind == QueueError::WorkerPanicked { stats.stopped = true; }
            record_gap(&mut stats, kind, message);
        } else {
            stats.completed = stats.completed.saturating_add(1);
        }
        self.stage = Stage::Finalized;
    }
}

impl Drop for Reservation {
    fn drop(&mut self) {
        let mut stats = self.shared.stats.lock().unwrap_or_else(|e| e.into_inner());
        stats.bytes -= self.bytes;
        stats.reservations -= 1;
        match self.stage {
            Stage::Reserved => {
                stats.abandoned_reservations = stats.abandoned_reservations.saturating_add(1);
                record_gap(&mut stats, QueueError::ReservationDropped, "reserved capture was abandoned before submission");
            }
            Stage::Submitted => {
                stats.pending -= 1;
                stats.failed = stats.failed.saturating_add(1);
                stats.paused = true;
                record_gap(&mut stats, QueueError::JobDropped, "accepted raw observation was not written");
            }
            Stage::Finalized => {}
        }
        self.shared.changed.notify_all();
    }
}

// Drop the owned payload before returning its byte reservation.
struct Envelope<T> { job: T, reservation: Reservation }

pub struct WriterQueue<T: Send + 'static> {
    shared: Arc<Shared>,
    sender: Mutex<Option<SyncSender<Envelope<T>>>>,
    // Dropping the queue closes admission but never joins on a game thread.
    _worker: JoinHandle<()>,
}

impl<T: Send + 'static> WriterQueue<T> {
    pub fn new<F>(max_bytes: usize, queue_capacity: usize, writer: F) -> io::Result<Self>
    where F: FnMut(T) -> Result<(), String> + Send + 'static {
        if max_bytes == 0 || queue_capacity == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "writer byte and queue capacities must be positive"));
        }
        let shared = Arc::new(Shared::new(max_bytes));
        let (sender, receiver) = mpsc::sync_channel(queue_capacity);
        let worker_shared = shared.clone();
        let worker = thread::Builder::new().name("ura-raw-writer".into()).spawn(move || {
            let outcome = catch_unwind(AssertUnwindSafe(|| run_worker(receiver, writer)));
            let mut stats = worker_shared.stats.lock().unwrap_or_else(|e| e.into_inner());
            if outcome.is_err() {
                stats.paused = true;
                record_gap(&mut stats, QueueError::WorkerPanicked, "writer loop terminated unexpectedly");
            }
            stats.worker_alive = false;
            stats.stopped = true;
            worker_shared.changed.notify_all();
        })?;
        Ok(Self { shared, sender: Mutex::new(Some(sender)), _worker: worker })
    }

    /// Call before allocating/copying the raw payload. A rejection records loss.
    pub fn reserve(&self, bytes: usize) -> Result<Reservation, QueueError> {
        let mut stats = self.shared.stats.lock().unwrap_or_else(|e| e.into_inner());
        let error = if stats.stopped || !stats.worker_alive { Some(QueueError::Stopped) }
            else if stats.paused { Some(QueueError::Paused) }
            else if bytes == 0 { Some(QueueError::EmptyCapture) }
            else if stats.bytes.checked_add(bytes).is_none_or(|n| n > self.shared.max_bytes) { Some(QueueError::OverBudget) }
            else { None };
        if let Some(error) = error {
            stats.rejected = stats.rejected.saturating_add(1);
            record_gap(&mut stats, error, error.code());
            return Err(error);
        }
        stats.bytes += bytes;
        stats.peak_bytes = stats.peak_bytes.max(stats.bytes);
        stats.reservations += 1;
        Ok(Reservation { shared: self.shared.clone(), bytes, stage: Stage::Reserved })
    }

    /// Non-waiting admission: full, paused, closed and disconnected queues reject
    /// explicitly. On every failure the payload and its reservation are dropped.
    pub fn try_submit(&self, mut reservation: Reservation, job: T) -> Result<(), QueueError> {
        if !Arc::ptr_eq(&reservation.shared, &self.shared) {
            reservation.reject(QueueError::ForeignReservation);
            drop(job);
            return Err(QueueError::ForeignReservation);
        }
        let mut stats = self.shared.stats.lock().unwrap_or_else(|e| e.into_inner());
        let error = if stats.stopped || !stats.worker_alive { Some(QueueError::Stopped) }
            else if stats.paused { Some(QueueError::Paused) } else { None };
        if let Some(error) = error {
            stats.rejected = stats.rejected.saturating_add(1);
            record_gap(&mut stats, error, error.code());
            reservation.stage = Stage::Finalized;
            drop(stats);
            drop(job);
            return Err(error);
        }
        reservation.stage = Stage::Submitted;
        stats.admitted = stats.admitted.saturating_add(1);
        stats.pending += 1;
        let envelope = Envelope { job, reservation };
        let send = self.sender.lock().unwrap_or_else(|e| e.into_inner());
        let result = match send.as_ref() {
            Some(sender) => sender.try_send(envelope),
            None => Err(TrySendError::Disconnected(envelope)),
        };
        match result {
            Ok(()) => Ok(()),
            Err(error) => {
                let (kind, mut rejected) = match error {
                    TrySendError::Full(envelope) => (QueueError::QueueFull, envelope),
                    TrySendError::Disconnected(envelope) => (QueueError::WorkerDisconnected, envelope),
                };
                stats.admitted -= 1;
                stats.pending -= 1;
                stats.rejected = stats.rejected.saturating_add(1);
                if kind == QueueError::WorkerDisconnected {
                    stats.paused = true;
                    stats.stopped = true;
                    stats.worker_alive = false;
                }
                record_gap(&mut stats, kind, kind.code());
                rejected.reservation.stage = Stage::Finalized;
                drop(send);
                drop(stats);
                drop(rejected);
                Err(kind)
            }
        }
    }

    pub fn stats(&self) -> Stats {
        self.shared.stats.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// HTTP/control-thread barrier only. Waits for caller-held reservations too.
    /// A drained queue with historical gaps is explicitly incomplete.
    pub fn drain(&self, timeout: Duration) -> Result<Stats, DrainError> {
        let started = Instant::now();
        let mut stats = self.shared.stats.lock().unwrap_or_else(|e| e.into_inner());
        while stats.reservations != 0 {
            let remaining = timeout.saturating_sub(started.elapsed());
            if remaining.is_zero() { return Err(DrainError::Timeout(stats.clone())); }
            let (next, _) = self.shared.changed.wait_timeout(stats, remaining).unwrap_or_else(|e| e.into_inner());
            stats = next;
        }
        if stats.gaps != 0 { Err(DrainError::Incomplete(stats.clone())) } else { Ok(stats.clone()) }
    }

    /// Explicitly resume after a writer error; all previous gap evidence remains.
    /// A panicked/disconnected worker requires a new queue, not a fake resume.
    pub fn resume(&self) -> Result<(), QueueError> {
        let mut stats = self.shared.stats.lock().unwrap_or_else(|e| e.into_inner());
        if stats.stopped || !stats.worker_alive { return Err(QueueError::Stopped); }
        stats.paused = false;
        Ok(())
    }

    /// Stop new captures, then let the worker finish every already accepted job.
    /// A blocking writer cannot safely be killed; drain reports its timeout.
    pub fn close(&self) {
        {
            let mut stats = self.shared.stats.lock().unwrap_or_else(|e| e.into_inner());
            stats.stopped = true;
        }
        let sender = self.sender.lock().unwrap_or_else(|e| e.into_inner()).take();
        drop(sender);
        self.shared.changed.notify_all();
    }
}

impl<T: Send + 'static> Drop for WriterQueue<T> {
    fn drop(&mut self) { self.close(); }
}

fn run_worker<T, F>(receiver: Receiver<Envelope<T>>, mut writer: F)
where T: Send + 'static, F: FnMut(T) -> Result<(), String> {
    while let Ok(envelope) = receiver.recv() {
        let Envelope { job, mut reservation } = envelope;
        match catch_unwind(AssertUnwindSafe(|| writer(job))) {
            Ok(Ok(())) => reservation.finish(None),
            Ok(Err(message)) => reservation.finish(Some((QueueError::WriterError, &message))),
            Err(_) => {
                reservation.finish(Some((QueueError::WorkerPanicked, "raw writer panicked; pending observations will be marked failed")));
                // Drop releases this job; receiver Drop explicitly fails all
                // accepted jobs still queued through their Reservation guards.
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::{Read, Write};
    use std::sync::{atomic::{AtomicUsize, Ordering}, Barrier};

    fn wait_until(mut predicate: impl FnMut() -> bool) {
        let until = Instant::now() + Duration::from_secs(3);
        while !predicate() {
            assert!(Instant::now() < until, "worker failed to make progress");
            thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn reserve_precedes_copy_and_drop_releases_budget_with_gap() {
        let queue = WriterQueue::<Vec<u8>>::new(8, 1, |_| Ok(())).unwrap();
        assert!(matches!(queue.reserve(9), Err(QueueError::OverBudget)));
        assert_eq!(queue.stats().bytes, 0);
        let reservation = queue.reserve(8).unwrap();
        assert_eq!(reservation.bytes(), 8);
        assert_eq!(queue.stats().bytes, 8);
        assert!(matches!(queue.reserve(1), Err(QueueError::OverBudget)));
        drop(reservation);
        let stats = queue.stats();
        assert_eq!((stats.bytes, stats.reservations, stats.abandoned_reservations, stats.gaps), (0, 0, 1, 3));
    }

    #[test]
    fn reservations_cannot_transfer_between_queues() {
        let first = WriterQueue::<Vec<u8>>::new(8, 1, |_| Ok(())).unwrap();
        let second = WriterQueue::<Vec<u8>>::new(8, 1, |_| Ok(())).unwrap();
        let reservation = first.reserve(8).unwrap();
        assert_eq!(second.try_submit(reservation, vec![1; 8]), Err(QueueError::ForeignReservation));
        assert_eq!((first.stats().bytes, second.stats().bytes), (0, 0));
        assert_eq!(first.stats().gaps, 1);
    }

    #[test]
    fn full_queue_rejects_immediately_and_releases_only_rejected_budget() {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let mut first = true;
        let queue = WriterQueue::new(3, 1, move |_: u8| {
            if first { first = false; entered_tx.send(()).unwrap(); release_rx.recv().unwrap(); }
            Ok(())
        }).unwrap();
        queue.try_submit(queue.reserve(1).unwrap(), 1).unwrap();
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        queue.try_submit(queue.reserve(1).unwrap(), 2).unwrap();
        assert_eq!(queue.try_submit(queue.reserve(1).unwrap(), 3), Err(QueueError::QueueFull));
        assert_eq!((queue.stats().pending, queue.stats().bytes), (2, 2));
        release_tx.send(()).unwrap();
        assert!(matches!(queue.drain(Duration::from_secs(2)), Err(DrainError::Incomplete(_))));
        let stats = queue.stats();
        assert_eq!((stats.admitted, stats.completed, stats.bytes, stats.gaps), (2, 2, 0, 1));
    }

    #[test]
    fn simultaneous_reservations_never_exceed_byte_budget() {
        let queue = Arc::new(WriterQueue::<()>::new(64, 1, |_| Ok(())).unwrap());
        let ready = Arc::new(Barrier::new(17));
        let release = Arc::new(Barrier::new(17));
        let joins: Vec<_> = (0..16).map(|_| {
            let (queue, ready, release) = (queue.clone(), ready.clone(), release.clone());
            thread::spawn(move || { let held = queue.reserve(8); ready.wait(); release.wait(); drop(held); })
        }).collect();
        ready.wait();
        assert_eq!((queue.stats().bytes, queue.stats().reservations, queue.stats().peak_bytes), (64, 8, 64));
        release.wait();
        for join in joins { join.join().unwrap(); }
        assert_eq!(queue.stats().bytes, 0);
    }

    #[test]
    fn write_error_pauses_new_capture_but_accepted_jobs_and_gap_history_survive() {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let queue = WriterQueue::new(3, 2, move |job: u8| {
            if job == 1 { entered_tx.send(()).unwrap(); release_rx.recv().unwrap(); return Err("disk full".into()); }
            Ok(())
        }).unwrap();
        queue.try_submit(queue.reserve(1).unwrap(), 1).unwrap();
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        queue.try_submit(queue.reserve(1).unwrap(), 2).unwrap();
        release_tx.send(()).unwrap();
        assert!(matches!(queue.drain(Duration::from_secs(2)), Err(DrainError::Incomplete(_))));
        assert!(queue.stats().paused);
        assert!(matches!(queue.reserve(1), Err(QueueError::Paused)));
        let old_gaps = queue.stats().gaps;
        queue.resume().unwrap();
        queue.try_submit(queue.reserve(1).unwrap(), 3).unwrap();
        assert!(matches!(queue.drain(Duration::from_secs(2)), Err(DrainError::Incomplete(_))));
        let stats = queue.stats();
        assert_eq!((stats.admitted, stats.completed, stats.failed, stats.gaps), (3, 2, 1, old_gaps));
    }

    #[test]
    fn panicked_worker_fails_and_releases_all_accepted_work() {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let queue = WriterQueue::new(2, 1, move |_: u8| {
            entered_tx.send(()).unwrap(); release_rx.recv().unwrap(); panic!("writer failure");
        }).unwrap();
        queue.try_submit(queue.reserve(1).unwrap(), 1).unwrap();
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        queue.try_submit(queue.reserve(1).unwrap(), 2).unwrap();
        release_tx.send(()).unwrap();
        assert!(matches!(queue.drain(Duration::from_secs(2)), Err(DrainError::Incomplete(_))));
        wait_until(|| !queue.stats().worker_alive);
        let stats = queue.stats();
        assert_eq!((stats.pending, stats.bytes, stats.failed, stats.gaps), (0, 0, 2, 2));
        assert_eq!(queue.resume(), Err(QueueError::Stopped));
    }

    #[test]
    fn disconnected_receiver_rejects_and_releases_reservation() {
        let (sender, receiver) = mpsc::sync_channel::<Envelope<u8>>(1);
        drop(receiver);
        let queue = WriterQueue { shared: Arc::new(Shared::new(8)), sender: Mutex::new(Some(sender)), _worker: thread::spawn(|| {}) };
        assert_eq!(queue.try_submit(queue.reserve(8).unwrap(), 1), Err(QueueError::WorkerDisconnected));
        let stats = queue.stats();
        assert_eq!((stats.admitted, stats.pending, stats.bytes, stats.gaps), (0, 0, 0, 1));
        assert!(stats.stopped && stats.paused && !stats.worker_alive);
    }

    #[test]
    fn close_and_drop_are_non_waiting_and_preserve_already_accepted_work() {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let completed = Arc::new(AtomicUsize::new(0));
        let count = completed.clone();
        let queue = WriterQueue::new(2, 1, move |_: u8| {
            entered_tx.send(()).unwrap(); release_rx.recv().unwrap(); count.fetch_add(1, Ordering::SeqCst); Ok(())
        }).unwrap();
        queue.try_submit(queue.reserve(1).unwrap(), 1).unwrap();
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let held = queue.reserve(1).unwrap();
        queue.close();
        assert_eq!(queue.try_submit(held, 2), Err(QueueError::Stopped));
        assert_eq!(queue.stats().bytes, 1);
        let evidence = queue.shared.clone();
        drop(queue);
        release_tx.send(()).unwrap();
        wait_until(|| !evidence.stats.lock().unwrap().worker_alive);
        assert_eq!(completed.load(Ordering::SeqCst), 1);
        assert_eq!(evidence.stats.lock().unwrap().bytes, 0);
    }

    #[test]
    fn drain_timeout_reports_live_bytes_until_writer_finishes() {
        let (release_tx, release_rx) = mpsc::channel();
        let queue = WriterQueue::new(8, 1, move |_: u8| { release_rx.recv().unwrap(); Ok(()) }).unwrap();
        queue.try_submit(queue.reserve(8).unwrap(), 1).unwrap();
        match queue.drain(Duration::from_millis(20)) {
            Err(DrainError::Timeout(stats)) => assert_eq!((stats.pending, stats.bytes), (1, 8)),
            other => panic!("unexpected flush result: {:?}", other),
        }
        release_tx.send(()).unwrap();
        let stats = queue.drain(Duration::from_secs(2)).unwrap();
        assert_eq!((stats.completed, stats.pending, stats.bytes), (1, 0, 0));
    }

    #[test]
    fn diagnostic_history_is_bounded_utf8_and_never_cleared_by_resume() {
        let queue = WriterQueue::new(8, 1, |_: u8| Err("界".repeat(4096))).unwrap();
        for _ in 0..32 {
            queue.resume().unwrap();
            queue.try_submit(queue.reserve(1).unwrap(), 1).unwrap();
            assert!(matches!(queue.drain(Duration::from_secs(2)), Err(DrainError::Incomplete(_))));
        }
        let before = queue.stats();
        assert_eq!((before.gaps, before.failed, before.diagnostics.len()), (32, 32, MAX_DIAGNOSTICS));
        assert!(before.diagnostics.iter().all(|d| d.message.len() <= MAX_ERROR_BYTES && !d.message.is_empty()));
        queue.resume().unwrap();
        assert_eq!(queue.stats().diagnostics, before.diagnostics);
        assert_eq!(queue.stats().gaps, 32);
    }

    fn complete_chunked_file(mebibytes: usize) {
        let folder = std::env::var_os("OBSERVATION_QUEUE_TEST_DIR").map(std::path::PathBuf::from).unwrap_or_else(std::env::temp_dir);
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let path = folder.join(format!("observation-{}-{}-{}.raw", std::process::id(), mebibytes, stamp));
        let mut file = OpenOptions::new().create_new(true).write(true).open(&path).unwrap();
        let queue = WriterQueue::new(256 * 1024, 2, move |chunk: Vec<u8>| file.write_all(&chunk).map_err(|e| e.to_string())).unwrap();
        let bytes = mebibytes * 1024 * 1024;
        let chunk_size = 64 * 1024;
        for start in (0..bytes).step_by(chunk_size) {
            let reservation = queue.reserve(chunk_size).unwrap();
            let chunk = (start..start + chunk_size).map(|i| (i % 251) as u8).collect();
            queue.try_submit(reservation, chunk).unwrap();
            queue.drain(Duration::from_secs(3)).unwrap();
        }
        queue.close();
        wait_until(|| !queue.stats().worker_alive);
        let stats = queue.stats();
        assert_eq!(stats.completed, (bytes / chunk_size) as u64);
        assert_eq!((stats.bytes, stats.gaps, stats.failed), (0, 0, 0));
        assert!(stats.peak_bytes <= 256 * 1024);
        assert_eq!(std::fs::metadata(&path).unwrap().len(), bytes as u64);
        let mut reader = std::fs::File::open(&path).unwrap();
        let mut buffer = vec![0u8; chunk_size];
        for start in (0..bytes).step_by(chunk_size) {
            reader.read_exact(&mut buffer).unwrap();
            assert!(buffer.iter().enumerate().all(|(i, byte)| *byte == ((start + i) % 251) as u8));
        }
        drop(reader);
        std::fs::remove_file(path).unwrap();
    }

    #[test] fn one_mib_raw_is_written_completely_in_bounded_chunks() { complete_chunked_file(1); }
    #[test] fn sixteen_mib_raw_is_written_completely_in_bounded_chunks() { complete_chunked_file(16); }
    #[test] fn sixty_four_mib_raw_is_written_completely_in_bounded_chunks() { complete_chunked_file(64); }
}
