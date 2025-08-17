mod error;
mod evm;
mod icp;
mod state;
mod tick;

use candid::Principal;
use ic_cdk::{
    api::canister_self, call::Call, export_candid, init, post_upgrade, pre_upgrade, query, update,
};
use shared::{
    Asset, AuctionInfo, EscrowParams, evm::signer::derive_evm_address, make_dst_params,
    make_src_params,
};

use crate::{
    evm::escrow::withdraw_escrow,
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
    let auc: AuctionInfo = Call::unbounded_wait(get_config().relayer_id, "get_auction")
        .with_arg(order_hash.clone())
        .await
        .map_err(|e| format!("{e:?}"))?
        .candid::<Result<AuctionInfo, String>>()
        .map_err(|e| e)
        .map_err(|e| e.to_string())??;

    let resolver_principal = ic_cdk::api::canister_self();
    let resolver_evm = derive_evm_address(get_config().evm.ecdsa_key_id).await;

    // Build params for both legs and classify
    let dst_params = make_dst_params(&auc.order, resolver_principal, &resolver_evm);
    let src_params = make_src_params(
        &auc.order,
        auc.current_price,
        resolver_principal,
        &resolver_evm,
    );
    let (icp_params, evm_params, chain_id) =
        classify_one_icp_one_evm(&src_params, &dst_params).map_err(|e| format!("{e}"))?;

    // 1) Create ICP escrow via factory
    let icp_id = icp::create_icp_escrow(&icp_params)
        .await
        .map_err(|e| e.to_string())?;

    // 2) EVM calldata (same as tick::prepare_evm_calldata)
    let evm_calldata = tick::prepare_evm_calldata(&auc, &evm_params).map_err(|e| e.to_string())?;

    // 3) Deploy via your existing helper → returns deployed clone address (0x-hex)
    let evm_addr = evm::deploy::deploy_evm_escrow(chain_id, evm_calldata)
        .await
        .map_err(|e| e.to_string())?;

    let handles = EscrowHandles {
        evm_addr: evm_addr.clone(),
        icp_id,
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
async fn withdraw_evm_manual(order_hash: String, secret: Vec<u8>) -> Result<(), String> {
    // load handles & auction to rebuild EVM params
    let handles = ACTIVE_ESCROWS
        .with(|m| m.borrow().get(&order_hash).cloned())
        .ok_or("escrow not deployed")?;

    let relayer = get_config().relayer_id;
    let auc_opt: Option<AuctionInfo> = Call::unbounded_wait(relayer, "get_active_auction")
        .with_arg(order_hash.clone())
        .await
        .map_err(|e| format!("get_active_auction: {e:?}"))?
        .candid()
        .map_err(|e| format!("decode: {e:?}"))?;
    let auc = if let Some(a) = auc_opt {
        a
    } else {
        Call::unbounded_wait(relayer, "get_finished_auction")
            .with_arg(order_hash.clone())
            .await
            .map_err(|e| format!("get_finished_auction: {e:?}"))?
            .candid::<Option<AuctionInfo>>()
            .map_err(|e| format!("decode: {e:?}"))?
            .ok_or("auction not found")?
    };

    let resolver_principal = canister_self();
    let resolver_evm = derive_evm_address(get_config().evm.ecdsa_key_id).await;

    let dst_params = make_dst_params(&auc.order, resolver_principal, &resolver_evm);
    let src_params = make_src_params(
        &auc.order,
        auc.current_price,
        resolver_principal,
        &resolver_evm,
    );
    let (_icp_params, evm_params, chain_id) =
        classify_one_icp_one_evm(&src_params, &dst_params).map_err(|e| format!("{e}"))?;

    withdraw_escrow(
        chain_id,
        &handles.evm_addr,
        secret,
        &evm_params,
        &auc.order.order_hash,
    )
    .await
    .map_err(|e| e.to_string())?;

    ACTIVE_ESCROWS.with(|m| {
        if let Some(h) = m.borrow_mut().get_mut(&order_hash) {
            h.revealed = true;
        }
    });
    Ok(())
}

fn classify_one_icp_one_evm<'a>(
    a: &'a EscrowParams,
    b: &'a EscrowParams,
) -> Result<(&'a EscrowParams, &'a EscrowParams, u64), String> {
    match (&a.asset, &b.asset) {
        (Asset::Erc20 { chain_id, .. }, Asset::ICP)
        | (Asset::Erc20 { chain_id, .. }, Asset::ICRC(_)) => Ok((b, a, *chain_id)),
        (Asset::ICP, Asset::Erc20 { chain_id, .. })
        | (Asset::ICRC(_), Asset::Erc20 { chain_id, .. }) => Ok((a, b, *chain_id)),
        _ => Err("swap must involve exactly one EVM and one ICP/ICRC asset".into()),
    }
}

/// Emergency escape hatch – withdraw ICP escrow even if something stalled.
#[update]
async fn emergency_withdraw_icp(escrow_id: Principal, secret: Vec<u8>) -> Result<(), String> {
    icp::withdraw_icp(escrow_id, secret)
        .await
        .map_err(|e| e.to_string())
}

export_candid!();
