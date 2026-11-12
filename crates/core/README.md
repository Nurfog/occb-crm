# `occb-core` Blueprint

## System Overview

Crate de dominio puro del CRM escolar OCCB (`occb-core`): define el vocabulario compartido en español entre PostgreSQL, la API Axum y el frontal Dioxus. Agrupa tipos de dominio (colegios, personas, alumnos, apoderados, admisión, agenda y comunicaciones), validación sin E/S con errores en español, territorio chileno oficial (16 regiones, 346 comunas), textos de interfaz ES/EN y datos de demostración para el modo sin conexión. No toca red, disco ni base de datos.

## Subdirectories & System Boundaries

- `src`: [contrato del paquete](./src/README.md) — tipos (`dominio.rs`), validación pura (`validacion.rs`), territorio CUT (`territorio.rs`), textos ES/EN (`i18n.rs`), datos demo (`semilla.rs`) y re-exportaciones (`lib.rs`).

## Global Invariants

- Todo el dominio habla español por defecto (`nombre`, `correo`, `importe`, `etapa`) con conmutador ES/EN solo en la interfaz (`[src/i18n.rs:textos]`).
- Ningún invariante de negocio vive solo en la API o el frontal: la validación canónica está en `src/validacion.rs` y ambos la reutilizan.
- El territorio es un catálogo cerrado y generado (16 regiones, 346 comunas): no se edita a mano y cualquier pareja región/comuna inválida se rechaza.
- Los datos de demostración nunca sustituyen al directorio oficial de PostgreSQL.

## Global Side Effects

- Ninguno: `occb-core` no abre conexiones, no lee ni escribe ficheros y no genera tráfico. El único no-determinismo está aislado en `datos_semilla_*` (`Uuid::new_v4()`, `Utc::now()`).

## System Verification

- Run: `cargo test -p occb-core`
