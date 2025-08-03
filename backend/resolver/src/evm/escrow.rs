use crate::{error::*, state::get_config};
use ethers_core::types::U256;
use shared::{
    EscrowParams,
    evm::{
        broadcast::send_signed_transaction,
        erc20_addr,
        escrow::{cancel_calldata, withdraw_calldata},
        hex_to_bytes32, parse_evm_addr,
        signer::sign_transaction,
    },
};

pub async fn withdraw_escrow(
    chain_id: u64,
    escrow_addr: &str,
    secret: Vec<u8>,
    params: &EscrowParams,
    order_hash: &str,
) -> Result<()> {
    let cfg = get_config().evm.clone();

    let order_hash_bytes = hex_to_bytes32(order_hash)?;
    let maker_evm = parse_evm_addr(&params.counterparty)?;
    let taker_evm = parse_evm_addr(&params.initiator)?;
    let token_evm = erc20_addr(&params.asset)?;

    let data = withdraw_calldata(
        &secret,
        params,
        order_hash_bytes,
        maker_evm,
        taker_evm,
        token_evm,
    );

    let signed_tx = sign_transaction(
        &cfg,
        chain_id,
        escrow_addr,
        data,
        U256::from(500_000),
        U256::from(10_000_000_000u64),
        U256::from(2_000_000_000u64),
    )
    .await?;

    let tx_hash = send_signed_transaction(&cfg, chain_id, signed_tx).await?;
    ic_cdk::println!("Withdraw escrow tx: {}", tx_hash);
    Ok(())
}

pub async fn cancel_escrow(
    chain_id: u64,
    escrow_addr: &str,
    params: &EscrowParams,
    order_hash: &str,
) -> Result<(), ResolverError> {
    let cfg = get_config().evm.clone();

    let order_hash_bytes = hex_to_bytes32(order_hash)?;
    let maker_evm = parse_evm_addr(&params.counterparty)?;
    let taker_evm = parse_evm_addr(&params.initiator)?;
    let token_evm = erc20_addr(&params.asset)?;

    let data = cancel_calldata(params, order_hash_bytes, maker_evm, taker_evm, token_evm);

    let signed_tx = sign_transaction(
        &cfg,
        chain_id,
        escrow_addr,
        data,
        U256::from(500_000),
        U256::from(10_000_000_000u64),
        U256::from(2_000_000_000u64),
    )
    .await?;

    let tx_hash = send_signed_transaction(&cfg, chain_id, signed_tx).await?;
    ic_cdk::println!("Cancel escrow tx: {}", tx_hash);
    Ok(())
}
