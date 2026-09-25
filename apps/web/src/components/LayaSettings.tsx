"use client";

import { useEffect, useState } from "react";

type Preset = { id: string; label: string; url: string };

type Settings = {
  laya_url: string;
  laya_api_key_set: boolean;
  laya_model?: string | null;
  presets: Preset[];
  settings_path: string;
};

export function LayaSettings({ gateway }: { gateway: string }) {
  const [url, setUrl] = useState("");
  const [key, setKey] = useState("");
  const [model, setModel] = useState("");
  const [presets, setPresets] = useState<Preset[]>([]);
  const [path, setPath] = useState("~/.laya-gateway/settings.json");
  const [msg, setMsg] = useState("");
  const [ok, setOk] = useState<boolean | null>(null);

  async function load() {
    const res = await fetch(`${gateway}/v1/settings`, { cache: "no-store" });
    if (!res.ok) throw new Error(`settings ${res.status}`);
    const s = (await res.json()) as Settings;
    setUrl(s.laya_url || "");
    setModel(s.laya_model || "");
    setPresets(s.presets || []);
    setPath(s.settings_path || path);
  }

  useEffect(() => {
    void load().catch((e) => {
      setMsg(String(e));
      setOk(false);
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [gateway]);

  async function save() {
    setMsg("Saving…");
    setOk(null);
    const body: Record<string, string | null> = {
      laya_url: url.trim(),
      laya_model: model.trim() || null,
    };
    if (key.trim()) body.laya_api_key = key.trim();
    const res = await fetch(`${gateway}/v1/settings`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    const data = await res.json().catch(() => ({}));
    if (!res.ok) {
      setMsg(data.error || `Save failed (${res.status})`);
      setOk(false);
      return;
    }
    setKey("");
    setMsg(`Saved · ${data.laya_url}`);
    setOk(true);
    await load();
  }

  async function test() {
    setMsg("Testing…");
    setOk(null);
    await save();
    const res = await fetch(`${gateway}/v1/settings/test`, { method: "POST" });
    const data = await res.json();
    if (data.ok) {
      setMsg(`OK · HTTP ${data.http_status} · ${data.latency_ms} ms`);
      setOk(true);
    } else {
      setMsg(data.error || `Failed · HTTP ${data.http_status || "?"}`);
      setOk(false);
    }
  }

  return (
    <section className="rounded-xl border border-[var(--gray-200)] bg-white p-5">
      <h2 className="text-[15px] font-bold text-[var(--gray-900)]">
        Where is Laya?
      </h2>
      <p className="mb-4 mt-1 text-[13px] text-[var(--gray-400)]">
        Set the URL of your Laya System One service. You can run it on this
        machine or any reachable host.
      </p>
      <div className="mb-3 flex flex-wrap gap-2">
        {presets.map((p) => (
          <button
            key={p.id}
            type="button"
            className="rounded-lg border border-[var(--gray-200)] bg-white px-3 py-2 text-sm hover:border-[var(--primary)]"
            onClick={() => setUrl(p.url)}
          >
            {p.label}
          </button>
        ))}
      </div>
      <label className="mb-3 block text-xs font-semibold text-[var(--gray-500)]">
        Laya URL
        <input
          className="mt-1 w-full rounded-lg border border-[var(--gray-200)] bg-[var(--gray-100)] px-3 py-2 font-mono text-sm text-[var(--gray-900)]"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
        />
      </label>
      <label className="mb-3 block text-xs font-semibold text-[var(--gray-500)]">
        Optional API key
        <input
          type="password"
          className="mt-1 w-full rounded-lg border border-[var(--gray-200)] bg-[var(--gray-100)] px-3 py-2 font-mono text-sm"
          value={key}
          onChange={(e) => setKey(e.target.value)}
          placeholder="leave blank to keep current"
        />
      </label>
      <label className="mb-4 block text-xs font-semibold text-[var(--gray-500)]">
        Optional model id
        <input
          className="mt-1 w-full rounded-lg border border-[var(--gray-200)] bg-[var(--gray-100)] px-3 py-2 font-mono text-sm"
          value={model}
          onChange={(e) => setModel(e.target.value)}
          placeholder="e.g. multilingual"
        />
      </label>
      <div className="flex flex-wrap items-center gap-2">
        <button
          type="button"
          onClick={() => void save()}
          className="rounded-lg bg-[var(--primary)] px-4 py-2 text-sm font-semibold text-white hover:bg-[var(--primary-deep)]"
        >
          Save
        </button>
        <button
          type="button"
          onClick={() => void test()}
          className="rounded-lg border border-[var(--gray-200)] bg-white px-4 py-2 text-sm"
        >
          Test connection
        </button>
        <span
          className={`text-xs ${ok === true ? "text-[var(--primary)]" : ok === false ? "text-[#ff6a55]" : "text-[var(--gray-400)]"}`}
        >
          {msg}
        </span>
      </div>
    </section>
  );
}
