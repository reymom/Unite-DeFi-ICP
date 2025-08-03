use candid::Principal;
use ethers_core::utils::keccak256;
use ic_cdk::api::canister_self;
use ic_cdk::{api::msg_caller, init, post_upgrade, pre_upgrade, query, update};
use shared::funds::{
    pull_via_transfer_from, refund_funds, transfer_funds, transfer_safety_deposit, verify_funds,
};
use shared::{Asset, EscrowInfo, EscrowParams, EscrowState, Role, now_sec};

mod state;

use crate::state::{STATE, get_escrow_info};

#[init]
async fn init(params: EscrowParams) {
    STATE.with_borrow_mut(|s| {
        *s = Some(EscrowInfo {
            canister_id: canister_self(),
            params: params.clone(),
            status: EscrowState::Open,
            deployed_at: now_sec(),
            locked_at: None,
            claimed_secret: None,
            admin: msg_caller(),
        });
    });

    ic_cdk::println!(
        "Escrow deployed: {} - {:?}",
        canister_self(),
        get_escrow_info()
    )
}

#[pre_upgrade]
fn pre_upgrade() {
    let state = STATE.with_borrow(|s| s.clone());
    ic_cdk::storage::stable_save((state,)).unwrap();
}

#[post_upgrade]
fn post_upgrade() {
    let (state,): (Option<EscrowInfo>,) = ic_cdk::storage::stable_restore().unwrap();
    STATE.with_borrow_mut(|s| *s = state);
}

#[query]
fn get_escrow() -> EscrowInfo {
    get_escrow_info()
}

/// The factory sends the funds to the escrow on init in behalf of the maker
/// But it needs to signal that the transfer was successfull -> Lock state
#[update]
async fn factory_lock() -> Result<(), String> {
    let mut esc = get_escrow_info();
    if msg_caller() != esc.admin {
        return Err("not authorized".into());
    }
    if esc.status != EscrowState::Open {
        return Err("escrow not Open".into());
    }

    // verify funds ≥ amount + safety_deposit
    verify_funds(&esc.params, esc.params.amount + esc.params.safety_deposit).await?;

    esc.status = EscrowState::Locked;
    esc.locked_at = Some(now_sec());
    STATE.with_borrow_mut(|s| *s = Some(esc));
    Ok(())
}

/// Emergency method for locking maker funds. Role & caller rules enforced.
/// The maker needs to give allowance to the escrow before calling this lock.
#[update]
async fn lock() -> Result<(), String> {
    let mut escrow = get_escrow_info();

    if escrow.status != EscrowState::Open {
        return Err("Not in Open state".into());
    }

    let caller: Principal = match escrow.params.role {
        Role::Source => Principal::from_text(&escrow.params.counterparty),
        Role::Destination => Principal::from_text(&escrow.params.initiator),
    }
    .expect("invalid principal in escrow params");
    if msg_caller() != caller {
        return Err("Only counterparty (taker) can lock in Source role".into());
    }

    // Allow lock until the private‑withdrawal window opens.
    let tl = &escrow.params.timelock;
    if now_sec() >= tl.withdrawal_start(escrow.deployed_at) {
        return Err("lock period has ended".into());
    }

    // If token supports ICRC‑2, try pull; otherwise expect funds already sent
    match escrow.params.asset {
        Asset::ICP | shared::Asset::ICRC(_) => {
            let _ = pull_via_transfer_from(&escrow.params, msg_caller(), canister_self())
                .await
                .ok();
        }
        _ => return Err("asset not valid for ICP escrow".into()),
    }

    verify_funds(
        &escrow.params,
        escrow.params.amount + escrow.params.safety_deposit,
    )
    .await?;

    escrow.status = EscrowState::Locked;
    escrow.locked_at = Some(now_sec());
    STATE.with_borrow_mut(|s| *s = Some(escrow));
    Ok(())
}

