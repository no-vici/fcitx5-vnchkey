//! Sentinel Barrier (N+1 Protocol) and Tri-Layer Race Condition Prevention.
//!
//! Prevents letter swallowing and double characters in asynchronous Web/Electron
//! event queues (VS Code, Chrome, Slack, Discord).

/// Barrier status after key event dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarrierStatus {
    /// Idle - safe to proceed directly
    Idle,
    /// Barrier armed with pending backspace sequence
    Armed(usize),
    /// Flushed and committed
    Flushed,
}

/// Sentinel Barrier implementation.
#[derive(Debug, Clone)]
pub struct SentinelBarrier {
    status: BarrierStatus,
    watchdog_timeout_ms: u64,
}

impl SentinelBarrier {
    /// Creates a new SentinelBarrier with 200ms default watchdog timeout.
    pub fn new() -> Self {
        Self {
            status: BarrierStatus::Idle,
            watchdog_timeout_ms: 200,
        }
    }

    /// Resets barrier state.
    pub fn reset(&mut self) {
        self.status = BarrierStatus::Idle;
    }

    /// Arms the barrier for N backspaces before emitting new glyphs.
    pub fn arm(&mut self, backspace_count: usize) {
        if backspace_count > 0 {
            self.status = BarrierStatus::Armed(backspace_count);
        } else {
            self.status = BarrierStatus::Idle;
        }
    }

    /// Releases the barrier.
    pub fn release(&mut self) {
        self.status = BarrierStatus::Flushed;
    }

    /// Current status.
    pub fn status(&self) -> BarrierStatus {
        self.status
    }

    /// Configured watchdog timeout in milliseconds.
    pub fn watchdog_timeout_ms(&self) -> u64 {
        self.watchdog_timeout_ms
    }
}

impl Default for SentinelBarrier {
    fn default() -> Self {
        Self::new()
    }
}
