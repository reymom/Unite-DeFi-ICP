use std::time::Duration;

use ic_cdk::call::Call;
use shared::{Order, now_sec};

use crate::{
    auction,
    state::{self, LAST_PROCESSED_ORDER_ID, get_config},
};

const POLL_INTERVAL_SEC: u64 = 5;

pub fn spawn_tick() {
    ic_cdk_timers::set_timer_interval(Duration::from_secs(POLL_INTERVAL_SEC), || {
        ic_cdk::futures::spawn(async {
            if let Err(err) = tick().await {
                ic_cdk::println!("Tick error: {}", err);
            }
        });
    });
}

async fn tick() -> Result<(), String> {
    let last_id = LAST_PROCESSED_ORDER_ID.with(|c| *c.borrow());
    ic_cdk::println!("Relayer tick - fetching orders after ID {}", last_id);

    // Ensure configuration exists
    let orderbook_pid = get_config().orderbook_id;

    // Fetch up to 50 *pending* orders newer than `last_id`
    let (orders, _next): (Vec<(u64, Order)>, Option<u64>) =
        Call::unbounded_wait(orderbook_pid, "list_auctionable_orders")
            .with_args(&(Some(last_id), Some(50u64)))
            .await
            .map_err(|e| format!("{e:?}"))?
            .candid_tuple()
            .map_err(|e| format!("decode: {e:?}"))?;

    if orders.is_empty() {
        ic_cdk::println!("No new auctionable orders found.");
    }

    let now_sec = now_sec();
    for (id, order) in orders {
        ic_cdk::println!("Tracking order id {id} (hash {})", order.order_hash);
        state::auctions::insert_order(order, now_sec);
        LAST_PROCESSED_ORDER_ID.with(|c| *c.borrow_mut() = id);
    }

    /* advance all auctions */
    auction::step(now_sec);

    Ok(())
}
