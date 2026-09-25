use std::sync::Arc;
use std::time::Instant;

use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::config::{
    normalize_laya_url, save_persisted, Config, PersistedSettings, DEFAULT_LAYA_URL,
};
use crate::decide::{self, Decision};
use crate::error::ApiError;
use crate::laya::LayaClient;
use crate::stats::{DecisionMode, GatewayEvent, Stats};

pub struct AppState {
    pub config: Config,
    pub laya: LayaClient,
    pub http: Client,
    pub stats: Arc<Stats>,
}

#[derive(Deserialize)]
pub struct RoutingBody {
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct SettingsBody {
    pub laya_url: String,
    #[serde(default)]
    pub laya_api_key: Option<String>,
    #[serde(default)]
    pub laya_model: Option<String>,
}

pub async fn dashboard_page() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .insert_header(("Cache-Control", "no-store"))
        .body(include_str!("../dashboard.html"))
}

pub async fn logo_aios() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/jpeg")
        .insert_header(("Cache-Control", "public, max-age=86400"))
        .body(include_bytes!("../../assets/aios.jpeg").as_ref())
}

pub async fn health_live(state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(json!({
            "status": "ok",
            "service": state.config.service_name,
        }))
}

pub async fn health_ready(state: web::Data<AppState>) -> HttpResponse {
    let ep = state.laya.snapshot();
    HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(json!({
            "status": "ok",
            "service": state.config.service_name,
            "routing": state.stats.routing_enabled(),
            "laya_url": ep.url,
        }))
}

pub async fn get_stats(state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(state.stats.snapshot())
}

pub async fn set_routing(
    state: web::Data<AppState>,
    body: web::Json<RoutingBody>,
) -> HttpResponse {
    state.stats.set_routing(body.enabled);
    HttpResponse::Ok().json(json!({
        "routing_enabled": state.stats.routing_enabled(),
    }))
}

pub async fn get_settings(state: web::Data<AppState>) -> HttpResponse {
    let ep = state.laya.snapshot();
    HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(json!({
            "laya_url": ep.url,
            "laya_api_key_set": ep.api_key.as_ref().is_some_and(|k| !k.is_empty()),
            "laya_model": ep.model,
            "default_laya_url": DEFAULT_LAYA_URL,
            "presets": [
                {
                    "id": "octor-dev",
                    "label": "Octor DEV (VPN)",
                    "url": "http://10.8.0.9:8343/v1/systemone"
                },
                {
                    "id": "local",
                    "label": "Local Laya",
                    "url": "http://127.0.0.1:8000/v1/systemone"
                },
                {
                    "id": "local-docker",
                    "label": "Local Docker :8343",
                    "url": "http://127.0.0.1:8343/v1/systemone"
                }
            ],
            "settings_path": state.config.settings_path.display().to_string(),
        }))
}

pub async fn set_settings(
    state: web::Data<AppState>,
    body: web::Json<SettingsBody>,
) -> Result<HttpResponse, ApiError> {
    let url = normalize_laya_url(&body.laya_url).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let current = state.laya.snapshot();
    // Empty api key in the form means "keep current"; send "-" or explicit clear later.
    let api_key = match body.laya_api_key.as_deref().map(str::trim) {
        None | Some("") => current.api_key.clone(),
        Some("__clear__") => None,
        Some(v) => Some(v.to_string()),
    };
    let model = match body.laya_model.as_deref().map(str::trim) {
        None | Some("") => current.model.clone(),
        Some("__clear__") => None,
        Some(v) => Some(v.to_string()),
    };
    let persisted = PersistedSettings {
        laya_url: url,
        laya_api_key: api_key,
        laya_model: model,
    };
    state
        .laya
        .apply(&persisted)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    save_persisted(&state.config.settings_path, &persisted)
        .map_err(ApiError::Other)?;
    Ok(HttpResponse::Ok().json(json!({
        "ok": true,
        "laya_url": persisted.laya_url,
        "laya_api_key_set": persisted.laya_api_key.as_ref().is_some_and(|k| !k.is_empty()),
        "laya_model": persisted.laya_model,
        "settings_path": state.config.settings_path.display().to_string(),
    })))
}

