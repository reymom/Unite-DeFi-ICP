use ethers_core::utils::hex;
use evm_rpc_canister_types::{CallArgs, CallResult, EVM_RPC, MultiCallResult, TransactionRequest};

use shared::{
    EscrowParams, Role,
    evm::{
        config::EvmConfig,
        erc20_addr,
        factory::{ADDR_FACTORY, encode_dst_immutables, encode_src_immutables},
        parse_evm_addr,
    },
};

const CALL_CYCLES: u128 = 25_000_000_000_000;

/// Verify that a deployed **clone** matches the expected immutables.
///
/// * `clone_addr` – 0x-prefixed hex string supplied by resolver.
/// * `params`     – **Src** escrow parameters (same side as clone).
/// * `order_hash` – bytes32 order id.
pub async fn verify_escrow(
    cfg: &EvmConfig,
    chain_id: u64,
    clone_addr: &str,
    params: &EscrowParams,
    order_hash: &[u8; 32],
) -> Result<bool, String> {
    //---------------- maker / taker / token ---------------------------------
    let maker_evm = parse_evm_addr(&params.counterparty)?; // maker on EVM
    let taker_evm = parse_evm_addr(&params.initiator)?; // resolver EVM addr
    let token_evm = erc20_addr(&params.asset)?;

    //---------------- calldata ----------------------------------------------
    let data = match params.role {
        Role::Source => encode_src_immutables(*order_hash, params, maker_evm, taker_evm, token_evm),
        Role::Destination => {
            encode_dst_immutables(*order_hash, params, maker_evm, taker_evm, token_evm)
        }
    };

    let rpc_services = cfg.rpc_services(chain_id)?;

    let resp = EVM_RPC
        .eth_call(
            rpc_services,
            None,
            CallArgs {
                transaction: TransactionRequest {
                    to: Some(ADDR_FACTORY.into()),
                    input: Some(format!("0x{}", hex::encode(data))),
                    ..Default::default()
                },
                block: None,
            },
            CALL_CYCLES,
        )
        .await
        .map_err(|e| format!("IC call failed: {:?}", e))?;

    let bytes = match resp.0 {
        MultiCallResult::Consistent(CallResult::Ok(hex)) => hex,
        MultiCallResult::Consistent(CallResult::Err(e)) => {
            return Err(format!("eth_call error: {e:?}"));
        }
        MultiCallResult::Inconsistent(vec) => {
            return Err(format!("inconsistent RPC responses: {vec:?}"));
        }
    };

    //---------------- compare clone address ---------------------------------
    let ret = hex::decode(bytes.trim_start_matches("0x"))
        .map_err(|e| format!("call output decode: {e}"))?;
    if ret.len() < 32 {
        return Err("eth_call returned <32 bytes".into());
    }
    let expected = &ret[12..32]; // right-most 20 bytes
    Ok(hex::encode(expected).eq_ignore_ascii_case(clone_addr.trim_start_matches("0x")))
}
