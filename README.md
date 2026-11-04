# OCCB CRM — clon de Twenty en Rust 2024 + Dioxus

Migración de [twentyhq/twenty](https://github.com/twentyhq/twenty) (React + NestJS + PostgreSQL)
a **Rust edition 2024 + Dioxus 0.7 + Axum + PostgreSQL 18**.

Idioma por defecto: **español**, con conmutador **ES/EN** (i18n) en el frontal.

## Estructura

```
occb-crm/
  Cargo.toml              # workspace Rust 2024
  docker-compose.yml      # bd (postgres:18) + redis (redis:8) + frontal (nginx)
  .env.example            # plantilla de variables (muestra)
  .env                    # valores locales (no versionado)
  crates/
    core/                 # dominio en español: dominio, validacion, i18n, semilla
    api/                  # backend Axum :3001 (solo llama a funciones SQL)
    app/                  # frontal Dioxus web (Dockerfile + nginx.conf)
  migrations/             # 001 esquema → 006 funciones SECURITY DEFINER + RLS + siembra
```

### Mapa Twenty → Rust

| Twenty original | OCCB Rust |
|---|---|
| `twenty-front` (React) | `crates/app` (Dioxus 0.7 + Router + señales + i18n ES/EN) |
| `twenty-server` (NestJS) | `crates/api` (Axum; CRUD solo vía funciones PG) |
| PostgreSQL + Redis | `docker-compose.yml` (postgres:18 + redis:8 con healthchecks) |

## Requisitos

- Rust 1.98+, `wasm32-unknown-unknown`, Dioxus CLI 0.7 (`dx`), Docker.

```bash
rustup target add wasm32-unknown-unknown
cp .env.example .env   # solo la primera vez
```

## Arranque rápido (todo en contenedores salvo la API)

```bash
docker compose up -d bd redis        # Postgres 18 + Redis
cargo run -p occb-api                # API :3001 (migra y siembra solo)
docker compose up -d frontal         # frontal :8080 (build WASM + nginx)
```

- Frontal: http://localhost:8080 (/, /personas, /empresas, /oportunidades, /ajustes, /acceso)
- API: http://localhost:3001/api/salud · `/api/panel` · `/api/empresas` · `/api/personas` · `/api/oportunidades`

## API (rutas en español + alias en inglés)

- `GET /api/salud` — estado del servicio
- `GET/POST /api/empresas` · `GET/PATCH/DELETE /api/empresas/{id}`
- `GET/POST /api/personas` · `GET/DELETE /api/personas/{id}`
- `GET/POST /api/oportunidades` · `PATCH /api/oportunidades/{id}/etapa` · `DELETE /api/oportunidades/{id}`
- `GET /api/panel` — resumen calculado en la base de datos

Seguridad: la API jamás toca tablas con SQL directo; solo ejecuta funciones
`SECURITY DEFINER` con validación (`migrations/002–005`) y RLS activado.
Errores unificados en español: `{ "codigo": "...", "mensaje": "..." }`.

## Verificación

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd crates/app && dx build --web
curl http://127.0.0.1:3001/api/panel
curl http://127.0.0.1:8080/ -o /dev/null -w "%{http_code}\n"
```

## Fase 1 hecha · siguiente

Hecho: núcleo CRM en español + i18n, PG 18 en Docker, CRUD por funciones + RLS,
frontal en contenedor. Siguiente: conectar el frontal a la API en vivo,
autenticación, y API también en contenedor.
