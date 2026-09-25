import type { GatewayEvent } from "@/lib/gateway";

type Props = {
  events: GatewayEvent[];
};

function modeColor(mode: GatewayEvent["mode"]): string {
  if (mode === "forced") return "var(--ok)";
  if (mode === "none") return "var(--accent)";
  return "var(--muted)";
}

export function EventsTable({ events }: Props) {
  return (
    <section className="overflow-hidden rounded-lg border border-[var(--line)] bg-[var(--panel)]/60">
      <div className="border-b border-[var(--line)] px-4 py-3">
        <h2 className="text-sm font-medium tracking-wide uppercase">
          Requests recentes
        </h2>
      </div>
      <div className="overflow-x-auto">
        <table className="min-w-full text-left text-sm">
          <thead className="text-xs text-[var(--muted)] uppercase">
            <tr>
              <th className="px-4 py-2 font-medium">Quando</th>
              <th className="px-4 py-2 font-medium">Mode</th>
              <th className="px-4 py-2 font-medium">Tool</th>
              <th className="px-4 py-2 font-medium">Conf</th>
              <th className="px-4 py-2 font-medium">Laya ms</th>
              <th className="px-4 py-2 font-medium">Tokens</th>
              <th className="px-4 py-2 font-medium">Motivo</th>
            </tr>
          </thead>
          <tbody>
            {events.length === 0 ? (
              <tr>
                <td
                  colSpan={7}
                  className="px-4 py-8 text-center text-[var(--muted)]"
                >
                  Ainda sem tráfego. Aponte o agent para{" "}
                  <code className="font-[family-name:var(--font-mono)]">
                    http://127.0.0.1:8790/v1
                  </code>
                  .
                </td>
              </tr>
            ) : (
              events.map((ev) => (
                <tr
                  key={ev.id}
                  className="border-t border-[var(--line)]/70 font-[family-name:var(--font-mono)] text-[13px]"
                >
                  <td className="px-4 py-2 whitespace-nowrap text-[var(--muted)]">
                    {new Date(ev.at).toLocaleTimeString()}
                  </td>
                  <td
                    className="px-4 py-2"
                    style={{ color: modeColor(ev.mode) }}
                  >
                    {ev.mode}
                  </td>
                  <td className="px-4 py-2">{ev.tool ?? "—"}</td>
                  <td className="px-4 py-2">
                    {ev.confidence != null
                      ? ev.confidence.toFixed(2)
                      : "—"}
                  </td>
                  <td className="px-4 py-2">
                    {ev.laya_latency_ms ?? "—"}
                  </td>
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
