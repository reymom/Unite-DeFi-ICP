use ethers_core::{abi::Token, types::U256};

use crate::{EscrowParams, Role, Timelocks};

const SRC_WITHDRAW: u8 = 224;
const SRC_PUBLIC_WITHDRAW: u8 = 192;
const SRC_CANCEL: u8 = 160;
const SRC_PUBLIC_CANCEL: u8 = 128;
const DST_WITHDRAW: u8 = 96;
const DST_PUBLIC_WITHDRAW: u8 = 64;
const DST_CANCEL: u8 = 32;
const DST_PUBLIC_CANCEL: u8 = 0;

pub fn build_immutable_tuple(
    order_hash: [u8; 32],
    params: &EscrowParams,
    maker: [u8; 20],
    taker: [u8; 20],
    token: [u8; 20],
) -> Token {
    let timelock_word = U256::from(Packed(&params.timelock, params.role.clone()));
    Token::Tuple(vec![
        Token::FixedBytes(order_hash.to_vec()),
        Token::FixedBytes(params.hashlock.to_vec()),
        Token::Uint(U256::from_big_endian(&maker)),
        Token::Uint(U256::from_big_endian(&taker)),
        Token::Uint(U256::from_big_endian(&token)),
        Token::Uint(params.amount.into()),
        Token::Uint(params.safety_deposit.into()),
        Token::Uint(timelock_word),
    ])
}

fn pack_timelocks(t: &Timelocks, role: Role) -> U256 {
    let mut v = U256::zero();

    match role {
        Role::Source => {
            v |= U256::from(t.withdrawal) << SRC_WITHDRAW;
            v |= U256::from(t.public_withdrawal) << SRC_PUBLIC_WITHDRAW;
            v |= U256::from(t.cancellation) << SRC_CANCEL;
            v |= U256::from(t.public_cancellation.unwrap_or(0)) << SRC_PUBLIC_CANCEL;
            // the four *Dst* fields are filled with the **same** relative
            // delays.  The factory just needs numbers – they will be
            // interpreted on the other chain.
            v |= U256::from(t.withdrawal) << DST_WITHDRAW;
            v |= U256::from(t.public_withdrawal) << DST_PUBLIC_WITHDRAW;
            v |= U256::from(t.cancellation) << DST_CANCEL;
            v |= U256::from(t.public_cancellation.unwrap_or(0)) << DST_PUBLIC_CANCEL;
        }
        Role::Destination => {
            v |= U256::from(t.withdrawal) << DST_WITHDRAW;
            v |= U256::from(t.public_withdrawal) << DST_PUBLIC_WITHDRAW;
            v |= U256::from(t.cancellation) << DST_CANCEL;
            v |= U256::from(t.public_cancellation.unwrap_or(0)) << DST_PUBLIC_CANCEL;
            // for completeness fill the Src half too
            v |= U256::from(t.withdrawal) << SRC_WITHDRAW;
            v |= U256::from(t.public_withdrawal) << SRC_PUBLIC_WITHDRAW;
            v |= U256::from(t.cancellation) << SRC_CANCEL;
            v |= U256::from(t.public_cancellation.unwrap_or(0)) << SRC_PUBLIC_CANCEL;
        }
    }
    v
}

struct Packed<'a>(&'a Timelocks, Role);
impl<'a> From<Packed<'a>> for U256 {
    fn from(p: Packed<'a>) -> U256 {
        pack_timelocks(p.0, p.1)
    }
}
