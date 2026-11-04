# OCCB CRM

CRM en **Rust edition 2024**: frontal **Dioxus 0.7**, API **Axum** y **PostgreSQL 18**.
Todo en contenedores y funcional de punta a punta.

Idioma por defecto: **español**, con conmutador **ES/EN** en el frontal.

## Estructura

```
occb-crm/
  Cargo.toml              # workspace Rust 2024
  docker-compose.yml      # bd + redis + api + frontal
  .env.example            # plantilla de variables (muestra)
  .env                    # valores locales (no versionado)
  crates/
    core/                 # dominio en español: dominio, validacion, i18n, semilla
    api/                  # backend Axum (solo llama a funciones SQL) + Dockerfile
    app/                  # frontal Dioxus web (cliente API + Dockerfile + nginx)
  migrations/             # esquema, funciones seguras + RLS y datos de ejemplo
```

## Arranque rápido (todo funcional)

```bash
cp .env.example .env   # solo la primera vez
docker compose up -d --build   # bd + redis + api (:3001) + frontal (:8080)
```

- Frontal: http://localhost:8080 (/, /personas, /empresas, /oportunidades, /ajustes, /acceso)
- API: http://localhost:3001/api/salud · `/api/panel` · `/api/empresas` · `/api/personas` · `/api/oportunidades`

El frontal lee y escribe en la API (crear/eliminar empresas y personas,
crear oportunidades, avanzar etapas en el kanban). Si la API no responde,
muestra datos de demostración locales.

## Desarrollo sin Docker

```bash
docker compose up -d bd redis
cargo run -p occb-api          # :3001 (aplica migraciones y siembra)
cd crates/app && dx serve --web  # :8080 con recarga
```

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
node e2e/prueba.mjs   # E2E en Chrome headless (frontal + API arriba)
```
