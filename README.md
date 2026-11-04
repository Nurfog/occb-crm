# OCCB CRM — clon de Twenty en Rust 2024 + Dioxus

Migración MVP de [twentyhq/twenty](https://github.com/twentyhq/twenty) (React + NestJS + PostgreSQL)
a **Rust edition 2024 + Dioxus 0.7 + Axum + PostgreSQL**.

> Twenty pesa ~2.5GB / 17k commits. Este repo es un clon funcional MVP:
> misma UX (sidebar, tablas People/Companies, kanban Opportunities, dashboard, login/settings),
> mismo dominio (companies, people, opportunities, pipeline), backend real REST listo para Postgres.

## Estructura

```
occb-crm/
  Cargo.toml              # workspace Rust 2024
  crates/
    core/                 # dominio compartido (equivale a twenty-server entities + seed)
    api/                  # backend Axum :3001 (equivale a NestJS + BullMQ simplificado)
    app/                  # frontend Dioxus web (equivale a twenty-front React)
      Dioxus.toml
      assets/main.css     # clon visual Twenty (claro, sidebar 250px, kanban 5 col)
  migrations/001_init.sql # esquema Postgres espejo (companies, people, opportunities)
  docker-compose.yml      # postgres:16 + redis:7
```

### Mapa Twenty → Rust

| Twenty original | OCCB Rust |
|---|---|
| `twenty-front` (React + Jotai + Linaria) | `crates/app` (Dioxus 0.7 + Router + signals) |
| `twenty-server` (NestJS + GraphQL + BullMQ) | `crates/api` (Axum + tokio + tower-http) |
| PostgreSQL + Redis | `docker-compose.yml` + `migrations/001_init.sql` (API hoy en memoria con seed, migrable a SQLx) |
| Nx monorepo | Cargo workspace `resolver = "2"`, `edition = "2024"` |

## Requisitos

- Rust 1.85+ (probado con 1.98), `wasm32-unknown-unknown`, Dioxus CLI 0.7 (`dx`)
- Opcional: Docker para Postgres/Redis

```bash
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --locked # o binstall
```

## Correr

Backend (puerto 3001):

```bash
cargo run -p occb-api
curl http://127.0.0.1:3001/api/health
curl http://127.0.0.1:3001/api/dashboard
```

Frontend web (puerto 8080):

```bash
cd crates/app
dx serve --web
# abre http://127.0.0.1:8080
```

Rutas: `/` dashboard · `/people` · `/companies` · `/opportunities` (kanban con ▶ para avanzar etapa) · `/settings` · `/login`.

Build web verificado:

```bash
cd crates/app && dx build --web
```

Tests:

```bash
cargo test --workspace
```

## API

- `GET /api/health`
- `GET/POST /api/companies` · `GET/PATCH/DELETE /api/companies/{id}`
- `GET/POST /api/people` · `GET/DELETE /api/people/{id}`
- `GET/POST /api/opportunities` · `PATCH /api/opportunities/{id}/stage {"stage":"meeting"}`
- `GET /api/dashboard` → `{total_companies, total_people, total_opportunities, pipeline_value, won_value, win_rate}`

## Siguiente (para paridad total)

1. SQLx + `DATABASE_URL=postgres://occb:occb@localhost:5432/occb` (quitar `RwLock<Store>`, usar pool).
2. Auth JWT + multi-workspace (hoy login mock).
3. GraphQL (`async-graphql`) para paridad con twenty-server.
4. Sincronía frontend↔backend con `reqwest` + `use_resource` (hoy frontend usa seed local + API lista).
5. Notes/Tasks/Workflows/AI-agents como nuevos crates.
