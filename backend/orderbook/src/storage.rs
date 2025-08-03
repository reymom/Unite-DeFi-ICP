use ic_stable_structures::{
    DefaultMemoryImpl, StableBTreeMap,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
};
use std::cell::RefCell;

use shared::{Order, OrderStatus};

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub static ORDER_BOOK: RefCell<StableBTreeMap<u64, (Order, OrderStatus), Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(0))),
        )
    );

    pub static ORDER_COUNTER: RefCell<u64> = RefCell::new(0);
}
