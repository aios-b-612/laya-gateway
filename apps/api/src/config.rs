use std::env;
use std::time::Duration;

use anyhow::{bail, Context, Result};

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_address: String,
    pub routing_enabled: bool,
    pub min_confidence: f64,
    pub laya_url: String,
    pub laya_api_key: Option<String>,
    pub laya_model: Option<String>,
    pub laya_timeout: Duration,
    pub llm_upstream_url: String,
    pub llm_api_key: Option<String>,
    pub upstream_timeout: Duration,
    pub max_events: usize,
    pub service_name: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv();

        let bind_address =
            env::var("LAYA_GATEWAY_BIND").unwrap_or_else(|_| "127.0.0.1:8790".into());
        if !bind_address.starts_with("127.0.0.1") && !bind_address.starts_with("localhost") {
            bail!("LAYA_GATEWAY_BIND must be loopback (127.0.0.1 / localhost) — this gateway stays on the developer machine");
        }

        let routing_raw = env::var("LAYA_GATEWAY_ROUTING").unwrap_or_else(|_| "on".into());
        let routing_enabled = matches!(
            routing_raw.to_ascii_lowercase().as_str(),
            "1" | "true" | "on" | "yes"
        );

        let min_confidence: f64 = env::var("LAYA_GATEWAY_MIN_CONFIDENCE")
            .unwrap_or_else(|_| "0.55".into())
            .parse()
            .context("LAYA_GATEWAY_MIN_CONFIDENCE")?;

        let laya_url = env::var("LAYA_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8000/v1/systemone".into());

        Ok(Self {
            bind_address,
            routing_enabled,
            min_confidence,
            laya_url,
            laya_api_key: env::var("LAYA_API_KEY").ok().filter(|s| !s.trim().is_empty()),
            laya_model: env::var("LAYA_MODEL").ok().filter(|s| !s.trim().is_empty()),
            laya_timeout: Duration::from_millis(
                env::var("LAYA_GATEWAY_LAYA_TIMEOUT_MS")
                    .unwrap_or_else(|_| "800".into())
                    .parse()
                    .context("LAYA_GATEWAY_LAYA_TIMEOUT_MS")?,
            ),
            llm_upstream_url: env::var("LLM_UPSTREAM_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".into())
                .trim_end_matches('/')
                .to_string(),
            llm_api_key: env::var("LLM_API_KEY").ok().filter(|s| !s.trim().is_empty()),
            upstream_timeout: Duration::from_millis(
                env::var("LAYA_GATEWAY_UPSTREAM_TIMEOUT_MS")
                    .unwrap_or_else(|_| "120000".into())
                    .parse()
                    .context("LAYA_GATEWAY_UPSTREAM_TIMEOUT_MS")?,
            ),
            max_events: env::var("LAYA_GATEWAY_MAX_EVENTS")
                .unwrap_or_else(|_| "200".into())
                .parse()
                .context("LAYA_GATEWAY_MAX_EVENTS")?,
            service_name: "laya-gateway".into(),
        })
    }
}
