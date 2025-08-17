mod auction;
mod state;
mod tick;
mod verifier;

use candid::Principal;
use ethers_core::utils::{hex, keccak256};
// use ethers_core::utils::{hex, keccak256};
use ic_cdk::{
    api::msg_caller, call::Call, export_candid, init, post_upgrade, pre_upgrade, query, update,
};
use shared::{Asset, AuctionInfo, Order, make_dst_params, make_src_params, now_sec};

use crate::{
    state::{
        ACTIVE_AUCTIONS, FINISHED_AUCTIONS, LAST_PROCESSED_ORDER_ID,
        config::Config,
        get_config, initialize_config, mutate_config,
        secret::{self, store_secret},
        spawn_check_evm_address,
    },
    tick::spawn_tick,
    verifier::{evm, icp},
};

#[init]
fn init(cfg: Config) {
    initialize_config(cfg);
    spawn_check_evm_address();
    spawn_tick();

    let config = get_config();
    ic_cdk::println!("[init] config = {:?}", config);
}

#[pre_upgrade]
fn pre_upgrade() {
    let cursor = LAST_PROCESSED_ORDER_ID.with_borrow(|c| c.clone());
    let (active, finished) = state::auctions::export_auctions();
    ic_cdk::storage::stable_save((get_config(), cursor, active, finished)).unwrap();
}

#[post_upgrade]
fn post_upgrade() {
    let (config, cursor, active, finished): (
        Config,
        u64,
        Vec<(String, AuctionInfo)>,
        Vec<(String, AuctionInfo)>,
    ) = ic_cdk::storage::stable_restore().unwrap();
    initialize_config(config);
    LAST_PROCESSED_ORDER_ID.with_borrow_mut(|c| *c = cursor);
    state::auctions::import_auctions((active, finished));
    spawn_tick();
}

#[update]
fn set_orderbook_canister_id(canister_id: Principal) {
    mutate_config(|cfg| cfg.orderbook_id = canister_id)
}

#[query]
fn get_orderbook_canister_id() -> Principal {
    get_config().orderbook_id
}

/// Front-end posts the freshly-signed order **and** its 32-byte secret.
/// Relayer validates, persists to Orderbook and starts the auction.
#[update]
async fn submit_order(order: Order, secret: Vec<u8>) -> Result<(), String> {
    ic_cdk::println!("[submit_order]");
    // quick sanity
    if secret.len() != 32 {
        return Err("secret must be 32 bytes".into());
    }
    if keccak256(&secret) != order.hashlock {
        return Err("secret does not match hashlock".into());
    }

    // persist in Orderbook (cross-canister call)
    let orderbook_id = get_config().orderbook_id;

    ic_cdk::println!("orderbook_id: {:?}", orderbook_id);

    let add_order_result: Result<(), String> = Call::unbounded_wait(orderbook_id, "add_order")
        .with_arg(order.clone())
        .with_cycles(3_000_000_000)
        .await
        .map_err(|e| format!("Orderbook.add_order call failed: {e:?}"))?
        .candid()
        .map_err(|e| format!("decode: {e:?}"))?;
    add_order_result.map_err(|e| format!("orderbook.add_order returned error: {e}"))?;

    // store secret locally
    store_secret(&order.order_hash, secret);

    Ok(())
}

#[query]
pub fn get_active_auction(order_hash: String) -> Option<AuctionInfo> {
    ACTIVE_AUCTIONS.with_borrow(|m| m.get(&order_hash).map(|o| o.clone()))
}

#[query]
pub fn get_finished_auction(order_hash: String) -> Option<AuctionInfo> {
    FINISHED_AUCTIONS.with_borrow(|m| m.get(&order_hash).map(|o| o.clone()))
}

/// Resolver tries to accept current price.
#[update]
pub fn accept_price(order_hash: String, resolver_evm: String) -> Result<(), String> {
    let caller = msg_caller();

    ACTIVE_AUCTIONS.with(|m| {
        let mut map = m.borrow_mut();
        let auc = map.get_mut(&order_hash).ok_or("unknown auction")?;

        if auc.finished {
            return Err("auction finished".into());
        }
        if auc.winner.is_some() {
            return Err("already accepted".into());
        }
        if now_sec() < auc.order.auction_start_at {
            return Err("auction not started".into());
        }

        auc.winner = Some((caller, resolver_evm));
        auc.finished = true;
        Ok(())
    })
}

