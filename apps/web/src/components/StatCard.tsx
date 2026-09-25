type Props = {
  label: string;
  value: string | number;
  accent?: boolean;
};

export function StatCard({ label, value, accent }: Props) {
  return (
    <div className="rounded-xl border border-[var(--gray-200)] bg-white px-4 py-4">
      <p className="text-[11px] font-semibold tracking-wide text-[var(--gray-400)] uppercase">
        {label}
      </p>
      <p
        className="mt-2 text-2xl font-bold text-[var(--gray-900)]"
        style={{ color: accent ? "var(--primary)" : undefined }}
      >
        {value}
      </p>
    </div>
  );
}
