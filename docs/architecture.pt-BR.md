# Arquitetura

> English: [architecture.md](./architecture.md)

Gateway **local** (Rust) no laptop do desenvolvedor; Laya (System One) pode ser o serviço compartilhado no DEV; o dashboard Next só observa metadados.

## Diagramas

HTML Archify interativo (abra no navegador):

- [Arquitetura no laptop](./diagramas/archify/architecture.laptop.html) — quem fala com quem
- [Workflow da decisão de tool](./diagramas/archify/workflow.tool-decision.html) — o que acontece em cada request

```
agent ──► laya-gateway:8790 ──┬── Laya (/v1/systemone)
                              └── LLM upstream
dashboard:3000 ──► GET /v1/stats
```

## Por que essa divisão

| Peça | Onde | Por quê |
|------|------|---------|
| Modelo Laya | Em geral compartilhado (DEV) | Um checkpoint quente; respostas consistentes |
| Proxy gateway | `127.0.0.1` por desenvolvedor | Intercepta agent↔LLM; keys e prompts ficam locais |
| Dashboard | Next local | Métricas de routing/tokens da *sua* sessão |

Não rode o proxy transparente de LLM como serviço multi-tenant compartilhado: isso mandaria todo prompt de código por um box central.

## Metáfora para leigos

O gateway é o ajudante de cozinha. O Laya responde “qual utensílio?” em milissegundos. O chef-mestre (seu LLM) cozinha — e se o ajudante estiver inseguro, o chef decide como sempre (fail-open).
