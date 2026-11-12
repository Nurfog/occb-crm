# `occb-app` Blueprint

## System Overview

Frontal web del CRM escolar (`occb-app`, Dioxus 0.7 compilado a WASM y servido por `nginx` con fallback SPA a `index.html`): disposición común con barra lateral/superior, 19 rutas en español (18 bajo `layout(DisposicionApp)` más `/acceso` pública, además de la captura `/:..segmentos`), y cliente HTTP con renovación automática de sesión. La `URL` base de la API se inyecta en compilación (`API_BASE_URL`, defecto `http://127.0.0.1:3001`); en `docker-compose` el frontal expone `:8080`.

## Subdirectories & System Boundaries

- `src`: [contrato del paquete](./src/README.md) — `main.rs` (rutas, disposición y 20 vistas) y `cliente.rs` (sesión en memoria + recursos HTTP).
- `assets`: [contrato de estáticos](./assets/README.md) — `main.css` global con caché larga en `nginx`.

## Global Invariants

- Sin sesión no hay pantallas protegidas: el guardia redirige a acceso y regresa al destino.
- Textos ES/EN desde `occb-core`; `VistaAcceso` siempre fuera del `layout`.
- Nada se borra de verdad: las vistas llaman a deshabilitación lógica y anonimización admin-only.

## Global Side Effects

- Solo red hacia la API (`reqwest`) y navegación SPA; ningún estado persistente en disco o base local.

## System Verification

- Run: `cargo test -p occb-app`
