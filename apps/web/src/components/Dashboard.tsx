"use client";

import { useCallback, useEffect, useState } from "react";
import {
  fetchStats,
  setRouting,
  type StatsSnapshot,
} from "@/lib/gateway";
import { type Locale, t } from "@/lib/i18n";
import { StatCard } from "@/components/StatCard";
import { EventsTable } from "@/components/EventsTable";

const POLL_MS = 2000;
const LOCALE_KEY = "laya-gateway-locale";

export function Dashboard() {
  const [stats, setStats] = useState<StatsSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [locale, setLocale] = useState<Locale>("en");

  useEffect(() => {
    const saved = window.localStorage.getItem(LOCALE_KEY);
    if (saved === "en" || saved === "pt-BR") setLocale(saved);
  }, []);

  function changeLocale(next: Locale) {
    setLocale(next);
    window.localStorage.setItem(LOCALE_KEY, next);
  }

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
      setError(e instanceof Error ? e.message : "routing toggle failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="mx-auto flex min-h-screen max-w-6xl flex-col gap-8 px-6 py-10">
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <p className="mb-1 text-sm tracking-[0.18em] text-[var(--accent)] uppercase">
            {t(locale, "eyebrow")}
          </p>
          <h1 className="text-4xl font-semibold tracking-tight">
            {t(locale, "title")}
          </h1>
          <p className="mt-2 max-w-xl text-[var(--muted)]">
            {t(locale, "subtitle")}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <label className="text-xs text-[var(--muted)]">
            {t(locale, "lang")}
            <select
              className="ml-2 rounded-md border border-[var(--line)] bg-[var(--panel)] px-2 py-1 text-sm text-[var(--text)]"
              value={locale}
              onChange={(e) => changeLocale(e.target.value as Locale)}
            >
              <option value="en">English</option>
              <option value="pt-BR">Português (Brasil)</option>
            </select>
          </label>
          <button
            type="button"
            onClick={() => void toggleRouting()}
            disabled={!stats || busy}
            className="rounded-md border border-[var(--line)] bg-[var(--panel)] px-4 py-2 text-sm font-medium transition hover:border-[var(--accent)] disabled:opacity-50"
          >
            {t(locale, "routing")}:{" "}
            <span
              style={{
                color: stats?.routing_enabled ? "var(--ok)" : "var(--warn)",
              }}
            >
              {stats?.routing_enabled ? "on" : "off"}
            </span>
          </button>
        </div>
      </header>

      {error ? (
        <div className="rounded-md border border-[var(--danger)]/40 bg-[var(--danger)]/10 px-4 py-3 text-sm">
          {t(locale, "offlinePrefix", { error })}
          <code className="font-[family-name:var(--font-mono)]">make api</code>
          {t(locale, "offlineSuffix")}
        </div>
      ) : null}

      <section className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard label={t(locale, "requests")} value={stats?.requests ?? "—"} />
        <StatCard
          label={t(locale, "forced")}
          value={stats?.forced ?? "—"}
          accent
        />
        <StatCard
          label={t(locale, "passthrough")}
          value={stats?.passthrough ?? "—"}
        />
        <StatCard
          label={t(locale, "layaAvg")}
          value={stats ? stats.laya_avg_latency_ms.toFixed(0) : "—"}
        />
      </section>

      <section className="grid gap-4 sm:grid-cols-3">
        <StatCard label={t(locale, "routed")} value={stats?.routed ?? "—"} />
        <StatCard label={t(locale, "none")} value={stats?.none ?? "—"} />
        <StatCard
          label={t(locale, "layaErrors")}
          value={stats?.laya_errors ?? "—"}
        />
      </section>

      <EventsTable events={stats?.events ?? []} locale={locale} />

      <footer className="border-t border-[var(--line)] pt-4 text-xs text-[var(--muted)]">
        {t(locale, "updated")}{" "}
        {stats?.updated_at
          ? new Date(stats.updated_at).toLocaleString(
              locale === "pt-BR" ? "pt-BR" : "en-US",
            )
          : "—"}{" "}
        · {t(locale, "privacy")}
      </footer>
    </main>
  );
}
