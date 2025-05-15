// References:
// https://github.com/jito-labs/jito-rust-rpc/tree/master
// https://docs.jito.wtf/lowlatencytxnsend/#api
// https://docs.jito.wtf/lowlatencytxnsend/#getting-started

use reqwest::Client;
use serde_json::{json, Value};
use std::fmt;
use anyhow::{anyhow, Result};
use rand::seq::SliceRandom;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use tracing::info;

// For help in naming spans
use crate::constants::QTRADE_LANDER_TRACER_NAME;
const JITO_JSON_RPC_SDK: &str = "rpc::jito::JitoJsonRpcSDK";

pub struct JitoJsonRpcSDK {
    base_url: String,
    uuid: Option<String>,
    client: Client,
}

#[derive(Debug)]
pub struct PrettyJsonValue(pub Value);

impl fmt::Display for PrettyJsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self.0).unwrap())
    }
}

impl From<Value> for PrettyJsonValue {
    fn from(value: Value) -> Self {
        PrettyJsonValue(value)
    }
}

impl JitoJsonRpcSDK {
    pub fn new(base_url: &str, uuid: Option<String>) -> Self {
        Self {
            base_url: base_url.to_string(),
            uuid,
            client: Client::new(),
        }
    }

    async fn send_request(&self, endpoint: &str, method: &str, params: Option<Value>) -> Result<Value, reqwest::Error> {
        let tracer = global::tracer(QTRADE_LANDER_TRACER_NAME);
        let span_name = format!("{}::send_request", JITO_JSON_RPC_SDK);

        let result = tracer.in_span(span_name, |_cx| async move {
            // Create the JSON-RPC request
            let id = format!("{}", rand::random::<u64>());

            let request = json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": method,
                "params": params.unwrap_or_else(|| json!({}))
            });

            // Log the request
            info!("Sending request to {}/{}: {}", self.base_url, endpoint, PrettyJsonValue(request.clone()));

            // Send the request
            let response = self.client
                .post(&format!("{}/{}", self.base_url, endpoint))
                .json(&request)
                .send()
                .await?;

            // Parse the response
            let res = response.json::<Value>().await?;

            // Log the response
            info!("Received response: {}", PrettyJsonValue(res.clone()));

            Ok(res)
        }).await;

        result
    }

    pub async fn get_tip_accounts(&self) -> Result<Value, reqwest::Error> {
        let tracer = global::tracer(QTRADE_LANDER_TRACER_NAME);
        let span_name = format!("{}::get_tip_acccounts", JITO_JSON_RPC_SDK);

        let result = tracer.in_span(span_name, |_cx| async move {
            let endpoint = "jito-tips".to_string();
            self.send_request(&endpoint, "getTipAccounts", None).await
        }).await;

        result
    }

    // Get a random tip account
    pub async fn get_random_tip_account(&self) -> Result<String> {
        let tracer = global::tracer(QTRADE_LANDER_TRACER_NAME);
        let span_name = format!("{}::get_random_tip_account", JITO_JSON_RPC_SDK);

        let result = tracer.in_span(span_name, |_cx| async move {
            let tip_accounts_response = self.get_tip_accounts().await
                .map_err(|e| anyhow!("Failed to get tip accounts: {}", e))?;

            if let Some(result) = tip_accounts_response.get("result") {
                if let Some(accounts) = result.as_array() {
                    if accounts.is_empty() {
                        return Err(anyhow!("No tip accounts found"));
                    }

                    // Choose a random account from the list
                    if let Some(account) = accounts.choose(&mut rand::thread_rng()) {
                        if let Some(pubkey) = account.as_str() {
                            return Ok(pubkey.to_string());
                        }
                    }
                }
            }

            Err(anyhow!("Failed to parse tip accounts response"))
        }).await;

        result
    }

    pub async fn get_bundle_statuses(&self, bundle_uuids: Vec<String>) -> Result<Value> {
        let tracer = global::tracer(QTRADE_LANDER_TRACER_NAME);
        let span_name = format!("{}::get_bundle_statuses", JITO_JSON_RPC_SDK);

        let result = tracer.in_span(span_name, |_cx| async move {
            let endpoint = "bundles".to_string();
            let params = json!({
                "uuids": bundle_uuids
            });

            self.send_request(&endpoint, "getBundleStatuses", Some(params))
                .await
                .map_err(|e| anyhow!("Failed to get bundle statuses: {}", e))
        }).await;

        result
    }

    pub async fn send_bundle(&self, params: Option<Value>, uuid: Option<&str>) -> Result<Value, anyhow::Error> {
        let tracer = global::tracer(QTRADE_LANDER_TRACER_NAME);
        let span_name = format!("{}::send_bundle", JITO_JSON_RPC_SDK);

        let result = tracer.in_span(span_name, |_cx| async move {
            let endpoint = "bundles".to_string();
            let mut bundle_params = params.unwrap_or_else(|| json!({}));

            // Add the UUID if provided
            if let Some(uuid_str) = uuid.or(self.uuid.as_deref()) {
                bundle_params["uuid"] = json!(uuid_str);
            }

            self.send_request(&endpoint, "sendBundle", Some(bundle_params))
                .await
                .map_err(|e| anyhow!("Failed to send bundle: {}", e))
        }).await;

        result
    }

    pub async fn send_txn(&self, params: Option<Value>, bundle_only: bool) -> Result<Value, reqwest::Error> {
        let tracer = global::tracer(QTRADE_LANDER_TRACER_NAME);
        let span_name = format!("{}::send_txn", JITO_JSON_RPC_SDK);

        let result = tracer.in_span(span_name, |_cx| async move {
            let endpoint = "tx".to_string();
            let mut tx_params = params.unwrap_or_else(|| json!({}));

            // Set the bundle_only flag
            tx_params["bundle_only"] = json!(bundle_only);

            self.send_request(&endpoint, "sendTransaction", Some(tx_params)).await
        }).await;

        result
    }

    pub async fn get_in_flight_bundle_statuses(&self, bundle_uuids: Vec<String>) -> Result<Value> {
        let tracer = global::tracer(QTRADE_LANDER_TRACER_NAME);
        let span_name = format!("{}::get_in_flight_bundle_statuses", JITO_JSON_RPC_SDK);

        let result = tracer.in_span(span_name, |_cx| async move {
            let endpoint = "bundles".to_string();
            let params = json!({
                "uuids": bundle_uuids
            });

            self.send_request(&endpoint, "getInFlightBundleStatuses", Some(params))
                .await
                .map_err(|e| anyhow!("Failed to get in-flight bundle statuses: {}", e))
        }).await;

        result
    }

    // Helper method to convert Value to PrettyJsonValue
    pub fn prettify(value: Value) -> PrettyJsonValue {
        PrettyJsonValue(value)
    }
}
