# OCCB CRM

CRM en **Rust edition 2024**: frontal **Dioxus 0.7**, API **Axum** y **PostgreSQL 18**.

Idioma por defecto: **español**, con conmutador **ES/EN** en el frontal.

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
  migrations/             # esquema, funciones seguras + RLS y datos de ejemplo
```

## Requisitos

- Rust 1.98+, `wasm32-unknown-unknown`, Dioxus CLI 0.7 (`dx`), Docker.

```bash
rustup target add wasm32-unknown-unknown
cp .env.example .env   # solo la primera vez
```

## Arranque rápido

```bash
docker compose up -d bd redis        # PostgreSQL 18 + Redis
cargo run -p occb-api                # API :3001 (aplica migraciones y siembra)
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
`SECURITY DEFINER` con validación (`migrations/`) y RLS activado.
Errores unificados en español: `{ "codigo": "...", "mensaje": "..." }`.

## Verificación

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd crates/app && dx build --web
curl http://127.0.0.1:3001/api/panel
curl http://127.0.0.1:8080/ -o /dev/null -w "%{http_code}\n"
```
