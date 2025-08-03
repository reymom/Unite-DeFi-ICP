use candid::{CandidType, Deserialize};
use evm_rpc_canister_types::{RpcApi, RpcServices};
use ic_cdk::management_canister::EcdsaKeyId;
use std::collections::HashMap;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct EvmChainConfig {
    pub rpc_urls: Vec<String>, // Primary + fallbacks
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct EvmConfig {
    pub ecdsa_key_id: EcdsaKeyId,             // ECDSA key_id
    pub chains: HashMap<u64, EvmChainConfig>, // chain_id → config
}

impl EvmConfig {
    pub fn rpc_services(&self, chain_id: u64) -> Result<RpcServices, String> {
        let chain = self
            .chains
            .get(&chain_id)
            .ok_or_else(|| format!("RPC config missing for chain {}", chain_id))?;

        let services = chain
            .rpc_urls
            .iter()
            .map(|url| RpcApi {
                url: url.clone(),
                headers: None,
            })
            .collect();

        Ok(RpcServices::Custom {
            chainId: chain_id,
            services,
        })
    }
}
