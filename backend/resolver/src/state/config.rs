use candid::{CandidType, Deserialize, Principal};
use shared::evm::config::EvmConfig;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AssetFilters {
    pub allow_assets: Vec<String>, // ERC-20 symbols or ICP canister ids
    pub deny_assets: Vec<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct TickConfig {
    pub max_slippage_bps: u16, // 100 = 1 %
    pub poll_interval_sec: u64,
    pub min_profit_icp: u64, // absolute ICP units
    pub filters: AssetFilters,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Config {
    pub relayer_id: Principal,
    pub factory_icp: Principal,
    pub evm: EvmConfig,
    pub automatic_tick: Option<TickConfig>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct EscrowHandles {
    pub evm_addr: String,
    pub icp_id: Principal,
    pub revealed: bool,
}
