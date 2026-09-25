# AGENTS.md — laya-gateway

> Portuguese: [AGENTS.pt-BR.md](./AGENTS.pt-BR.md)

Public monorepo (`aios-b-612/laya-gateway`): local gateway + dashboard.

## Stack

| App | Role | Stack |
|-----|------|--------|
| `apps/api` | OpenAI-compatible proxy + Laya decision | Rust / Actix (patterns from `0ctor/template-api-rust`) |
| `apps/web` | Metrics dashboard | Next.js 16 (`0ctor/template-frontend` stack, no SSO) |

## Rules for this repo

1. Gateway **always** on loopback — do not expose it on LAN/VPN as an LLM proxy.
2. Laya may be remote (DEV); the chat proxy **must not**.
3. Fail-open if Laya fails.
4. Event timestamps in **UTC** (`Z`) on the API; the UI formats in the operator’s timezone.
5. No secrets in git — only `.env.example`.
6. Agent username: **Alfred**.

## Documentation language

- Canonical docs: **English** (`README.md`, `docs/*.md`).
- Optional locale: **Portuguese (Brazil)** (`README.pt-BR.md`, `docs/*.pt-BR.md`, `AGENTS.pt-BR.md`).

## Commands

```bash
make api    # cargo run in apps/api
make web    # next dev in apps/web
make test
```
