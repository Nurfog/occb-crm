# `designs` — Diseños y RFCs OCCB CRM

## Overview

Índice de documentos de diseño del CRM escolar: mapa del sitio y auditoría por crate con fases de remediación. Solo Markdown, sin código fuente.

## Interface

- `SITEMAP.md`: índice de RFCs con enlace a `./auditoria-occb-crm.md` (`[SITEMAP.md:1]`).
- `auditoria-occb-crm.md`: auditoría 2026-11-12 por crate (`core`, `api`, `app`, `migrations`, `e2e`+`docs`) + bloque transversal, severidades `Blocker`/`Major`/`Nit`, fases 0-4.

## Invariants

- Sin extensiones fuente reconocidas: clasificado como Intermediate Blueprint (Hay).
- Enlaces solo relativos entre `.md` existentes; `*_DRIFT_REPORT.md` excluido por `.egmignore` y no se enlaza.

## Side Effects

- Ninguno (documentación estática).

## Verification

- Run: `ls *.md`
