use std::collections::BTreeMap;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::Config;

#[derive(Clone)]
pub struct LayaClient {
    http: Client,
    url: String,
    api_key: Option<String>,
    model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SystemOneRequest {
    pub state: Value,
    pub questions: BTreeMap<String, Question>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Question {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub instructions: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criteria: Option<BTreeMap<String, Option<String>>>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SystemOneResponse {
    pub answers: BTreeMap<String, Answer>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Answer {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub choice: Option<String>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub probabilities: Option<BTreeMap<String, f64>>,
    #[serde(default)]
    pub noul: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Usage {
    #[serde(default)]
    pub input_tokens: u64,
}

#[derive(Debug, Clone)]
pub struct LayaCall {
    pub response: SystemOneResponse,
    pub latency_ms: u64,
}

impl LayaClient {
    pub fn new(config: &Config) -> Result<Self> {
        let http = Client::builder()
            .timeout(config.laya_timeout)
            .build()
            .context("cliente HTTP Laya")?;
        Ok(Self {
            http,
            url: config.laya_url.clone(),
            api_key: config.laya_api_key.clone(),
            model: config.laya_model.clone(),
        })
    }

    pub async fn decide(&self, mut req: SystemOneRequest) -> Result<LayaCall> {
        if req.model.is_none() {
            req.model = self.model.clone();
        }
        let started = Instant::now();
        let mut builder = self
            .http
            .post(&self.url)
            .header("content-type", "application/json")
            .header("user-agent", "laya-gateway/0.1");
        if let Some(key) = &self.api_key {
            builder = builder.bearer_auth(key);
        }
        let response = builder
            .json(&req)
            .send()
            .await
            .context("POST Laya /v1/systemone")?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!("Laya HTTP {status}: {}", body.chars().take(200).collect::<String>()));
        }
        let parsed: SystemOneResponse =
            serde_json::from_str(&body).context("parse resposta Laya")?;
        Ok(LayaCall {
            response: parsed,
            latency_ms: started.elapsed().as_millis() as u64,
        })
    }
}
