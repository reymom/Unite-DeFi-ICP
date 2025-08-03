use ethers_core::{
    k256::{
        PublicKey,
        ecdsa::{RecoveryId, Signature as K256Signature, VerifyingKey},
        elliptic_curve::sec1::ToEncodedPoint,
    },
    types::{Address, Eip1559TransactionRequest, Signature, U256},
    utils::{hex, keccak256},
};
use ic_cdk::management_canister::{
    EcdsaKeyId, EcdsaPublicKeyArgs, SignWithEcdsaArgs, ecdsa_public_key, sign_with_ecdsa,
};

use crate::evm::config::EvmConfig;

pub async fn sign_transaction(
    cfg: &EvmConfig,
    chain_id: u64,
    to: &str,
    data: Vec<u8>,
    gas: U256,
    max_fee_per_gas: U256,
    max_priority_fee_per_gas: U256,
) -> Result<String, String> {
    let tx = Eip1559TransactionRequest {
        chain_id: Some(chain_id.into()),
        to: Some(to.parse().unwrap()),
        data: Some(data.into()),
        gas: Some(gas),
        max_fee_per_gas: Some(max_fee_per_gas),
        max_priority_fee_per_gas: Some(max_priority_fee_per_gas),
        ..Default::default()
    };

    let unsigned_bytes = [&[0x02u8], tx.rlp().as_ref()].concat();
    let tx_hash = keccak256(&unsigned_bytes);
    let signature = sign_tx_hash(cfg.ecdsa_key_id.clone(), &tx_hash).await;
    let pubkey = derive_pubkey(cfg.ecdsa_key_id.clone()).await;
    let v = recover_v(&tx_hash, &signature, &pubkey);
    let sig = Signature {
        v,
        r: signature[..32].into(),
        s: signature[32..64].into(),
    };

    let signed_bytes = [&[0x02u8], tx.rlp_signed(&sig).as_ref()].concat();
    Ok(format!("0x{}", hex::encode(signed_bytes)))
}

pub async fn derive_pubkey(key_id: EcdsaKeyId) -> Vec<u8> {
    let res = ecdsa_public_key(&EcdsaPublicKeyArgs {
        canister_id: None,
        derivation_path: vec![],
        key_id,
    })
    .await
    .expect("ECDSA pubkey call failed");
    res.public_key
}

pub fn pubkey_to_address(pubkey_bytes: &[u8]) -> String {
    let key = PublicKey::from_sec1_bytes(pubkey_bytes).expect("invalid pubkey SEC1");
    let point = key.to_encoded_point(false);
    let hash = keccak256(&point.as_bytes()[1..]);
    ethers_core::utils::to_checksum(&Address::from_slice(&hash[12..32]), None)
}

pub async fn derive_evm_address(key_id: EcdsaKeyId) -> String {
    let pubkey = derive_pubkey(key_id).await;
    pubkey_to_address(&pubkey)
}

pub async fn sign_tx_hash(key_id: EcdsaKeyId, tx_hash: &[u8]) -> Vec<u8> {
    let res = sign_with_ecdsa(&SignWithEcdsaArgs {
        message_hash: tx_hash.to_vec(),
        derivation_path: vec![],
        key_id,
    })
    .await
    .expect("ECDSA sign failed");
    res.signature
}

pub fn recover_v(tx_hash: &[u8], signature: &[u8], pubkey: &[u8]) -> u64 {
    let orig_key = VerifyingKey::from_sec1_bytes(pubkey).unwrap();
    let sig = K256Signature::try_from(signature).unwrap();
    for parity in [0u8, 1] {
        let recid = RecoveryId::try_from(parity).unwrap();
        if let Ok(recovered_key) = VerifyingKey::recover_from_prehash(tx_hash, &sig, recid) {
            if recovered_key == orig_key {
                return parity as u64;
            }
        }
    }
    panic!("v parity recovery failed");
}
