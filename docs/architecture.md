# Arquitetura em uma frase

Gateway **local** (Rust) no laptop do dev; Laya (System One) pode ser o serviço compartilhado no DEV; dashboard Next só observa metadados.

```
agent ──► laya-gateway:8790 ──┬── Laya (/v1/systemone)
                              └── LLM upstream
dashboard:3000 ──► GET /v1/stats
```
