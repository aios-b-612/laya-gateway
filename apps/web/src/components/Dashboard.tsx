"use client";

import { useCallback, useEffect, useState } from "react";
import {
  fetchStats,
  setRouting,
  type StatsSnapshot,
} from "@/lib/gateway";
import { StatCard } from "@/components/StatCard";
import { EventsTable } from "@/components/EventsTable";

const POLL_MS = 2000;

export function Dashboard() {
  const [stats, setStats] = useState<StatsSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    try {
      const next = await fetchStats();
      setStats(next);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "offline");
    }
  }, []);

  useEffect(() => {
    void load();
    const id = window.setInterval(() => void load(), POLL_MS);
    return () => window.clearInterval(id);
  }, [load]);

  async function toggleRouting() {
    if (!stats || busy) return;
    setBusy(true);
    try {
      await setRouting(!stats.routing_enabled);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "falha ao alternar routing");
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="mx-auto flex min-h-screen max-w-6xl flex-col gap-8 px-6 py-10">
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <p className="mb-1 text-sm tracking-[0.18em] text-[var(--accent)] uppercase">
            local · 127.0.0.1
          </p>
          <h1 className="text-4xl font-semibold tracking-tight">laya-gateway</h1>
          <p className="mt-2 max-w-xl text-[var(--muted)]">
            Laya escolhe a tool; o LLM caro só preenche argumentos ou responde.
            Inspirado no jev-gateway — motor System One open-weight.
          </p>
        </div>
        <button
          type="button"
          onClick={() => void toggleRouting()}
          disabled={!stats || busy}
          className="rounded-md border border-[var(--line)] bg-[var(--panel)] px-4 py-2 text-sm font-medium transition hover:border-[var(--accent)] disabled:opacity-50"
        >
          Routing:{" "}
          <span
            style={{
              color: stats?.routing_enabled ? "var(--ok)" : "var(--warn)",
            }}
          >
            {stats?.routing_enabled ? "on" : "off"}
          </span>
        </button>
      </header>

      {error ? (
        <div className="rounded-md border border-[var(--danger)]/40 bg-[var(--danger)]/10 px-4 py-3 text-sm">
          Gateway offline ou inacessível ({error}). Suba com{" "}
          <code className="font-[family-name:var(--font-mono)]">make api</code>.
        </div>
      ) : null}

      <section className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard label="Requests" value={stats?.requests ?? "—"} />
        <StatCard label="Forced tools" value={stats?.forced ?? "—"} accent />
        <StatCard label="Passthrough" value={stats?.passthrough ?? "—"} />
        <StatCard
          label="Laya avg ms"
          value={
            stats
              ? stats.laya_avg_latency_ms.toFixed(0)
              : "—"
          }
        />
      </section>

      <section className="grid gap-4 sm:grid-cols-3">
        <StatCard label="Routed" value={stats?.routed ?? "—"} />
        <StatCard label="No tool" value={stats?.none ?? "—"} />
        <StatCard label="Laya errors" value={stats?.laya_errors ?? "—"} />
      </section>

      <EventsTable events={stats?.events ?? []} />

      <footer className="border-t border-[var(--line)] pt-4 text-xs text-[var(--muted)]">
        Atualizado{" "}
        {stats?.updated_at
          ? new Date(stats.updated_at).toLocaleString()
          : "—"}{" "}
        · prompts e args nunca aparecem aqui · só metadados
      </footer>
    </main>
  );
}
