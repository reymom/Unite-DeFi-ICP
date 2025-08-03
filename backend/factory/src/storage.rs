use candid::Principal;
use ic_stable_structures::{
    DefaultMemoryImpl, StableBTreeMap,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
};
use std::cell::RefCell;

use shared::EscrowInfo;

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub static ESCROW_REGISTRY: RefCell<StableBTreeMap<Principal, EscrowInfo, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(0))),
        )
    );

    pub static ESCROW_WASM: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}
