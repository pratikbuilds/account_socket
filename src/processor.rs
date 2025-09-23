use async_trait::async_trait;
use std::sync::Arc;

use carbon_core::{
    account::AccountProcessorInputType, error::Error, metrics::MetricsCollection,
    processor::Processor,
};
use carbon_drift_v2_decoder::accounts::DriftAccount;
pub struct DriftAccountProcessor;

#[async_trait]
impl Processor for DriftAccountProcessor {
    type InputType = AccountProcessorInputType<DriftAccount>;

    async fn process(
        &mut self,
        input: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> Result<(), Error> {
        let (metadata, decoded_account, solana_account) = input;

        // println!("=== Account Update ===");
        // println!("Account: {}", metadata.pubkey);
        // println!("Slot: {}", metadata.slot);

        // println!("Owner: {}", solana_account.owner);
        // println!("Lamports: {}", solana_account.lamports);

        // println!("Decoded Account Lamports: {}", decoded_account.lamports);
        // println!("Decoded Account Owner: {}", decoded_account.owner);
        // println!("Decoded Account Executable: {}", decoded_account.executable);
        // println!("Decoded Account Rent Epoch: {}", decoded_account.rent_epoch);

        match decoded_account.data {
            carbon_drift_v2_decoder::accounts::DriftAccount::User(user) => {
                println!("🎯 USER ACCOUNT FOUND!");
                println!("User: {:?}", user);
            }

            _ => {
                println!("❓ Other account type (skipping)");
                return Ok(());
            }
        }

        println!("===================");

        Ok(())
    }
}
