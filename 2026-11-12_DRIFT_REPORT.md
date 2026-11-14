# Informe de deriva — contratos EGM (2026-11-12)

## `EGM.md`

- Garantía “Ningún SQL directo desde la app: todo vía funciones” contradicha por `crates/api/src/auth.rs:372` (`select nombre, rol from usuarios where id = $1` directo en `renovar`), sin `app.usuario_id` ni función `obtener_credencial_sesion`. Misma causa que Blocker de `docs/designs/auditoria-occb-crm.md`.

## `crates/README.md`

No drift detected.

## `crates/core/README.md`

No drift detected.

## `crates/core/src/README.md`

No drift detected.

## `crates/api/README.md`

No drift detected.

## `crates/api/src/README.md`

- Referencias de línea desactualizadas: documenta `[main.rs:62]` (escritura colegios protegida), `[main.rs:78]` y `[main.rs:70]` (territorio/panel públicos); en código actual lectura pública está en `main.rs:72-76` (`/api/colegios`, `/api/regiones`, `/api/comunas`, `/api/panel`) y `POST /api/colegios` protegido en `main.rs:170`.
- “PostgreSQL en cada petición vía `transaccion_con_actor`” contradicho por `auth.rs:372` (`select ... from usuarios` directo sin actor) en `renovar`.

## `crates/app/README.md`

No drift detected.

## `crates/app/src/README.md`

No drift detected.

## `crates/app/assets/README.md`

No drift detected.

## `migrations/README.md`

No drift detected.

## `e2e/README.md`

No drift detected.

## `docs/README.md`

No drift detected.

## `docs/manual/README.md`

- `video/recorrido.mp4` documentado como `1280×720 H.264`; `ffprobe` real: `width=1280 height=900 duration=37.5`. Altura 900 vs 720.
- Duración redondeada a `37 s` vs `37.5` real (menor, se deja como nota).

## `docs/designs/README.md`

No drift detected.

## `ROADMAP.md` (no es contrato, se incluye por verdad canónica)

- Declara `E2E 120/120` (`Fase 10`) y `113/113` (`Fase 9`); `e2e/prueba.mjs` contiene 122 `afirma(` verificados y `EGM.md`/`e2e/README.md` declaran 122. Hoja desactualizada frente a código.
