# `docs` Blueprint

## System Overview

Documentación del proyecto OCCB CRM: actualmente solo el manual de uso con fotos y video del recorrido.

## Subdirectories & System Boundaries

- `manual`: [contrato del manual](./manual/README.md) — `MANUAL.md` (14 secciones ES) + `img/` (15 capturas) + `video/recorrido.mp4`.

## Global Invariants

- La documentación de usuario vive en `manual/`; los contratos EGM de código viven junto a cada crate.

## Global Side Effects

- Ninguno.

## System Verification

- Run: `ls docs/manual/img/*.png docs/manual/video/recorrido.mp4`
