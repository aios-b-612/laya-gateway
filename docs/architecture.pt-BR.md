# Arquitetura

> English: [architecture.md](./architecture.md)

Gateway **local** (Rust) no laptop do desenvolvedor; Laya (System One) pode ser o serviço compartilhado no DEV; o dashboard Next só observa metadados.

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
