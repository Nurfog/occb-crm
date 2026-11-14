# OCCB Twenty-Style SaaS with Holdings and Modules

* **Author:** Nurfog
* **Status:** Draft
* **Date:** 2026-11-12

--------------------------------------------------------------------------------

## Overview

OCCB moves from single-school admission CRM to school-core SaaS with admission as an optional module. Small covers one school, Group covers two to five with combined totals, Holding covers six or more with custom price. Prospect data fills alumno, apoderados, and salud at matriculation; without admission the school loads CSV into core. Non-matriculated stays as marketing pool with consent and expiry. Holdings see aggregated ingresos, resultados, and metas plus individual schools by grant; schools see only own contexto with dominio, accesos, and web link and no cross-compare. Interface follows Twenty patterns for views, pipelines, command menu, search, favorites, filters, saved views, workflows, and import. Spanish stays default with ES and EN toggle. Deletes stay soft with admin-only anonymize. The 122 E2E checks stay green.

## The Problem

New schools need isolated contexto while holdings need combined totals, but the codebase has no tenant scope. JWT carries no holding or school claim, listar, panel, buscar, and timeline read global state, and RLS has no policies behind SECURITY DEFINER functions. The 12k MINEDUC directory shares the colegios table with tenants, so licencia, dominio, contacto, and representantes would pollute directory queries. Oportunidad has no RUT, apoderados, or salud, and no atomic matriculation function exists, so promotion risks partial writes and duplicate RUT errors. RUT uniqueness is global where activo, prospect has no RUT field, and the sentinel passes validation, so the same RUT across two schools collides or leaks. Non-matriculated retention with auto-filled salud and no origen, consent, or TTL conflicts with soft-delete law and data rules.

Proposed solution: tenant foundation first with contracted schools plus memberships plus scoped reads, CSV as quarantined inbox leading to contacto postulacion, single idempotent matriculation transaction reusing the forward arrow, and marketing pool with origen prospecto plus consent plus TTL. Twenty chrome stays minimal until tenancy is green.

## Goals

* Contracted schools live separate from the 12k directory with RBD foreign key, licencia, dominio, and flags plus user memberships; directory stays read-only for traslados.
* All reads take scope with holding totals plus drill-down and school own-only enforced in SQL; panel, buscar, and listar never rely on frontend filtering.
* CSV acts as quarantined inbox with RUT check plus RBD resolve plus consent flag leading to contacto postulacion, not direct loader; fixed MINEDUC and RUT template first.
* Single matricular postulacion transaction is idempotent and reuses the forward arrow; prospecto_rut is nullable validated not unique with origen, consentimiento, and TTL; salud stages only at secretaria confirmation as draft.
* Non-matriculated stays activo with origen prospecto plus consent plus TTL and 12 to 24 month auto-anonymize; no salud retention for non-matriculated; suppression list blocks re-contact.
* Every new label ships in ES plus EN; Deshabilitar plus Anonimizar confirm preserved; 122 E2E plus cargo test plus clippy with deny warnings stay green with no stacking on red.

## Non-Goals

* No custom dominio hosting, no pagina-web builder or CMS, no custom permission matrix.
* No ISP discount engine or billing metering, no unified BI or forecasting.
* No Flow canvas, no drag kanban or calendar, no week or day views, no threading.
* No column configurator or sharing, no merge engine, no cross-colegio compare for colegio role.
* No backup-scheduling UI, no new dep, no main.rs split in this design beyond per-view pure fns.

## Open Questions

* **Question 1**: RUT scope — global with traslado and merge versus RBD-scoped with quarantine view — needs author sign-off before migration 029.
* **Question 2**: Holding metric formulas — ingresos, resultados, metas as sum versus average with currency, academic period, etapa homologation, importe versus real arancel — needs author sign-off before panel change.
* **Question 3**: Value metric for licensing — per alumno, per matricula, per usuario — needs author sign-off; Small, Group, Holding display names already fixed.

## Critical User Journeys (CUJs)

### CUJ 1: Secretaria matriculates prospect without partial write

* **Given**: Postulacion in Evaluado with prospecto_rut valid, apoderados RUTs valid, target colegio_rbd contracted and in user scope.
* **When**: Secretaria confirms matricula draft with salud.
* **Then**: Single transaction creates alumno plus vinculos plus salud draft, moves etapa to Matriculado, preserves traslados and actividad history, returns alumno id; duplicate submit is idempotent; E2E asserts PG row plus UI ficha.

