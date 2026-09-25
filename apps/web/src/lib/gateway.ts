export type DecisionMode = "forced" | "none" | "passthrough";

export type GatewayEvent = {
  id: string;
  at: string;
  mode: DecisionMode;
  reason?: string | null;
  tool?: string | null;
  confidence?: number | null;
  laya_latency_ms?: number | null;
  upstream_latency_ms?: number | null;
  prompt_tokens?: number | null;
  completion_tokens?: number | null;
  model?: string | null;
  tools_count: number;
};

export type StatsSnapshot = {
  service: string;
  routing_enabled: boolean;
  requests: number;
  routed: number;
  forced: number;
  none: number;
  passthrough: number;
  laya_errors: number;
  laya_avg_latency_ms: number;
  laya_calls: number;
  events: GatewayEvent[];
  updated_at: string;
};

export function gatewayBaseUrl(): string {
  return (
    process.env.NEXT_PUBLIC_GATEWAY_URL?.replace(/\/$/, "") ||
    "http://127.0.0.1:8790"
  );
}

export async function fetchStats(): Promise<StatsSnapshot> {
  const res = await fetch(`${gatewayBaseUrl()}/v1/stats`, {
    cache: "no-store",
  });
  if (!res.ok) {
    throw new Error(`gateway ${res.status}`);
  }
  return res.json();
}

export async function setRouting(enabled: boolean): Promise<void> {
  const res = await fetch(`${gatewayBaseUrl()}/v1/routing`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ enabled }),
  });
  if (!res.ok) {
    throw new Error(`routing ${res.status}`);
  }
}
