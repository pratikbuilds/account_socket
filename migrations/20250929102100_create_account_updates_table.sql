-- Create account_updates table for storing Solana account state changes
CREATE TABLE account_updates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pubkey TEXT NOT NULL,
    slot INTEGER NOT NULL,
    account_type TEXT NOT NULL,
    owner TEXT NOT NULL,
    lamports INTEGER NOT NULL,
    data_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for efficient querying
CREATE INDEX idx_pubkey ON account_updates(pubkey);
CREATE INDEX idx_slot ON account_updates(slot);
CREATE INDEX idx_account_type ON account_updates(account_type);
CREATE INDEX idx_pubkey_slot ON account_updates(pubkey, slot DESC);
