#!/usr/bin/env node

/**
 * WebSocket Client Test for Account Socket Server
 *
 * This script demonstrates how to connect to the account socket server,
 * subscribe to account updates, and handle real-time data.
 *
 * Usage: node test_client.js [account_pubkey]
 */

const WebSocket = require('ws');

// Configuration
const WS_URL = process.env.WS_URL || 'ws://localhost:8080/ws';
const TEST_PUBKEY = process.argv[2] || 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'; // USDC token account

// Colors for console output
const colors = {
    reset: '\x1b[0m',
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    magenta: '\x1b[35m',
    cyan: '\x1b[36m'
};

function log(color, prefix, message) {
    const timestamp = new Date().toISOString();
    console.log(`${color}[${timestamp}] ${prefix}${colors.reset} ${message}`);
}

function logInfo(message) {
    log(colors.blue, 'INFO', message);
}

function logSuccess(message) {
    log(colors.green, 'SUCCESS', message);
}

function logWarning(message) {
    log(colors.yellow, 'WARNING', message);
}

function logError(message) {
    log(colors.red, 'ERROR', message);
}

function logData(message) {
    log(colors.cyan, 'DATA', message);
}

class AccountSocketClient {
    constructor(url) {
        this.url = url;
        this.ws = null;
        this.subscriptions = new Set();
        this.messageCount = 0;
        this.startTime = Date.now();
    }

    connect() {
        return new Promise((resolve, reject) => {
            logInfo(`Connecting to ${this.url}...`);

            this.ws = new WebSocket(this.url);

            this.ws.on('open', () => {
                logSuccess('Connected to account socket server');
                this.startTime = Date.now();
                resolve();
            });

            this.ws.on('message', (data) => {
                this.handleMessage(data);
            });

            this.ws.on('close', (code, reason) => {
                logWarning(`Connection closed: ${code} ${reason}`);
                this.cleanup();
            });

            this.ws.on('error', (error) => {
                logError(`WebSocket error: ${error.message}`);
                reject(error);
            });

            // Timeout after 10 seconds
            setTimeout(() => {
                if (this.ws.readyState !== WebSocket.OPEN) {
                    reject(new Error('Connection timeout'));
                }
            }, 10000);
        });
    }

    handleMessage(data) {
        try {
            const message = JSON.parse(data.toString());
            this.messageCount++;

            const elapsed = ((Date.now() - this.startTime) / 1000).toFixed(1);

            logData(`Message #${this.messageCount} (${elapsed}s):`);
            console.log(JSON.stringify(message, null, 2));

            // Log key information
            if (message.pubkey && message.account) {
                logInfo(`Account: ${message.pubkey.substring(0, 8)}...`);
                logInfo(`Type: ${message.account.account_type}`);
                logInfo(`Source: ${message.source}`);
                logInfo(`Slot: ${message.account.slot}`);
                logInfo(`Lamports: ${message.account.lamports}`);
            }

            console.log('─'.repeat(80));

        } catch (error) {
            logError(`Failed to parse message: ${error.message}`);
            console.log('Raw data:', data.toString());
        }
    }

    subscribe(pubkey) {
        if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
            throw new Error('WebSocket not connected');
        }

        const request = {
            action: 'subscribe',
            pubkey: pubkey
        };

        logInfo(`Subscribing to account: ${pubkey}`);
        this.ws.send(JSON.stringify(request));
        this.subscriptions.add(pubkey);
        logSuccess(`Subscription request sent for ${pubkey}`);
    }

    unsubscribe(pubkey) {
        if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
            throw new Error('WebSocket not connected');
        }

        const request = {
            action: 'unsubscribe',
            pubkey: pubkey
        };

        logInfo(`Unsubscribing from account: ${pubkey}`);
        this.ws.send(JSON.stringify(request));
        this.subscriptions.delete(pubkey);
        logSuccess(`Unsubscription request sent for ${pubkey}`);
    }

    sendInvalidMessage() {
        if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
            throw new Error('WebSocket not connected');
        }

        logInfo('Sending invalid message to test error handling...');
        this.ws.send('invalid json');
    }

    getStats() {
        const elapsed = (Date.now() - this.startTime) / 1000;
        const rate = this.messageCount / elapsed;

        return {
            messageCount: this.messageCount,
            elapsed: elapsed.toFixed(1),
            rate: rate.toFixed(2),
            subscriptions: Array.from(this.subscriptions)
        };
    }

    cleanup() {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
        this.subscriptions.clear();
    }

    close() {
        logInfo('Closing connection...');
        this.cleanup();
    }
}

