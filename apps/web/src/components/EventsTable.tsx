import type { GatewayEvent } from "@/lib/gateway";
import type { Locale } from "@/lib/i18n";
import { t } from "@/lib/i18n";

type Props = {
  events: GatewayEvent[];
  locale: Locale;
};

function modeColor(mode: GatewayEvent["mode"]): string {
  if (mode === "forced") return "var(--ok)";
  if (mode === "none") return "var(--accent)";
  return "var(--muted)";
}

export function EventsTable({ events, locale }: Props) {
  return (
    <section className="overflow-hidden rounded-lg border border-[var(--line)] bg-[var(--panel)]/60">
      <div className="border-b border-[var(--line)] px-4 py-3">
        <h2 className="text-sm font-medium tracking-wide uppercase">
          {t(locale, "recent")}
        </h2>
      </div>
      <div className="overflow-x-auto">
        <table className="min-w-full text-left text-sm">
          <thead className="text-xs text-[var(--muted)] uppercase">
            <tr>
              <th className="px-4 py-2 font-medium">{t(locale, "when")}</th>
              <th className="px-4 py-2 font-medium">{t(locale, "mode")}</th>
              <th className="px-4 py-2 font-medium">{t(locale, "tool")}</th>
              <th className="px-4 py-2 font-medium">{t(locale, "conf")}</th>
              <th className="px-4 py-2 font-medium">{t(locale, "layaMs")}</th>
              <th className="px-4 py-2 font-medium">{t(locale, "tokens")}</th>
              <th className="px-4 py-2 font-medium">{t(locale, "reason")}</th>
            </tr>
          </thead>
          <tbody>
            {events.length === 0 ? (
              <tr>
                <td
                  colSpan={7}
                  className="px-4 py-8 text-center text-[var(--muted)]"
                >
                  {t(locale, "emptyPrefix")}
                  <code className="font-[family-name:var(--font-mono)]">
                    http://127.0.0.1:8790/v1
                  </code>
                  {t(locale, "emptySuffix")}
                </td>
              </tr>
            ) : (
              events.map((ev) => (
                <tr
                  key={ev.id}
                  className="border-t border-[var(--line)]/70 font-[family-name:var(--font-mono)] text-[13px]"
                >
                  <td className="px-4 py-2 whitespace-nowrap text-[var(--muted)]">
                    {new Date(ev.at).toLocaleTimeString(
                      locale === "pt-BR" ? "pt-BR" : "en-US",
                    )}
                  </td>
                  <td
                    className="px-4 py-2"
                    style={{ color: modeColor(ev.mode) }}
                  >
                    {ev.mode}
                  </td>
                  <td className="px-4 py-2">{ev.tool ?? "—"}</td>
                  <td className="px-4 py-2">
                    {ev.confidence != null ? ev.confidence.toFixed(2) : "—"}
                  </td>
                  <td className="px-4 py-2">{ev.laya_latency_ms ?? "—"}</td>
                  <td className="px-4 py-2">
                    {ev.prompt_tokens ?? "—"} / {ev.completion_tokens ?? "—"}
                  </td>
                  <td className="max-w-[220px] truncate px-4 py-2 text-[var(--muted)]">
                    {ev.reason ?? "—"}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}
