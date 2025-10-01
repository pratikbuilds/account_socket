# Account Socket

Real-time Solana program account monitoring via WebSocket using Carbon RPC subscriptions.

## Overview

A WebSocket server that monitors Solana program accounts in real-time, decodes them, caches in Redis, stores in SQLite, and broadcasts updates to connected clients.

**Currently configured for:** Meteora DAMM V2 program accounts (but can be adapted for any Solana program)

## Features

- **Real-time monitoring**: Subscribe to any Solana program's accounts via Carbon RPC
- **WebSocket API**: Real-time bidirectional communication with subscription management
- **Caching**: Redis cache for fast account lookups
- **Persistence**: SQLite database for historical account states
- **Account decoding**: Automatic account data parsing based on program type

## Prerequisites

- **Rust** (1.70+) - Install from [rustup.rs](https://rustup.rs/)
- **Redis** - For caching
- **SQLite** - For database persistence
- **Solana RPC endpoint** - With subscription support (e.g., Helius, Triton)

### Install Dependencies

**macOS:**
```bash
brew install redis sqlite
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install redis-server sqlite3
```

**Windows:**
- Redis: [Download](https://github.com/microsoftarchive/redis/releases)
- SQLite: Usually pre-installed or [download](https://sqlite.org/download.html)

## Setup

### 1. Clone and Install

```bash
git clone <repository-url>
cd account_socket
cargo build --release
```

### 2. Configure Environment

Create a `.env` file:

```env
# Required: Solana RPC endpoint
RPC_URL=wss://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY

# Required: Database
DATABASE_URL=sqlite:account.db

# Optional: Redis (default shown)
REDIS_URL=redis://127.0.0.1:6379

# Optional: WebSocket server (defaults shown)
WEBSOCKET_HOST=127.0.0.1
WEBSOCKET_PORT=8080

# Optional: Database pool
DATABASE_MAX_CONNECTIONS=10

# Optional: Logging
RUST_LOG=info,account_socket=debug
```

### 3. Setup Database

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features sqlite

# Run migrations
sqlx migrate run --database-url sqlite:account.db
```

### 4. Start Redis

```bash
# Start Redis server
redis-server

# Or run as background service
brew services start redis  # macOS
sudo systemctl start redis # Linux
```

## Running the Server

```bash
# Development mode with debug logging
RUST_LOG=debug cargo run

# Production mode
cargo run --release
```

Server will start on `ws://127.0.0.1:8080/ws`

## Using the WebSocket API

### Connect

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => console.log('Connected');
ws.onmessage = (event) => {
    const update = JSON.parse(event.data);
    console.log('Account update:', update);
};
```

### Subscribe to Account

Send a subscription request:

```javascript
ws.send(JSON.stringify({
    action: "subscribe",
    pubkey: "CPpeWQrniBd4WARd3kEjS7XP1oxVtD8Fr3hie19F6gXV"
}));
```

You'll immediately receive the current account state (from cache or database), then real-time updates as the account changes.

### Unsubscribe from Account

```javascript
ws.send(JSON.stringify({
    action: "unsubscribe",
    pubkey: "CPpeWQrniBd4WARd3kEjS7XP1oxVtD8Fr3hie19F6gXV"
}));
```

### Response Format

```json
{
  "pubkey": "CPpeWQrniBd4WARd3kEjS7XP1oxVtD8Fr3hie19F6gXV",
  "source": "cache",
  "account": {
    "id": 123,
    "pubkey": "CPpeWQrniBd4WARd3kEjS7XP1oxVtD8Fr3hie19F6gXV",
    "slot": 370462731,
    "account_type": "Pool",
    "owner": "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB",
    "lamports": 8630400,
    "data_json": {
      "pool_fees": { ... },
      "token_a_mint": "...",
      "liquidity": "67020350331064650037410477447049",
      ...
    },
    "created_at": "2025-10-01T10:50:05Z"
  }
}
```

**Response fields:**
- `source`: Where the data came from (`cache`, `database`, or `realtime`)
- `account_type`: Type of account (Pool, Position, Config, etc.)
- `data_json`: Decoded account data (structure depends on account type)

## Testing with CLI Tools

### Using wscat

```bash
# Install wscat
npm install -g wscat

# Connect and subscribe
wscat -c ws://localhost:8080/ws
> {"action":"subscribe","pubkey":"CPpeWQrniBd4WARd3kEjS7XP1oxVtD8Fr3hie19F6gXV"}
```

### Using websocat

```bash
# Install websocat
cargo install websocat

# Connect and subscribe
echo '{"action":"subscribe","pubkey":"CPpeWQrniBd4WARd3kEjS7XP1oxVtD8Fr3hie19F6gXV"}' | websocat ws://localhost:8080/ws
```

## Adapting for Other Programs

Currently configured for **Meteora DAMM V2**. To monitor a different Solana program:

1. Add the program's decoder dependency to `Cargo.toml`
2. Update `src/main.rs` to use the new decoder:
   ```rust
   RpcProgramSubscribe::new(
       config.rpc_url.clone(),
       Filters::new(
           YOUR_PROGRAM_ID,  // Change this
           ...
       ),
   )
   ```
3. Update `src/processor.rs` to handle the new account types

## Database Queries

### View Recent Updates

```bash
sqlite3 account.db "SELECT pubkey, slot, account_type, created_at FROM account_updates ORDER BY created_at DESC LIMIT 10;"
```

### View Account History

```bash
sqlite3 account.db "SELECT slot, account_type, created_at FROM account_updates WHERE pubkey = 'YOUR_PUBKEY' ORDER BY slot DESC;"
```

## Architecture

```
Solana RPC (Geyser)
        ↓
Carbon Pipeline (Subscribe to Program Accounts)
        ↓
Decoder (Parse Account Data)
        ↓
Processor → SQLite + Redis
        ↓
WebSocket Broadcast → Connected Clients
```

## Technical Details

- **Cache-aside pattern**: On subscribe, checks Redis → SQLite, then streams real-time updates
- **JSON serialization**: Uses `serde_json` with `arbitrary_precision` for u128 values
- **Account types supported**: Pool, Position, Config, ClaimFeeOperator, TokenBadge, Vesting (Meteora DAMM V2)

## Troubleshooting

**Redis connection failed:**
```bash
redis-cli ping  # Should return "PONG"
```

**Database locked:**
```bash
# Close any open connections to account.db
lsof account.db
```

**No updates received:**
- Check RPC endpoint is valid and has subscription support
- Verify the account pubkey is owned by the program you're monitoring
- Check logs: `RUST_LOG=debug cargo run`
