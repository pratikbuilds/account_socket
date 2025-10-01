use async_trait::async_trait;
use carbon_meteora_damm_v2_decoder::accounts::MeteoraDammV2Account;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

use carbon_core::{
    account::AccountProcessorInputType, error::Error, metrics::MetricsCollection,
    processor::Processor,
};

use crate::cache::RedisCache;
use crate::database::{Database, NewAccountUpdate};
use crate::websocket::WebSocketServer;

// Global shared state for processor dependencies
#[derive(Debug)]
pub struct ProcessorState {
    pub database: Arc<Database>,
    pub cache: Arc<RedisCache>,
    pub websocket_server: Arc<WebSocketServer>,
}

// Thread-safe global state
pub static PROCESSOR_STATE: tokio::sync::OnceCell<ProcessorState> =
    tokio::sync::OnceCell::const_new();

// Unit struct for Carbon pipeline compatibility
pub struct MeteoraDammV2AccountProcessor;

#[async_trait]
impl Processor for MeteoraDammV2AccountProcessor {
    type InputType = AccountProcessorInputType<MeteoraDammV2Account>;

    #[instrument(skip(self, input, _metrics), fields(pubkey = %input.0.pubkey, slot = input.0.slot))]
    async fn process(
        &mut self,
        input: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> Result<(), Error> {
        let (metadata, decoded_account, solana_account) = input;

        info!(
            pubkey = %metadata.pubkey,
            slot = metadata.slot,
            lamports = solana_account.lamports,
            owner = %solana_account.owner,
            "🔄 Processing Meteora DAMM V2 account update"
        );

        // Get global state (should always be initialized by main.rs)
        let state = PROCESSOR_STATE
            .get()
            .expect("Processor state not initialized");

        // Determine account type and serialize the actual data
        let (account_type, account_json) = match decoded_account.data {
            carbon_meteora_damm_v2_decoder::accounts::MeteoraDammV2Account::Pool(pool_data) => {
                info!(pubkey = %metadata.pubkey, "🏊 Processing POOL account");
                // With arbitrary_precision feature, u128 values are serialized as strings
                ("Pool", serde_json::to_value(&pool_data).unwrap_or(serde_json::Value::Null))
            }
            carbon_meteora_damm_v2_decoder::accounts::MeteoraDammV2Account::Position(
                position_data,
            ) => {
                info!(pubkey = %metadata.pubkey, "📍 Processing POSITION account");
                ("Position", serde_json::to_value(&position_data).unwrap_or(serde_json::Value::Null))
            }
            carbon_meteora_damm_v2_decoder::accounts::MeteoraDammV2Account::Config(config_data) => {
                info!(pubkey = %metadata.pubkey, "⚙️ Processing CONFIG account");
                ("Config", serde_json::to_value(&config_data).unwrap_or(serde_json::Value::Null))
            }
            carbon_meteora_damm_v2_decoder::accounts::MeteoraDammV2Account::ClaimFeeOperator(
                operator_data,
            ) => {
                info!(pubkey = %metadata.pubkey, "💰 Processing CLAIM FEE OPERATOR account");
                ("ClaimFeeOperator", serde_json::to_value(&operator_data).unwrap_or(serde_json::Value::Null))
            }
            carbon_meteora_damm_v2_decoder::accounts::MeteoraDammV2Account::TokenBadge(
                badge_data,
            ) => {
                info!(pubkey = %metadata.pubkey, "🏆 Processing TOKEN BADGE account");
                ("TokenBadge", serde_json::to_value(&badge_data).unwrap_or(serde_json::Value::Null))
            }
            carbon_meteora_damm_v2_decoder::accounts::MeteoraDammV2Account::Vesting(
                vesting_data,
            ) => {
                info!(pubkey = %metadata.pubkey, "🔒 Processing VESTING account");
                ("Vesting", serde_json::to_value(&vesting_data).unwrap_or(serde_json::Value::Null))
            }
        };

        info!(account_type = %account_type, account_json = %account_json, "💾 Inserting account update into database");

        // Create database record
        let new_account_update = NewAccountUpdate {
            pubkey: metadata.pubkey.to_string(),
            slot: metadata.slot,
            account_type: account_type.to_string(),
            owner: solana_account.owner.to_string(),
            lamports: solana_account.lamports,
            data_json: account_json,
        };

        // Store in database
        // debug!(
        //     pubkey = %metadata.pubkey,
        //     account_type,
        //     slot = metadata.slot,
        //     "💾 Inserting account update into database"
        // );

        match state
            .database
            .insert_account_update(new_account_update)
            .await
        {
            Ok(account_update) => {
                // Update cache

                if let Err(e) = state
                    .cache
                    .set_account(&metadata.pubkey.to_string(), &account_update)
                    .await
                {
                    warn!(
                        pubkey = %metadata.pubkey,
                        error = %e,
                        "⚠️ Failed to cache account in Redis"
                    );
                } else {
                    debug!(pubkey = %metadata.pubkey, "✅ Account cached successfully");
                }

                // Broadcast to WebSocket clients
                debug!(pubkey = %metadata.pubkey, "📡 Broadcasting account update to WebSocket clients");
                state
                    .websocket_server
                    .broadcast_account_update(&metadata.pubkey.to_string(), &account_update)
                    .await;
            }
            Err(e) => {
                error!(
                    pubkey = %metadata.pubkey,
                    account_type,
                    error = %e,
                    "❌ Failed to store account in database"
                );
            }
        }

        Ok(())
    }
}

// #[async_trait]
// impl Processor for DriftAccountProcessor {
//     type InputType = AccountProcessorInputType<DriftAccount>;

//     async fn process(
//         &mut self,
//         input: Self::InputType,
//         _metrics: Arc<MetricsCollection>,
//     ) -> Result<(), Error> {
//         println!("🔥 DRIFT PROCESSOR CALLED!");
//         let (_metadata, decoded_account, _solana_account) = input;

//         // println!("=== Account Update ===");
//         // println!("Account: {}", metadata.pubkey);
//         // println!("Slot: {}", metadata.slot);

//         // println!("Owner: {}", solana_account.owner);
//         // println!("Lamports: {}", solana_account.lamports);

//         // println!("Decoded Account Lamports: {}", decoded_account.lamports);
//         // println!("Decoded Account Owner: {}", decoded_account.owner);
//         // println!("Decoded Account Executable: {}", decoded_account.executable);
//         // println!("Decoded Account Rent Epoch: {}", decoded_account.rent_epoch);

//         match decoded_account.data {
//             carbon_drift_v2_decoder::accounts::DriftAccount::User(user) => {
//                 println!("🎯 USER ACCOUNT FOUND!");
//                 println!("User: {:?}", user);
//             }

//             _ => {
//                 println!("❓ Other account type (skipping)");
//                 return Ok(());
//             }
//         }

//         println!("===================");

//         Ok(())
//     }
// }

// #[async_trait]
// impl Processor for MeteoraDammV2InstructionProcessor {
//     type InputType = InstructionProcessorInputType<MeteoraDammV2Instruction>;

//     async fn process(
//         &mut self,
//         input: Self::InputType,
//         _metrics: Arc<MetricsCollection>,
//     ) -> Result<(), Error> {
//         println!("🎯 METEORA INSTRUCTION PROCESSOR CALLED!");
//         let (metadata, decoded_instruction, _nested_instructions, _raw_instruction) = input;

//         println!(
//             "Transaction Signature: {:?}",
//             metadata.transaction_metadata.signature
//         );
//         println!("Slot: {}", metadata.transaction_metadata.slot);
//         println!("Program ID: {}", decoded_instruction.program_id);

//         match decoded_instruction.data {
//             MeteoraDammV2Instruction::Swap(swap) => {
//                 println!("🔄 SWAP INSTRUCTION!");
//                 println!("Swap: {:?}", swap);
//             }
//             MeteoraDammV2Instruction::AddLiquidity(add_liquidity) => {
//                 println!("💧 ADD LIQUIDITY INSTRUCTION!");
//                 println!("Add Liquidity: {:?}", add_liquidity);
//             }
//             MeteoraDammV2Instruction::RemoveLiquidity(remove_liquidity) => {
//                 println!("🚰 REMOVE LIQUIDITY INSTRUCTION!");
//                 println!("Remove Liquidity: {:?}", remove_liquidity);
//             }
//             MeteoraDammV2Instruction::CreatePosition(create_position) => {
//                 println!("📍 CREATE POSITION INSTRUCTION!");
//                 println!("Create Position: {:?}", create_position);
//             }
//             MeteoraDammV2Instruction::ClosePosition(close_position) => {
//                 println!("❌ CLOSE POSITION INSTRUCTION!");
//                 println!("Close Position: {:?}", close_position);
//             }
//             MeteoraDammV2Instruction::InitializePool(initialize_pool) => {
//                 println!("🏊 INITIALIZE POOL INSTRUCTION!");
//                 println!("Initialize Pool: {:?}", initialize_pool);
//             }
//             MeteoraDammV2Instruction::ClaimReward(claim_reward) => {
//                 println!("💰 CLAIM REWARD INSTRUCTION!");
//                 println!("Claim Reward: {:?}", claim_reward);
//             }
//             MeteoraDammV2Instruction::LockPosition(lock_position) => {
//                 println!("🔒 LOCK POSITION INSTRUCTION!");
//                 println!("Lock Position: {:?}", lock_position);
//             }
//             _ => {
//                 println!("📋 Other instruction type");
//             }
//         }

//         println!("===================");

//         Ok(())
//     }
// }
