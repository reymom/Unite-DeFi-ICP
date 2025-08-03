use ethers_core::utils::hex;

use crate::Asset;

pub mod broadcast;
pub mod config;
pub mod escrow;
pub mod factory;
pub mod immutable;
pub mod signer;

pub fn hex_to_bytes32(hex_str: &str) -> Result<[u8; 32], String> {
    let mut bytes = [0u8; 32];
    let clean_hex = hex_str.trim_start_matches("0x");
    ethers_core::utils::hex::decode_to_slice(clean_hex, &mut bytes)
        .map_err(|e| format!("Invalid hex: {e}"))?;
    Ok(bytes)
}

/// Convert a hex string (with/without `0x`) into fixed-20-byte array.
pub fn parse_evm_addr(s: &str) -> Result<[u8; 20], String> {
    let bytes =
        hex::decode(s.trim_start_matches("0x")).map_err(|e| format!("bad address {s}: {e}"))?;
    if bytes.len() != 20 {
        return Err(format!("address {s} length != 20 bytes"));
    }
    let mut arr = [0u8; 20];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

/// Convert `Asset::Erc20{..}` into raw 20-byte token address.
pub fn erc20_addr(asset: &Asset) -> Result<[u8; 20], String> {
    match asset {
        Asset::Erc20 {
            address: Some(a), ..
        } => parse_evm_addr(a),
        _ => Err("asset is not ERC-20 with address".into()),
    }
}
