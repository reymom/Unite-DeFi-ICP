use candid::Principal;
use ic_cdk::call::Call;
use shared::EscrowParams;

use crate::{error::*, state::get_config};

pub async fn create_icp_escrow(params: &EscrowParams) -> Result<Principal> {
    let cfg = get_config();
    let factory = cfg.factory_icp;

    let result: Result<Principal, String> = Call::unbounded_wait(factory, "create_escrow")
        .with_arg(params.clone())
        .with_cycles(600_000_000_000u128) // 0.6 T cycles (creation + buffer)
        .await
        .map_err(|e| ResolverError::Factory(format!("{e:?}")))?
        .candid()
        .map_err(|e| ResolverError::Factory(format!("decode: {e:?}")))?;

    let id = result.map_err(|e| ResolverError::Factory(e))?;
    Ok(id)
}

// withdraw after secret reveal
pub async fn withdraw_icp(canister: Principal, secret_bytes: Vec<u8>) -> Result<()> {
    let res: Result<(), String> = Call::unbounded_wait(canister, "withdraw")
        .with_arg(secret_bytes)
        .await
        .map_err(|e| ResolverError::Other(format!("{e:?}")))?
        .candid()
        .map_err(|e| ResolverError::Other(format!("decode: {e:?}")))?;

    res?;
    Ok(())
}