### CUJ 2: Holding sees totals, colegio sees own only

* **Given**: Holding with schools A, B, grants for role; colegio user scoped to A only.
* **When**: Holding opens panel with scope holding, colegio opens panel with scope A.
* **Then**: Holding sees combined postulantes per etapa plus drill to A or B; colegio sees A only with no neighbor names or values; all scope checks run in SQL with JWT claims.

### CUJ 3: CSV without admission feeds quarantine, not core

* **Given**: School without admission module uploads CSV with rut, nombres, apellidos plus consent flag.
* **When**: Import runs with RBD resolve and RUT check.
* **Then**: Rows land as contacto postulacion in quarantine with origen csv, per-row errors reported, valid rows create prospects; no direct alumno insert; sunset after migration window.

## Technical Plan (Architecture)

Tenancy is a database and claim change first, Twenty chrome last. No frontend scope switcher ships until scoped reads are green.

```text
[JWT sesion+holding_id+colegio_rbd[]] --> [exigir_sesion + es_miembro] --> [transaccion_con_actor + app.holding_id]
[colegios_contratados(RBD FK) + membresias_usuario] <-- [029 tenant] --> [listar_*/panel/buscar(p_scope)]
[CSV inbox + prospecto_rut/origen/consentimiento/TTL] --> [matricular_postulacion tx idempotente] --> [alumnos + vinculos + salud draft]
[scope=A] --SQL enforce--> [colegio A only] | [scope=holding] --SQL enforce--> [totals + drill A/B on grant]
[main.rs scope picker + chip Vista + Path/highlights] <-- derived from scoped props only
```

Components:

* `029 tenant fundacion`: new tables `colegios_contratados(rbd FK, licencia Small/Group/Holding, dominio, flags)`, `membresias_usuario(usuario_id, holding_id, colegio_rbd)`; JWT adds `holding_id` plus `colegio_rbd[]`; `transaccion_con_actor` sets `app.holding_id`; rewrite `listar_*`, `panel_resumen(p_scope)`, `buscar_espacio(p_scope)` with scope param; directory `colegios` stays read-only for traslados.
* `030 matricula y consentimiento`: add `prospecto_rut nullable validated not unique`, `origen`, `consentimiento`, `ttl` to oportunidad; add `matricular_postulacion(oportunidad_id, alumno_rut, apoderados_rut[], salud?)` atomic idempotent; health staged only at secretaria confirmation; `anonimizar` keeps history with contact scrubbed; `supresion` list blocks re-contact; auto-anonymize job 12 to 24 months for non-matriculated, no salud retention.
* `CSV inbox`: extend `importar_alumnos/apoderados` to quarantine with RUT check plus RBD resolve plus consent flag, fixed MINEDUC template first, per-row report with `job_id` history; no mapper until heterogeneous schemas proven.
* `API scope`: `exigir_sesion` checks `es_miembro(holding_id, colegio_rbd)`; `transaccion_con_actor` binds actor plus scope; `panel_resumen`, `buscar_espacio`, scoped `listar_*` take `p_scope`; holding totals plus drill on grant, colegio own-only; benchmark as anonymized percentiles only.
* `Frontend Twenty minimal`: scope picker plus `ChipVistaActiva` with clear reusing `SelectorVistas`; Path strip plus highlights plus urgency dot plus task grouping plus UTC suffix plus rule sentence plus static chat limit hint from existing props only; preserve all E2E classes and text with wrapping divs only.
* `i18n.rs`: paired keys for Small, Group, Holding, holding, sede, licencia, meta, dominio, ingresos, scope, consent, TTL, buckets, UTC, Cuando, Si, Entonces; both arms required; build fails if either arm missing.

Data flow stays as today with scope added: Dioxus signals fetch once through scoped `cliente::*` with scope param, render derives strings, mutator calls same scoped endpoint, signal reloads. No new store, no polling, no new dep. Migration uses add nullable plus batched backfill plus index concurrently, never single transaction over 12k; traslados audit reused for moves; favoritos, timeline, actividad joins carry scope.

Failure modes closed by construction: unknown scope rejects with 403; unscoped call never falls back to global; RBD without contract hides from licensed lists but stays in directory for traslados; invalid RUT rejects at validation with Spanish mensaje; duplicate submit returns existing alumno; missing consent blocks marketing retention; E2E text preserved by wrapping outside asserted substrings.

