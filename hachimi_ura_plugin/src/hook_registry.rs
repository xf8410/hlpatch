//! Host-independent hook ownership. Native installation requires a caller that
//! can verify rollback by comparing the original target bytes, not merely the
//! host API's return address. Callback lookups never acquire the install lock.
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Registration {
    pub target: usize,
    pub handler: usize,
    pub trampoline: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistryError {
    InvalidAddress,
    TargetOwned,
    HandlerOwned,
    RollbackUnsupported,
    Quarantined,
    InstallFailed,
    InvalidTrampoline,
    LookupMismatch,
    NativePanic,
    RollbackUnverified,
}

impl RegistryError {
    pub fn code(self) -> &'static str {
        match self {
            Self::InvalidAddress => "invalid_address",
            Self::TargetOwned => "target_owned",
            Self::HandlerOwned => "handler_owned",
            Self::RollbackUnsupported => "rollback_unsupported",
            Self::Quarantined => "quarantined",
            Self::InstallFailed => "install_failed",
            Self::InvalidTrampoline => "invalid_trampoline",
            Self::LookupMismatch => "lookup_mismatch",
            Self::NativePanic => "native_panic",
            Self::RollbackUnverified => "rollback_unverified",
        }
    }
}

impl fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Diagnostics {
    pub registered: usize,
    pub quarantined_targets: usize,
    pub install_attempts: u64,
    pub installed: u64,
    pub idempotent: u64,
    pub conflicts: u64,
    pub unsupported: u64,
    pub failures: u64,
    pub rollbacks_verified: u64,
    pub rollbacks_failed: u64,
    pub last_error: Option<RegistryError>,
}

#[derive(Default)]
struct State {
    by_handler: HashMap<usize, Registration>,
    by_target: HashMap<usize, usize>,
    quarantined_targets: HashSet<usize>,
    quarantined_handlers: HashSet<usize>,
    diagnostics: Diagnostics,
}

