use candid::Principal;

use crate::{Asset, EscrowParams, Order, Role};

/// Build **Src** params for a resolver at `price`.
pub fn make_src_params(
    order: &Order,
    price: f64,
    resolver_principal: Principal,
    resolver_evm: &str,
) -> EscrowParams {
    let asset = order.taker_asset.clone();
    let role = Role::Source;

    let (initiator, counterparty, payout_addr) = match &asset {
        &Asset::ICP | Asset::ICRC(_) => (
            resolver_principal.to_text(),
            order.maker_icp.to_text(),
            resolver_principal.to_text(),
        ),
        Asset::Erc20 { .. } => (
            resolver_evm.to_string(),
            order.maker_evm.clone(),
            resolver_evm.to_string(),
        ),
    };

    EscrowParams {
        role,
        initiator,
        counterparty,
        asset,
        amount: amount_for_price(order, price),
        hashlock: order.hashlock,
        timelock: order.timelocks.clone(),
        payout_addr,
        safety_deposit: order.safety_deposit,
    }
}

pub fn make_dst_params(
    order: &Order,
    resolver_principal: Principal,
    resolver_evm: &str,
) -> EscrowParams {
    let role = Role::Destination;
    let asset = order.maker_asset.clone();

    let (initiator, counterparty, payout_addr) = match &asset {
        Asset::ICP | Asset::ICRC(_) => (
            resolver_principal.to_text(),
            order.maker_icp.to_text(),
            order.maker_icp.to_text(),
        ),
        Asset::Erc20 { .. } => (
            resolver_evm.to_string(),
            order.maker_evm.clone(),
            order.maker_evm.clone(),
        ),
    };

    EscrowParams {
        role,
        initiator,
        counterparty,
        asset,
        amount: order.making_amount as u64,
        hashlock: order.hashlock,
        timelock: order.timelocks.clone(),
        payout_addr,
        safety_deposit: order.safety_deposit,
    }
}

/// amount of `taker_asset` a resolver must post at the given price
fn amount_for_price(order: &Order, price: f64) -> u64 {
    if price.is_finite() && price > 0.0 {
        // maker / price  (rounded *up* so user never gets less)
        ((order.making_amount as f64 / price).ceil()) as u64
    } else {
        // fallback to static taking_amount (floor-price case)
        order.min_return_amount as u64
    }
}