/// Verify both escrows (ICP & EVM), then reveal secret to the winner.
#[update]
async fn verify_and_reveal_secret(
    evm_escrow_addr: String,
    icp_escrow_canister: String,
    order_hash: String,
    resolver_evm: String,
) -> Result<Vec<u8>, String> {
    let resolver_principal = msg_caller();

    // 0. pull AuctionInfo (active or finished, doesn’t matter)
    let info = ACTIVE_AUCTIONS
        .with(|m| m.borrow().get(&order_hash).cloned())
        .or_else(|| FINISHED_AUCTIONS.with(|m| m.borrow().get(&order_hash).cloned()))
        .ok_or("unknown order")?;

    if info.winner != Some((resolver_principal, resolver_evm.clone())) {
        return Err("caller is not winner".into());
    }

    // --- Build expected params for both legs (shared helpers) ---
    let dst_params = make_dst_params(&info.order, resolver_principal, &resolver_evm);
    let src_params = make_src_params(
        &info.order,
        info.current_price,
        resolver_principal,
        &resolver_evm,
    );

    // Decide which leg is ICP/EVM
    let (params_icp, params_evm) = match (&src_params.asset, &dst_params.asset) {
        (Asset::ICP | Asset::ICRC(_), Asset::Erc20 { .. }) => (&src_params, &dst_params),
        (Asset::Erc20 { .. }, Asset::ICP | Asset::ICRC(_)) => (&dst_params, &src_params),
        _ => return Err("swap must involve exactly one EVM and one ICP asset".into()),
    };

    // 1. Verify ICP escrow
    icp::verify_escrow(&icp_escrow_canister, &params_icp)
        .await?
        .then_some(())
        .ok_or("ICP escrow verification failed")?;

    // 2. Verify EVM escrow
    let chain_id = match params_evm.asset {
        Asset::Erc20 { chain_id, .. } => chain_id,
        _ => return Err("source asset is not ERC-20".into()),
    };
    let mut hash_bytes = [0u8; 32];
    {
        let h = order_hash.trim_start_matches("0x");
        let b = hex::decode(h).map_err(|e| format!("bad order_hash: {e}"))?;
        if b.len() != 32 {
            return Err("order_hash must be 32-byte hex".into());
        }
        hash_bytes.copy_from_slice(&b);
    }
    evm::verify_escrow(
        &get_config().evm,
        chain_id,
        &evm_escrow_addr,
        &params_evm,
        &hash_bytes,
    )
    .await?
    .then_some(())
    .ok_or("EVM escrow verification failed")?;

    if dst_params.hashlock != src_params.hashlock {
        return Err("hashlock mismatch".into());
    }

    // 4. Reveal secret
    let secret = secret::take_secret(&order_hash).ok_or("secret not found / already revealed")?;
    if keccak256(&secret) != dst_params.hashlock {
        return Err("secret mismatch with hashlock".into());
    }

    // mark auction closed & move to FINISHED
    ACTIVE_AUCTIONS.with(|a| {
        FINISHED_AUCTIONS.with(|f| {
            if let Some(mut auc) = a.borrow_mut().remove(&order_hash) {
                auc.winner = Some((resolver_principal, resolver_evm));
                auc.finished = true;
                f.borrow_mut().insert(order_hash.clone(), auc);
            }
        })
    });

    Ok(secret)
}

#[query]
pub fn get_public_secret(order_hash: String) -> Result<Vec<u8>, String> {
    let auction = ACTIVE_AUCTIONS
        .with_borrow(|m| m.get(&order_hash).cloned())
        .or_else(|| FINISHED_AUCTIONS.with_borrow(|m| m.get(&order_hash).cloned()))
        .ok_or("Auction not found in active or finished auctions")?;

    if auction.finished
        && now_sec() > auction.order.timelocks.public_withdrawal + auction.order.auction_start_at
    {
        let secret =
            secret::take_secret(&order_hash).ok_or("secret not found / already revealed")?;
        Ok(secret)
    } else {
        return Err("auction is not finished yet".into());
    }
}

#[query]
pub fn list_active_auctions() -> Vec<AuctionInfo> {
    ACTIVE_AUCTIONS.with(|m| m.borrow().values().cloned().collect::<Vec<_>>())
}

#[query]
pub fn list_finished_auctions(after: Option<String>, limit: u64) -> Vec<String> {
    let mut v = FINISHED_AUCTIONS.with(|m| m.borrow().keys().cloned().collect::<Vec<_>>());
    v.sort(); // deterministic order by hash
    let start = after
        .and_then(|h| v.iter().position(|x| x == &h))
        .map(|idx| idx + 1)
        .unwrap_or(0);
    v.into_iter().skip(start).take(limit as usize).collect()
}

export_candid!();
