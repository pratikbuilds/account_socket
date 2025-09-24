use anyhow::Result;
use carbon_core::pipeline::{Pipeline, ShutdownStrategy};

use carbon_meteora_damm_v2_decoder::{MeteoraDammV2Decoder, PROGRAM_ID};
use carbon_rpc_program_subscribe_datasource::{Filters, RpcProgramSubscribe};
use dotenv::dotenv;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};

mod config;
mod processor;
use crate::processor::{
    DriftAccountProcessor, MeteoraDammV2AccountProcessor, MeteoraDammV2InstructionProcessor,
};
use carbon_log_metrics::LogMetrics;
use config::ServiceConfig;
use std::{env, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for Carbon logging

    dotenv().ok();
    env_logger::init();
    let config = ServiceConfig::from_env()?;

    println!("{}", config.rpc_url);

    let mut pipeline = Pipeline::builder()
        .datasource(RpcProgramSubscribe::new(
            config.rpc_url,
            Filters::new(
                PROGRAM_ID,
                Some(RpcProgramAccountsConfig {
                    filters: None,
                    account_config: RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            ),
        ))
        .account(MeteoraDammV2Decoder, MeteoraDammV2AccountProcessor)
        .metrics(Arc::new(LogMetrics::new()))
        .shutdown_strategy(ShutdownStrategy::ProcessPending)
        .build()?;

    println!("Starting pipeline...");
    pipeline.run().await?;

    Ok(())
}
