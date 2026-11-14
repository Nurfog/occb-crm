# Stage 8 Intermediate Zoom Out — `docs/designs/saas-tenancy-matriculacion.md` (L1-205) — Consolidated

> Mode: REVIEW. Standard inaccessible (permission.rejected external_directory). Fallback applied. No files written in subagent. Prior compared: zoom_out (Stage 5: 4 Blockers + 8 Majors) vs comprehension (21 findings) vs critic (14 Blockers + 13 Majors + 9 Nits).

## Verdict: CONDITIONAL PASS — 10 Blockers + 7 Majors must close before implementation

Mitigation check: NONE. Target unchanged since Stage 5. All 4 Stage-5 Blockers persist. Comprehension + Critic add confirmations and elevations.

## Blockers B1-B10

B1 Public panel leak not closed. panel to protegidas + exigir_sesion+es_miembro+p_scope; listar_colegios public read-only; contracted require scope; 403 unknown scope.
B2 holdings entity missing. Add holdings + colegios_contratados(holding_id FK) or defer billing to manual flag-only; define es_miembro + login enrichment.
B3 RUT scope undecided blocks idempotent matriculation. Close Q1 before 029, define key + sentinel rejection + reject CUJs, freeze to oportunidad + lowercase etapas.
B4 Open registration + scope-less API-key bypass. validar_clave_api to holding/colegio + scope; registro invite-only or wont-fix + risk.
B5 CSV quarantine vacuous + sunset unowned. Freeze header list, origen csv + consent, RBD contracted else hold, per-row report + job_id, sunset 90d.
B6 Retention TTL + suppression unimplementable + salud contradiction. Single TTL 18m from ultimo_contacto, suppression blocks re-contact, cron owner, no salud until Matriculado commit.
B7 Licensing tiers unenforceable + billing contradiction. CHECK + 403 or display-only + no enforcement; define metric or defer.
B8 Scope binding incomplete + unowned joins + migrator risk. p_scope type + fail-closed 403, SET app.holding_id + app.colegio_rbd, own all join carriers, FK RESTRICT vs NULL-RBD flow, batched backfill + CONCURRENTLY runbook.
B9 Aggregation money-math undefined. Formulas + glossary + fixture before panel change.
B10 Rollout sequencing conflict, Twenty bundle too big. M0 freeze+split first, then tenant, CSV, matriculation; defer Twenty to M4+; map M to S gates.

## Majors M1-M7

M1 No numbered requirements/acceptance/traceability. G-01/FR-01/NFR-01/CUJ-01/AC-01 + Trace.
M2 NFR run-ons unmeasurable. Split to one-assertion NFR + metric/threshold.
M3 E2E/auth migration fixture gap. Extend prueba.mjs without renaming copy/selectors.
M4 Directory vs tenant FK trap. FK RESTRICT + admin import or NULL-RBD flow + CUJ.
M5 Consent detail hole + fail-closed secret. Consent schema + revocation; fail-closed prod secret.
M6 Structure scope leakage + misplaced alternatives + numbering collisions. <=5-line approach in feature, move detail to L136+; CUJ-01/ALT-01/Q-01 namespace.
M7 Duplication + zero tables + language. Normative in FR/NFR only, tables, active voice, glossary-first.

## Nits N1-N5

N1 Metadata incomplete (Doc-ID, Version, Reviewers/Approver). N2 Broken heading hierarchy + --- separators. N3 Dangling /tmp refs. N4 Glossary + i18n key drift. N5 Status/Dates without exit criteria + long paragraph + casing clean-up.

## Passes

Tenancy-first staging; contracted-vs-directory split; CSV quarantined inbox intent; atomic idempotent matriculation reusing forward arrow; consented pool with expiry; server-side scope; Twenty-minimal + non-goals; Alternatives A-D; 122-green + cargo/clippy gates.
