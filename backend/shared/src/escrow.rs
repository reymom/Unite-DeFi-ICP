use std::borrow::Cow;

use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{Storable, storable::Bound};
use serde::Serialize;

use crate::Timelocks;

#[derive(Clone, Debug, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum Role {
    /// Escrow that holds the *origin‑chain* funds (maker side in Fusion terminology)
    Source,
    /// Escrow that holds the *destination‑chain* funds (taker side)
    Destination,
}

#[derive(Clone, Debug, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum Asset {
    ICP,
    ICRC(Principal),
    Erc20 {
        chain_id: u64,
        address: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub struct EscrowParams {
    pub role: Role,
    pub initiator: String,
    pub counterparty: String,
    pub asset: Asset,
    pub amount: u64,
    pub hashlock: [u8; 32],
    pub timelock: Timelocks,
    pub payout_addr: String,
    pub safety_deposit: u64,
    // pub access_asset: Option<Asset>,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, PartialEq, Eq)]
pub enum EscrowState {
    Open,
    Locked,
    Partial(u64),
    Completed,
    Cancelled,
    Rescued,
    Expired,
}

// #[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
// pub struct EscrowInfo {
//     pub canister_id: Principal,
//     pub params: EscrowParams,
//     pub state: EscrowState,
// }

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct EscrowInfo {
    pub canister_id: Principal,
    pub params: EscrowParams,
    pub status: EscrowState,
    pub deployed_at: u64,
    pub locked_at: Option<u64>,
    pub claimed_secret: Option<[u8; 32]>,
    pub admin: Principal, // factory that deploys the escrow
}

impl Storable for EscrowInfo {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).expect("escrow encode"))
    }

    fn into_bytes(self) -> Vec<u8> {
        candid::encode_one(self).expect("EscrowInfo encode")
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(bytes.as_ref()).expect("escrow decode")
    }

    const BOUND: Bound = Bound::Bounded {
        // 4 KiB per record should be ample for params + metadata.
        max_size: 4 * 1024,
        is_fixed_size: false,
    };
}
