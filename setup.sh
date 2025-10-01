#!/bin/bash

# Account Socket Server Setup Script
# This script sets up the development environment for the account socket server

set -e

echo "🚀 Setting up Account Socket Server..."

# Color definitions for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Rust is installed
check_rust() {
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Please install from https://rustup.rs/"
        exit 1
    fi
    print_success "Rust $(rustc --version) found"
}

# Check if Redis is available
check_redis() {
    if ! command -v redis-server &> /dev/null; then
        print_warning "Redis not found. Installing via package manager..."

        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS
            if command -v brew &> /dev/null; then
                brew install redis
                print_success "Redis installed via Homebrew"
            else
                print_error "Homebrew not found. Please install Redis manually"
                exit 1
            fi
        elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
            # Linux
            if command -v apt-get &> /dev/null; then
                sudo apt-get update
                sudo apt-get install -y redis-server
                print_success "Redis installed via apt"
            elif command -v yum &> /dev/null; then
                sudo yum install -y redis
                print_success "Redis installed via yum"
            else
                print_error "Package manager not found. Please install Redis manually"
                exit 1
            fi
        else
            print_warning "Unsupported OS. Please install Redis manually"
        fi
    else
        print_success "Redis found"
    fi
}

# Check if SQLite is available
check_sqlite() {
    if ! command -v sqlite3 &> /dev/null; then
        print_warning "SQLite not found. Installing..."

        if [[ "$OSTYPE" == "darwin"* ]]; then
            brew install sqlite
        elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
            if command -v apt-get &> /dev/null; then
                sudo apt-get install -y sqlite3
            elif command -v yum &> /dev/null; then
                sudo yum install -y sqlite
            fi
        fi
        print_success "SQLite installed"
    else
        print_success "SQLite found"
    fi
}

# Install SQLx CLI for database migrations
install_sqlx_cli() {
    if ! command -v sqlx &> /dev/null; then
        print_status "Installing SQLx CLI for database migrations..."
        cargo install sqlx-cli --no-default-features --features sqlite
        print_success "SQLx CLI installed"
    else
        print_success "SQLx CLI found"
    fi
}

# Create .env file if it doesn't exist
create_env_file() {
    if [[ ! -f .env ]]; then
        print_status "Creating .env file..."
        cat > .env << EOF
# Solana RPC endpoint with Geyser support
RPC_URL=wss://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY_HERE

# Database configuration
DATABASE_URL=sqlite:account.db

# Redis configuration (optional - defaults shown)
REDIS_URL=redis://127.0.0.1:6379

# WebSocket server configuration (optional - defaults shown)
WEBSOCKET_HOST=127.0.0.1
WEBSOCKET_PORT=8080

# Database configuration (optional - defaults shown)
DATABASE_MAX_CONNECTIONS=10

# Logging configuration (optional)
RUST_LOG=info,account_socket=debug
EOF
        print_success ".env file created"
        print_warning "Please update the RPC_URL in .env with your actual API key"
    else
        print_success ".env file already exists"
    fi
}

# Run database migrations
setup_database() {
    print_status "Setting up database..."

    # Check if database exists
    if [[ -f account.db ]]; then
        print_warning "Database file already exists"
    fi

    # Run migrations
    print_status "Running database migrations..."
    sqlx migrate run --database-url sqlite:account.db
    print_success "Database migrations completed"

    # Create indexes for performance
    print_status "Creating additional performance indexes..."
    sqlite3 account.db << EOF
CREATE INDEX IF NOT EXISTS idx_created_at ON account_updates(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_lamports ON account_updates(lamports);
.quit
EOF
    print_success "Performance indexes created"
}

# Start Redis if not running
start_redis() {
    if ! pgrep -x "redis-server" > /dev/null; then
        print_status "Starting Redis server..."

        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS - try brew services first
            if command -v brew &> /dev/null; then
                brew services start redis
            else
                redis-server &
            fi
        else
            # Linux
            if command -v systemctl &> /dev/null; then
                sudo systemctl start redis
            else
                redis-server &
            fi
        fi

        # Wait a moment for Redis to start
        sleep 2

        # Test Redis connection
        if redis-cli ping > /dev/null 2>&1; then
            print_success "Redis server started successfully"
        else
            print_error "Failed to start Redis server"
            exit 1
        fi
    else
        print_success "Redis server is already running"
    fi
}

# Build the project
build_project() {
    print_status "Building the project..."
    cargo build
    print_success "Project built successfully"
}

# Test the setup
test_setup() {
    print_status "Testing setup..."

    # Test Redis connection
    if redis-cli ping > /dev/null 2>&1; then
        print_success "Redis connection test passed"
    else
        print_error "Redis connection test failed"
        return 1
    fi

    # Test database connection
    if sqlite3 account.db "SELECT COUNT(*) FROM account_updates;" > /dev/null 2>&1; then
        print_success "Database connection test passed"
    else
        print_error "Database connection test failed"
        return 1
    fi

    print_success "All tests passed!"
}

# Main setup function
main() {
    echo "🔧 Environment Setup"
    check_rust
    check_redis
    check_sqlite
    install_sqlx_cli

    echo ""
    echo "📁 Project Setup"
    create_env_file
    setup_database

    echo ""
    echo "🚀 Service Setup"
    start_redis
    build_project

    echo ""
    echo "🧪 Testing Setup"
    test_setup

    echo ""
    echo "✅ Setup completed successfully!"
    echo ""
    echo "📖 Next steps:"
    echo "1. Update the RPC_URL in .env with your Helius API key"
    echo "2. Run the server: cargo run"
    echo "3. Connect to WebSocket: ws://localhost:8080/ws"
    echo ""
    echo "📚 For testing examples, see the README.md file"
}

# Run main function
main "$@"