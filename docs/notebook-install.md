# Notebook install (Linux)

User-level install used on this machine (not committed secrets).

```bash
# binary
install -m 755 apps/api/target/release/laya-gateway ~/.local/bin/laya-gateway

# env → ~/.laya-gateway/.env  (mode 600)
# systemd → ~/.config/systemd/user/laya-gateway.service
systemctl --user enable --now laya-gateway
```

Check:

```bash
systemctl --user status laya-gateway
curl -fsS http://127.0.0.1:8790/health/live
curl -fsS http://127.0.0.1:8790/v1/stats
```

OpenCode: provider `laya-gateway` in `~/.config/opencode/opencode.json`.  
Cursor hosted models do not route through the proxy; see README.