## Alternatives Considered

### Alternative A: Keep CRM standalone plus handoff

* **Pros**: Least scope; preserves 122 checks; avoids SIS absorb; CSV as 90-day sunset tool with RUT quarantine.
* **Cons**: Holding asks for combined view remain unmet; dual source for directory versus tenant persists.
* **Reason for Rejection**: Rejected by author as sole outcome because holding needs combined totals; kept as fallback if tenancy turns red.

### Alternative B: Invert with CRM as core

* **Pros**: Kills dual source with one RUT namespace and one audit; asistencia, notas, arancel subscribe to matriculado.
* **Cons**: Schools wanting only notas pay for CRM; schoolccb admin becomes adapter; forces SIS absorb.
* **Reason for Rejection**: Rejected by author: cost exceeds tenancy benefit for schools without admission; adapter risk outweighs single-namespace gain.

### Alternative C: Pool as consent asset with hashed routing

* **Pros**: Monetizes non-convertidos with hashed RUT plus comuna and nivel for intra-holding demand routing plus apoderado-controlled re-contact.
* **Cons**: Hardest privacy lift needing consent ledger plus settlement; anonymized pool loses marketing value if over-scrubbed.
* **Reason for Rejection**: Rejected by author: privacy and settlement cost outweigh routing gain for MVP; revisit after consent ledger exists.

### Alternative D: Per-matriculado pricing with billable ledger

* **Pros**: Zero base plus price per effective matriculado with mover_etapa to matriculado as billable ledger; CSV free as funnel; fits marzo cashflow.
* **Cons**: Anti-gaming for reversos and traslados double-count, cobranza overhead, variable-cost fear, immutable ledger conflicts with soft-delete reuse.
* **Reason for Rejection**: Rejected by author: gaming and collection risk plus ledger versus soft-delete conflict outweigh pricing simplicity.

--------------------------------------------------------------------------------

## Non-Functional Requirements

* Security and privacy: scope enforced in SQL with JWT holding_id plus colegio_rbd claims; holding sees totals plus granted schools, colegio sees own; no named neighbor leak; consent plus TTL plus suppression enforced; history survives anonymize with contact scrubbed; secrets stay in env with fail-closed prod.
* Performance: scoped server params with batched backfill plus index concurrently; no client fetch-then-filter for holding rollup; WASM delta limited to scope picker plus chips plus Path and highlights; no new dep.
* Rollout and rollback: M0 freeze plus split with no behavior change, M1 tenant plus licencia read-only, M2 CSV inbox per tenant, M3 matriculation plus retention; each gated on cargo test plus clippy plus 122 E2E with no stacking on red; migrations forward only with restore from datos_pg on red.

--------------------------------------------------------------------------------

### Additional Resources

* `concepts_brief.md` Priority 1 intake (Nurfog 2026-11-12, tenancy-first)
* `EGM.md` root blueprint plus `migrations/README.md` 28 plus tenant 029-030
* `e2e/prueba.mjs` 122 checks plus scope and matriculation fixtures
* `/tmp/opencode/schoolccb/frontend/src/pages/ManagedSchoolsView.jsx` plus `SchoolDetailView.jsx` admin reference (CRM ignored)

### Future Improvements

* AND/OR builder with guided chips first; mapper only if heterogeneous schemas proven; workflows expansion beyond postulaciones triggers; custom domains; permission matrix beyond hacer_admin and quitar_admin; unified BI.

--------------------------------------------------------------------------------

## Detailed Implementation Plan

### Target Files to Create/Modify

#### 1. `migrations/029_tenant_fundacion.sql` (Create)

* **Type / Scope**: PostgreSQL DDL plus SECURITY DEFINER helpers, RLS scope base.
* **Proposed Changes**: Create colegios_contratados with RBD foreign key to colegios, licencia Small, Group, Holding, dominio, flags; create membresias_usuario with usuario_id, holding_id, colegio_rbd; add scope param pattern for listar, panel, buscar; directory colegios stays read-only; use add nullable plus batched backfill plus index concurrently.

#### 2. `migrations/030_matricula_consentimiento.sql` (Create)

* **Type / Scope**: PostgreSQL DDL plus atomic function plus retention.
* **Proposed Changes**: Add prospecto_rut nullable validated not unique plus origen plus consentimiento plus ttl to oportunidad; add supresion list; add matricular_postulacion with idempotent alumno plus vinculos plus salud draft plus etapa Matriculado; health staged only at secretaria confirmation; auto-anonymize 12 to 24 months for non-matriculated with no salud retention.

