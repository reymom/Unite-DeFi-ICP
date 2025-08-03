use ethers_core::{
    types::{Bytes, Eip1559TransactionRequest},
    utils::{hex, keccak256},
};
use shared::evm::{
    factory::ADDR_FACTORY,
    signer::{recover_v, sign_tx_hash},
};

use crate::{error::*, state::get_config};

/// Deploy escrow on EVM chain via `evm_rpc` helper canister.
/// Returns deployed address (hex string, 0x-prefixed).
pub async fn deploy_evm_escrow(chain_id: u64, calldata: Vec<u8>) -> Result<String> {
    let cfg = get_config();
    let tx = Eip1559TransactionRequest {
        to: Some(ADDR_FACTORY.into()),
        data: Some(Bytes::from(calldata)),
        gas: Some(300_000u64.into()),
        max_fee_per_gas: Some(10_000_000_000u64.into()),
        max_priority_fee_per_gas: Some(2_000_000_000u64.into()),
        chain_id: Some(chain_id.into()),
        ..Default::default()
    };

    let tx_rlp_bytes = tx.rlp();
    let unsigned_bytes = [&[0x02u8], tx_rlp_bytes.as_ref()].concat();
    let tx_hash = keccak256(&unsigned_bytes);

    let signature = sign_tx_hash(cfg.evm.ecdsa_key_id.clone(), &tx_hash).await;
    let pubkey = shared::evm::signer::derive_pubkey(cfg.evm.ecdsa_key_id.clone()).await;
    let v = recover_v(&tx_hash, &signature, &pubkey);

    let sig = ethers_core::types::Signature {
        v,
        r: signature[..32].into(),
        s: signature[32..64].into(),
    };

    let signed_rlp_bytes = tx.rlp_signed(&sig);
    let signed_bytes = [&[0x02u8], signed_rlp_bytes.as_ref()].concat();

    let signed_tx_hex = format!("0x{}", hex::encode(&signed_bytes));
    ic_cdk::println!("Signed tx: {}", signed_tx_hex);

    Ok(signed_tx_hex)
}
