.PHONY: api web dev check test fmt lint

api:
	cd apps/api && cargo run

web:
	cd apps/web && npm run dev

dev:
	@echo "Terminal 1: make api"
	@echo "Terminal 2: make web"
	@echo "Dashboard:  http://127.0.0.1:3000"
	@echo "Gateway:    http://127.0.0.1:8790"

check:
	cd apps/api && cargo check
	cd apps/web && npm run lint

test:
	cd apps/api && cargo test
	cd apps/web && npm test

fmt:
	cd apps/api && cargo fmt
	cd apps/web && npx prettier src --write

lint:
	cd apps/api && cargo clippy -- -D warnings
	cd apps/web && npm run lint
