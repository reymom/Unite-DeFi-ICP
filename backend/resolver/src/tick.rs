use std::time::Duration;

use crate::{
    error::*,
    evm::{
        deploy::deploy_evm_escrow,
        escrow::{cancel_escrow, withdraw_escrow},
    },
    icp::{create_icp_escrow, withdraw_icp},
    state::{
        ACTIVE_ESCROWS,
        config::{Config, EscrowHandles},
        get_config,
    },
};
use candid::Principal;
use ic_cdk::{api::canister_self, call::Call, futures::spawn};
use shared::{
    Asset, AuctionInfo, EscrowParams, Role,
    evm::{
        erc20_addr,
        factory::{encode_dst_immutables, encode_src_immutables},
        hex_to_bytes32, parse_evm_addr,
        signer::derive_evm_address,
    },
    make_dst_params, make_src_params, now_sec,
};

pub fn spawn_tick() {
    let poll = get_config().automatic_tick.unwrap().poll_interval_sec;
    ic_cdk_timers::set_timer_interval(Duration::from_secs(poll), || {
        spawn(async {
            if let Err(e) = tick().await {
                ic_cdk::println!("[resolver::tick] {e}");
            }
        });
    });
}

async fn tick() -> Result<()> {
    // pull config
    let cfg = get_config();

    // list active auctions
    let auctions: Vec<AuctionInfo> = Call::unbounded_wait(cfg.relayer_id, "list_active_auctions")
        .with_arg(())
        .await
        .map_err(|e| ResolverError::Relayer(format!("{e:?}")))?
        .candid()
        .map_err(|e| ResolverError::Relayer(format!("decode: {e:?}")))?;

    let our_principal = canister_self();

    for auc in auctions {
        if auc.finished {
            continue; // competition already resolved
        }

        let resolver_evm = derive_evm_address(cfg.evm.ecdsa_key_id.clone()).await;
        match auc.winner {
            Some((winner_icp, ref winner_evm)) if winner_icp == our_principal => {
                // We won previously, ensure escrows deployed
                if ACTIVE_ESCROWS.with(|m| m.borrow().contains_key(&auc.order.order_hash.clone())) {
                    continue; // already handled
                }

                ic_cdk::println!("We confirmed winner for {}", auc.order.order_hash);

                let (icp_params, evm_params, chain_id) =
                    classify_assets(&auc, our_principal, winner_evm)?;
                let icp_id = create_icp_escrow(&icp_params).await?;

                let evm_calldata = prepare_evm_calldata(&auc, &evm_params)?;
                let evm_addr = deploy_evm_escrow(chain_id, evm_calldata.clone()).await?;

                ACTIVE_ESCROWS.with(|m| {
                    m.borrow_mut().insert(
                        auc.order.order_hash.clone(),
                        EscrowHandles {
                            evm_addr: evm_addr.clone(),
                            icp_id,
                            revealed: false,
                        },
                    )
                });

                // Now verify and reveal secret
                let secret_bytes: Vec<u8> =
                    Call::unbounded_wait(cfg.relayer_id, "verify_and_reveal_secret")
                        .with_args(&(
                            evm_addr.clone(),
                            icp_id.to_text(),
                            auc.order.order_hash.clone(),
                            resolver_evm,
                        ))
                        .await?
                        .candid::<Result<Vec<u8>, String>>()
                        .map_err(|e| ResolverError::Relayer(format!("decode: {e:?}")))? // outer Result
                        .map_err(|e| ResolverError::Relayer(e))?; // inner Err variant

                withdraw_icp(icp_id, secret_bytes.clone()).await?;
                withdraw_escrow(
                    chain_id,
                    &evm_addr,
                    secret_bytes,
                    &evm_params,
                    &auc.order.order_hash,
                )
                .await?;

                ACTIVE_ESCROWS.with(|m| {
                    if let Some(h) = m.borrow_mut().get_mut(&auc.order.order_hash) {
                        h.revealed = true;
                    }
                });

                ic_cdk::println!("Completed swap for {}", auc.order.order_hash);
            }

            Some((winner_principal, ref winner_evm)) => {
                let now = now_sec();

                if let Some(handles) =
                    ACTIVE_ESCROWS.with(|m| m.borrow().get(&auc.order.order_hash).cloned())
                {
                    let (_, evm_params, chain_id) =
                        classify_assets(&auc, winner_principal, winner_evm)?;

                    let deployed_at = auc.order.auction_start_at;
                    let cancellation_start = evm_params.timelock.cancellation_start(deployed_at);

                    if evm_params.timelock.in_public_withdrawal_window(
                        now,
                        deployed_at,
                        cancellation_start,
                    ) && !handles.revealed
                    {
                        let secret: Vec<u8> =
                            Call::unbounded_wait(cfg.relayer_id, "get_public_secret")
                                .with_arg(auc.order.order_hash.clone())
                                .await
                                .map_err(|e| ResolverError::Other(format!("{e:?}")))?
                                .candid::<Result<Vec<u8>, String>>()
                                .map_err(|e| ResolverError::Other(format!("decode: {e:?}")))?
                                .map_err(|e| ResolverError::Other(e))?;

                        withdraw_icp(handles.icp_id, secret.clone()).await?;
                        withdraw_escrow(
                            chain_id,
                            &handles.evm_addr,
                            secret,
                            &evm_params,
                            &auc.order.order_hash,
                        )
                        .await?;

                        ACTIVE_ESCROWS.with(|m| {
                            if let Some(h) = m.borrow_mut().get_mut(&auc.order.order_hash) {
                                h.revealed = true;
                            }
                        });

                        ic_cdk::println!("Public withdrawal completed: {}", auc.order.order_hash);
                    } else if evm_params
                        .timelock
                        .in_private_cancellation_window(now, deployed_at)
                        && !handles.revealed
                    {
                        cancel_escrow(
                            chain_id,
                            &handles.evm_addr,
                            &evm_params,
                            &auc.order.order_hash,
                        )
                        .await?;
                        ic_cdk::println!("Escrow cancelled: {}", auc.order.order_hash);
                    }
                }
            }

            None => {
                // Auction open, we may try to accept price
                if price_ok(&auc, &cfg) {
                    Call::unbounded_wait(cfg.relayer_id, "accept_price")
                        .with_args(&(auc.order.order_hash.clone(), resolver_evm.clone()))
                        .await
                        .map_err(|e| ResolverError::Relayer(format!("{e:?}")))? // IC call error
                        .candid::<Result<(), String>>() // decode into inner Result
                        .map_err(|e| ResolverError::Relayer(format!("decode: {e:?}")))??;

                    ic_cdk::println!("Attempted to accept price for {}", auc.order.order_hash);
                }
            }
        }
    }

    Ok(())
}

