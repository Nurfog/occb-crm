# `occb-api` Blueprint

## System Overview

Binario de la API HTTP OCCB (`occb-api` sobre Axum + PostgreSQL 18): separa rutas públicas mínimas (salud, acceso, regiones/comunas, lectura de colegios y panel) de rutas protegidas con JWT o clave API. Todo el CRUD pasa por funciones `SECURITY DEFINER` con RLS, validación espejo de `occb-core` en español y trazabilidad por `app.usuario_id`. Incluye admisión, escuela (alumnos/apoderados/RUT/ingresos), agenda, correos, búsqueda/favoritos/vistas, automatizaciones con reintentos aislados, asistente con herramientas, claves API, webhooks HMAC e importación CSV.

## Subdirectories & System Boundaries

- `src`: [contrato del paquete](./src/README.md) — `main.rs` (enrutado público/protegido), `auth.rs` (JWT + rotación + claves), `bd.rs` (pool, migraciones, `fila_a_*`), `errores.rs`, `configuracion.rs` y `rutas_*` (colegios, territorio, personas, alumnos, apoderados, ingresos, oportunidades, tareas, notas, eventos, correos, espacio, automatizaciones, agentes, integraciones, importación).

## Global Invariants

- Nada se borra de verdad: deshabilitación lógica (`activo`) + anonimización a valores por defecto solo admin, con UUID estables para informes.
- La escritura exige sesión; la lectura del directorio y del panel es pública.
- La validación canónica vive en `occb-core`; la API la reaplica antes de cada escritura y mapea errores PG a JSON en español.

## Global Side Effects

- PostgreSQL como único estado mutable (lecturas/escrituras por `sqlx`, migraciones al arrancar).
- Sesiones en cookie `HttpOnly`; red saliente solo hacia webhooks firmados y hacia el proveedor LLM del asistente.

## System Verification

- Run: `cargo test -p occb-api`
