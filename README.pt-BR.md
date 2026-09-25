# laya-gateway

> [English](./README.md)

Gateway LLM local (no laptop do desenvolvedor) inspirado no [jev-gateway](https://github.com/vinilana/jev-gateway): quando o coding agent vai decidir **qual tool chamar**, o gateway pergunta ao [Laya](https://github.com/NandhaKishorM/laya) (System One open-weight) em vez de gastar o LLM caro só nessa escolha. O resto do tráfego segue para o modelo usual.

Monorepo:

| Caminho | Stack | Inspirado em |
|---------|--------|----------------|
| `apps/api` | Rust + Actix | [`0ctor/template-api-rust`](https://github.com/0ctor/template-api-rust) |
| `apps/web` | Next.js 16 + Tailwind | [`0ctor/template-frontend`](https://github.com/0ctor/template-frontend) (dashboard local, sem SSO Octor) |

> Projeto independente. Laya é open-source; Jev/TypeSafe são de terceiros. Este gateway **não** é afiliado à TypeSafe.

## Arquitetura

```
IDE / agent (OpenCode, Codex, …)
        │  OpenAI-compatible
        ▼
laya-gateway  :8790   ← 127.0.0.1 na sua máquina
   ├─ “qual tool?” → Laya (local ou endpoint DEV compartilhado)
   └─ resto        → LLM_UPSTREAM_URL
        │
        ▼
dashboard Next  :3000  (só metadados / métricas)
```

- **Gateway = local** (como o jev-gateway): keys do LLM e prompts ficam no laptop.
- **Laya = pode ser compartilhado** (ex. URL na VPN de DEV do time para `/v1/systemone`).

Fail-open: se o Laya cair, o request segue intacto para o LLM.

## Início rápido

```bash
cp .env.example .env
# ajuste LAYA_URL e LLM_UPSTREAM_URL

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

### Apontar um client OpenAI-compatible

Base URL do client → `http://127.0.0.1:8790/v1` (ou conforme o client).  
O `Authorization` do LLM é encaminhado; opcionalmente use `LLM_API_KEY` no `.env`.

## Routing

| Mode | Efeito |
|------|--------|
| `forced` | `tool_choice` fixado na tool escolhida pelo Laya |
| `none` | `tool_choice: none` (texto puro) |
| `passthrough` | request intacto (baixa confiança, sem tools, routing off, erro Laya) |

Toggle: botão no dashboard ou `POST /v1/routing` com `{"enabled":false}` (baseline).

## Segurança

- Bind **somente** loopback (`LAYA_GATEWAY_BIND` deve ser `127.0.0.1` / `localhost`).
- O dashboard não mostra prompts, args de tools nem credentials — só contadores e motivos.

## Status do MVP

Implementado:

- Proxy OpenAI Chat Completions
- Decisão Laya `choice` + `noul`
- Stats + toggle de routing
- Dashboard Next

Roadmap:

- Adapters Anthropic Messages / Gemini / Responses (como no jev-gateway)
- Launchers `laya-opencode` / `laya-codex`
- Shortlist quando há dezenas de tools
- Direct tool call sem LLM (args fechados)

## Licença

MIT — ver [LICENSE](./LICENSE).
