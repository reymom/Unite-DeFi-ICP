# Relayer canister — Dutch-auction matching engine and cross-chain swaps validations

## 0. Architecture overview

```text
+-------------+          submit_order()          +--------------+
|  Front-end  |  ───────────────────────────────▶ |   Relayer    |
|  (Maker)    |                                   |   Canister   |
+-------------+                                   +--------------+
                                                    │
                                                    │ add_order()
                                                    ▼
                                  +--------------+  +--------------+
                                  |  Orderbook   |◀─┤ACTIVE_AUCTIONS|
                                  |  Canister    |  +--------------+
                                  +--------------+          │  tick()
                                                            │
                     get_active_auction(), accept_price()   ▼
+-------------+   ────────────────────────────────────────────────────┐
|  Resolver   |                                                      │
|  Bot / UI   |◀──────────────────────────────────────────────────────┘
+-------------+                      ▲
        │ verify_and_reveal_secret() │
        ▼                            │
+-------------+            +------------------+            +-------------+
|  ICP Src/Dst |  withdraw |  Relayer checks  |  eth_call  |  EVM Src/Dst|
|  Escrow      |◀─────────▶|  both escrows &  |◀──────────▶|  Escrow     |
+-------------+   secret   |   returns secret |   address  +-------------+
                           +------------------+   helper
```

## 1. What the relayer does

| Responsibility                                                                      | How it is implemented                                                                                                                                                     |
| ----------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Keep an _eye_ on the `Orderbook` canister and import fresh orders.                  | `tick()` runs every **5 s**. It calls `list_auctionable_orders(last_id, 50)` and stores the orders in `ACTIVE_AUCTIONS`.                                                  |
| Run a Dutch auction for every imported order.                                       | `auction::step(now)` recomputes each order’s price curve every tick and flips `finished=true` once the floor price is reached.                                            |
| Expose public endpoints for resolvers.                                              | `get_active_auction(order_hash)` returns an `AuctionPublic` with the current price and the src/dst escrow parameters at that price.                                       |
| Verify both escrows after the resolver deploys them and reveal the secret.          | `verify_and_reveal_secret(evm_addr, icp_canister, order_hash)` – does ICP-side checks, EVM `eth_call`, cross-checks the hash-lock and finally returns the 32-byte secret. |
| Persist the 32-byte secret that the **maker** supplies when the order is submitted. | `submit_order(order, secret)` stores the secret in local `ORDER_SECRETS`, then forwards the order to the `Orderbook`.                                                     |
| Move auctions from _active_ to _finished_ baskets.                                  | Once `.finished == true` **or** `.winner.is_some()` the entry is copied to `FINISHED_AUCTIONS` and dropped from the active map.                                           |

## 2. Data layout (thread-locals)

```rust
LAST_PROCESSED_ORDER_ID : u64                    // high-water mark vs Orderbook
ORDERBOOK_CANISTER_ID   : Option<Principal>      // configured on init
ORDER_SECRETS           : BTreeMap<String, Vec<u8>>
ACTIVE_AUCTIONS         : BTreeMap<String, AuctionInfo>
FINISHED_AUCTIONS       : BTreeMap<String, AuctionInfo>
```

## 3. API surface

| Candid method                                                                                 | Caller                | Purpose                                                                       |
| --------------------------------------------------------------------------------------------- | --------------------- | ----------------------------------------------------------------------------- |
| `submit_order(order : Order, secret : vec nat8)`                                              | **Front-end / maker** | Validates `secret`, stores it, then calls `Orderbook.add_order`.              |
| `get_active_auction(order_hash : text) -> ?AuctionPublic`                                     | **Resolver UI**       | Current price and escrow params (pre-calculated for the resolver).            |
| `verify_and_reveal_secret(evm_addr : text, icp_canister : text, order_hash : text) -> [nat8]` | **Resolver bot**      | After escrows are deployed: verifies both legs, marks winner, reveals secret. |
| `list_finished_auctions() -> vec AuctionPublic`                                               | anybody               | Historical data (optional convenience).                                       |
| `configure(orderbook_id : principal)`                                                         | ops                   | One-shot setter for the `ORDERBOOK_CANISTER_ID`.                              |

## 4. Auction timing rules

- `insert_order()` chooses `next_drop_at`:

```text
if now < auction_start_at  →  next_drop_at = auction_start_at
else                        →  next_drop_at = now + STEP_SEC(=5s)
```

- `auction::step()` will recompute `current_price` whenever `now ≥ next_drop_at`, then `next_drop_at += STEP_SEC`.

- Floor-check: if `current_price <= min_return_amount` → `finished=true`.

## 5. Escrow verification logic

1. ICP-leg (`icp::verify_escrow`): Confirms canister's `EscrowInfo` matches the expected `dst_params`.
2. EVM-leg (`evm::verify_esrcow`): Builds the Solidity tuple, packs timelocks, does an `eth_call` to
   `EscrowFactory.addressOfEscrow{Src|Dst}` and checks the returned `CREATE2` address equals `clone_addr`.
3. Hash-lock cross-check (must be identical on both chains).
4. Secret integrity check `(keccak256(secret) == hashlock)`.

## 6. Testing checklist

- **Submit order** via `submit_order`, ensure:
  - secret is stored (`ORDER_SECRETS` non-empty);
  - `Orderbook` contains the order (`ORDER_COUNTER` incremented).
- Wait one tick (5 s) – order appears in `ACTIVE_AUCTIONS`.
- Call `get_active_auction` – verify price & escrow params.
- Deploy escrows (mock or real), then call `verify_and_reveal_secret`.
  - Should return the same 32-byte secret, mark winner, move auction to `FINISHED`.
- Subsequent call returns `Err "secret not found / already revealed"`.

## 7. Future-proofing notes

- **Multi-EVM**: `rpc_url_for(chain_id)` is abstracted – just extend the match with more networks.
- **Non-Dutch pricing**: plug another curve function in `curve_price_at()`.
- **Automatic secret pruning**: currently `ORDER_SECRETS` is cleaned on reveal; consider a periodic GC for expired orders.