/// The counter‑party reveals `secret` to unlock funds.
#[update]
async fn withdraw(secret: [u8; 32]) -> Result<(), String> {
    let mut escrow = get_escrow_info();
    if escrow.status != EscrowState::Locked {
        return Err("Not in Locked state".into());
    }

    // // window check
    // let now = now_sec();
    // let tl = &escrow.params.timelock;
    // if !tl.in_private_withdrawal_window(
    //     now,
    //     escrow.deployed_at,
    //     tl.cancellation_start(escrow.deployed_at),
    // ) {
    //     return Err("withdrawal window closed".into());
    // }

    // // Verify secret with hashlock
    // if keccak256(&secret) != escrow.params.hashlock {
    //     return Err("Invalid secret".into());
    // }

    // Transfer funds
    // transfer_funds(&escrow.params).await?;
    // transfer_safety_deposit(&escrow.params, msg_caller()).await?;

    escrow.status = EscrowState::Completed;
    escrow.claimed_secret = Some(secret);
    STATE.with_borrow_mut(|s| *s = Some(escrow));

    ic_cdk::println!("[withdraw_icp] transferred 4.5 ICP + safety deposit to taker");
    Ok(())
}

#[update]
async fn public_withdraw(secret: [u8; 32]) -> Result<(), String> {
    let mut escrow = get_escrow_info();
    if escrow.status != EscrowState::Locked {
        return Err("Not in Locked state".into());
    }

    let now = now_sec();
    let tl = &escrow.params.timelock;
    if !tl.in_public_withdrawal_window(
        now,
        escrow.deployed_at,
        tl.cancellation_start(escrow.deployed_at),
    ) {
        return Err("public withdrawal window closed".into());
    }

    // Verify secret with hashlock
    if keccak256(&secret) != escrow.params.hashlock {
        return Err("Invalid secret".into());
    }

    transfer_funds(&escrow.params).await?;
    transfer_safety_deposit(&escrow.params, msg_caller()).await?;

    escrow.status = EscrowState::Completed;
    escrow.claimed_secret = Some(secret);
    STATE.with_borrow_mut(|s| *s = Some(escrow));

    Ok(())
}

/// After `cancellation` seconds, any side can cancel and refund.
#[update]
async fn cancel() -> Result<(), String> {
    let mut escrow = get_escrow_info();

    // Only allow cancel if in Open or Locked state and after cancellation timelock
    if !(escrow.status == EscrowState::Open || escrow.status == EscrowState::Locked) {
        return Err("Cannot cancel in current state".into());
    }

    // Enforce cancellation window
    if now_sec()
        < escrow
            .params
            .timelock
            .cancellation_start(escrow.deployed_at)
    {
        return Err("cancellation window not reached".into());
    }

    refund_funds(&escrow.params).await?;
    transfer_safety_deposit(&escrow.params, msg_caller()).await?;

    escrow.status = EscrowState::Cancelled;
    STATE.with_borrow_mut(|s| *s = Some(escrow));
    Ok(())
}

#[update]
async fn public_cancel() -> Result<(), String> {
    let mut escrow = get_escrow_info();
    if !(escrow.status == EscrowState::Open || escrow.status == EscrowState::Locked) {
        return Err("Cannot cancel in current state".into());
    }
    if escrow.params.role == Role::Destination {
        return Err("destination escrow has no public cancel".into());
    } else {
        match escrow
            .params
            .timelock
            .public_cancellation_start(escrow.deployed_at)
        {
            Some(pcs) if now_sec() < pcs => (),
            _ => return Err("public cancel window not reached".into()),
        }
    }

    // Refund the funds to maker or taker (based on role)
    refund_funds(&escrow.params).await?;

    // Send the safety deposit to the caller
    transfer_safety_deposit(&escrow.params, msg_caller()).await?;

    escrow.status = EscrowState::Cancelled;
    STATE.with_borrow_mut(|s| *s = Some(escrow));
    Ok(())
}

#[update]
async fn rescue() -> Result<(), String> {
    let mut esc = get_escrow();
    if msg_caller().to_text() != esc.params.counterparty {
        return Err("only taker".into());
    }
    let delay = 86_400; // 24h hard‑coded for now
    if now_sec() < esc.params.timelock.cancellation_start(esc.deployed_at) + delay {
        return Err("rescue delay not reached".into());
    }
    refund_funds(&esc.params).await?;
    transfer_safety_deposit(&esc.params, msg_caller()).await?;
    esc.status = EscrowState::Rescued;
    STATE.with(|s| *s.borrow_mut() = Some(esc));
    Ok(())
}

ic_cdk::export_candid!();
