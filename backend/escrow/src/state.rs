use shared::EscrowInfo;
use std::cell::RefCell;

thread_local! {
    pub static STATE: RefCell<Option<EscrowInfo>> = RefCell::default();
}

pub fn get_escrow_info() -> EscrowInfo {
    STATE.with_borrow(|state| {
        state
            .as_ref()
            .expect("BUG: state is not initialized")
            .clone()
    })
}
