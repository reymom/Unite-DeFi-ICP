use super::config::EvmConfig;
use evm_rpc_canister_types::{
    EVM_RPC, MultiSendRawTransactionResult, RpcConfig, SendRawTransactionResult,
    SendRawTransactionStatus,
};

const TX_CYCLES: u128 = 10_000_000_000;

#[derive(Debug)]
pub enum TxSendError {
    EmptyTxHash,
    NonceTooLow,
    NonceTooHigh,
    InsufficientFunds,
    ReplacementUnderpriced,
    RpcError(String),
    IcError(String),
}

pub async fn send_signed_transaction(
    cfg: &EvmConfig,
    chain_id: u64,
    signed_tx: String,
) -> Result<String, TxSendError> {
    let rpc_providers = cfg.rpc_services(chain_id).map_err(TxSendError::RpcError)?;

    let rpc_config = Some(RpcConfig {
        responseConsensus: None,
        responseSizeEstimate: Some(1024),
    });

    match EVM_RPC
        .eth_send_raw_transaction(rpc_providers, rpc_config, signed_tx, TX_CYCLES)
        .await
    {
        Ok((res,)) => match res {
            MultiSendRawTransactionResult::Consistent(status) => match status {
                SendRawTransactionResult::Ok(status) => match status {
                    SendRawTransactionStatus::Ok(tx_hash) => {
                        ic_cdk::println!("[send_signed_transaction] tx_hash = {:?}", tx_hash);
                        tx_hash.ok_or_else(|| TxSendError::EmptyTxHash)
                    }
                    SendRawTransactionStatus::NonceTooLow => Err(TxSendError::NonceTooLow),
                    SendRawTransactionStatus::NonceTooHigh => Err(TxSendError::NonceTooHigh),
                    SendRawTransactionStatus::InsufficientFunds => {
                        Err(TxSendError::InsufficientFunds)
                    }
                },
                SendRawTransactionResult::Err(e) => {
                    Err(TxSendError::RpcError(format!("rpc error: {:?}", e)))
                }
            },
            MultiSendRawTransactionResult::Inconsistent(_) => {
                Err(TxSendError::RpcError("Inconsistent status".into()))
            }
        },
        Err(err) => Err(TxSendError::IcError(format!("IC reject: {}", err))),
    }
}
