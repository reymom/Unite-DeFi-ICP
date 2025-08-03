use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use shared::{Order, OrderStatus};

use crate::storage::{ORDER_BOOK, ORDER_COUNTER};

mod storage;

#[init]
fn init() {
    ic_cdk::println!("[init] orderbook initialised");
}

#[pre_upgrade]
fn pre_upgrade() {
    let counter = ORDER_COUNTER.with_borrow(|w| w.clone());
    ic_cdk::storage::stable_save((counter,)).unwrap();
}

#[post_upgrade]
fn post_upgrade() {
    let (counter,): (u64,) = ic_cdk::storage::stable_restore().unwrap();
    ORDER_COUNTER.with_borrow_mut(|c| *c = counter);
}

#[update]
async fn add_order(order: Order) -> Result<(), String> {
    let id = ORDER_COUNTER.with(|counter| {
        let mut ctr = counter.borrow_mut();
        *ctr += 1;
        *ctr
    });

    ORDER_BOOK.with_borrow_mut(|book| book.insert(id, (order, OrderStatus::Pending)));

    Ok(())
}

/// Update the status of an existing order.
/// Only relayer should call this logically (not enforced here).
#[update]
async fn update_order_status(id: u64, status: OrderStatus) -> Result<(), String> {
    ORDER_BOOK.with_borrow_mut(|books| {
        let order = books.get(&id);
        if order.is_none() {
            return Err("order not found".into());
        };
        books.insert(id, (order.unwrap().0, status));

        Ok(())
    })
}

#[query]
fn get_order(id: u64) -> Option<(Order, OrderStatus)> {
    ORDER_BOOK.with_borrow(|book| book.get(&id))
}

#[query]
fn list_orders() -> Vec<(Order, OrderStatus)> {
    ORDER_BOOK.with_borrow(|r| r.iter().map(|entry| entry.value().clone()).collect())
}

/// List active auctionable orders with pagination and page size.
/// Skip all non-Pending orders.
#[query]
fn list_auctionable_orders(
    after: Option<u64>, // cursor: last sequence number seen
    limit: Option<u64>, // max number of orders to return
) -> (Vec<(u64, Order)>, Option<u64>) {
    // (orders with sequence number, next cursor)
    let start_after = after.unwrap_or(0);
    let max_limit = limit.unwrap_or(10);

    let mut results = Vec::new();
    let mut next_cursor = None;

    ORDER_BOOK.with_borrow(|book| {
        // Iterate in ascending order starting after the cursor
        for entry in book.range((start_after + 1)..) {
            let seq = *entry.key();
            let (order, status) = entry.value();
            if status == OrderStatus::Pending {
                results.push((seq, order.clone()));
                if results.len() as u64 == max_limit {
                    next_cursor = Some(seq);
                    break;
                }
            }
        }
    });

    (results, next_cursor)
}

ic_cdk::export_candid!();