pub async fn test_laya(state: web::Data<AppState>) -> HttpResponse {
    match state.laya.health_probe().await {
        Ok((status, body, ms)) => HttpResponse::Ok().json(json!({
            "ok": (200..300).contains(&status),
            "http_status": status,
            "latency_ms": ms,
            "body": body,
            "laya_url": state.laya.snapshot().url,
        })),
        Err(err) => HttpResponse::Ok().json(json!({
            "ok": false,
            "error": err.to_string(),
            "laya_url": state.laya.snapshot().url,
        })),
    }
}

pub async fn chat_completions(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<Value>,
) -> Result<HttpResponse, ApiError> {
    let mut payload = body.into_inner();
    let model = payload
        .get("model")
        .and_then(|m| m.as_str())
        .map(str::to_string);
    let input = decide::extract_from_chat(&payload);
    let tools_count = input.tools.len();

    let mut mode = DecisionMode::Passthrough;
    let mut reason = Some("routing_off".into());
    let mut tool = None;
    let mut confidence = None;
    let mut laya_latency_ms = None;

    if state.stats.routing_enabled() {
        if let Some(skip) = decide::skip_reason(&input) {
            reason = Some(skip.into());
        } else {
            let laya_req = decide::build_request(&input, state.laya.snapshot().model.clone());
            match state.laya.decide(laya_req).await {
                Ok(call) => {
                    laya_latency_ms = Some(call.latency_ms);
                    let decision =
                        decide::interpret(&call.response.answers, state.config.min_confidence);
                    match decision {
                        Decision::Forced { tool: t, confidence: c } => {
                            mode = DecisionMode::Forced;
                            reason = None;
                            confidence = Some(c);
                            tool = Some(t.clone());
                            if let Some(obj) = payload.as_object_mut() {
                                obj.insert(
                                    "tool_choice".into(),
                                    json!({ "type": "function", "function": { "name": t } }),
                                );
                            }
                        }
                        Decision::None { confidence: c } => {
                            mode = DecisionMode::None;
                            reason = None;
                            confidence = Some(c);
                            if let Some(obj) = payload.as_object_mut() {
                                obj.insert("tool_choice".into(), json!("none"));
                            }
                        }
                        Decision::Passthrough { reason: r } => {
                            mode = DecisionMode::Passthrough;
                            reason = Some(r);
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!(error = %err, "Laya failed — fail-open to upstream");
                    state.stats.bump_laya_error();
                    mode = DecisionMode::Passthrough;
                    reason = Some(format!("laya_error:{err}"));
                }
            }
        }
    }

    let auth_header = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    let upstream_started = Instant::now();
    let upstream = forward_chat(&state, &payload, auth_header.as_deref()).await?;
    let upstream_latency_ms = Some(upstream_started.elapsed().as_millis() as u64);

    let (prompt_tokens, completion_tokens) = usage_of(&upstream.body);

    state.stats.record(GatewayEvent {
        id: Uuid::new_v4().to_string(),
        at: Utc::now(),
        mode,
        reason,
        tool,
        confidence,
        laya_latency_ms,
        upstream_latency_ms,
        prompt_tokens,
        completion_tokens,
        model,
        tools_count,
    });

    Ok(HttpResponse::build(
        actix_web::http::StatusCode::from_u16(upstream.status)
            .unwrap_or(actix_web::http::StatusCode::BAD_GATEWAY),
    )
    .content_type("application/json")
    .body(upstream.body))
}

struct UpstreamReply {
    status: u16,
    body: String,
}

async fn forward_chat(
    state: &AppState,
    payload: &Value,
    client_auth: Option<&str>,
) -> Result<UpstreamReply, ApiError> {
    let url = format!("{}/chat/completions", state.config.llm_upstream_url);
    let mut builder = state
        .http
        .post(&url)
        .header("content-type", "application/json")
        .json(payload);

    if let Some(auth) = client_auth {
        builder = builder.header("authorization", auth);
    } else if let Some(key) = &state.config.llm_api_key {
        builder = builder.bearer_auth(key);
    }

    let response = builder.send().await.map_err(|e| ApiError::Upstream(e.to_string()))?;
    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::Upstream(e.to_string()))?;
    Ok(UpstreamReply { status, body })
}

fn usage_of(body: &str) -> (Option<u64>, Option<u64>) {
    let Ok(v) = serde_json::from_str::<Value>(body) else {
        return (None, None);
    };
    let usage = v.get("usage");
    (
        usage
            .and_then(|u| u.get("prompt_tokens"))
            .and_then(|n| n.as_u64()),
        usage
            .and_then(|u| u.get("completion_tokens"))
            .and_then(|n| n.as_u64()),
    )
}
