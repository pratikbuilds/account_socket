use anyhow::Result;
use carbon_core::pipeline::Pipeline;
use carbon_drift_v2_decoder::DriftDecoder;
use carbon_drift_v2_decoder::PROGRAM_ID;
use carbon_rpc_program_subscribe_datasource::{Filters, RpcProgramSubscribe};
use dotenv::dotenv;

mod config;
mod processor;
use config::ServiceConfig;
use processor::DriftAccountProcessor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let config = ServiceConfig::from_env()?;

    print!("{}", config.rpc_url);

    let mut pipeline = Pipeline::builder()
        .datasource(RpcProgramSubscribe::new(
            config.rpc_url,
            Filters::new(PROGRAM_ID, None),
        ))
        .account(DriftDecoder, DriftAccountProcessor)
        .build()?;

    println!("Starting pipeline...");
    pipeline.run().await?;

    Ok(())
}
