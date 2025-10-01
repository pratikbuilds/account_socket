use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{info, warn, error, debug, instrument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUpdate {
    pub id: i64,
    pub pubkey: String,
    pub slot: i64,
    pub account_type: String,
    pub owner: String,
    pub lamports: i64,
    pub data_json: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAccountUpdate {
    pub pubkey: String,
    pub slot: u64,
    pub account_type: String,
    pub owner: String,
    pub lamports: u64,
    pub data_json: serde_json::Value,
}

#[derive(Debug)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    #[instrument(skip(database_url))]
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        debug!("Establishing database connection");
        let pool = SqlitePool::connect(database_url).await?;
        info!("Database connection pool created successfully");
        Ok(Self { pool })
    }

    #[instrument(skip(self, update), fields(pubkey = %update.pubkey, account_type = %update.account_type, slot = update.slot))]
    pub async fn insert_account_update(
        &self,
        update: NewAccountUpdate,
    ) -> Result<AccountUpdate, sqlx::Error> {
        let created_at = Utc::now();

        // Convert to i64 first to avoid temporary value issues
        let slot_i64 = update.slot as i64;
        let lamports_i64 = update.lamports as i64;

        debug!(
            pubkey = %update.pubkey,
            account_type = %update.account_type,
            slot = update.slot,
            lamports = update.lamports,
            "💾 Executing database insert for account update"
        );

        let row = sqlx::query!(
            r#"
            INSERT INTO account_updates (pubkey, slot, account_type, owner, lamports, data_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            RETURNING id, pubkey, slot, account_type, owner, lamports, data_json, created_at
            "#,
            update.pubkey,
            slot_i64,
            update.account_type,
            update.owner,
            lamports_i64,
            update.data_json,
            created_at
        ).fetch_one(&self.pool).await?;

        let account_update = AccountUpdate {
            id: row.id,
            pubkey: row.pubkey,
            slot: row.slot,
            account_type: row.account_type,
            owner: row.owner,
            lamports: row.lamports,
            data_json: serde_json::from_str(&row.data_json).unwrap(),
            created_at: DateTime::from_naive_utc_and_offset(row.created_at.unwrap(), Utc),
        };

        info!(
            id = account_update.id,
            pubkey = %account_update.pubkey,
            account_type = %account_update.account_type,
            "✅ Account update inserted successfully into database"
        );

        Ok(account_update)
    }

    #[instrument(skip(self), fields(pubkey = %pubkey))]
    pub async fn get_latest_account_state(
        &self,
        pubkey: &str,
    ) -> Result<Option<AccountUpdate>, sqlx::Error> {
        debug!(pubkey = %pubkey, "🔍 Querying database for latest account state");

        let row = sqlx::query!(
            r#"
            SELECT id,pubkey,slot,account_type,owner,lamports,data_json,created_at
            FROM account_updates
            WHERE pubkey = ?1
            ORDER BY slot DESC
            LIMIT 1
            "#,
            pubkey
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let account_update = AccountUpdate {
                id: row.id.unwrap(),
                pubkey: row.pubkey,
                slot: row.slot,
                account_type: row.account_type,
                owner: row.owner,
                lamports: row.lamports,
                data_json: serde_json::from_str(&row.data_json).unwrap(),
                created_at: DateTime::from_naive_utc_and_offset(row.created_at.unwrap(), Utc),
            };

            info!(
                pubkey = %pubkey,
                id = account_update.id,
                slot = account_update.slot,
                account_type = %account_update.account_type,
                "✅ Latest account state retrieved from database"
            );

            Ok(Some(account_update))
        } else {
            debug!(pubkey = %pubkey, "🔍 No account state found in database");
            Ok(None)
        }
    }
}