#### 3. `crates/api/src/auth.rs` (Modify)

* **Type / Scope**: Axum auth plus JWT claims plus scope guard.
* **Proposed Changes**: Extend claims with holding_id plus colegio_rbd list; add es_miembro check; enforce exigir_sesion scope on licensed routes; fail-closed prod secret; keep 15 min acceso plus 30d rotativa plus HttpOnly SameSite Lax.

#### 4. `crates/api/src/bd.rs` (Modify)

* **Type / Scope**: SQLx pool plus scoped transaction.
* **Proposed Changes**: Extend transaccion_con_actor to set app.holding_id plus app.usuario_id; keep max 10 connections plus migrar_y_sembrar applying 001 to 030; no direct table writes outside functions.

#### 5. `crates/api/src/rutas_matricula.rs` (Create) + `crates/api/src/main.rs` (Modify)

* **Type / Scope**: Axum handlers plus route registration.
* **Proposed Changes**: New matricular endpoint reusing forward arrow semantics with DB-string etapa; register mod plus POST route with exigir_sesion plus es_miembro; no new state values beyond Matriculado terminal.

#### 6. `crates/api/src/rutas_oportunidades.rs`, `crates/api/src/rutas_espacio.rs` (Modify)

* **Type / Scope**: Scoped reads for panel and search.
* **Proposed Changes**: Add p_scope to listar, panel_resumen, buscar_espacio; holding totals plus drill on grant, colegio own-only; benchmark as anonymized percentiles only if demanded.

#### 7. `crates/api/src/rutas_importacion.rs` (Modify)

* **Type / Scope**: CSV quarantine inbox.
* **Proposed Changes**: Extend importar to RUT check plus RBD resolve plus consent flag with fixed MINEDUC template, per-row report with job_id history, preview five, max 5 MB and 1000 rows; rows create contacto postulacion with origen csv, never direct alumno.

#### 8. `crates/core/src/i18n.rs` (Modify)

* **Type / Scope**: Shared ES and EN strings.
* **Proposed Changes**: Add paired Textos fields for Small, Group, Holding, holding, sede, licencia, meta, dominio, ingresos, scope, consent, TTL, buckets, UTC, Cuando, Si, Entonces; both Idioma arms required; build fails if either arm missing.

#### 9. `crates/app/src/main.rs`, `crates/app/src/cliente.rs` (Modify)

* **Type / Scope**: Dioxus frontend plus scoped client.
* **Proposed Changes**: Add scope picker plus ChipVistaActiva with clear reusing SelectorVistas; Path strip plus highlights plus urgency dot plus task grouping plus UTC suffix plus rule sentence plus static chat limit hint from scoped props only; scoped cliente fns with p_scope reusing existing mutators; preserve E2E classes and text with wrapping divs only.

#### 10. `crates/app/assets/main.css` (Modify)

* **Type / Scope**: Global stylesheet with long cache in nginx.
* **Proposed Changes**: Add scope chip, path chevrons, highlights sticky, urgency dot, group header, chip pill; preserve all E2E-pinned selectors; no inline styles.

#### 11. `e2e/prueba.mjs` (Modify)

* **Type / Scope**: Headless Chrome suite, 122 must stay green.
* **Proposed Changes**: Add scope fixtures (holding totals versus colegio own-only), matriculation idempotent submit, CSV quarantine per-row report, consent TTL suppression; extend without renaming asserted copy or selectors.

### Verification Plan

* **Compilation/Build**: `cargo build -p occb-api` plus `dx build --web` local-optional in `crates/app`.
* **Unit Tests**: `cargo test --workspace` requires zero failures (core 17 plus api plus app suites).
* **Static Analysis & Linting**: `cargo clippy --workspace -- -D warnings`.
* **Test Coverage Requirement**: WASM delta under 5 KB, Cargo.toml unchanged for SaaS base, scoped SQL has EXPLAIN with index use, no client fetch-then-filter for rollup.
* **E2E/Integration**: `node e2e/prueba.mjs` with FRONTAL http://127.0.0.1:8080 API http://127.0.0.1:3001 Chrome headless; 122/122 green plus new scope, matriculation, CSV quarantine asserts; grep asserted selectors and copy before merge; S1, S2, S3 each gated with no stacking on red.

--------------------------------------------------------------------------------

*Drafted with EGM*
