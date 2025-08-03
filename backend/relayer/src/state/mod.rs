use shared::AuctionInfo;
use shared::evm::signer::derive_evm_address;
use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::state::config::Config;

pub mod auctions;
pub mod config;
pub mod secret;

thread_local! {
    pub static CONFIG: RefCell<Option<Config>> = RefCell::new(None);
    pub static LAST_PROCESSED_ORDER_ID: RefCell<u64> = RefCell::new(0);
    pub static ORDER_SECRETS: RefCell<BTreeMap<String, Vec<u8>>> = RefCell::new(BTreeMap::new());
    pub static ACTIVE_AUCTIONS: RefCell<BTreeMap<String, AuctionInfo>> = RefCell::new(BTreeMap::new());
    pub static FINISHED_AUCTIONS: RefCell<BTreeMap<String, AuctionInfo>> = RefCell::new(BTreeMap::new());
    // for legal compliance, we can add later a registry of kyc'ed resolvers as 1inch does for its network
    // pub static RESOLVER_REGISTRY: RefCell<BTreeMap<Principal, String>> = RefCell::new(BTreeMap::new());
}

/// Mutates (part of) the current config using `f`.
///
/// Panics if there is no config.
pub fn mutate_config<F, R>(f: F) -> R
where
    F: FnOnce(&mut Config) -> R,
{
    CONFIG.with_borrow_mut(|s| f(s.as_mut().expect("BUG: config is not initialized")))
}

pub fn initialize_config(config: Config) {
    CONFIG.set(Some(config));
}

pub fn get_config() -> Config {
    CONFIG.with_borrow(|cfg| {
        cfg.as_ref()
            .expect("BUG: config is not initialized")
            .clone()
    })
}

pub fn spawn_check_evm_address() {
    ic_cdk_timers::set_timer(std::time::Duration::ZERO, || {
        ic_cdk::futures::spawn(async {
            let cfg = get_config();
            let evm_addr = derive_evm_address(cfg.evm.ecdsa_key_id.clone()).await;
            ic_cdk::println!("Resolver EVM address: {}", evm_addr);
        });
    });
}
