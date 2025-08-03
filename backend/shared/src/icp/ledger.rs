use candid::Principal;
use ic_cdk::call::Call;
use icrc_ledger_types::{
    icrc1::{
        account::Account,
        transfer::{BlockIndex, NumTokens, TransferArg, TransferError},
    },
    icrc2::transfer_from::{TransferFromArgs, TransferFromError},
};

const ICP_LEDGER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

fn ledger_pid(pid_opt: Option<Principal>) -> Principal {
    pid_opt.unwrap_or_else(|| Principal::from_text(ICP_LEDGER_ID).unwrap())
}

/* -------------------------------------------------------------------------
Generic helpers
---------------------------------------------------------------------- */
pub async fn icrc1_balance_of(
    ledger: Option<Principal>,
    account: Account,
) -> Result<NumTokens, String> {
    Call::unbounded_wait(ledger_pid(ledger), "icrc1_balance_of")
        .with_arg(account)
        .await
        .map_err(|e| format!("{e:?}"))?
        .candid::<NumTokens>()
        .map_err(|e| format!("decode: {e:?}"))
}

pub async fn icrc1_transfer(
    ledger: Option<Principal>,
    arg: TransferArg,
) -> Result<BlockIndex, String> {
    Call::unbounded_wait(ledger_pid(ledger), "icrc1_transfer")
        .with_arg(arg)
        .await
        .map_err(|e| format!("{e:?}"))?
        .candid::<Result<BlockIndex, TransferError>>()
        .map_err(|e| format!("decode: {e:?}"))?
        .map_err(|e| format!("transfer error: {e:?}"))
}

pub async fn icrc2_transfer_from(
    ledger: Option<Principal>,
    arg: TransferFromArgs,
) -> Result<BlockIndex, String> {
    Call::unbounded_wait(ledger_pid(ledger), "icrc2_transfer_from")
        .with_arg(arg)
        .await
        .map_err(|e| format!("{e:?}"))?
        .candid::<Result<BlockIndex, TransferFromError>>()
        .map_err(|e| format!("decode: {e:?}"))?
        .map_err(|e| format!("transfer_from error: {e:?}"))
}
