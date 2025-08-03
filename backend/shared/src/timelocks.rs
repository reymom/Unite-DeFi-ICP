use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct Timelocks {
    pub finality_lock: u64,               // seconds after deployed_at
    pub withdrawal: u64,                  // after finality
    pub public_withdrawal: u64,           // after finality
    pub cancellation: u64,                // after finality
    pub public_cancellation: Option<u64>, // None if Dst escrow, Some if Src escrow
}

impl Timelocks {
    /// Return the timestamp for finality lock period start (absolute, given deployed_at)
    pub fn finality_start(&self, deployed_at: u64) -> u64 {
        deployed_at + self.finality_lock
    }

    /// Return the absolute timestamp for withdrawal window start
    pub fn withdrawal_start(&self, deployed_at: u64) -> u64 {
        self.finality_start(deployed_at) + self.withdrawal
    }

    /// Return the absolute timestamp for public withdrawal window start
    pub fn public_withdrawal_start(&self, deployed_at: u64) -> u64 {
        self.finality_start(deployed_at) + self.public_withdrawal
    }

    /// Return the absolute timestamp for cancellation window start
    pub fn cancellation_start(&self, deployed_at: u64) -> u64 {
        self.finality_start(deployed_at) + self.cancellation
    }

    /// Return the absolute timestamp for public cancellation window start
    pub fn public_cancellation_start(&self, deployed_at: u64) -> Option<u64> {
        self.public_cancellation
            .map(|pc| self.finality_start(deployed_at) + pc)
    }

    /// Utility to check if now is in private withdrawal window (inclusive start, exclusive end)
    pub fn in_private_withdrawal_window(
        &self,
        now: u64,
        deployed_at: u64,
        cancellation_start: u64,
    ) -> bool {
        let start = self.withdrawal_start(deployed_at);
        now >= start && now < cancellation_start
    }

    /// Utility to check if now is in public withdrawal window
    pub fn in_public_withdrawal_window(
        &self,
        now: u64,
        deployed_at: u64,
        cancellation_start: u64,
    ) -> bool {
        let start = self.public_withdrawal_start(deployed_at);
        now >= start && now < cancellation_start
    }

    /// Utility to check if now is in cancellation window (private)
    pub fn in_private_cancellation_window(&self, now: u64, deployed_at: u64) -> bool {
        now >= self.cancellation_start(deployed_at)
    }

    /// Utility to check if now is in public cancellation window
    pub fn in_public_cancellation_window(&self, now: u64, deployed_at: u64) -> Option<bool> {
        self.public_cancellation_start(deployed_at)
            .map(|pc| now >= pc)
    }

    pub fn is_finality_passed(&self, now: u64, deployed_at: u64) -> bool {
        now >= self.finality_start(deployed_at)
    }
}
