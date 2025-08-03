use crate::state::ORDER_SECRETS;

/// Called when maker creates the order.
pub fn store_secret(order_hash: &str, secret: Vec<u8>) {
    ORDER_SECRETS.with(|m| {
        m.borrow_mut().insert(order_hash.to_string(), secret);
    });
}

/// Takes & removes secret; returns None if not present.
pub fn take_secret(order_hash: &str) -> Option<Vec<u8>> {
    ORDER_SECRETS.with(|m| m.borrow_mut().remove(order_hash))
}
