# laya-gateway

> [Português (Brasil)](./README.pt-BR.md)

A local LLM gateway (runs on the developer’s laptop) inspired by [jev-gateway](https://github.com/vinilana/jev-gateway): when a coding agent is about to decide **which tool to call**, the gateway asks [Laya](https://github.com/NandhaKishorM/laya) (open-weight System One) instead of spending the expensive LLM on that choice alone. Everything else goes to your usual model untouched.

Monorepo layout:

| Path | Stack | Inspired by |
|------|--------|-------------|
| `apps/api` | Rust + Actix | [`0ctor/template-api-rust`](https://github.com/0ctor/template-api-rust) |
| `apps/web` | Next.js 16 + Tailwind | [`0ctor/template-frontend`](https://github.com/0ctor/template-frontend) (local dashboard, no Octor SSO) |

> Independent project. Laya is open source; Jev/TypeSafe are third parties. This gateway is **not** affiliated with TypeSafe.

## Architecture

```
IDE / agent (OpenCode, Codex, …)
        │  OpenAI-compatible
        ▼
laya-gateway  :8790   ← 127.0.0.1 on your machine
   ├─ “which tool?” → Laya (local or shared DEV endpoint)
   └─ everything else → LLM_UPSTREAM_URL
        │
        ▼
Next dashboard  :3000  (metadata / metrics only)
```

- **Gateway = local** (same idea as jev-gateway): LLM keys and prompts stay on the laptop.
- **Laya = may be shared** (e.g. a team DEV VPN URL for `/v1/systemone`).

Fail-open: if Laya is down, the request is forwarded to the LLM unchanged.

## Quick start

```bash
cp .env.example .env
# set LAYA_URL and LLM_UPSTREAM_URL

# terminal 1 — API
cd apps/api && cargo run

# terminal 2 — dashboard
cd apps/web && npm install && npm run dev
```

- Gateway: `http://127.0.0.1:8790`
- Health: `GET /health/live`
- Stats: `GET /v1/stats`
- Chat proxy: `POST /v1/chat/completions` (alias `/chat/completions`)
- Dashboard: `http://127.0.0.1:3000`

### Point an OpenAI-compatible client

Set the client base URL to `http://127.0.0.1:8790/v1` (or as required by the client).  
LLM `Authorization` is forwarded; optionally set `LLM_API_KEY` in `.env`.

## Routing

| Mode | Effect |
|------|--------|
| `forced` | `tool_choice` pinned to the tool Laya selected |
| `none` | `tool_choice: none` (plain text reply) |
| `passthrough` | request unchanged (low confidence, no tools, routing off, Laya error) |

Toggle: dashboard button or `POST /v1/routing` with `{"enabled":false}` (baseline).

## Security

- Bind **loopback only** (`LAYA_GATEWAY_BIND` must be `127.0.0.1` / `localhost`).
- The dashboard never shows prompts, tool arguments, or credentials — only counters and skip reasons.

## MVP status

Shipped:

- OpenAI Chat Completions proxy
- Laya `choice` + `noul` decision
- Stats + routing toggle
- Next dashboard

Roadmap:

- Anthropic Messages / Gemini / Responses adapters (as in jev-gateway)
- Launchers `laya-opencode` / `laya-codex`
- Shortlist when there are dozens of tools
- Direct tool call without an LLM (closed args)

## License

MIT — see [LICENSE](./LICENSE).
