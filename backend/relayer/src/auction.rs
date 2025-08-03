use shared::{Order, PriceCurve};

use crate::state::{ACTIVE_AUCTIONS, FINISHED_AUCTIONS};

/// Advance all auctions; called by relayer every tick.
pub fn step(now: u64) {
    let mut to_archive = vec![];

    ACTIVE_AUCTIONS.with(|m| {
        for (key, auction) in m.borrow_mut().iter_mut() {
            ic_cdk::println!(
                "[auction {}] now={}, next_drop_at={}, current_price={:.6}, finished={}, winner={:?}",
                key,
                now,
                auction.next_drop_at,
                auction.current_price,
                auction.finished,
                auction.winner
            );
            if auction.finished || auction.winner.is_some() {
                ic_cdk::println!("[auction {}] finished", key);
                to_archive.push(key.clone());
                continue;
            }

            // before auction start – keep start_price
            if now < auction.order.auction_start_at {
                ic_cdk::println!(
                    "[auction {}] not started yet: now={} start_at={}",
                    key, now, auction.order.auction_start_at
                );
                continue;
            }

            if now >= auction.next_drop_at {
                let elapsed = now - auction.order.auction_start_at;
                let old_price = auction.current_price;
                let floor_ratio_val = floor_ratio(&auction.order);
                let floor_price = floor_ratio_val * auction.order.auction_start_rate;

                let new_price = curve_price_at(
                    &auction.order.price_curve,
                    elapsed,
                    auction.order.auction_start_rate,
                    floor_ratio_val,
                );
                if (new_price - auction.current_price).abs() > f64::EPSILON {
                    ic_cdk::println!(
                        "[auction {}] elapsed={} old_price={:.6} new_price={:.6} floor_price={:.6} start_rate={:.6} floor_ratio={:.6}",
                        key,
                        elapsed,
                        old_price,
                        new_price,
                        floor_price,
                        auction.order.auction_start_rate,
                        floor_ratio_val
                    );
                    auction.current_price = new_price;
                }
                // schedule next step
                auction.next_drop_at = now + 5;
                ic_cdk::println!(
                    "[auction {}] next_drop_at updated to {}",
                    key, auction.next_drop_at
                );
                // expire if reached floor
                if (new_price - auction.order.min_return_amount as f64).abs() < 1e-12 {
                    auction.finished = true;
                    ic_cdk::println!(
                        "[auction {}] reached floor/expired: new_price={:.6} min_return_amount={} floor_price={:.6}",
                        key, new_price, auction.order.min_return_amount, floor_price
                    );
                }
            }
        }
    });

    // move finished ones out of ACTIVE
    if !to_archive.is_empty() {
        ACTIVE_AUCTIONS.with_borrow_mut(|src| {
            FINISHED_AUCTIONS.with_borrow_mut(|dst| {
                for k in to_archive {
                    if let Some(auc) = src.remove(&k) {
                        ic_cdk::println!("[auction {}] archiving (finished or has winner)", k);
                        dst.insert(k, auc);
                    }
                }
            })
        });
    };
}

pub fn curve_price_at(curve: &PriceCurve, t: u64, start_rate: f64, floor_ratio: f64) -> f64 {
    let m = curve.price_at(t);
    let p = start_rate * m;
    p.max(floor_ratio * start_rate)
}

pub fn floor_ratio(order: &Order) -> f64 {
    order.min_return_amount as f64 / (order.making_amount) as f64 / order.auction_start_rate
}
