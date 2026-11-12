# OCCB CRM Blueprint

## System Overview

CRM escolar en Rust 2024 (español por defecto, conmutador ES/EN): frontal Dioxus 0.7 → WASM servido por `nginx` (`:8080`), API Axum (`:3001`) que solo ejecuta funciones PostgreSQL 18 `SECURITY DEFINER` con RLS, y 28 migraciones `sqlx` con semillas (territorio SUBDERE, 12 038 colegios MINEDUC). Suite E2E en Chrome headless (122 afirmaciones, filas dedicadas) y manual de uso con fotos y video. Orquestación local vía `docker-compose` (`bd` + `redis` + `api` + `frontal`).

## Local Interface & Invariants

- Workspace (`Cargo.toml`): miembros `crates/core`, `crates/api`, `crates/app`; dependencias compartidas (`axum`, `dioxus`, `sqlx`, `argon2`, `jsonwebtoken`, `validator`).
- Servicios (`docker-compose.yml`): `bd` (postgres:18, `:5432`, volumen `datos_pg`, healthcheck `pg_isready`), `redis` (redis:8, `:6379`), `api` (`occb-api:local`, `:3001`, `APP_SECRETO` sobreescribible), `frontal` (`occb-frontal:local`, `:8080→80`, healthcheck `wget`); la API espera `bd` sano.
- Arranque: `docker compose up -d --build`; o `docker compose up -d bd redis` + `cargo run -p occb-api` + `dx serve` en `crates/app`.
- Invariantes: JWT acceso 15 min + renovación opaca rotativa; registro abierto por diseño; RUT chileno obligatorio en alumnos/apoderados; etapas de admisión `contacto→…→matriculado`; nada se borra (deshabilitación + anonimización admin-only); E2E verde antes de cerrar cada fase.

## Subdirectories & System Boundaries

- `crates/core`: [Blueprint](./crates/core/README.md) — dominio puro, validación, territorio CUT, i18n ES/EN, semilla demo ([paquete](./crates/core/src/README.md)).
- `crates/api`: [Blueprint](./crates/api/README.md) — API Axum + JWT/claves/webhooks/importación ([paquete](./crates/api/src/README.md)).
- `crates/app`: [Blueprint](./crates/app/README.md) — frontal Dioxus, sesión en memoria ([paquete](./crates/app/src/README.md), [estáticos](./crates/app/assets/README.md)).
- `migrations`: [Blueprint](./migrations/README.md) — 28 migraciones, funciones seguras + RLS + semillas.
- `e2e`: [Blueprint](./e2e/README.md) — suite Chrome headless, 122 afirmaciones.
- `docs`: [Blueprint](./docs/README.md) — manual de uso ([manual](./docs/manual/README.md), medios excluidos por `.egmignore`).

## Global Invariants

- Español en código, BD, API y errores (`{codigo, mensaje}`); vocabulario compartido (`nombre`, `correo`, `importe`, `etapa`).
- Ningún SQL directo desde la app: todo vía funciones con validación espejo.
- `.egmignore` + `.gitignore` definen lo excluido del mapa (builds, secretos, medios, informes de deriva).

## Global Side Effects

- Estado mutable solo en PostgreSQL (volúmenes `datos_pg`, `datos_redis`); red local entre contenedores más salidas firmadas (webhooks HMAC, proveedor LLM).

## System Verification

- Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings && node e2e/prueba.mjs`