#[derive(Default)]
pub struct HookRegistry {
    install_lock: Mutex<()>,
    state: Mutex<State>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// The handler is the host interceptor's lookup key, never the target.
    /// During native installation this can return None; callers must not invent
    /// a trampoline or call a patched target recursively to fill that gap.
    pub fn lookup(&self, handler: usize) -> Option<Registration> {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).by_handler.get(&handler).copied()
    }

    pub fn diagnostics(&self) -> Diagnostics {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let mut result = state.diagnostics.clone();
        result.registered = state.by_handler.len();
        result.quarantined_targets = state.quarantined_targets.len();
        result
    }

    /// Serialize native patching without holding the entries lock. The caller's
    /// preflight must establish that rollback_verified can safely restore and
    /// compare the original prologue. If unavailable, no native call is made.
    ///
    /// install returns the host trampoline; lookup independently confirms it.
    /// rollback_verified must check target bytes even when install returned 0
    /// or panicked: an API failure does not prove that no native patch occurred.
    pub fn install<I, L, R>(
        &self,
        target: usize,
        handler: usize,
        rollback_available: bool,
        install: I,
        lookup: L,
        rollback_verified: R,
    ) -> Result<Registration, RegistryError>
    where
        I: FnOnce() -> usize,
        L: FnOnce() -> usize,
        R: FnOnce() -> bool,
    {
        let _installation = self.install_lock.lock().unwrap_or_else(|e| e.into_inner());
        {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            let error = if target == 0 || handler == 0 || target == handler {
                Some(RegistryError::InvalidAddress)
            } else if state.quarantined_targets.contains(&target) || state.quarantined_handlers.contains(&handler) {
                Some(RegistryError::Quarantined)
            } else if let Some(existing) = state.by_handler.get(&handler).copied() {
                if existing.target == target {
                    state.diagnostics.idempotent += 1;
                    return Ok(existing);
                }
                state.diagnostics.conflicts += 1;
                Some(RegistryError::HandlerOwned)
            } else if state.by_target.contains_key(&target) {
                state.diagnostics.conflicts += 1;
                Some(RegistryError::TargetOwned)
            } else if !rollback_available {
                state.diagnostics.unsupported += 1;
                Some(RegistryError::RollbackUnsupported)
            } else {
                None
            };
            if let Some(error) = error {
                state.diagnostics.last_error = Some(error);
                return Err(error);
            }
            state.diagnostics.install_attempts += 1;
        }

        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let trampoline = install();
            if trampoline == 0 {
                return Err(RegistryError::InstallFailed);
            }
            if trampoline == target || trampoline == handler {
                return Err(RegistryError::InvalidTrampoline);
            }
            if lookup() != trampoline {
                return Err(RegistryError::LookupMismatch);
            }
            Ok(Registration { target, handler, trampoline })
        })).unwrap_or(Err(RegistryError::NativePanic));

        match outcome {
            Ok(registration) => {
                let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
                state.by_handler.insert(handler, registration);
                state.by_target.insert(target, handler);
                state.diagnostics.installed += 1;
                Ok(registration)
            }
            Err(error) => {
                let restored = catch_unwind(AssertUnwindSafe(rollback_verified)).unwrap_or(false);
                let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
                state.diagnostics.failures += 1;
                if restored {
                    state.diagnostics.rollbacks_verified += 1;
                    state.diagnostics.last_error = Some(error);
                    Err(error)
                } else {
                    state.quarantined_targets.insert(target);
                    state.quarantined_handlers.insert(handler);
                    state.diagnostics.rollbacks_failed += 1;
                    state.diagnostics.last_error = Some(RegistryError::RollbackUnverified);
                    Err(RegistryError::RollbackUnverified)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier, atomic::{AtomicUsize, Ordering}};
    use std::thread;

    #[test]
    fn same_pair_is_idempotent_and_both_ownership_conflicts_are_rejected() {
        let registry = HookRegistry::new();
        assert_eq!(registry.install(10, 20, true, || 30, || 30, || false).unwrap().trampoline, 30);
        assert_eq!(registry.install(10, 20, false, || panic!("duplicate install"), || 0, || false).unwrap().trampoline, 30);
        assert_eq!(registry.install(10, 21, true, || panic!("target stolen"), || 0, || false), Err(RegistryError::TargetOwned));
        assert_eq!(registry.install(11, 20, true, || panic!("handler rebound"), || 0, || false), Err(RegistryError::HandlerOwned));
        let d = registry.diagnostics();
        assert_eq!((d.registered, d.installed, d.idempotent, d.conflicts), (1, 1, 1, 2));
    }

    #[test]
    fn unavailable_rollback_rejects_before_touching_native_code() {
        let registry = HookRegistry::new();
        assert_eq!(registry.install(10, 20, false, || panic!("must not install"), || panic!("must not query"), || panic!("must not rollback")), Err(RegistryError::RollbackUnsupported));
        assert_eq!(registry.diagnostics().install_attempts, 0);
        assert_eq!(registry.diagnostics().unsupported, 1);
    }

    #[test]
    fn callback_can_query_during_native_install_without_entries_deadlock() {
        let registry = Arc::new(HookRegistry::new());
        let lookup_registry = registry.clone();
        let result = registry.install(10, 20, true, || {
            assert_eq!(registry.lookup(20), None);
            let (tx, rx) = std::sync::mpsc::channel();
            thread::spawn(move || tx.send(lookup_registry.lookup(20)).unwrap());
            assert_eq!(rx.recv_timeout(std::time::Duration::from_secs(1)).unwrap(), None);
            30
        }, || 30, || false);
        assert!(result.is_ok());
        assert_eq!(registry.lookup(20).unwrap().trampoline, 30);
    }

    #[test]
    fn simultaneous_installations_patch_only_once() {
        let registry = Arc::new(HookRegistry::new());
        let barrier = Arc::new(Barrier::new(8));
        let calls = Arc::new(AtomicUsize::new(0));
        let joins: Vec<_> = (0..8).map(|_| {
            let (registry, barrier, calls) = (registry.clone(), barrier.clone(), calls.clone());
            thread::spawn(move || {
                barrier.wait();
                registry.install(10, 20, true, || { calls.fetch_add(1, Ordering::SeqCst); 30 }, || 30, || false).unwrap()
            })
        }).collect();
        for join in joins { assert_eq!(join.join().unwrap().trampoline, 30); }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(registry.diagnostics().idempotent, 7);
    }

    #[test]
    fn failed_install_and_mismatched_lookup_do_not_publish_and_can_retry_after_verified_rollback() {
        let registry = HookRegistry::new();
        let rollbacks = AtomicUsize::new(0);
        assert_eq!(registry.install(10, 20, true, || 0, || panic!("zero result must not query"), || { rollbacks.fetch_add(1, Ordering::SeqCst); true }), Err(RegistryError::InstallFailed));
        assert_eq!(registry.lookup(20), None);
        assert_eq!(registry.install(10, 20, true, || 30, || 31, || { rollbacks.fetch_add(1, Ordering::SeqCst); true }), Err(RegistryError::LookupMismatch));
        assert_eq!(registry.lookup(20), None);
        assert!(registry.install(10, 20, true, || 30, || 30, || false).is_ok());
        assert_eq!(rollbacks.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn unverified_rollback_quarantines_target_and_handler_and_never_retries() {
        let registry = HookRegistry::new();
        assert_eq!(registry.install(10, 20, true, || 30, || 0, || false), Err(RegistryError::RollbackUnverified));
        assert_eq!(registry.lookup(20), None);
        for (target, handler) in [(10, 20), (10, 21), (11, 20)] {
            assert_eq!(registry.install(target, handler, true, || panic!("quarantined"), || 0, || false), Err(RegistryError::Quarantined));
        }
        let d = registry.diagnostics();
        assert_eq!((d.registered, d.quarantined_targets, d.rollbacks_failed), (0, 1, 1));
    }

    #[test]
    fn native_panic_rolls_back_and_does_not_poison_later_install() {
        let registry = HookRegistry::new();
        assert_eq!(registry.install(10, 20, true, || panic!("native adapter"), || 0, || true), Err(RegistryError::NativePanic));
        assert!(registry.install(10, 20, true, || 30, || 30, || false).is_ok());
    }

    #[test]
    fn rollback_panic_is_quarantined_and_invalid_addresses_do_not_install() {
        let registry = HookRegistry::new();
        for (target, handler) in [(0, 20), (10, 0), (10, 10)] {
            assert_eq!(registry.install(target, handler, true, || panic!("invalid"), || 0, || false), Err(RegistryError::InvalidAddress));
        }
        assert_eq!(registry.install(10, 20, true, || 0, || 0, || panic!("rollback")), Err(RegistryError::RollbackUnverified));
    }

    #[test]
    fn recursive_trampoline_addresses_are_rolled_back_without_publication() {
        let registry = HookRegistry::new();
        for trampoline in [10, 20] {
            assert_eq!(registry.install(10, 20, true, || trampoline, || panic!("invalid trampoline"), || true), Err(RegistryError::InvalidTrampoline));
            assert_eq!(registry.lookup(20), None);
        }
        assert_eq!(registry.diagnostics().rollbacks_verified, 2);
    }
}
