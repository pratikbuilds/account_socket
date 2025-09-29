use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

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

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn insert_account_update(
        &self,
        update: NewAccountUpdate,
    ) -> Result<AccountUpdate, sqlx::Error> {
        let created_at = Utc::now();

        // Convert to i64 first to avoid temporary value issues
        let slot_i64 = update.slot as i64;
        let lamports_i64 = update.lamports as i64;

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

        Ok(AccountUpdate {
            id: row.id,
            pubkey: row.pubkey,
            slot: row.slot,
            account_type: row.account_type,
            owner: row.owner,
            lamports: row.lamports,
            data_json: serde_json::from_str(&row.data_json).unwrap(),
            created_at: DateTime::from_naive_utc_and_offset(row.created_at.unwrap(), Utc),
        })
    }

    pub async fn get_latest_account_state(
        &self,
        pubkey: &str,
    ) -> Result<Option<AccountUpdate>, sqlx::Error> {
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
            Ok(Some(AccountUpdate {
                id: row.id.unwrap(),
                pubkey: row.pubkey,
                slot: row.slot,
                account_type: row.account_type,
                owner: row.owner,
                lamports: row.lamports,
                data_json: serde_json::from_str(&row.data_json).unwrap(),
                created_at: DateTime::from_naive_utc_and_offset(row.created_at.unwrap(), Utc),
            }))
        } else {
            Ok(None)
        }
    }
}
