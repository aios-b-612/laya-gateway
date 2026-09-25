import type { GatewayEvent } from "@/lib/gateway";
import type { Locale } from "@/lib/i18n";
import { t } from "@/lib/i18n";

type Props = {
  events: GatewayEvent[];
  locale: Locale;
};

function modeColor(mode: GatewayEvent["mode"]): string {
  if (mode === "forced") return "var(--primary)";
  if (mode === "none") return "#2a85ff";
  return "var(--gray-400)";
}

export function EventsTable({ events, locale }: Props) {
  return (
    <section className="overflow-hidden rounded-xl border border-[var(--gray-200)] bg-white">
      <div className="border-b border-[var(--gray-200)] px-4 py-3">
        <h2 className="text-sm font-bold tracking-wide text-[var(--gray-900)] uppercase">
          {t(locale, "recent")}
        </h2>
      </div>
      <div className="overflow-x-auto">
        <table className="min-w-full text-left text-sm">
          <thead className="text-[11px] font-semibold text-[var(--gray-400)] uppercase">
            <tr>
              <th className="px-4 py-2">{t(locale, "when")}</th>
              <th className="px-4 py-2">{t(locale, "mode")}</th>
              <th className="px-4 py-2">{t(locale, "tool")}</th>
              <th className="px-4 py-2">{t(locale, "conf")}</th>
              <th className="px-4 py-2">{t(locale, "layaMs")}</th>
              <th className="px-4 py-2">{t(locale, "tokens")}</th>
              <th className="px-4 py-2">{t(locale, "reason")}</th>
            </tr>
          </thead>
          <tbody>
            {events.length === 0 ? (
              <tr>
                <td
                  colSpan={7}
                  className="px-4 py-8 text-center text-[var(--gray-400)]"
                >
                  {t(locale, "emptyPrefix")}
                  <code className="font-mono">http://127.0.0.1:8790/v1</code>
                  {t(locale, "emptySuffix")}
                </td>
              </tr>
            ) : (
              events.map((ev) => (
                <tr
                  key={ev.id}
                  className="border-t border-[var(--gray-100)] font-mono text-[12px]"
                >
                  <td className="px-4 py-2 whitespace-nowrap text-[var(--gray-400)]">
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
                  <td className="max-w-[220px] truncate px-4 py-2 text-[var(--gray-400)]">
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
