# Stage 5 Initial Zoom Out — `docs/designs/saas-tenancy-matriculacion.md` (L1-205) — Consolidated

> Mode: REVIEW. Standard: `/home/juan/dev/egm/resources/document_standards/zoom-out-standard.md` — **inaccessible** (`permission.rejected: external_directory` in both reviewers). Fallback applied: altitude / scope / invariants (tenancy-first, school-first, consent+RUT, visibility server-side, Twenty minimal, 122 E2E green) + GoldfishContext verified (EGM.md, concepts_brief.md, Cargo.toml, auth.rs, bd.rs, main.rs, dominio.rs, i18n.rs, 010_colegios_admision.sql, 028_borrado_logico.sql, prueba.mjs). No files written.

## Verdict: CONDITIONAL PASS — 4 Blockers + 8 Majors must close before Stage 6

Direction is sound (tenancy-first staging, contracted-vs-directory split, quarantined CSV inbox, atomic idempotent matriculation, consent+TTL+suppression, server-side scope, Twenty-minimal intent, 122-green gates). Not shovel-ready: scope-enforcement, holdings entity, RUT-scope, and money-vs-counts undecided at point 029/030 + panel change scheduled.

## Blockers

### B1 — Public panel leak not closed (visibility server-side)
Target 76,79,117,153 vs main.rs:63-76, auth.rs:34-43,150-212, bd.rs:18-28. panel_resumen(p_scope) plus holding totals scheduled while main.rs:75-76 keeps /api/panel + /api/dashboard on publicas; Reclamos has no holding_id/colegio_rbd[], exigir_sesion has no scope check; transaccion_con_actor only sets app.usuario_id. Design never moves panel to protegidas nor splits public colegios (12k directory) vs licensed colegios_contratados. Fix: panel/dashboard to protegidas + exigir_sesion+es_miembro+p_scope; listar_colegios directory stays public read-only; contracted lists require scope; 403 on unknown scope, no global fallback.

### B2 — holdings entity missing — holding_id dangles
Target 68,150-154,158 vs concepts_brief.md:24, 010:4-22. colegios_contratados + membresias_usuario + JWT holding_id+colegio_rbd[] + app.holding_id, but no holdings table, no lifecycle (Group 2-5 vs Holding 6+ enforcement). Without FK target es_miembro unimplementable. Fix: add holdings + colegios_contratados(holding_id FK, rbd FK, licencia, dominio, flags) + membresias or explicitly defer billing to manual licencia flag-only.

### B3 — RUT scope undecided blocks idempotent matriculation
Target 38,48,77 + 028:20-25,860,874, dominio.rs:74-113. Q1 asks global vs RBD-scoped yet 030 already assumes prospecto_rut nullable not unique + idempotent. 028 uq indexes global; sentinel passes validation. Idempotency key owner undefined. Fix: choose before 029 with sentinel rejection, uq replacement, prospect dedup, (oportunidad_id, alumno_rut) key plus Redis Idempotency-Key.

### B4 — Open registration + API-key bypass unscoped
Target absent vs auth.rs:151-212,258-279, main.rs:99-134, EGM.md:13. POST /api/acceso/registro open by design; exigir_sesion accepts x-clave-api with only scope-less check. Fix: validar_clave_api to holding_id/colegio_rbd + scope check, registro to invite-only or holding-scoped or explicit wont-fix with risk accepted.

## Majors (M1-M8) + Nits (N1-N2) + Passes + Opportunities

See synthesizer full output: M1 aggregation formulas missing; M2 retention TTL unenforceable; M3 E2E/auth migration fixture gap; M4 altitude file-level while Q1-Q3 open; M5 Twenty bundle too big; M6 RLS/p_scope matrix incomplete; M7 schoolccb/CSV second-truth dropped; M8 fail-closed + consent-detail holes. N1 ephemeral /tmp schoolccb ref; N2 licensing/Cargo/WASM copy in zoom-in. Passes: tenancy-first staging, directory vs tenant split, CSV quarantine, salud staged at secretaria, Twenty minimal non-goals, Alternatives A-D. Opportunities: reuse traslados audit for RUT mobility, idempotency via unique plus Redis TTL, hashed routing post-consent, CSV inbox as onboarding product, scope picker as saved view, holding totals single-SQL GROUP BY, consent UX as trust asset. Required edits lines 75-79,117,153,161-169; 68,150-158; 38,48,77,148; 11,22,39; 25,77; 17,64-71,83; 140-201; 128.
