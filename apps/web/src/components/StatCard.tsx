type Props = {
  label: string;
  value: string | number;
  accent?: boolean;
};

export function StatCard({ label, value, accent }: Props) {
  return (
    <div className="rounded-lg border border-[var(--line)] bg-[var(--panel)]/80 px-4 py-5">
      <p className="text-xs tracking-wide text-[var(--muted)] uppercase">
        {label}
      </p>
      <p
        className="mt-2 font-[family-name:var(--font-mono)] text-3xl font-medium"
        style={{ color: accent ? "var(--accent)" : "var(--text)" }}
      >
        {value}
      </p>
    </div>
  );
}
