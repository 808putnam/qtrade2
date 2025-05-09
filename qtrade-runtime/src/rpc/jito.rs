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
use crate::QTRADE_RUNTIME_TRACER_NAME;
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
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::send_request", JITO_JSON_RPC_SDK);

        let result = tracer.in_span(span_name, |_cx| async move {
            let url = format!("{}{}", self.base_url, endpoint);
        
            let data = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
                "params": params.unwrap_or(json!([]))
            });
    
            info!("Sending request to: {}", url);
            info!("Request body: {}", serde_json::to_string_pretty(&data).unwrap());
    
            let response = self.client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&data)
                .send()
                .await?;
    
            let status = response.status();
            info!("Response status: {}", status);
    
            let body = response.json::<Value>().await?;
            info!("Response body: {}", serde_json::to_string_pretty(&body).unwrap());
    
            Ok(body)
    
        }).await;

        result
    }

    pub async fn get_tip_accounts(&self) -> Result<Value, reqwest::Error> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::get_tip_acccounts", JITO_JSON_RPC_SDK);

        let result = tracer.in_span(span_name, |_cx| async move {
            let endpoint = if let Some(uuid) = &self.uuid {
                format!("/bundles?uuid={}", uuid)
            } else {
                "/bundles".to_string()
            };

            self.send_request(&endpoint, "getTipAccounts", None).await
        }).await;

        result
    }

    // Get a random tip account
    pub async fn get_random_tip_account(&self) -> Result<String> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::get_random_tip_account", JITO_JSON_RPC_SDK);
        
        let result = tracer.in_span(span_name, |_cx| async move {
            let tip_accounts_response = self.get_tip_accounts().await?;
            
            let tip_accounts = tip_accounts_response["result"]
                .as_array()
                .ok_or_else(|| anyhow!("Failed to parse tip accounts as array"))?;

            if tip_accounts.is_empty() {
                return Err(anyhow!("No tip accounts available"));
            }

            let random_account = tip_accounts
                .choose(&mut rand::thread_rng())
                .ok_or_else(|| anyhow!("Failed to choose random tip account"))?;

            random_account
                .as_str()
                .ok_or_else(|| anyhow!("Failed to parse tip account as string"))
                .map(String::from)
        }).await;

        result
    }

    pub async fn get_bundle_statuses(&self, bundle_uuids: Vec<String>) -> Result<Value> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::get_bundle_statuses", JITO_JSON_RPC_SDK);

        let result = tracer.in_span(span_name, |_cx| async move {
            let endpoint = if let Some(uuid) = &self.uuid {
                format!("/bundles?uuid={}", uuid)
            } else {
                "/bundles".to_string()
            };

            // Construct the params as a list within a list
            let params = json!([bundle_uuids]);

            self.send_request(&endpoint, "getBundleStatuses", Some(params))
                .await
                .map_err(|e| anyhow!("Request error: {}", e))
        }).await;

        result
    }

    pub async fn send_bundle(&self, params: Option<Value>, uuid: Option<&str>) -> Result<Value, anyhow::Error> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::send_bundle", JITO_JSON_RPC_SDK);

        let result = tracer.in_span(span_name, |_cx| async move {
            let mut endpoint = "/bundles".to_string();
            
            if let Some(uuid) = uuid {
                endpoint = format!("{}?uuid={}", endpoint, uuid);
            }
        
            // Ensure params is an array of transactions
            let transactions = match params {
                Some(Value::Array(transactions)) => {
                    if transactions.is_empty() {
                        return Err(anyhow!("Bundle must contain at least one transaction"));
                    }
                    if transactions.len() > 5 {
                        return Err(anyhow!("Bundle can contain at most 5 transactions"));
                    }
                    transactions
                },
                _ => return Err(anyhow!("Invalid bundle format: expected an array of transactions")),
            };
        
            // Wrap the transactions array in another array
            let params = json!([transactions]);
        
            // Send the wrapped transactions array
            self.send_request(&endpoint, "sendBundle", Some(params))
                .await
                .map_err(|e| anyhow!("Request error: {}", e))
        }).await;

        result
    }

    pub async fn send_txn(&self, params: Option<Value>, bundle_only: bool) -> Result<Value, reqwest::Error> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::send_txn", JITO_JSON_RPC_SDK);

        let result = tracer.in_span(span_name, |_cx| async move {
            let mut query_params = Vec::new();

            if bundle_only {
                query_params.push("bundleOnly=true".to_string());
            }

            let endpoint = if query_params.is_empty() {
                "/transactions".to_string()
            } else {
                format!("/transactions?{}", query_params.join("&"))
            };

            // Construct params as an array instead of an object
            let params = match params {
                Some(Value::Object(map)) => {
                    let tx = map.get("tx").and_then(Value::as_str).unwrap_or_default();
                    let skip_preflight = map.get("skipPreflight").and_then(Value::as_bool).unwrap_or(false);
                    json!([
                        tx,
                        {
                            "encoding": "base64",
                            "skipPreflight": skip_preflight
                        }
                    ])
                },
                _ => json!([]),
            };

            self.send_request(&endpoint, "sendTransaction", Some(params)).await
        }).await;

        result
    }

    pub async fn get_in_flight_bundle_statuses(&self, bundle_uuids: Vec<String>) -> Result<Value> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::get_in_flight_bundle_statuses", JITO_JSON_RPC_SDK);

        let result = tracer.in_span(span_name,  |_cx| async move {
            let endpoint = if let Some(uuid) = &self.uuid {
                format!("/bundles?uuid={}", uuid)
            } else {
                "/bundles".to_string()
            };

            // Construct the params as a list within a list
            let params = json!([bundle_uuids]);

            self.send_request(&endpoint, "getInflightBundleStatuses", Some(params))
                .await
                .map_err(|e| anyhow!("Request error: {}", e))
        }).await;

        result
    }

    // Helper method to convert Value to PrettyJsonValue
    pub fn prettify(value: Value) -> PrettyJsonValue {
        PrettyJsonValue(value)
    }
    
}