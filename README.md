# Account Socket Server

A high-performance Rust WebSocket server that streams real-time Solana account updates using the Carbon framework and Meteora DAMM V2 decoder. The server provides real-time account data with Redis caching and SQLite persistence.

## Features

- 🔥 **Real-time Account Streaming**: Uses Carbon pipeline with Yellowstone Geyser for low-latency Solana account updates
- 🌐 **WebSocket API**: Real-time bidirectional communication with subscription management
- 🔴 **Redis Caching**: High-performance caching layer with TTL management
- 💾 **SQLite Database**: Persistent storage with efficient indexing
- 📊 **Structured Logging**: Comprehensive tracing with multiple log levels
- 🏊 **Multiple Account Types**: Supports Pool, Position, Config, ClaimFeeOperator, TokenBadge, and Vesting accounts

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Solana RPC    │───▶│  Carbon Pipeline │───▶│   Processor     │
│   (Geyser)      │    │  (Meteora DAMM)  │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                         │
                                                         ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  WebSocket      │◀───│  Broadcast       │◀───│  Database +     │
│  Clients        │    │  Channel         │    │  Cache          │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Prerequisites

- **Rust** (1.70+): Install from [rustup.rs](https://rustup.rs/)
- **Redis**: For caching layer
- **SQLite**: For database persistence
- **Solana RPC Endpoint**: With Geyser support (like Helius)

### Installing Dependencies

#### macOS (via Homebrew)
```bash
brew install redis sqlite
```

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install redis-server sqlite3
```

#### Windows
- Download Redis from [releases page](https://github.com/microsoftarchive/redis/releases)
- SQLite comes with Windows or download from [sqlite.org](https://sqlite.org/download.html)

## Quick Start

### 1. Clone and Setup

```bash
git clone <repository-url>
cd account_socket
```

### 2. Environment Configuration

Create a `.env` file in the project root:

```env
# Required: Solana RPC endpoint with Geyser support
RPC_URL=wss://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY

# Required: Database URL
DATABASE_URL=sqlite:account.db

# Optional: Redis configuration (defaults shown)
REDIS_URL=redis://127.0.0.1:6379

# Optional: WebSocket server configuration (defaults shown)
WEBSOCKET_HOST=127.0.0.1
WEBSOCKET_PORT=8080

# Optional: Database configuration (defaults shown)
DATABASE_MAX_CONNECTIONS=10

# Optional: Logging configuration
RUST_LOG=info,account_socket=debug
```

### 3. Database Setup

Run the database migration to create the required tables:

```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features sqlite

# Run migrations
sqlx migrate run --database-url sqlite:account.db
```

### 4. Start Services

Start Redis (if not already running):

```bash
# macOS/Linux
redis-server

# Or as a service
brew services start redis  # macOS
sudo systemctl start redis # Linux
```

### 5. Run the Server

```bash
# Development mode with debug logging
RUST_LOG=debug cargo run

# Release mode
cargo run --release
```

The server will start on `http://127.0.0.1:8080/ws` by default.

## Usage

### WebSocket Connection

Connect to the WebSocket endpoint:

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = function() {
    console.log('Connected to account socket server');
};

ws.onmessage = function(event) {
    const data = JSON.parse(event.data);
    console.log('Account update:', data);
};
```

### Subscription Protocol

#### Subscribe to Account Updates

```javascript
// Subscribe to a specific account
ws.send(JSON.stringify({
    action: "subscribe",
    pubkey: "ACCOUNT_PUBLIC_KEY_HERE"
}));
```

#### Unsubscribe from Account Updates

```javascript
// Unsubscribe from a specific account
ws.send(JSON.stringify({
    action: "unsubscribe",
    pubkey: "ACCOUNT_PUBLIC_KEY_HERE"
}));
```

### Message Format

#### Subscription Request
```json
{
    "action": "subscribe" | "unsubscribe",
    "pubkey": "string"
}
```

#### Account Update Response
```json
{
    "pubkey": "string",
    "source": "cache" | "database" | "realtime",
    "account": {
        "id": 123,
        "pubkey": "string",
        "slot": 123456789,
        "account_type": "Pool" | "Position" | "Config" | "ClaimFeeOperator" | "TokenBadge" | "Vesting",
        "owner": "string",
        "lamports": 1000000,
        "data_json": {}, // Decoded account data
        "created_at": "2024-09-30T12:00:00Z"
    }
}
```

## Testing

### 1. Manual WebSocket Testing

You can test the WebSocket connection using various tools:

#### Using `websocat`

```bash
# Install websocat
cargo install websocat

# Connect and send subscription
echo '{"action":"subscribe","pubkey":"YOUR_ACCOUNT_PUBKEY"}' | websocat ws://localhost:8080/ws
```

#### Using Node.js

```javascript
const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8080/ws');

ws.on('open', function open() {
    console.log('Connected');

    // Subscribe to an account
    ws.send(JSON.stringify({
        action: 'subscribe',
        pubkey: 'ACCOUNT_PUBKEY_HERE'
    }));
});

ws.on('message', function message(data) {
    const update = JSON.parse(data);
    console.log('Account update:', JSON.stringify(update, null, 2));
});
```

#### Using Python

```python
import asyncio
import websockets
import json

async def test_websocket():
    uri = "ws://localhost:8080/ws"
    async with websockets.connect(uri) as websocket:
        # Subscribe to an account
        await websocket.send(json.dumps({
            "action": "subscribe",
            "pubkey": "ACCOUNT_PUBKEY_HERE"
        }))

        # Listen for updates
        async for message in websocket:
            data = json.loads(message)
            print(f"Account update: {json.dumps(data, indent=2)}")

asyncio.run(test_websocket())
```

### 2. Database Inspection

Check stored account data:

```bash
# Open SQLite database
sqlite3 account.db

# View recent account updates
.headers on
.mode table
SELECT * FROM account_updates ORDER BY created_at DESC LIMIT 10;

# View specific account history
SELECT * FROM account_updates WHERE pubkey = 'YOUR_PUBKEY' ORDER BY slot DESC;

# Check indexes
.schema account_updates
```

### 3. Redis Cache Inspection

Check cached data:

```bash
# Connect to Redis
redis-cli

# View all account keys
KEYS account:*

# Get specific account data
GET account:YOUR_PUBKEY

# Check TTL
TTL account:YOUR_PUBKEY
```

## Logging

The application uses structured logging with multiple levels:

- **ERROR**: Critical errors that require attention
- **WARN**: Warning conditions that might need investigation
- **INFO**: General information about application state
- **DEBUG**: Detailed information for debugging

### Log Configuration

Set the `RUST_LOG` environment variable to control logging:

```bash
# Debug level for the application
export RUST_LOG=debug

# Info level with debug for specific modules
export RUST_LOG=info,account_socket=debug

# Specific module debugging
export RUST_LOG=account_socket::websocket=trace
```

### Key Log Events

- 🚀 **Server startup and initialization**
- 🔌 **WebSocket client connections**
- 📝 **Subscription management**
- 🔄 **Account processing pipeline**
- 💾 **Database operations**
- 🔴 **Redis cache operations**
- 📡 **Real-time broadcasting**

## Performance Tuning

### Database Optimization

```sql
-- Additional indexes for performance
CREATE INDEX IF NOT EXISTS idx_created_at ON account_updates(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_lamports ON account_updates(lamports);

-- Analyze query performance
EXPLAIN QUERY PLAN SELECT * FROM account_updates WHERE pubkey = ? ORDER BY slot DESC LIMIT 1;
```

### Redis Configuration

```bash
# Increase memory limit in redis.conf
maxmemory 1gb
maxmemory-policy allkeys-lru

# Enable RDB snapshots
save 900 1
save 300 10
save 60 10000
```

### Environment Variables for Production

```env
# Production configuration
RUST_LOG=info
DATABASE_MAX_CONNECTIONS=20
WEBSOCKET_PORT=8080

# Redis persistence
REDIS_URL=redis://127.0.0.1:6379/0
```

## Troubleshooting

### Common Issues

1. **Connection Refused**
   - Ensure Redis is running: `redis-cli ping`
   - Check database file permissions
   - Verify RPC endpoint connectivity

2. **High Memory Usage**
   - Increase Redis maxmemory limit
   - Implement account data cleanup policies
   - Monitor active WebSocket connections

3. **Slow Performance**
   - Check database indexes
   - Monitor Redis cache hit rates
   - Verify network latency to Solana RPC

### Debug Commands

```bash
# Check server health
curl -v ws://localhost:8080/ws

# Monitor Redis performance
redis-cli monitor

# Database query performance
sqlite3 account.db "EXPLAIN QUERY PLAN SELECT * FROM account_updates WHERE pubkey = 'test'"

# Check log output
tail -f /var/log/account_socket.log
```

## Development

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

### Documentation

```bash
cargo doc --open
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## License

[Add your license information here]

---

For more detailed information about the Carbon framework and Meteora DAMM V2, see the [Carbon documentation](https://docs.carbon.so/).