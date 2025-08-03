/* -------------------------------------------------------------------------
Public API used by escrow logic
---------------------------------------------------------------------- */

use candid::Principal;
use ic_cdk::api::canister_self;
use icrc_ledger_types::{
    icrc1::{
        account::Account,
        transfer::{NumTokens, TransferArg},
    },
    icrc2::transfer_from::TransferFromArgs,
};

use crate::ledger::{icrc1_balance_of, icrc1_transfer, icrc2_transfer_from};
use crate::{Asset, EscrowParams, Role};

fn p_to_account(p: Principal) -> Account {
    Account {
        owner: p,
        subaccount: None,
    }
}

/// Ensure the canister’s balance ≥ expected (amount+safety).
pub async fn verify_funds(params: &EscrowParams, expected: u64) -> Result<(), String> {
    let acct = Account {
        owner: canister_self(),
        subaccount: None,
    };
    let bal = match &params.asset {
        Asset::ICP => icrc1_balance_of(None, acct).await?,
        Asset::ICRC(led) => icrc1_balance_of(Some(*led), acct).await?,
        _ => return Err("asset not valid for ICP escrow".into()),
    };
    if bal < NumTokens::from(expected) {
        Err(format!("balance {} < expected {}", bal, expected))
    } else {
        Ok(())
    }
}

/// Send the escrowed amount to payout_addr.
pub async fn transfer_funds(params: &EscrowParams) -> Result<(), String> {
    use icrc_ledger_types::icrc1::transfer::TransferArg;
    let ledger_pid = match &params.asset {
        Asset::ICP => None,
        Asset::ICRC(pid) => Some(*pid),
        _ => return Err("asset not valid for ICP escrow".into()),
    };

    let arg = TransferArg {
        from_subaccount: None,
        to: p_to_account(
            Principal::from_text(params.payout_addr.clone()).expect("invalid principal"),
        ),
        amount: NumTokens::from(params.amount),
        fee: None,
        memo: None,
        created_at_time: None,
    };
    icrc1_transfer(ledger_pid, arg).await.map(|_| ())
}

/// Refund all funds to the locker (depends on role).
pub async fn refund_funds(params: &EscrowParams) -> Result<(), String> {
    let dst: Principal = match params.role {
        Role::Source => Principal::from_text(&params.counterparty),
        Role::Destination => Principal::from_text(&params.initiator),
    }
    .expect("invalid principal in escrow params");
    let arg = TransferArg {
        from_subaccount: None,
        to: p_to_account(dst),
        amount: NumTokens::from(params.amount),
        fee: None,
        memo: None,
        created_at_time: None,
    };
    let ledger_pid = match &params.asset {
        Asset::ICP => None,
        Asset::ICRC(pid) => Some(*pid),
        _ => return Err("asset not valid for ICP escrow".into()),
    };
    icrc1_transfer(ledger_pid, arg).await.map(|_| ())
}

/// Pay out the safety deposit to `dst`.
pub async fn transfer_safety_deposit(
    params: &EscrowParams,
    recipient: Principal,
) -> Result<(), String> {
    if params.safety_deposit == 0 {
        return Ok(());
    }
    let arg = TransferArg {
        from_subaccount: None,
        to: p_to_account(recipient),
        amount: NumTokens::from(params.safety_deposit),
        fee: None,
        memo: None,
        created_at_time: None,
    };
    let ledger_pid = match &params.asset {
        Asset::ICP => None,
        Asset::ICRC(pid) => Some(*pid),
        _ => return Err("asset not valid for ICP escrow".into()),
    };
    icrc1_transfer(ledger_pid, arg).await.map(|_| ())
}

/* -------------------------------------------------------------------------
Pull funds via ICRC‑2 approve/transfer_from — used in `init()`
---------------------------------------------------------------------- */
/// Called in init(): pulls tokens into this canister after the
/// factory is given permissions to transfer via `icrc2_approve`.
pub async fn pull_via_transfer_from(
    params: &EscrowParams,
    from: Principal,
    to: Principal,
) -> Result<(), String> {
    let ledger_pid = match &params.asset {
        Asset::ICP => None,
        Asset::ICRC(pid) => Some(*pid),
        _ => return Err("asset not valid for ICP escrow".into()),
    };
    let arg = TransferFromArgs {
        from: p_to_account(from),
        to: p_to_account(to),
        amount: NumTokens::from(params.amount + params.safety_deposit),
        spender_subaccount: None,
        fee: None,
        memo: None,
        created_at_time: Some(ic_cdk::api::time()),
    };
    icrc2_transfer_from(ledger_pid, arg).await.map(|_| ())
}
