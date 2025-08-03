mod escrow;
pub mod evm;
mod icp;
mod order;
mod params;
mod timelocks;

pub use escrow::*;
pub use icp::*;
pub use order::*;
pub use params::*;
pub use timelocks::*;

pub fn now_sec() -> u64 {
    ic_cdk::api::time() / 1_000_000_000
}
