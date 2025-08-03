// use candid::Principal;
// use ic_cdk::call::Call;
// use shared::{EscrowInfo, EscrowParams};

// pub async fn verify_escrow(escrow_canister: &str, expected: &EscrowParams) -> Result<bool, String> {
//     ic_cdk::println!("[verify_escrow]");
//     let pid =
//         Principal::from_text(escrow_canister).map_err(|e| format!("invalid canister id: {e}"))?;

//     let info: EscrowInfo = Call::unbounded_wait(pid, "get_escrow")
//         .await
//         .map_err(|e| format!("{e:?}"))?
//         .candid()
//         .map_err(|e| format!("decode: {e:?}"))?;

//     Ok(&info.params == expected)
// }
