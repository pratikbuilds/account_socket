use async_trait::async_trait;
use carbon_meteora_damm_v2_decoder::accounts::MeteoraDammV2Account;
use carbon_meteora_damm_v2_decoder::instructions::MeteoraDammV2Instruction;
use std::sync::Arc;

use carbon_core::{
    account::AccountProcessorInputType, error::Error, instruction::InstructionProcessorInputType,
    metrics::MetricsCollection, processor::Processor,
};
use carbon_drift_v2_decoder::accounts::DriftAccount;

pub struct MeteoraDammV2AccountProcessor;

#[async_trait]
impl Processor for MeteoraDammV2AccountProcessor {
    type InputType = AccountProcessorInputType<MeteoraDammV2Account>;

    async fn process(
        &mut self,
        input: Self::InputType,
        metrics: Arc<MetricsCollection>,
    ) -> Result<(), Error> {
        println!("IN the process");
        let (metadata, decoded_account, solana_account) = input;

        println!("=== Account Update ===");
        println!("Account: {}", metadata.pubkey);
        println!("Slot: {}", metadata.slot);

        println!("Owner: {}", solana_account.owner);
        println!("Lamports: {}", solana_account.lamports);

        println!("Decoded Account Lamports: {}", decoded_account.lamports);
        println!("Decoded Account Owner: {}", decoded_account.owner);

        match decoded_account.data {
            carbon_meteora_damm_v2_decoder::accounts::MeteoraDammV2Account::Pool(pool) => {
                // println!("🏊 POOL ACCOUNT FOUND!");
                // println!("Pool: {:?}", pool);
            }
            carbon_meteora_damm_v2_decoder::accounts::MeteoraDammV2Account::Position(position) => {
                println!("📍 POSITION ACCOUNT FOUND!");
                println!("Position: {:?}", position);
            }
            carbon_meteora_damm_v2_decoder::accounts::MeteoraDammV2Account::Config(config) => {
                println!("⚙️ CONFIG ACCOUNT FOUND!");
                // println!("Config: {:?}", config);
            }
            carbon_meteora_damm_v2_decoder::accounts::MeteoraDammV2Account::ClaimFeeOperator(
                operator,
            ) => {
                println!("💰 CLAIM FEE OPERATOR FOUND!");
                // println!("Claim Fee Operator: {:?}", operator);
            }
            carbon_meteora_damm_v2_decoder::accounts::MeteoraDammV2Account::TokenBadge(badge) => {
                println!("🏆 TOKEN BADGE FOUND!");
                // println!("Token Badge: {:?}", badge);
            }
            carbon_meteora_damm_v2_decoder::accounts::MeteoraDammV2Account::Vesting(vesting) => {
                println!("🔒 VESTING ACCOUNT FOUND!");
                // println!("Vesting: {:?}", vesting);
            }
        }

        println!("===================");

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
