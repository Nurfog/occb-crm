# `docs` Blueprint

## System Overview

Documentación del proyecto OCCB CRM: manual de uso con fotos y video del recorrido (`manual/`) más diseños y RFCs (`designs/`).

## Subdirectories & System Boundaries

- `manual`: [contrato del manual](./manual/README.md) — `MANUAL.md` (14 secciones ES) + `img/` (15 capturas) + `video/recorrido.mp4`.
- `designs`: [contrato de diseños](./designs/README.md) — `SITEMAP.md` + `auditoria-occb-crm.md` (auditoría por crate + transversal, fases 0-4).

## Global Invariants

- La documentación de usuario vive en `manual/`; los contratos EGM de código viven junto a cada crate.

## Global Side Effects

- Ninguno.

## System Verification

- Run: `ls docs/manual/img/*.png docs/manual/video/recorrido.mp4`
