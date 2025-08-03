use ethers_core::abi::encode;

use crate::{EscrowParams, evm::immutable::build_immutable_tuple};

pub const FACTORY_ABI: &str = include_str!("abis/escrow_factory.json");
pub const ADDR_FACTORY: &str = "0xa7bCb4EAc8964306F9e3764f67Db6A7af6DdF99A";

pub const SEL_ADDR_SRC: [u8; 4] = [0x5b, 0x4c, 0xcc, 0xfd]; // addressOfEscrowSrc(...)
pub const SEL_ADDR_DST: [u8; 4] = [0xa4, 0x2f, 0xc8, 0x93]; // addressOfEscrowDst(...)

/// Encode the Solidity tuple for **SRC** helper.
pub fn encode_src_immutables(
    order_hash: [u8; 32],
    params: &EscrowParams,
    maker: [u8; 20],
    taker: [u8; 20],
    token: [u8; 20],
) -> Vec<u8> {
    let tuple = build_immutable_tuple(order_hash, params, maker, taker, token);
    [SEL_ADDR_SRC.to_vec(), encode(&[tuple])].concat()
}

/// Encode tuple for **DST** helper.
pub fn encode_dst_immutables(
    order_hash: [u8; 32],
    params: &EscrowParams,
    maker: [u8; 20],
    taker: [u8; 20],
    token: [u8; 20],
) -> Vec<u8> {
    let tuple = build_immutable_tuple(order_hash, params, maker, taker, token);
    [SEL_ADDR_DST.to_vec(), encode(&[tuple])].concat()
}
