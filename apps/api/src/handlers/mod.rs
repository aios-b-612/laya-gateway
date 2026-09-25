use std::sync::Arc;
use std::time::Instant;

use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::config::Config;
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

pub async fn dashboard_page() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .insert_header(("Cache-Control", "no-store"))
        .body(include_str!("../dashboard.html"))
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
    HttpResponse::Ok()
        .insert_header(("Cache-Control", "no-store"))
        .json(json!({
            "status": "ok",
            "service": state.config.service_name,
            "routing": state.stats.routing_enabled(),
            "laya_url": state.config.laya_url,
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
            let laya_req = decide::build_request(&input, state.config.laya_model.clone());
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
