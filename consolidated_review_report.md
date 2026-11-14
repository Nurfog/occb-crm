# Consolidated Review Report — `docs/designs/saas-tenancy-matriculacion.md`

**Target:** `/home/juan/dev/occb-crm/docs/designs/saas-tenancy-matriculacion.md` (L1-205, Feature 1-135 + Detailed 136-205)
**Mode:** Consolidated full flow (Stages 5-9, no checkpoints)
**Verdict:** NOT READY — do not implement S1 until consolidated Blockers close. Direction (tenancy-first, contracted-vs-directory, quarantined CSV, atomic idempotent matriculation, consent+TTL+suppression, server-side scope, Twenty-minimal, 122-green gates) is sound.

## Prioritized Findings

### 🔴 Blockers

- **C-01 Tenancy auth + public panel leak (S5-B1, S8-B1, S9-B06, S7-B04).** `/api/panel` + `/api/dashboard` stay public while `panel_resumen(p_scope)` scheduled; `Reclamos` + `UsuarioActual` carry no `holding_id/colegio_rbd[]`; `exigir_sesion` + `x-clave-api` + `registro` unscoped. Fix: panel to `protegidas` + `exigir_sesion+es_miembro+p_scope`; `listar_colegios` directory public read-only; contracted lists require scope; `validar_clave_api` returns scope; `registro` invite-only or wont-fix + risk; 403 unknown scope, never global. Files: `auth.rs`, `main.rs`, `029`.
- **C-02 Holdings entity + membership lifecycle missing (S5-B2, S8-B2, S9-B01/B-03).** `colegios_contratados` + `membresias_usuario` + JWT `holding_id` without `holdings` table, Group 2-5 vs Holding 6+ unenforced, single JWT `holding_id` vs multi-holding users. Fix: `holdings(id,nombre,plan)` + `colegios_contratados(holding_id FK,rbd FK,licencia,dominio,flags)` + `membresias(usuario_id,holding_id,colegio_rbd)` + `es_miembro(pool,uid,hid,rbd)` + login enrichment + `transaccion_con_actor(pool,uid,hid,rbd)` setting `app.holding_id` + `app.colegio_rbd`. Files: `029`, `auth.rs`, `bd.rs`.
- **C-03 RUT scope + idempotent matriculation undefined (S5-B3, S8-B3, S7-B01/B-05, S9-B07).** Q1 global vs RBD-scoped open while 030 assumes `prospecto_rut nullable not unique`; `028` uq global + sentinel `00000000-0`; `matricular_postulacion(oportunidad_id,alumno_rut,...)` omits `colegio_rbd`, renames prospecto, no idempotency key; `Evaluado→Matriculado` skips `Aceptado`. Fix: close Q1 before 029 (RBD-scoped UNIQUE or global+traslado), reject sentinel, define `(oportunidad_id,alumno_rut)` + `Idempotency-Key`/Redis key, freeze lowercase DB etapas, require `Aceptado` or explicit skip + reject CUJs + `curso/vinculo/salud-draft` mapping.
- **C-04 SQL scope hand-wave + RLS absent + join carriers unowned (S7-B04, S8-B8, S9-B05, S6-#11/#12).** `p_scope` undefined/spoofable, `SECURITY DEFINER` no policies, `transaccion_con_actor` only `app.usuario_id`; `favoritos/timeline/actividad` + `traslados` joins unscoped; `CONCURRENTLY` in-txn impossible. Fix: freeze `p_holding_id uuid + p_colegio_rbd text`, fail-closed 403, `SET LOCAL app.holding_id + app.colegio_rbd`, RLS policies + `EXPLAIN` index proof, own all join carriers, batched backfill + separate `CONCURRENTLY` runbook.
- **C-05 Directory vs tenant conflation + new-school FK trap (S5-pass w/ gap, S7-M04, S6-#13).** 12k `colegios(RBD)` public vs `colegios_contratados(RBD FK)`; new private school without RBD cannot FK; `crear_colegio` vs read-only conflict; hide-without-contract unowned. Fix: `FK RESTRICT` + admin import or NULL-RBD manual flow + CUJ, RBD-without-contract hides from licensed lists stays in directory.
- **C-06 CSV quarantine vacuous + sunset + second-truth (S8-B5, S7-B06, S9-B08, S6-#1/#15).** Fixed 5MB/1000/preview5/no mapping vs frozen MINEDUC+RUT template, RBD resolve, consent encoding, per-row codes, `job_id` history, `contacto postulacion` meaning, sunset owner missing; Overview says into core vs Goals never direct alumno. Fix: header list + `origen=csv` + `consentimiento{canal,fecha,prueba}` + RBD contracted else hold + per-row `{fila,codigo,mensaje}` + `importaciones(job_id)` creating `oportunidades(contacto,origen csv)` never `alumnos` + 90d sunset + owner.
- **C-07 Retention TTL + suppression + salud contradiction (S8-B6, S7-B07, S9-B09, S6-#14).** 12-24mo range no anchor/job/schema, `supresion` undefined, re-import vs resurrect ambiguous, salud staged vs fills contradiction, `crear_ingreso(motivo)` required. Fix: single TTL 18m from `ultimo_contacto`, `origen/canal/fecha/titular`, `supresion(rut_hash,motivo)` blocks re-contact (`suprimido`), nightly cron owner, no salud row until `Matriculado` commit (draft in tx payload).
- **C-08 Licensing tiers + money math + value metric unenforceable (S8-B7/B9, S7-B08/B-02, S6-#2/#7).** Small 1 / Group 2-5 / Holding 6+ vs Q3 per alumno/matricula/usuario vs Non-Goal no billing; `ingresos/resultados/metas` vs `importe/valor_tuberia` vs currency/period/homologation undefined. Fix: `CHECK` + 403 on exceed or display-only + no enforcement; define metric or defer; money formulas + glossary + single-SQL `GROUP BY` + fixture before panel change; remove billing language if non-goal.
- **C-09 i18n + scope-frontend + Twenty bundle underspecified (S5-M5/M8, S9-B11, S7-M02).** Paired `Textos` list mixes capitals/acronyms, zero exact identifiers + ES strings; scope picker + `ChipVistaActiva` vs `SelectorVistas` false reuse; Path/highlights/dot/grouping/UTC/rule/chat-hint bundled with `main.css` generics + `main.rs` monolith + WASM delta. Fix: exact snake_case `Textos` + both ES/EN verbatim (fail build if arm missing); `SCOPE` signal + scoped `cliente::*` + `ScopePicker` + `ChipVistaActiva(vista,busqueda,on_limpiar)` wrapping div only; defer Twenty to M4+ after tenancy green.
- **C-10 Verification + rollout + E2E migration unmeasurable (S9-B12, S8-B10, S7-M03).** `dx` vs `cargo build` ambiguous, `cargo test` duplicated, manual Path checks, WASM 5KB/Cargo/EXPLAIN/S-gates vague; M0-M3 vs S1-S3 unmapped; scope fixtures, idempotent resubmit, CSV per-row, TTL/suppression, negative 403 missing; `migrar_y_sembrar 001→030` single-txn risk + restore runbook absent. Fix: pin `cargo build -p occb-api`, `cargo test` zero-fail, `clippy -D warnings`, WASM baseline + stat gate, Cargo diff exit-code, `EXPLAIN` Index Scan, `grep filter 0`, `S1=M1 S2=M2 S3=M3`, E2E 122/122 + additive asserts without renaming copy/selectors, `pg_restore` runbook, no stacking on red.
- **C-11 Spec structure blocks build (S7-B09/B-10/B-11/B-12/B-13/B-14).** No `G-/FR-/NFR-/CUJ-/AC-` IDs, no `## Acceptance Criteria`, no `## Glossary`, full Technical Plan L62-85 in feature spec while Q1-Q3 open, `###` without parent `##`, metadata missing Doc-ID/Version/Reviewers/Approver. Fix: `G-01/FR-01/NFR-01/CUJ-01/AC-01` + Trace table; `## Acceptance Criteria` one assertion each; `## Glossary` before Goals; ≤5-line Proposed Approach in feature, move how to L136+; `CUJ-01/ALT-01/Q-01` namespace with owner/due/Resolution; header Doc-ID `OCCB-SAAS-001` v0.1.0 + History; promote `## References` + `## Future Work`, delete `---`.
- **C-12 Matriculation route + DTO + stage naming missing (S9-B03/M-03).** No `rutas_matricula.rs`, no `mod`, no `POST /api/oportunidades/:id/matricular`, `CuerpoMatricula` undefined, forward-arrow vague, DB lowercase vs frontend capitalized. Fix: create file + `mod` + protected POST with `es_miembro`, DTO `(oportunidad_id,alumno_rut,apoderados_rut,salud,idempotency_key)`, freeze lowercase `contacto..matriculado`, forbid new variants.

### 🟡 Majors

- Consent detail + fail-closed secret + sentinel + salud-draft holder (S7-M05, S8-M5, S9-N05). Orphan `accesos/web link`, `dominio` text-only no DNS (S6-#3). Personas/JTBD + grant vs No-matrix (S7-M01). Twenty chrome parent + E2E fragility (S7-M02). NFR run-ons + WASM/Cargo/EXPLAIN (S7-M03). CSV RBD/consent schema (S9-M04). Frontend scope state conflation (S9-M05). E2E fixtures without users/time-travel (S9-M06). Panel money-math at change point (S9-M07). main.css baseline + frozen selectors (S9-M08). migrar_y_sembrar ordering vs seed/demo (S9-M09). Fail-closed + API-key scope (S9-M10). M vs S mapping + restore vague (S9-M11). Heading/file traceability + one-assertion bullets + code-span consistency (S9-M12/M-13/M-14).

### 🟢 Nits

- Language drift importe/ingresos, /tmp schoolccb ref, status without exit criteria, 130-word Overview paragraph, casing/code-span/pronoun clean-up, Future dump, separator pseudo-structure (S7-N01/N-09). Capitalization, main.rs ambiguity, preview copy, Deshabilitar wrap, salud payload, plus-chaining (S9-N01/N-06). Glossary/i18n drift, metadata, hierarchy, dangling refs, status/exit (S8-N1/N-05).

## Traceability Matrix

| Consolidated | Raw findings | Target lines |
|---|---|---|
| C-01 | S5-B1, S8-B1, S7-B04, S9-B06 | 76,79,117,153,161-169, auth, main |
| C-02 | S5-B2, S8-B2, S9-B01/B-03 | 68,150-158 |
| C-03 | S5-B3, S8-B3, S7-B01/B-05, S9-B07, S6-#4/#9/#10 | 38,48,77,147-148,162 |
| C-04 | S7-B04, S8-B8, S9-B05 | 17,22,54,67,76,83-85,158 |
| C-05 | S7-M04, S6-#13 | 21, 010, 028 |
| C-06 | S8-B5, S7-B06, S9-B08 | 11,23,56-60,77,172-173 |
| C-07 | S8-B6, S7-B07, S9-B09 | 11,15,25,77,117,148 |
| C-08 | S8-B7/B9, S7-B08/B-02 | 11,31,39-40,143 |
| C-09 | S5-M5/M8, S9-B11, S7-M02 | 80-81,178,182-187 |
| C-10 | S9-B12, S8-B10, S7-M03 | 191-201, 119 |
| C-11 | S7-B09/B-10/B-11/B-12/B-13/B-14 | 1-5,19-26,42-60,62-85,115-134 |
| C-12 | S9-B03/M-03 | 160-164 |

Stages: S5 zoom_out_report.md (full 1-205), S6 comprehension_report.md (1-135 vs 136-205, 21 findings), S7 critic_report.md (1-135, 14B+13M+9N), S8 intermediate_zoom_out_report.md (full delta, 10B+7M), S9 implementation_readiness_report.md (136-205, 12B+14M+6N). Canonical standards inaccessible (external_directory); fallback invariants applied; re-run once perms fixed.
