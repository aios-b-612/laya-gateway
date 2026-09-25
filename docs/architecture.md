# Architecture

> Portuguese: [architecture.pt-BR.md](./architecture.pt-BR.md)

**Local** gateway (Rust) on the developer’s laptop; Laya (System One) may be a shared DEV service; the Next dashboard only observes metadata.

```
agent ──► laya-gateway:8790 ──┬── Laya (/v1/systemone)
                              └── LLM upstream
dashboard:3000 ──► GET /v1/stats
```

## Why this split

| Piece | Where | Why |
|-------|--------|-----|
| Laya model | Often shared (DEV) | One warm checkpoint; consistent answers |
| Gateway proxy | `127.0.0.1` per developer | Intercepts agent↔LLM traffic; keys and prompts stay local |
| Dashboard | Local Next app | Live routing/token stats for *your* sessions |

Do not run the transparent LLM proxy as a shared multi-tenant service: that would ship every coding prompt through a central box.
