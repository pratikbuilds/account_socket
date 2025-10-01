# Account Socket

WebSocket server for monitoring Solana program accounts in real-time.

## What It Does

Subscribes to Solana program accounts via Carbon RPC, decodes them, stores in SQLite + Redis, and broadcasts updates to WebSocket clients.

**Currently monitoring:** Meteora DAMM V2 program (can be changed to any Solana program)

## Prerequisites

- Rust 1.70+
- Redis
- SQLite
- Solana RPC endpoint with subscription support

### Install Dependencies

**macOS:**
```bash
brew install redis sqlite
```

**Linux:**
```bash
sudo apt install redis-server sqlite3
```

## Setup

### 1. Clone and Build

```bash
git clone <repo-url>
cd account_socket
cargo build
```

### 2. Configure

Create `.env` file:

```env
RPC_URL=wss://mainnet.helius-rpc.com/?api-key=YOUR_KEY
DATABASE_URL=sqlite:account.db
REDIS_URL=redis://127.0.0.1:6379
WEBSOCKET_PORT=8080
```

### 3. Setup Database

```bash
cargo install sqlx-cli --no-default-features --features sqlite
sqlx migrate run
```

### 4. Start Redis

```bash
redis-server
```

## Run

```bash
cargo run
```

Server starts on `ws://127.0.0.1:8080/ws`

## Usage

### Connect

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
ws.onmessage = (event) => console.log(JSON.parse(event.data));
```

### Subscribe to Account

```javascript
ws.send(JSON.stringify({
    action: "subscribe",
    pubkey: "CPpeWQrniBd4WARd3kEjS7XP1oxVtD8Fr3hie19F6gXV"
}));
```

Returns current state immediately, then streams updates.

### Unsubscribe

```javascript
ws.send(JSON.stringify({
    action: "unsubscribe",
    pubkey: "CPpeWQrniBd4WARd3kEjS7XP1oxVtD8Fr3hie19F6gXV"
}));
```

### Response Format

```json
{
  "pubkey": "...",
  "source": "cache|database|realtime",
  "account": {
    "slot": 370462731,
    "account_type": "Pool",
    "owner": "...",
    "lamports": 8630400,
    "data_json": { /* decoded account data */ },
    "created_at": "2025-10-01T10:50:05Z"
  }
}
```

## CLI Testing

**Using wscat:**
```bash
npm install -g wscat
wscat -c ws://localhost:8080/ws
> {"action":"subscribe","pubkey":"CPpeWQrniBd4WARd3kEjS7XP1oxVtD8Fr3hie19F6gXV"}
```

**Using websocat:**
```bash
cargo install websocat
echo '{"action":"subscribe","pubkey":"CPpeWQrniBd4WARd3kEjS7XP1oxVtD8Fr3hie19F6gXV"}' | websocat ws://localhost:8080/ws
```

## Adapting for Other Programs

To monitor a different Solana program:

1. Add decoder to `Cargo.toml`
2. Update `PROGRAM_ID` in `src/main.rs`
3. Update account types in `src/processor.rs`

## Database Queries

```bash
# Recent updates
sqlite3 account.db "SELECT pubkey, slot, account_type FROM account_updates ORDER BY created_at DESC LIMIT 10;"

# Account history
sqlite3 account.db "SELECT slot, account_type FROM account_updates WHERE pubkey = 'YOUR_PUBKEY' ORDER BY slot DESC;"
```

## How It Works

```
Solana RPC → Carbon Pipeline → Decoder → Processor → SQLite + Redis → WebSocket Broadcast
```

- On subscribe: checks Redis → SQLite for current state, then streams real-time updates
- Uses `serde_json` with `arbitrary_precision` for u128 values
- Supports account types: Pool, Position, Config, ClaimFeeOperator, TokenBadge, Vesting

## Troubleshooting

**Redis not running:**
```bash
redis-cli ping  # should return PONG
```

**No updates:**
- Check RPC endpoint supports subscriptions
- Verify account is owned by the program
- Check logs: `RUST_LOG=debug cargo run`
