mod error;
mod evm;
mod icp;
mod state;
mod tick;

use candid::Principal;
use ic_cdk::{api::canister_self, export_candid, init, post_upgrade, pre_upgrade, query, update};
use shared::evm::signer::derive_evm_address;

use crate::{
    icp::withdraw_icp,
    state::{
        ACTIVE_ESCROWS,
        config::{Config, EscrowHandles},
        get_config, initialize_config, mutate_config, spawn_check_evm_address,
    },
};

#[init]
fn init(cfg: Config) {
    initialize_config(cfg.clone());
    spawn_check_evm_address();
    if cfg.automatic_tick.is_some() {
        tick::spawn_tick();
    }
}

#[pre_upgrade]
fn pre_upgrade() {
    ic_cdk::storage::stable_save((get_config(),)).unwrap();
}

#[post_upgrade]
fn post_upgrade() {
    let (cfg,): (Config,) = ic_cdk::storage::stable_restore().unwrap();
    initialize_config(cfg);
    tick::spawn_tick();
}

#[query]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[query]
fn get_resolver_config() -> Config {
    get_config()
}

#[query]
async fn get_resolver_evm_address() -> String {
    derive_evm_address(get_config().evm.ecdsa_key_id).await
}

#[query]
async fn get_relayer_canister_id() -> Principal {
    get_config().relayer_id
}

#[update]
async fn set_relayer_canister_id(canister_id: Principal) {
    mutate_config(|cfg| cfg.relayer_id = canister_id)
}

#[update]
async fn deploy_evm_escrow(order_hash: String) -> Result<EscrowHandles, String> {
    // cheap check: already deployed?
    if let Some(h) = ACTIVE_ESCROWS.with(|m| m.borrow().get(&order_hash).cloned()) {
        return Ok(h);
    }

    // fetch auction info from relayer
    // let auc: AuctionInfo = Call::unbounded_wait(get_config().relayer_id, "get_auction")
    //     .with_arg(order_hash.clone())
    //     .await
    //     .map_err(|e| format!("{e:?}"))?
    //     .candid::<Result<AuctionInfo, String>>()
    //     .map_err(|e| e)
    //     .map_err(|e| e.to_string())??;

    // let resolver_principal = ic_cdk::api::canister_self();
    // let resolver_evm = derive_evm_address(get_config().evm.ecdsa_key_id).await;

    // // classify assets and deploy ICP escrow only (EVM mocked)
    // let (icp_params, _evm_params, _cid) =
    //     classify_assets(&auc, resolver_principal, &resolver_evm).map_err(|e| e.to_string())?;
    // let icp_id = create_icp_escrow(&icp_params)
    //     .await
    //     .map_err(|e| e.to_string())?;

    let handles = EscrowHandles {
        evm_addr: String::from("0xeC265Bec77B1dBd83a40C07DFda3396A36BFE30f"),
        icp_id: canister_self(),
        revealed: false,
    };
    ACTIVE_ESCROWS.with(|m| m.borrow_mut().insert(order_hash, handles.clone()));

    Ok(handles)
}

#[update]
async fn withdraw_icp_manual(order_hash: String, secret: Vec<u8>) -> Result<(), String> {
    let handles = ACTIVE_ESCROWS
        .with(|m| m.borrow().get(&order_hash).cloned())
        .ok_or("escrow not deployed")?;

    withdraw_icp(handles.icp_id, secret)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[update]
async fn withdraw_evm_manual(order_hash: String, _secret: Vec<u8>) -> Result<(), String> {
    // let handles = ACTIVE_ESCROWS
    //     .with(|m| m.borrow().get(&order_hash).cloned())
    //     .ok_or("escrow not deployed")?;

    // ⸺ Mocked call: just print & mark as done ⸺
    ic_cdk::println!(
        "[manual-evm-withdraw] escrow=0x13527f566c20645fb75d12ee30897902afce044f token=T1INCH amount=5_343_750_000  safety_deposit=500_000_000 secret=0xb0950b2960a7070bc50a3ded6fd65abf44058b32938d9302d63827a3b9694731"
    );

    ACTIVE_ESCROWS.with(|m| {
        if let Some(h) = m.borrow_mut().get_mut(&order_hash) {
            h.revealed = true;
        }
    });

    Ok(())
}

/// Emergency escape hatch – withdraw ICP escrow even if something stalled.
#[update]
async fn emergency_withdraw_icp(escrow_id: Principal, secret: Vec<u8>) -> Result<(), String> {
    icp::withdraw_icp(escrow_id, secret)
        .await
        .map_err(|e| e.to_string())
}

export_candid!();
