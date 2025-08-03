use candid::{CandidType, Deserialize, Principal};
use shared::evm::config::EvmConfig;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Config {
    pub evm: EvmConfig,
    pub orderbook_id: Principal,
}
