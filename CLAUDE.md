# Account Socket

Real-time Solana account monitoring via WebSocket using Carbon RPC subscriptions.

## Stack

- **WebSocket**: Warp-based server for real-time client connections
- **Carbon Pipeline**: RPC program subscription for Meteora DAMM V2 accounts
- **Cache**: Redis for fast account lookups
- **Database**: SQLite for persistent account state
- **Decoder**: carbon-meteora-damm-v2-decoder for account parsing

## Architecture

```
Carbon RPC Subscribe → Decode Account → Store (SQLite + Redis) → Broadcast (WebSocket)
```

**Data Flow:**
1. Subscribe to Meteora DAMM V2 program accounts via Carbon
2. Decode account data (Pool, Position, Config, etc.)
3. Save to SQLite + cache in Redis
4. Broadcast updates to subscribed WebSocket clients

**WebSocket Protocol:**
- Subscribe: `{"action":"subscribe","pubkey":"<pubkey>"}`
- Unsubscribe: `{"action":"unsubscribe","pubkey":"<pubkey>"}`
- Updates: Real-time account state changes

## Notes

- Uses `serde_json` with `arbitrary_precision` for u128 serialization
- Cache-aside pattern: checks Redis → SQLite on subscription
- Supports Pool, Position, Config, ClaimFeeOperator, TokenBadge, Vesting account types