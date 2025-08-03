# Resolver Canister — Fully onchain resolver of 1inch auctions between ICP and EVM

## 0. Architecture overview

```text
+-------------+        list_active_auctions()        +--------------+
|  Resolver   | <----------------------------------- |    Relayer    |
+-------------+                                     +--------------+
       | accept_price(order_hash) if willing
       v
+-------------+       deploy escrows       +--------------+
|  Resolver   | -------------------------> |   ICP/EVM    |
+-------------+                            |   Escrows    |
       |                                    +--------------+
       | verify_and_reveal_secret()
       v
+-------------+ <------------------------- +--------------+
|  Resolver   |          secret            |    Relayer    |
+-------------+                            +--------------+
       | withdraw(secret) both chains
```

## 1. What the resolver does

- Periodically polls relayer for `list_active_auctions()`.
- Checks if:
  - No winner → `accept_price(order_hash)`.
  - We are winner → deploy escrows and request `verify_and_reveal_secret()`.
  - Another is winner → try to `withdraw` if window open or `cancel` if expired.

## 2. Lifecycle:

1. **Polling**: Every N seconds via `spawn_tick`, call relayer for active auctions.
2. **Check winner**:
   - If winner is `self`:
     - Deploy ICP & EVM escrows.
     - Call `verify_and_reveal_secret()` and extract `secret`.
     - Call both `withdraw_icp(secret)` and `withdraw_escrow(secret)`.
   - If another is winner:
     - If in public withdrawal window → use `get_public_secret` and withdraw.
     - Else if in cancellation window → call `cancel_escrow()`.
3. **Mark revealed** in in-memory state to avoid reprocessing.