// Interactive testing function
async function runInteractiveTest() {
    const client = new AccountSocketClient(WS_URL);

    try {
        await client.connect();

        // Subscribe to test account
        client.subscribe(TEST_PUBKEY);

        // Set up graceful shutdown
        process.on('SIGINT', () => {
            logInfo('Received SIGINT, shutting down gracefully...');

            const stats = client.getStats();
            logInfo(`Final statistics:`);
            logInfo(`  Messages received: ${stats.messageCount}`);
            logInfo(`  Elapsed time: ${stats.elapsed}s`);
            logInfo(`  Average rate: ${stats.rate} msg/s`);
            logInfo(`  Active subscriptions: ${stats.subscriptions.length}`);

            client.close();
            process.exit(0);
        });

        // Demo commands after 5 seconds
        setTimeout(() => {
            logInfo('Running demo commands...');

            // Test invalid message (should be handled gracefully)
            client.sendInvalidMessage();

            // Example of subscribing to another account
            const anotherAccount = 'So11111111111111111111111111111111111111112'; // SOL token
            setTimeout(() => client.subscribe(anotherAccount), 2000);

            // Example of unsubscribing
            setTimeout(() => client.unsubscribe(TEST_PUBKEY), 10000);

        }, 5000);

        // Print statistics every 30 seconds
        setInterval(() => {
            const stats = client.getStats();
            logInfo(`Statistics: ${stats.messageCount} messages in ${stats.elapsed}s (${stats.rate} msg/s)`);
        }, 30000);

        logInfo('Client is running. Press Ctrl+C to exit.');
        logInfo(`Listening for updates on account: ${TEST_PUBKEY}`);

    } catch (error) {
        logError(`Failed to start client: ${error.message}`);
        process.exit(1);
    }
}

// Stress test function
async function runStressTest(accountCount = 10, duration = 60) {
    logInfo(`Starting stress test: ${accountCount} accounts for ${duration} seconds`);

    const client = new AccountSocketClient(WS_URL);

    try {
        await client.connect();

        // Generate test accounts (these might not exist, but will test subscription handling)
        const testAccounts = [];
        for (let i = 0; i < accountCount; i++) {
            // Generate dummy pubkeys for testing
            const pubkey = 'Test' + '1'.repeat(40) + i.toString().padStart(4, '0');
            testAccounts.push(pubkey);
        }

        // Subscribe to all test accounts
        for (const pubkey of testAccounts) {
            client.subscribe(pubkey);
            await new Promise(resolve => setTimeout(resolve, 100)); // Small delay
        }

        logInfo(`Subscribed to ${testAccounts.length} accounts`);

        // Run for specified duration
        setTimeout(() => {
            const stats = client.getStats();
            logInfo('Stress test completed!');
            logInfo(`Results:`);
            logInfo(`  Messages received: ${stats.messageCount}`);
            logInfo(`  Duration: ${stats.elapsed}s`);
            logInfo(`  Average rate: ${stats.rate} msg/s`);
            logInfo(`  Subscriptions: ${stats.subscriptions.length}`);

            client.close();
            process.exit(0);
        }, duration * 1000);

    } catch (error) {
        logError(`Stress test failed: ${error.message}`);
        process.exit(1);
    }
}

// Command line interface
function showHelp() {
    console.log(`
Account Socket Server Test Client

Usage:
  node test_client.js [command] [options]

Commands:
  interactive [pubkey]          Run interactive test (default)
  stress [accounts] [duration]  Run stress test with multiple subscriptions
  help                         Show this help message

Examples:
  node test_client.js
  node test_client.js interactive EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
  node test_client.js stress 20 120

Environment Variables:
  WS_URL                       WebSocket server URL (default: ws://localhost:8080/ws)
`);
}

// Main execution
function main() {
    const command = process.argv[2] || 'interactive';

    switch (command) {
        case 'interactive':
            runInteractiveTest();
            break;

        case 'stress':
            const accountCount = parseInt(process.argv[3]) || 10;
            const duration = parseInt(process.argv[4]) || 60;
            runStressTest(accountCount, duration);
            break;

        case 'help':
        case '--help':
        case '-h':
            showHelp();
            break;

        default:
            // Treat unknown commands as pubkey for interactive mode
            if (command.length === 44) { // Typical Solana pubkey length
                process.argv[2] = command;
                runInteractiveTest();
            } else {
                logError(`Unknown command: ${command}`);
                showHelp();
                process.exit(1);
            }
    }
}

// Check if WebSocket is available
if (typeof WebSocket === 'undefined') {
    try {
        require('ws');
    } catch (error) {
        logError('WebSocket library not found. Install with: npm install ws');
        process.exit(1);
    }
}

// Run the main function
if (require.main === module) {
    main();
}

module.exports = { AccountSocketClient };