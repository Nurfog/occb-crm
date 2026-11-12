# `assets` — Estáticos del frontal

## Overview

Ficheros estáticos que Dioxus incrusta con `asset!` y `nginx` sirve con caché larga (`location /assets/`, `expires 30d`): la hoja de estilos global de la aplicación.

## Interface

- `main.css`: estilos globales enlazados desde `Aplicacion` vía `document::Link` (`[main.rs:CSS]`).

## Invariants

- Un solo punto de estilo: los componentes no llevan CSS en línea salvo clases de `main.css`.
- `watch_path = ["src", "assets"]` en `Dioxus.toml`: tocar `assets` recarga el frontal en desarrollo.

## Side Effects

- Ninguno en tiempo de ejecución (solo lectura de estáticos; la caché larga exige renombrar si el contenido cambia de forma incompatible).

## Verification

- Run: `cargo test -p occb-app`
