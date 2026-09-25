"use client";

import Image from "next/image";
import { useCallback, useEffect, useState } from "react";
import {
  fetchStats,
  setRouting,
  type StatsSnapshot,
} from "@/lib/gateway";
import { type Locale, t } from "@/lib/i18n";
import { StatCard } from "@/components/StatCard";
import { EventsTable } from "@/components/EventsTable";
import { LayaSettings } from "@/components/LayaSettings";

const POLL_MS = 2000;
const LOCALE_KEY = "laya-gateway-locale";
const GATEWAY =
  process.env.NEXT_PUBLIC_GATEWAY_URL?.replace(/\/$/, "") ||
  "http://127.0.0.1:8790";

type Panel = "overview" | "settings" | "traffic";

export function Dashboard() {
  const [stats, setStats] = useState<StatsSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [locale, setLocale] = useState<Locale>("en");
  const [panel, setPanel] = useState<Panel>("overview");

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

  const titles: Record<Panel, string> = {
    overview: "Overview",
    settings: "Laya endpoint",
    traffic: "Traffic",
  };

  return (
    <div className="flex min-h-screen bg-[var(--gray-100)] text-[var(--gray-500)]">
      <aside
        className="flex w-[var(--side-nav-width)] flex-none flex-col border-r border-[var(--gray-200)] bg-white"
      >
        <div className="flex flex-col items-start gap-2.5 border-b border-[var(--gray-200)] px-5 pb-4 pt-5">
          <Image
            src="/img/logo/aios.jpeg"
            alt="AIOS"
            width={180}
            height={44}
            className="h-11 w-auto max-w-[180px] object-contain"
            priority
          />
          <div className="text-[13px] font-semibold text-[var(--gray-700)]">
            Laya Gateway
          </div>
        </div>
        <nav className="grid gap-1 p-3">
          {(
            [
              ["overview", "Overview"],
              ["settings", "Laya endpoint"],
              ["traffic", "Traffic"],
            ] as const
          ).map(([id, label]) => (
            <button
              key={id}
              type="button"
              onClick={() => setPanel(id)}
              className={`rounded-[10px] px-3 py-2.5 text-left text-sm ${
                panel === id
                  ? "bg-[var(--primary-subtle)] font-semibold text-[var(--primary-deep)]"
                  : "text-[var(--gray-700)] hover:bg-[var(--gray-100)]"
              }`}
            >
              {label}
            </button>
          ))}
        </nav>
      </aside>

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-[var(--header-height)] items-center justify-between gap-3 border-b border-[var(--gray-200)] bg-white px-6">
          <div>
            <h1 className="text-lg font-bold text-[var(--gray-900)]">
              {titles[panel]}
            </h1>
            <p className="text-xs text-[var(--gray-400)]">
              local · 127.0.0.1:8790
            </p>
          </div>
          <div className="flex items-center gap-2">
            <label className="text-xs text-[var(--gray-400)]">
              {t(locale, "lang")}
              <select
                className="ml-2 rounded-lg border border-[var(--gray-200)] bg-white px-2 py-1 text-sm text-[var(--gray-700)]"
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
              className="rounded-lg border border-[var(--gray-200)] bg-white px-3 py-2 text-sm font-medium text-[var(--gray-700)] hover:border-[var(--primary)] disabled:opacity-50"
            >
              {t(locale, "routing")}:{" "}
              <span
                style={{
                  color: stats?.routing_enabled
                    ? "var(--primary)"
                    : "#f59e0b",
                }}
              >
                {stats?.routing_enabled ? "on" : "off"}
              </span>
            </button>
          </div>
        </header>

        <main className="grid gap-4 p-6">
          {error ? (
            <div className="rounded-[10px] border border-[#ff6a5540] bg-[#ff6a5514] px-4 py-3 text-sm text-[#ff6a55]">
              {t(locale, "offlinePrefix", { error })}
              <code className="font-mono">make api</code>
              {t(locale, "offlineSuffix")}
            </div>
          ) : null}

          {panel === "overview" ? (
            <>
              <section className="rounded-xl border border-[var(--gray-200)] bg-white p-5">
                <h2 className="text-[15px] font-bold text-[var(--gray-900)]">
                  Gateway status
                </h2>
                <p className="mb-4 mt-1 text-[13px] text-[var(--gray-400)]">
                  {t(locale, "subtitle")}
                </p>
                <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
                  <StatCard
                    label={t(locale, "requests")}
                    value={stats?.requests ?? "—"}
                  />
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
                    value={
                      stats ? stats.laya_avg_latency_ms.toFixed(0) : "—"
                    }
                  />
                </div>
              </section>
              <section className="grid gap-3 sm:grid-cols-3">
                <StatCard
                  label={t(locale, "routed")}
                  value={stats?.routed ?? "—"}
                />
                <StatCard
                  label={t(locale, "none")}
                  value={stats?.none ?? "—"}
                />
                <StatCard
                  label={t(locale, "layaErrors")}
                  value={stats?.laya_errors ?? "—"}
                />
              </section>
            </>
          ) : null}

          {panel === "settings" ? <LayaSettings gateway={GATEWAY} /> : null}

          {panel === "traffic" ? (
            <EventsTable events={stats?.events ?? []} locale={locale} />
          ) : null}
        </main>
      </div>
    </div>
  );
}
