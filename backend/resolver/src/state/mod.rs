use std::cell::RefCell;

use shared::evm::signer::derive_evm_address;

use crate::state::config::{Config, EscrowHandles};

pub mod config;

thread_local! {
    pub static CONFIG: RefCell<Option<Config>> = RefCell::new(None);
    pub static ACTIVE_ESCROWS: RefCell<std::collections::BTreeMap<String, EscrowHandles>> = RefCell::new(Default::default());
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
