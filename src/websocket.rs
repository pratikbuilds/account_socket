use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use warp::{Filter, ws::{Message, WebSocket}};
use tracing::{info, warn, error, debug, instrument};

use crate::cache::RedisCache;
use crate::database::{AccountUpdate, Database};

pub type ClientId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionRequest {
    pub action: String, // "subscribe" or "unsubscribe"
    pub pubkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUpdateMessage {
    pub pubkey: String,
    pub account: AccountUpdate,
    pub source: String, // "cache" or "database"
}

#[derive(Debug, Clone)]
pub struct WebSocketServer {
    clients: Arc<RwLock<HashMap<ClientId, broadcast::Sender<AccountUpdateMessage>>>>,
    subscriptions: Arc<RwLock<HashMap<String, Vec<ClientId>>>>,
    database: Arc<Database>,
    cache: Arc<RedisCache>,
    next_client_id: Arc<RwLock<u64>>,
}

impl WebSocketServer {
    pub fn new(database: Arc<Database>, cache: Arc<RedisCache>) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            database,
            cache,
            next_client_id: Arc::new(RwLock::new(1)),
        }
    }

    // Create Warp WebSocket filter
    pub fn create_websocket_filter(
        self: Arc<Self>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("ws")
            .and(warp::ws())
            .and(warp::any().map(move || self.clone()))
            .and_then(|ws: warp::ws::Ws, server: Arc<WebSocketServer>| async move {
                Ok::<_, warp::Rejection>(ws.on_upgrade(move |socket| {
                    server.handle_websocket_connection(socket)
                }))
            })
    }

    // Handle new WebSocket connection via Warp
    #[instrument(skip(self, ws))]
    pub async fn handle_websocket_connection(self: Arc<Self>, ws: WebSocket) {
        info!("🔌 New WebSocket client attempting to connect");

        // Generate unique client ID
        let client_id = {
            let mut next_id = self.next_client_id.write().await;
            let id = *next_id;
            *next_id += 1;
            id
        };

        info!(client_id, "✅ WebSocket client connected successfully");

        // Split WebSocket into sender/receiver
        let (mut ws_sender, mut ws_receiver) = ws.split();

        // Create broadcast channel for this client
        let (broadcast_tx, mut broadcast_rx) = broadcast::channel(100);

        // Register client in our clients HashMap
        {
            let mut clients = self.clients.write().await;
            clients.insert(client_id, broadcast_tx);
        }

        // Clone server for tasks
        let server_for_incoming = self.clone();

        // Task to handle incoming messages from client (subscription requests)
        let incoming_task = tokio::spawn(async move {
            debug!(client_id, "🔄 Starting incoming message handler for client");

            while let Some(result) = ws_receiver.next().await {
                match result {
                    Ok(msg) => {
                        if let Ok(text) = msg.to_str() {
                            debug!(client_id, message = %text, "📨 Received message from client");

                            // Parse subscription request
                            match serde_json::from_str::<SubscriptionRequest>(text) {
                                Ok(request) => {
                                    info!(
                                        client_id,
                                        action = %request.action,
                                        pubkey = %request.pubkey,
                                        "📝 Processing subscription request"
                                    );

                                    server_for_incoming
                                        .handle_subscription(client_id, request)
                                        .await;
                                }
                                Err(e) => {
                                    warn!(
                                        client_id,
                                        error = %e,
                                        raw_message = %text,
                                        "❌ Failed to parse subscription request"
                                    );
                                }
                            }
                        } else if msg.is_close() {
                            info!(client_id, "👋 Client sent close message");
                            break;
                        }
                    }
                    Err(e) => {
                        error!(client_id, error = %e, "❌ WebSocket error occurred");
                        break;
                    }
                }
            }
            debug!(client_id, "📤 Incoming message handler finished");
        });

        // Task to handle outgoing messages to client (account updates from broadcast)
        let outgoing_task = tokio::spawn(async move {
            debug!(client_id, "🔄 Starting outgoing message handler for client");

            while let Ok(update) = broadcast_rx.recv().await {
                debug!(
                    client_id,
                    pubkey = %update.pubkey,
                    account_type = %update.account.account_type,
                    source = %update.source,
                    "📡 Broadcasting account update to client"
                );

                match serde_json::to_string(&update) {
                    Ok(json) => {
                        if let Err(e) = ws_sender.send(Message::text(json)).await {
                            warn!(client_id, error = %e, "❌ Failed to send update to client");
                            break;
                        } else {
                            debug!(client_id, pubkey = %update.pubkey, "✅ Account update sent successfully");
                        }
                    }
                    Err(e) => {
                        error!(client_id, error = %e, pubkey = %update.pubkey, "❌ Failed to serialize update");
                    }
                }
            }
            debug!(client_id, "📤 Outgoing message handler finished");
        });

        // Wait for either task to complete (client disconnect or error)
        tokio::select! {
            _ = incoming_task => {
                info!(client_id, "🔄 Incoming task completed, client likely disconnected");
            },
            _ = outgoing_task => {
                info!(client_id, "🔄 Outgoing task completed, client likely disconnected");
            }
        }

        // Clean up client
        info!(client_id, "🧹 Starting client cleanup");
        self.cleanup_client(client_id).await;
    }

    #[instrument(skip(self), fields(client_id, action = %request.action, pubkey = %request.pubkey))]
    async fn handle_subscription(&self, client_id: ClientId, request: SubscriptionRequest) {
        match request.action.as_str() {
            "subscribe" => {
                info!(
                    client_id,
                    pubkey = %request.pubkey,
                    "📝 Client subscribing to account updates"
                );

                // Add client to subscription list for this pubkey
                {
                    let mut subs = self.subscriptions.write().await;
                    subs.entry(request.pubkey.clone())
                        .or_insert_with(Vec::new)
                        .push(client_id);
                }

                // Send current account state immediately
                debug!(client_id, pubkey = %request.pubkey, "🔍 Fetching current account state for new subscription");
                if let Some((account, source)) = self.get_account_data(&request.pubkey).await {
                    let message = AccountUpdateMessage {
                        pubkey: request.pubkey.clone(),
                        account,
                        source,
                    };

                    info!(
                        client_id,
                        pubkey = %request.pubkey,
                        source = %message.source,
                        account_type = %message.account.account_type,
                        "📤 Sending current account state to new subscriber"
                    );

                    // Send to this specific client
                    let clients = self.clients.read().await;
                    if let Some(tx) = clients.get(&client_id) {
                        if let Err(_) = tx.send(message) {
                            warn!(client_id, "⚠️ Failed to send initial account state - client may have disconnected");
                        }
                    }
                } else {
                    debug!(client_id, pubkey = %request.pubkey, "🔍 No current account state found");
                }
            }
            "unsubscribe" => {
                info!(
                    client_id,
                    pubkey = %request.pubkey,
                    "📝 Client unsubscribing from account updates"
                );

                // Remove client from subscription list
                let mut subs = self.subscriptions.write().await;
                if let Some(client_list) = subs.get_mut(&request.pubkey) {
                    client_list.retain(|&id| id != client_id);

                    // Clean up empty subscription lists
                    if client_list.is_empty() {
                        subs.remove(&request.pubkey);
                    }
                }
            }
            _ => {
                warn!(
                    client_id,
                    action = %request.action,
                    "❓ Unknown subscription action received"
                );
            }
        }
    }

    #[instrument(skip(self, account), fields(pubkey = %pubkey, account_type = %account.account_type))]
    pub async fn broadcast_account_update(&self, pubkey: &str, account: &AccountUpdate) {
        let subs = self.subscriptions.read().await;

        if let Some(client_ids) = subs.get(pubkey) {
            info!(
                pubkey = %pubkey,
                client_count = client_ids.len(),
                account_type = %account.account_type,
                "📡 Broadcasting account update to subscribed clients"
            );

            let message = AccountUpdateMessage {
                pubkey: pubkey.to_string(),
                account: account.clone(),
                source: "realtime".to_string(),
            };

            let clients = self.clients.read().await;

            for &client_id in client_ids {
                if let Some(tx) = clients.get(&client_id) {
                    if tx.send(message.clone()).is_err() {
                        // Client's receiver is dropped (client disconnected)
                        warn!(client_id, "⚠️ Client appears to be disconnected during broadcast");
                    } else {
                        debug!(client_id, pubkey = %pubkey, "✅ Account update sent to client");
                    }
                } else {
                    warn!(client_id, "⚠️ Client not found in clients map during broadcast");
                }
            }
        }
    }

    #[instrument(skip(self), fields(pubkey = %pubkey))]
    async fn get_account_data(&self, pubkey: &str) -> Option<(AccountUpdate, String)> {
        debug!(pubkey = %pubkey, "🔍 Retrieving account data using cache-aside pattern");

        // Cache-aside pattern: Try cache first, then database
        if let Ok(Some(account)) = self.cache.get_account(pubkey).await {
            debug!(pubkey = %pubkey, account_type = %account.account_type, "✅ Account retrieved from cache");
            return Some((account, "cache".to_string()));
        }

        debug!(pubkey = %pubkey, "🔍 Account not in cache, checking database");
        if let Ok(Some(account)) = self.database.get_latest_account_state(pubkey).await {
            info!(pubkey = %pubkey, account_type = %account.account_type, "✅ Account retrieved from database, caching for future requests");
            // Cache the result for next time
            if let Err(e) = self.cache.set_account(pubkey, &account).await {
                warn!(pubkey = %pubkey, error = %e, "⚠️ Failed to cache account after database retrieval");
            }
            return Some((account, "database".to_string()));
        }

        debug!(pubkey = %pubkey, "🔍 Account not found in cache or database");
        None
    }

    #[instrument(skip(self), fields(client_id))]
    async fn cleanup_client(&self, client_id: ClientId) {
        info!(client_id, "🧹 Starting client cleanup process");

        // Remove client from clients map
        {
            let mut clients = self.clients.write().await;
            if clients.remove(&client_id).is_some() {
                debug!(client_id, "✅ Client removed from clients map");
            } else {
                warn!(client_id, "⚠️ Client not found in clients map during cleanup");
            }
        }

        // Remove client from all subscriptions
        {
            let mut subs = self.subscriptions.write().await;
            let mut removed_from_subscriptions = 0;
            for (_pubkey, client_list) in subs.iter_mut() {
                let original_len = client_list.len();
                client_list.retain(|&id| id != client_id);
                if client_list.len() < original_len {
                    removed_from_subscriptions += 1;
                }
            }
            // Remove empty subscription lists
            let original_subs_count = subs.len();
            subs.retain(|_, client_list| !client_list.is_empty());
            let cleaned_empty_subs = original_subs_count - subs.len();

            if removed_from_subscriptions > 0 {
                debug!(
                    client_id,
                    subscription_count = removed_from_subscriptions,
                    cleaned_empty_subs,
                    "✅ Client removed from subscriptions"
                );
            }
        }

        info!(client_id, "✅ Client cleanup completed successfully");
    }
}
