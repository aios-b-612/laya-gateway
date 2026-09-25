use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionMode {
    Forced,
    None,
    Passthrough,
}

#[derive(Clone, Debug, Serialize)]
pub struct GatewayEvent {
    pub id: String,
    /// Instant in UTC (ISO-8601 with `Z`).
    pub at: DateTime<Utc>,
    pub mode: DecisionMode,
    pub reason: Option<String>,
    pub tool: Option<String>,
    pub confidence: Option<f64>,
    pub laya_latency_ms: Option<u64>,
    pub upstream_latency_ms: Option<u64>,
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub model: Option<String>,
    pub tools_count: usize,
}

#[derive(Default)]
struct Counters {
    requests: AtomicU64,
    routed: AtomicU64,
    forced: AtomicU64,
    none: AtomicU64,
    passthrough: AtomicU64,
    laya_errors: AtomicU64,
    laya_latency_sum_ms: AtomicU64,
    laya_calls: AtomicU64,
}

pub struct Stats {
    routing_enabled: AtomicBool,
    counters: Counters,
    events: Mutex<VecDeque<GatewayEvent>>,
    max_events: usize,
}

impl Stats {
    pub fn new(max_events: usize, routing_enabled: bool) -> Self {
        Self {
            routing_enabled: AtomicBool::new(routing_enabled),
            counters: Counters::default(),
            events: Mutex::new(VecDeque::with_capacity(max_events.min(512))),
            max_events,
        }
    }

    pub fn routing_enabled(&self) -> bool {
        self.routing_enabled.load(Ordering::Relaxed)
    }

    pub fn set_routing(&self, on: bool) {
        self.routing_enabled.store(on, Ordering::Relaxed);
    }

    pub fn record(&self, mut event: GatewayEvent) {
        self.counters.requests.fetch_add(1, Ordering::Relaxed);
        match event.mode {
            DecisionMode::Forced => {
                self.counters.routed.fetch_add(1, Ordering::Relaxed);
                self.counters.forced.fetch_add(1, Ordering::Relaxed);
            }
            DecisionMode::None => {
                self.counters.routed.fetch_add(1, Ordering::Relaxed);
                self.counters.none.fetch_add(1, Ordering::Relaxed);
            }
            DecisionMode::Passthrough => {
                self.counters.passthrough.fetch_add(1, Ordering::Relaxed);
            }
        }
        if let Some(ms) = event.laya_latency_ms {
            self.counters.laya_calls.fetch_add(1, Ordering::Relaxed);
            self.counters
                .laya_latency_sum_ms
                .fetch_add(ms, Ordering::Relaxed);
        }
        if event.id.is_empty() {
            event.id = Uuid::new_v4().to_string();
        }
        if let Ok(mut q) = self.events.lock() {
            if q.len() >= self.max_events {
                q.pop_front();
            }
            q.push_back(event);
        }
    }

    pub fn bump_laya_error(&self) {
        self.counters.laya_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> StatsSnapshot {
        let c = &self.counters;
        let laya_calls = c.laya_calls.load(Ordering::Relaxed);
        let sum = c.laya_latency_sum_ms.load(Ordering::Relaxed);
        let events = self
            .events
            .lock()
            .map(|q| q.iter().rev().cloned().collect())
            .unwrap_or_default();

        StatsSnapshot {
            service: "laya-gateway".into(),
            routing_enabled: self.routing_enabled(),
            requests: c.requests.load(Ordering::Relaxed),
            routed: c.routed.load(Ordering::Relaxed),
            forced: c.forced.load(Ordering::Relaxed),
            none: c.none.load(Ordering::Relaxed),
            passthrough: c.passthrough.load(Ordering::Relaxed),
            laya_errors: c.laya_errors.load(Ordering::Relaxed),
            laya_avg_latency_ms: if laya_calls == 0 {
                0.0
            } else {
                sum as f64 / laya_calls as f64
            },
            laya_calls,
            events,
            updated_at: Utc::now(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct StatsSnapshot {
    pub service: String,
    pub routing_enabled: bool,
    pub requests: u64,
    pub routed: u64,
    pub forced: u64,
    pub none: u64,
    pub passthrough: u64,
    pub laya_errors: u64,
    pub laya_avg_latency_ms: f64,
    pub laya_calls: u64,
    pub events: Vec<GatewayEvent>,
    /// Snapshot instant in UTC.
    pub updated_at: DateTime<Utc>,
}
