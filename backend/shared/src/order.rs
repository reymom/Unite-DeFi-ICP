use std::borrow::Cow;

use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{Storable, storable::Bound};
use serde::Serialize;

use crate::{Asset, Timelocks};

/// Auction info for the relayer
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AuctionInfo {
    pub order: Order,
    pub current_price: f64,
    pub next_drop_at: u64,
    pub winner: Option<(Principal, String)>,
    pub finished: bool,
}

/// Represents a point in the piecewise linear price curve: timestamp offset (seconds) → price multiplier.
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct PricePoint {
    pub time_offset_secs: u64, // seconds from auction start
    pub price_multiplier: f64, // multiplier factor (e.g., 1.0 for 100%)
}

/// The piecewise linear decreasing price curve of the Dutch auction.
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct PriceCurve {
    /// Ordered price points defining the price as a function of elapsed time.
    /// The auction price at any time after auction start is interpolated linearly between points.
    pub points: Vec<PricePoint>,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct Order {
    // ─── 1inch payload ────────────────────────────────────────────────
    pub order_hash: String, // bytes32
    pub signature: Vec<u8>, // EIP-712
    pub maker_asset: Asset, // ERC-20 | ICRC | ICP
    pub taker_asset: Asset,
    pub maker_icp: Principal,
    pub maker_evm: String,
    pub making_amount: u128,
    pub salt: String,
    pub maker_traits: Option<String>,
    // ─── Dutch-auction fields ──────────────────────────────────────────
    pub auction_start_at: u64,
    pub waiting_period: u64,
    pub auction_start_rate: f64,
    pub min_return_amount: u64,
    pub price_curve: PriceCurve,
    // ─── Escrow-specific ──────────────────────────────────────────────
    pub safety_deposit: u64,
    pub timelocks: Timelocks, // common set; contract derives Src/Dst windows
    pub hashlock: [u8; 32],
}

impl Storable for Order {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).expect("order encode"))
    }

    fn into_bytes(self) -> Vec<u8> {
        candid::encode_one(self).expect("Order encode")
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(bytes.as_ref()).expect("order decode")
    }

    const BOUND: Bound = Bound::Bounded {
        // 8 KiB per record
        max_size: 8 * 1024,
        is_fixed_size: false,
    };
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Expired,
}

impl Storable for OrderStatus {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).expect("order status encode"))
    }

    fn into_bytes(self) -> Vec<u8> {
        candid::encode_one(self).expect("Order status encode")
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(bytes.as_ref()).expect("order status decode")
    }

    const BOUND: Bound = Bound::Bounded {
        // 1 KiB per record
        max_size: 1 * 1024,
        is_fixed_size: false,
    };
}

impl PriceCurve {
    /// linear interpolation between defined `points`
    pub fn price_at(&self, t: u64) -> f64 {
        if self.points.is_empty() {
            return 1.0;
        }
        // before first point
        if t <= self.points[0].time_offset_secs {
            return self.points[0].price_multiplier;
        }
        // inside
        for w in self.points.windows(2) {
            let (a, b) = (&w[0], &w[1]);
            if t <= b.time_offset_secs {
                let span = b.time_offset_secs - a.time_offset_secs;
                let dt = t - a.time_offset_secs;
                let slope = (b.price_multiplier - a.price_multiplier) / span as f64;
                return a.price_multiplier + slope * dt as f64;
            }
        }
        // after last point
        self.points.last().unwrap().price_multiplier
    }
}
