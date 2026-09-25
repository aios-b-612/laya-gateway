# AGENTS.md — laya-gateway

Monorepo público (`aios-b-612/laya-gateway`): gateway local + dashboard.

## Stack

| App | Papel | Stack |
|-----|--------|--------|
| `apps/api` | Proxy OpenAI-compatible + decisão Laya | Rust / Actix (padrões `0ctor/template-api-rust`) |
| `apps/web` | Dashboard de métricas | Next.js 16 (stack `0ctor/template-frontend`, sem SSO) |

## Regras deste repo

1. Gateway **sempre** em loopback — não expor na LAN/VPN como proxy de LLM.
2. Laya pode ser remoto (DEV); o proxy de chat **não**.
3. Fail-open se Laya falhar.
4. Timestamps de eventos em **UTC** (`Z`) na API; UI formata no fuso do operador.
5. Sem secrets no git — só `.env.example`.
6. Agente: **Alfred**.

## Comandos

```bash
make api    # cargo run em apps/api
make web    # next dev em apps/web
make test
```
