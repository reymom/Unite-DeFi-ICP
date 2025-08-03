/// --------------------
/// Factory entry‑point: expose Candid API to create and query escrows while persisting state.
/// --------------------
use candid::{Encode, Principal};
use ic_cdk::{
    api::{canister_self, msg_caller, msg_cycles_accept, msg_cycles_available},
    call::Call,
    init,
    management_canister::{
        CanisterInstallMode, CanisterSettings, CreateCanisterArgs, InstallCodeArgs,
        create_canister_with_extra_cycles, install_code,
    },
    post_upgrade, pre_upgrade, query, update,
};
use shared::{
    Asset, EscrowInfo, EscrowParams, EscrowState, Role, funds::pull_via_transfer_from, now_sec,
};

mod storage;

use crate::storage::{ESCROW_REGISTRY, ESCROW_WASM};

// Cycle constants (adjust based on current network costs)
const CANISTER_CREATION_COST: u128 = 500_000_000_000; // 0.5T cycles
const INSTALLATION_BUFFER: u128 = 300_000_000_000; // 0.3T cycles buffer
const TOTAL_CREATION_COST: u128 = CANISTER_CREATION_COST + INSTALLATION_BUFFER;

#[init]
fn init() {
    ic_cdk::println!("[init] factory initialised");
}

#[pre_upgrade]
fn pre_upgrade() {
    let wasm = ESCROW_WASM.with_borrow(|w| w.clone());
    ic_cdk::storage::stable_save((wasm,)).unwrap();
}

#[post_upgrade]
fn post_upgrade() {
    let (wasm,): (Vec<u8>,) = ic_cdk::storage::stable_restore().unwrap();
    ESCROW_WASM.with_borrow_mut(|w| *w = wasm);
}

#[update]
async fn create_escrow(params: EscrowParams) -> Result<Principal, String> {
    let wasm = ESCROW_WASM.with(|w| w.borrow().clone());
    if wasm.is_empty() {
        return Err("Escrow WASM not set".into());
    }

    // Check and accept cycles
    let available_cycles = msg_cycles_available();
    if available_cycles < TOTAL_CREATION_COST {
        return Err(format!(
            "Insufficient cycles: required {}, provided {}",
            TOTAL_CREATION_COST, available_cycles
        ));
    }
    msg_cycles_accept(TOTAL_CREATION_COST);

    // Create new canister
    let create_result = create_canister_with_extra_cycles(
        &CreateCanisterArgs {
            settings: Some(CanisterSettings {
                controllers: Some(vec![canister_self()]),
                ..Default::default()
            }),
        },
        INSTALLATION_BUFFER,
    )
    .await
    .map_err(|err| format!("Creation failed: {}", err))?;

    // Install escrow code
    install_code(&InstallCodeArgs {
        mode: CanisterInstallMode::Install,
        canister_id: create_result.canister_id,
        wasm_module: wasm,
        arg: Encode!(&params).map_err(|e| e.to_string())?,
    })
    .await
    .map_err(|err| format!("Install failed: {}", err))?;

    ESCROW_REGISTRY.with_borrow_mut(|r| {
        r.insert(
            create_result.canister_id,
            EscrowInfo {
                canister_id: create_result.canister_id,
                params: params.clone(),
                status: EscrowState::Open,
                deployed_at: now_sec(),
                admin: canister_self(),
                claimed_secret: None,
                locked_at: None,
            },
        )
    });

    if matches!(params.asset, Asset::ICP | Asset::ICRC(_)) {
        // Determine who is the source of funds (maker)
        let from_principal_text = match params.role {
            Role::Source => params.initiator.clone(),
            Role::Destination => params.counterparty.clone(),
        };
        let from_principal = Principal::from_text(&from_principal_text)
            .map_err(|e| format!("bad principal: {e}"))?;

        // send the funds to the escrow
        pull_via_transfer_from(&params, from_principal, create_result.canister_id).await?;
        // signal escrow to flip escrow's Locked state
        let res: Result<(), String> =
            Call::unbounded_wait(create_result.canister_id, "factory_lock")
                .with_arg(()) // <- send empty tuple as arg
                .await
                .map_err(|e| format!("{e:?}"))?
                .candid::<Result<(), String>>()
                .map_err(|e| format!("decode: {e:?}"))?;
        res.map_err(|e| format!("escrow.factory_lock returned error: {e}"))?;
    }

    Ok(create_result.canister_id)
}

#[update]
fn set_escrow_wasm(wasm: Vec<u8>) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&msg_caller()) {
        return Err("Only controller can set WASM".into());
    }
    ESCROW_WASM.with_borrow_mut(|w| *w = wasm);
    Ok(())
}

#[query]
fn get_escrow(canister_id: Principal) -> Option<EscrowInfo> {
    ESCROW_REGISTRY.with_borrow(|r| r.get(&canister_id))
}

#[query]
fn list_escrows() -> Vec<EscrowInfo> {
    ESCROW_REGISTRY.with_borrow(|r| r.iter().map(|entry| entry.value().clone()).collect())
}

#[query]
fn get_wasm_length() -> usize {
    ESCROW_WASM.with(|w| w.borrow().len())
}

#[query]
fn get_wasm_hash() -> String {
    use sha2::{Digest, Sha256};
    // Compute hash of the stored wasm
    let hash = ESCROW_WASM.with_borrow(|w| {
        let mut hasher = Sha256::new();
        hasher.update(&*w);
        hasher.finalize()
    });
    hex::encode(hash)
}

ic_cdk::export_candid!();
