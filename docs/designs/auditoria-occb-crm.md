# Auditoría OCCB CRM (seguridad, prácticas, bugs, mejoras)

* **Author:** Nurfog
* **Date:** 2026-11-12

## Overview

Auditar todo el repo OCCB CRM para saber si lo construido tiene fallos, con intención de corregirlos después. El informe se organiza por crate más un bloque transversal (lo que cruza crates no se duplica: se referencia desde cada crate). Severidades `Blocker`/`Major`/`Nit`; cada fase cierra con `cargo test --workspace`, `clippy -- -D warnings` y E2E verde. Revisaron 7 lentes Goldfish en paralelo sin historial compartido.

## Analysis

### `crates/core` — mayormente sano

Sin Blockers verificados: validación pura sin E/S con 17/17 tests, RUT módulo 11 con normalización, territorio generado (16/346, no editar), `Textos` que obliga traducir ES/EN, `calcular_resumen` con tubería/ganado bien definidos. Nits: documentar que `semilla.rs` es solo demo (`@test.cl`, RUT de ejemplo) y que el directorio oficial vive en PG; nada que frene remediación.

### `crates/api` — foco principal

- Blocker: secreto JWT con fallback de desarrollo + `APP_SECRETO:-secreto-e2e-local` commiteado (JWT forjables); arranque fail-closed en `prod` y no devolver `renovacion` en JSON.
- Blocker: registro abierto sin límite (solo `login` pasa por `login_limitado`); cerrarlo o invitar + telemetría anti-abuso (`tower_governor` + `redis` ya desplegado).
- Blocker: `exigir_sesion` confía en `rol` del JWT sin revalidar; `salir` no verifica dueño; degradación tarda ≤15 min en propagar. Revalidar en middleware o caché Redis <60 s, denylist de `jti`.
- Blocker: `renovar()` hace `select` directo a `usuarios` (contradice "ningún SQL directo") sin `app.usuario_id` → crear `obtener_credencial_sesion(uuid)` en PG.
- Major: rate-limit local por proceso (60/min, `Mutex`, sin `Retry-After`); migrar a `redis` con bucket por IP+correo y backoff.
- Major: reuso de renovación solo devuelve 404 sin `error!` ni métrica; distinguir `sesion_revocada` de `no_encontrado`.
- Major: grants `EXECUTE to public` en `anonimizar_*` mientras la garantía admin-only vive solo en Axum; exigir `es_admin()` en PG + tabla de auditoría.
- Major: `GET /api/salud` no mira `redis`/pool/cola; worker webhooks único en memoria (doble entrega con 2 réplicas, sin DLQ); importación CSV sin auditoría de actor.
- Major: asistente sin presupuesto de tokens ni scrub de PII/salud antes del LLM.
- Nit: alias (`/api/schools`, `/api/dashboard`, `/api/health`) duplican superficie; catálogo `{codigo,mensaje}` inconsistente con `sin_sesion/prohibido`.

### `crates/app` — monolito y sesión

- Blocker: `main.rs` de 5134 líneas mezcla rutas, 20 vistas, fechas UTC, selección y estado global (`IDIOMA`, `VOLVER`, `SESION`, `FAVORITOS`, `NO_LEIDOS`); partir por dominio espejo de `rutas_*` antes de ampliar seguridad.
- Blocker: `renovacion` y `sesion_id` en JSON a WASM (accesible a XSS) pese a cookie `HttpOnly`; renovación solo-cookie + anti-CSRF, frontal sin almacenar secreto.
- Major: dos 401 paralelos disparan dos renovaciones y se auto-revocan; coordinador único de renovación en vuelo + ventana de gracia en PG.
- Major: `reqwest::Client` nuevo por intento, queries concatenadas sin encode (`San José` → 400), fichas que descargan 100 y filtran en cliente; cliente único, `Query` codificada, paginación servidor.
- Nit: sesión solo en memoria (recargar = re-entrar) es correcto pero debe quedar explícito como invariante auditable.

### `migrations/` — plano pero sólido

- Major: 28 ficheros planos con 6 `corrige_*`; imponer índice + prohibir `corrige_*` sin carpeta de dominio (solo convención, sin mover historial aplicado).
- Blocker (dato): anonimización a RUT comodín compartido re-identificable por correlación (UUID/historial intactos); valores aleatorios únicos + borrado en cascada o hash irreversible.
- Major: `UNIQUE WHERE activo` y `fin>=inicio`, RUT, grupos sanguíneos y etapas cerradas están bien; cualquier endurecimiento exige backfill (filas legacy lo rechazarían).
- Regla dura: jamás editar aplicadas; todo hacia adelante con E2E verde como puerta; rollback por restauración de `datos_pg`, nunca downgrade.

### `e2e/` + `docs/` — cobertura y verdad canónica

- Major: monolito de 1417 líneas / 122 afirmaciones, orden-dependiente y contra PG compartida; particionar por dominio, BD efímera en CI, cleanup de `SELLO` huérfano.
- Major: dos cabezas de verdad (`README.md` raíz obsoleto con `/api/empresas` vs `EGM.md` vs `ROADMAP.md` 120/120 frente a 122 reales); declarar `EGM.md` única raíz, `README.md` a puntero + arranque.
- Nit: 7 informes de deriva sin agregador; mantener solo el de raíz con enlaces.

### Transversal (no pertenece a un crate)

Compose no reproducible (imágenes flotantes, `occb/occb`, puertos al host, `redis` sin clave ni consumidor, volumen PG no estándar, sin `restart`/límites/respaldo); secretos (`APP_SECRETO`, `DATABASE_URL`, `LLM_CLAVE`) con defaults commiteados; CORS permisivo por split-origin `:8080/:3001`; glosario (RLS, `SECURITY DEFINER`, `jti`, RUT/RBD/CUT, etapas) y taxonomía pendientes de fijar; E2E 122 como línea base Fase 0.

## Goals vs. Non-Goals

* **In-Scope Goals:** por crate lo listado arriba; transversal una sola vez y referenciado; glosario mínimo; fases con puertas; migraciones solo hacia adelante.
* **Out-of-Scope Non-Goals:** carga, accesibilidad, inglés, reescribir el manual, `down -v` en limpia, correcciones de código (siguiente paso, no este brief).

## Alternatives Weighed

* Todo a la vez sin priorizar — rechazado: verticales por explotabilidad (auth → funciones/RLS → PII → integraciones → frontal/E2E).
* Un informe mezclado — rechazado: seguridad explotable primero; resto como reglas de fase.
* Taxonomía propia — rechazado: `Blocker/Major/Nit` + puertas existentes.
* Re-auditar migraciones completas — rechazado: estado final + `025` + `028`.

## Key Decisions & Staging for Next Steps

* Fase 0 (read-only, transversal): taxonomía, glosario, alcance cerrado por crate, E2E 122 verde base; excluir o eliminar superficie muerta.
* Fase 1 (transversal + `api`): secretos fail-closed, cookie estricta, CORS allowlist, compose reproducible + backup probado.
* Fase 2 (`api`): registro, revalidación de rol, propiedad de sesión, rate-limit en `redis`, `jti` denylist, compatibilidad de rutas endurecidas.
* Fase 3 (`migrations` + `api`): credencial en PG, `es_admin()` en funciones, grants mínimos, auditoría de anonimización, backfills.
* Fase 4 (`app` + `e2e`): partición del frontal, cliente único, paginación, métricas webhooks/observabilidad, E2E por dominio.
* Cierre de fase: `cargo test --workspace`, `clippy -- -D warnings`, `node e2e/prueba.mjs` en verde; rollback por restauración.
