//! Vim-style motions

use std::{
    time::{Duration, Instant},
    usize,
};

/// How long a lone `g` waits
const G_TIMEOUT: Duration = Duration::from_millis(500);

/// Upper bound on an accumulated count
const MAX_COUNT: usize = 9999;

/// Tracks the numeric count prefix and the pending `g` for `gg`.
#[derive(Debug, Default)]
pub struct InputState {
    count: Option<usize>,
    pending_g: Option<Instant>,
}

impl InputState {
    /// Append a typed digit to the pending count
    pub fn push_digit(&mut self, digit: u32) {
        let next = self
            .count
            .unwrap_or(0)
            .saturating_mul(10)
            .saturating_add(digit as usize)
            .min(MAX_COUNT);
        self.count = Some(next);
        self.pending_g = None;
    }

    /// Take the pending count, resolving to 1 when none was typed
    pub fn take_count(&mut self) -> usize {
        self.count.take().unwrap_or(1).max(1)
    }

    /// The pending count, if any.
    pub fn pending_count(&self) -> Option<usize> {
        self.count
    }

    /// Records a `g` keypress and reports whether it completes a `gg` within the timeout.
    pub fn register_g(&mut self) -> bool {
        let now = Instant::now();
        match self.pending_g {
            Some(first) if now.duration_since(first) < G_TIMEOUT => {
                self.pending_g = None;
                true
            }
            _ => {
                self.pending_g = Some(now);
                false
            }
        }
    }

    pub fn reset(&mut self) {
        self.count = None;
        self.pending_g = None;
    }
}
