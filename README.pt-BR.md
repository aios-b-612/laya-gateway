# laya-gateway

> [English](./README.md)

Gateway LLM local (no laptop do desenvolvedor) inspirado no [jev-gateway](https://github.com/vinilana/jev-gateway): quando o coding agent vai decidir **qual tool chamar**, o gateway pergunta ao [Laya](https://github.com/NandhaKishorM/laya) (System One open-weight) em vez de gastar o LLM caro só nessa escolha. O resto do tráfego segue para o modelo usual.

## Para leigos (linguagem simples)

Imagine o coding agent como um chef ocupado. A cada pouco ele precisa escolher um utensílio (ler um arquivo, rodar um comando, buscar na web…). Perguntar ao cérebro grande e caro (“qual colher?”) em toda rodada é lento e caro.

**Laya** é um especialista pequeno que só responde perguntas tipadas como “qual tool?” ou “sim/não?” — rápido, com uma nota de confiança. O **laya-gateway** fica entre o agent e o LLM de sempre: pergunta isso ao Laya e ou direciona o LLM para aquela tool, ou deixa o pedido intacto se o Laya estiver inseguro (fail-open).

| Peça | Metáfora | Onde roda |
|------|----------|-----------|
| Coding agent | O chef | OpenCode / Cursor no seu PC |
| laya-gateway | O ajudante de cozinha | Só no seu PC (`127.0.0.1`) |
| Laya | Especialista “qual tool?” | Seu PC ou box DEV compartilhado |
| Seu LLM | O chef-mestre do trabalho difícil | Modelo local ou API na nuvem |
| Dashboard | Placar | Seu PC (`:3000`) |

Diagramas interativos (abra o HTML no navegador):

- [Arquitetura — layout no laptop](./docs/diagramas/archify/architecture.laptop.html)
- [Workflow — decisão de tool](./docs/diagramas/archify/workflow.tool-decision.html)

Fontes JSON (Archify): [`docs/diagramas/archify/`](./docs/diagramas/archify/).

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

Detalhe: [docs/architecture.pt-BR.md](./docs/architecture.pt-BR.md).

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

**OpenCode** — provider em `opencode.json` apontando para `http://127.0.0.1:8790/v1`.

**Cursor:** os modelos hospedados do Composer/Agent **não** passam por este proxy. Use OpenCode (ou outro client OpenAI-compatible) para o roteamento completo. Em builds com BYOK OpenAI, a base URL pode ser `http://127.0.0.1:8790/v1`.

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
- Diagramas Archify (arquitetura + workflow)

Roadmap:

- Adapters Anthropic Messages / Gemini / Responses (como no jev-gateway)
- Launchers `laya-opencode` / `laya-codex`
- Shortlist quando há dezenas de tools
- Direct tool call sem LLM (args fechados)

## Licença

MIT — ver [LICENSE](./LICENSE).