/// price & filter logic
fn price_ok(auc: &AuctionInfo, cfg: &Config) -> bool {
    if auc.current_price
        > 1.0 + (cfg.automatic_tick.clone().unwrap().max_slippage_bps as f64 / 10_000.0)
    {
        return false;
    }
    true
}

/// Decide which params are on which chain
pub fn classify_assets(
    auc: &AuctionInfo,
    resolver_principal: Principal,
    resolver_evm: &str,
) -> Result<(EscrowParams, EscrowParams, u64)> {
    let src_params = make_src_params(
        &auc.order,
        auc.current_price,
        resolver_principal,
        resolver_evm,
    );
    let dst_params = make_dst_params(&auc.order, resolver_principal, resolver_evm);

    match (src_params.asset.clone(), dst_params.asset.clone()) {
        (Asset::ICP | Asset::ICRC(_), Asset::Erc20 { chain_id, .. }) => {
            Ok((src_params, dst_params, chain_id))
        }
        (Asset::Erc20 { chain_id, .. }, Asset::ICP | Asset::ICRC(_)) => {
            Ok((dst_params, src_params, chain_id))
        }
        _ => Err(ResolverError::Other("Unsupported asset pair".into())),
    }
}

pub fn prepare_evm_calldata(auc: &AuctionInfo, params: &EscrowParams) -> Result<Vec<u8>> {
    let order_hash_bytes = hex_to_bytes32(&auc.order.order_hash)?;
    let maker_evm = parse_evm_addr(&params.counterparty)?;
    let taker_evm = parse_evm_addr(&params.initiator)?;
    let token = erc20_addr(&params.asset)?;

    let calldata = match params.role {
        Role::Source => {
            encode_src_immutables(order_hash_bytes, params, maker_evm, taker_evm, token)
        }
        Role::Destination => {
            encode_dst_immutables(order_hash_bytes, params, maker_evm, taker_evm, token)
        }
    };

    Ok(calldata)
}
