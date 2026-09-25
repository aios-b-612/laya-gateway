use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// Default for Octor DEV VPN; override anytime in the dashboard or `.env`.
pub const DEFAULT_LAYA_URL: &str = "http://10.8.0.9:8343/v1/systemone";

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
    pub settings_path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistedSettings {
    pub laya_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub laya_api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub laya_model: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv();
        // Also load ~/.laya-gateway/.env when present (systemd install).
        if let Some(home) = env::var_os("HOME") {
            let _ = dotenvy::from_path(Path::new(&home).join(".laya-gateway/.env"));
        }

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

        let settings_path = settings_file_path();
        let mut laya_url = env::var("LAYA_URL").unwrap_or_else(|_| DEFAULT_LAYA_URL.into());
        let mut laya_api_key = env::var("LAYA_API_KEY").ok().filter(|s| !s.trim().is_empty());
        let mut laya_model = env::var("LAYA_MODEL").ok().filter(|s| !s.trim().is_empty());

        // Persisted UI settings override env (except first boot).
        if let Ok(persisted) = load_persisted(&settings_path) {
            laya_url = persisted.laya_url;
            if persisted.laya_api_key.is_some() {
                laya_api_key = persisted.laya_api_key;
            }
            if persisted.laya_model.is_some() {
                laya_model = persisted.laya_model;
            }
        }

        Ok(Self {
            bind_address,
            routing_enabled,
            min_confidence,
            laya_url: normalize_laya_url(&laya_url)?,
            laya_api_key,
            laya_model,
            laya_timeout: Duration::from_millis(
                env::var("LAYA_GATEWAY_LAYA_TIMEOUT_MS")
                    .unwrap_or_else(|_| "30000".into())
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
            settings_path,
        })
    }
}

pub fn settings_file_path() -> PathBuf {
    if let Ok(p) = env::var("LAYA_GATEWAY_SETTINGS") {
        return PathBuf::from(p);
    }
    if let Some(home) = env::var_os("HOME") {
        return Path::new(&home).join(".laya-gateway/settings.json");
    }
    PathBuf::from("settings.json")
}

pub fn load_persisted(path: &Path) -> Result<PersistedSettings> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let parsed: PersistedSettings = serde_json::from_str(&raw).context("parse settings.json")?;
    Ok(PersistedSettings {
        laya_url: normalize_laya_url(&parsed.laya_url)?,
        laya_api_key: parsed.laya_api_key.filter(|s| !s.trim().is_empty()),
        laya_model: parsed.laya_model.filter(|s| !s.trim().is_empty()),
    })
}

pub fn save_persisted(path: &Path, settings: &PersistedSettings) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let body = serde_json::to_string_pretty(settings)?;
    fs::write(path, body + "\n").with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Accept host:port, base URL, or full `/v1/systemone` URL.
pub fn normalize_laya_url(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("laya_url is empty");
    }
    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    };
    let mut url = with_scheme.trim_end_matches('/').to_string();
    if !url.contains("/v1/systemone") {
        url = format!("{url}/v1/systemone");
    }
    // Basic sanity: must look like http(s)://host...
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        bail!("laya_url must be http(s)");
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_host_port() {
        assert_eq!(
            normalize_laya_url("10.8.0.9:8343").unwrap(),
            "http://10.8.0.9:8343/v1/systemone"
        );
    }

    #[test]
    fn normalize_base() {
        assert_eq!(
            normalize_laya_url("http://127.0.0.1:8000").unwrap(),
            "http://127.0.0.1:8000/v1/systemone"
        );
    }

    #[test]
    fn normalize_full() {
        assert_eq!(
            normalize_laya_url("http://127.0.0.1:8000/v1/systemone/").unwrap(),
            "http://127.0.0.1:8000/v1/systemone"
        );
    }
}
