use shared::{AuctionInfo, Order};

use crate::state::{ACTIVE_AUCTIONS, FINISHED_AUCTIONS};

const STEP_SEC: u64 = 5;

pub fn insert_order(order: Order, now: u64) {
    let key = order.order_hash.clone();
    let start_price = order.auction_start_rate;
    let first_drop = if now < order.auction_start_at {
        order.auction_start_at // wait until start
    } else {
        now + STEP_SEC // already running → next 5-sec slot
    };

    let initial = AuctionInfo {
        order,
        current_price: start_price,
        next_drop_at: first_drop,
        winner: None,
        finished: false,
    };
    ACTIVE_AUCTIONS.with(|m| {
        m.borrow_mut().insert(key, initial);
    });
}

pub fn export_auctions() -> (Vec<(String, AuctionInfo)>, Vec<(String, AuctionInfo)>) {
    let act = ACTIVE_AUCTIONS.with(|m| {
        m.borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    });
    let fin = FINISHED_AUCTIONS.with(|m| {
        m.borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    });
    (act, fin)
}

pub fn import_auctions((act, fin): (Vec<(String, AuctionInfo)>, Vec<(String, AuctionInfo)>)) {
    ACTIVE_AUCTIONS.with(|m| m.borrow_mut().extend(act));
    FINISHED_AUCTIONS.with(|m| m.borrow_mut().extend(fin));
}
