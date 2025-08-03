use ethers_core::abi::{ParamType, Token, encode, short_signature};

use crate::{EscrowParams, evm::immutable::build_immutable_tuple};

pub const ESCROW_SRC_ABI: &str = include_str!("abis/escrow_src.json");
pub const ESCROW_DST_ABI: &str = include_str!("abis/escrow_dst.json");

// calldata for Escrow.withdraw(bytes32 secret, Immutables immutables)
pub fn withdraw_calldata(
    secret: &Vec<u8>,
    params: &EscrowParams,
    order_hash: [u8; 32],
    maker: [u8; 20],
    taker: [u8; 20],
    token: [u8; 20],
) -> Vec<u8> {
    let method = short_signature(
        "withdraw",
        &[
            ParamType::FixedBytes(32),
            ParamType::Tuple(vec![
                ParamType::FixedBytes(32), // orderHash
                ParamType::FixedBytes(32), // hashlock
                ParamType::Uint(256),      // maker
                ParamType::Uint(256),      // taker
                ParamType::Uint(256),      // token
                ParamType::Uint(256),      // amount
                ParamType::Uint(256),      // safetyDeposit
                ParamType::Uint(256),      // timelocks
            ]),
        ],
    );

    let tuple = build_immutable_tuple(order_hash, params, maker, taker, token);
    let args = encode(&[Token::FixedBytes(secret.to_vec()), tuple]);
    [method.to_vec(), args].concat()
}

// calldata for Escrow.cancel(Immutables immutables)
pub fn cancel_calldata(
    params: &EscrowParams,
    order_hash: [u8; 32],
    maker: [u8; 20],
    taker: [u8; 20],
    token: [u8; 20],
) -> Vec<u8> {
    let method = short_signature(
        "cancel",
        &[ParamType::Tuple(vec![
            ParamType::FixedBytes(32), // orderHash
            ParamType::FixedBytes(32), // hashlock
            ParamType::Uint(256),      // maker
            ParamType::Uint(256),      // taker
            ParamType::Uint(256),      // token
            ParamType::Uint(256),      // amount
            ParamType::Uint(256),      // safetyDeposit
            ParamType::Uint(256),      // timelocks
        ])],
    );

    let tuple = build_immutable_tuple(order_hash, params, maker, taker, token);
    let args = encode(&[tuple]);
    [method.to_vec(), args].concat()
}
