use anyhow::Result;
use carbon_core::pipeline::{Pipeline, ShutdownStrategy};

use carbon_meteora_damm_v2_decoder::{MeteoraDammV2Decoder, PROGRAM_ID};
use carbon_rpc_program_subscribe_datasource::{Filters, RpcProgramSubscribe};
use dotenv::dotenv;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use tracing::{info, warn, error, debug, Level};
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod cache;
mod config;
mod database;
mod processor;
mod websocket;

use crate::cache::RedisCache;
use crate::database::Database;
use crate::processor::{MeteoraDammV2AccountProcessor, PROCESSOR_STATE, ProcessorState};
use crate::websocket::WebSocketServer;
use carbon_log_metrics::LogMetrics;
use config::ServiceConfig;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Initialize structured logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,account_socket=debug"));

    tracing_subscriber::registry()
        .with(fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
        )
        .with(filter)
        .init();

    info!("🚀 Starting account socket server");

    let config = ServiceConfig::from_env()?;
    info!("📊 Configuration loaded");
    debug!("RPC URL: {}", config.rpc_url);
    debug!("WebSocket: {}:{}", config.websocket.host, config.websocket.port);
    debug!("Redis: {}", config.redis.url);
    debug!("Database: {}", config.database.url);

    // Initialize database
    info!("📦 Connecting to database");
    let database = Arc::new(Database::new(&config.database.url).await?);
    info!("✅ Database connection established");

    // Initialize Redis cache
    info!("🔴 Connecting to Redis");
    let cache = Arc::new(RedisCache::new(&config.redis.url).await?);
    info!("✅ Redis connection established");

    // Initialize WebSocket server
    info!("🌐 Setting up WebSocket server");
    let websocket_server = Arc::new(WebSocketServer::new(database.clone(), cache.clone()));
    info!("✅ WebSocket server initialized");

    // Initialize global processor state
    let processor_state = ProcessorState {
        database: database.clone(),
        cache: cache.clone(),
        websocket_server: websocket_server.clone(),
    };

    PROCESSOR_STATE.set(processor_state).expect("Failed to set processor state");
    info!("✅ Processor state initialized");

    // Create Warp WebSocket server using websocket module
    let ws_route = websocket_server.clone().create_websocket_filter();

    let server_addr = ([127, 0, 0, 1], config.websocket.port);
    info!("🌐 Starting Warp WebSocket server on http://{}:{}/ws", config.websocket.host, config.websocket.port);

    // Start the Warp server in background
    tokio::spawn(async move {
        info!("🚀 WebSocket server listening on {}", server_addr.1);
        warp::serve(ws_route)
            .run(server_addr)
            .await;
    });

    info!("⚙️  Building Carbon pipeline");
    let mut pipeline = Pipeline::builder()
        .datasource(RpcProgramSubscribe::new(
            config.rpc_url.clone(),
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

    info!("🔥 Starting Carbon pipeline for Meteora DAMM V2 accounts");
    info!("🎯 Target program: {}", PROGRAM_ID);
    pipeline.run().await?;

    Ok(())
}
