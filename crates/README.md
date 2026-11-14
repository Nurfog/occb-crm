# `crates`

## Overview

Agrupador de crates Rust del CRM escolar OCCB: `core` (dominio puro), `api` (Axum + PostgreSQL 18) y `app` (Dioxus 0.7 + nginx). No contiene código directo; todo el código vive en los subdirectorios.

## Subdirectories & Boundaries

- `core`: [Blueprint](./core/README.md) — dominio puro, validación, territorio CUT, i18n ES/EN, semilla demo ([paquete](./core/src/README.md)).
- `api`: [Blueprint](./api/README.md) — API Axum + JWT/claves/webhooks/importación ([paquete](./api/src/README.md)).
- `app`: [Blueprint](./app/README.md) — frontal Dioxus, sesión en memoria ([paquete](./app/src/README.md), [estáticos](./app/assets/README.md)).

## Verification

- Run: `cargo test --workspace`
