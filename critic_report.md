# Stage 7 Critic Report — FeatureRange 1–135 ONLY
**Target:** `/home/juan/dev/occb-crm/docs/designs/saas-tenancy-matriculacion.md`
**Scope:** L1–135 (feature spec; Detailed Plan L136+ excluded)
**Mode:** REVIEW / Antagonistic loop
**Reviewers:** `egm-design`, `egm-structural`, `egm-style` (parallel, fresh contexts via subagent)

**Notices:** agents and feature-spec-standard inaccessible (permission.rejected external_directory). Fallback built-in feature-spec expectations + GoldfishContext grounding used. No banned-adjective flattery found in L1–135. Locations verified against target L1–135 read.

## BLOCKERS (14)

B-01 RUT scope undecided while Goals/CUJ assume answer (L38 Q1 vs L25, L15, L46-48). 028 uq global, sentinel reused. Fix: close Q1, pick RBD-scoped UNIQUE or global with traslado, state migration, define dedup key + Idempotency-Key, add reject-case CUJ.
B-02 Holding money math undefined (L39 Q2 vs L11, L50-54, L22). panel sums importe, missing sum vs avg, currency, periodo, importe vs arancel, homologation. Fix: define formulas, glossary importe vs aggregates, EXPLAIN single-SQL GROUP BY + fixture.
B-03 Tenancy auth model missing (L21, L64-68 vs auth.rs). Reclamos no scope, exigir_sesion no membership, registro open, x-clave-api no scope. Missing holdings entity, membresias lifecycle, login enrichment, multi-holding. Fix: specify holdings, membresias lifecycle API, es_miembro signature, login JWT from memberships, registro invite-only or wont-fix + risk, validar_clave_api scope, 403 unknown scope.
B-04 SQL-enforcement hand-wave (L17, L22, L54). All reads SECURITY DEFINER, no RLS policies + SET app.holding_id/app.colegio_rbd + p_scope type, p_scope spoofable. transaccion_con_actor only app.usuario_id. Fix: p_scope type + fail-closed unknown/unscoped to 403 never global, move /api/panel to protected, keep listar_colegios public read-only, contracted require es_miembro, negative CUJ A cannot query B.
B-05 Matriculation stage-skip + partial spec (L44-48 vs dominio EtapaOportunidad). Evaluado to Matriculado skips Aceptado; missing curso, colegio_rbd, vinculo, salud draft vs crear_ingreso motivo, failure branches; Postulacion/Oportunidad + capitalized vs lowercase mismatch. Fix: require Aceptado pre-condition or allow Evaluado to Matriculado with rule; map mandatory fields; add reject CUJs; freeze to oportunidad table + lowercase etapas.
B-06 CSV quarantine acceptance vacuous (L56-60, L23). Fixed 5MB/1000/preview5/no mapping/no salud/vinculo/colegio_rbd; undefined template, RBD resolve, consent encoding, RUT check, per-row codes, job_id, TTL, contacto postulacion meaning. Fix: freeze header list, origen csv, consentimiento canal/fecha/prueba, RBD contracted else hold, per-row row/code/mensaje Spanish, sunset 90d.
B-07 Retention TTL range + suppression unimplementable + salud contradiction (L25, L15, L11, L77, L117). 12-24 range no anchor/job/schema, supresion undefined, re-import vs resurrect ambiguous; salud staged vs fills at matriculation contradiction. Fix: single TTL 18m from ultimo_contacto, origen/canal/timestamp/titular, supresion(rut_hash,motivo) blocks re-contact, cron owner, no salud row until Matriculado commit draft in tx payload only.
B-08 Licensing tiers unenforceable (L11, L40, L28-31). Small 1, Group 2-5, Holding 6+ vs Q3 per alumno/matricula/usuario vs Non-Goal no billing. No planes/suscripciones/metas/ingresos tables; no check/trigger/API. Fix: enforce CHECK + 403 on exceed or move counts display-only + no enforcement in 029-030; define metric or defer; remove billing language if non-goal.
B-09 No numbered testable requirements (L19-26, L42-60, L115-119). No REQ/FR/NFR/CUJ IDs, no traceability. Fix: G-01, FR-01, NFR-01, CUJ-01 with Trace each.
B-10 No Acceptance Criteria section (L44-60). Then bundles 5-6 assertions, no threshold/fixture/negative. Fix: add ## Acceptance Criteria with AC-01 one assertion each + FR/CUJ link.
B-11 No Glossary (L11, L21-26, L38-40, L76-81). Small/Group/Holding, contexto/scope/p_scope, licencia/dominio/accesos/web, prospecto_rut/origen/consentimiento/TTL/supresion, RBD/RUT/MINEDUC/JWT/RLS/CSV/WASM/PG/E2E/Twenty/forward arrow/idempotent/quarantine/sentinel/activo/salud, Cuando/Si/Entonces, both arms undefined. Fix: add ## Glossary before Goals with allowed values + canonical spelling.
B-12 Scope leakage full Technical Plan in feature spec (L62-85). Migration numbers, listar/panel/buscar, transaccion_con_actor, JWT shapes belong L136+. Fix: delete L62-85, replace with <=5-line Proposed Approach non-normative, move all to L136+.
B-13 Metadata incomplete (L1-5). Missing Doc ID, Version, Reviewers/Approver, Last-updated, Change history. Fix: add Doc-ID OCCB-SAAS-001, Version 0.1.0, Reviewers TBD, Approver for Q1-Q3, History table.
B-14 Broken heading hierarchy ### with no parent ## (L123, L130 under NFR L115). Fix: promote to ## References and ## Future Work or nest under ## Appendices as A.1/A.2; delete --- separators.

## MAJORS (13) M-01 to M-13

M-01 Personas/JTBD missing, grant contradicts No-matrix. Fix actor table + CUJ mapping or lift Non-Goal to minimal grant.
M-02 Twenty chrome without parent + E2E fragility + i18n gap. Defer Twenty to M4+ after tenancy green; list exact keys + Spanish copy.
M-03 NFR run-ons unmeasurable. Split to NFR-01 single assertion + metric/threshold + verification; move rollout gates to Rollout/Verification.
M-04 Directory vs tenant FK trap for new schools without RBD. Specify FK RESTRICT + admin import or NULL RBD manual flow + CUJ.
M-05 Consent detail hole. Define consentimiento canal/fecha/medio/titular + origen + revocar to suppression + anonymize, re-import on suppression rejected suprimido.
M-06 Solution inside The Problem. Cut L17 to Proposed Direction or delete.
M-07 Open Questions untrackable. Renumber Q-01 owner/due, add Resolution, cross-link Q to FR, imperative modality.
M-08 CUJ/Alternative/Question numbering collides. Single namespace CUJ-01, ALT-01, Q-01.
M-09 Alternatives misplaced + veto rationale. Move before approach, format ALT-01 Status/Criterion/Decided date/owner.
M-10 Dangling/ephemeral/forward refs. Relative versioned links, delete /tmp or vendor with hash, remove 029-030 from FeatureRange.
M-11 Duplication without single source. Normative wording in FR/NFR only, Overview pointers, resolve CSV/custom-domain/benchmark contradictions.
M-12 Zero tables, semicolon-crammed bullets. Tables ID/Statement/Rationale/Trace or one-assertion-per-bullet.
M-13 Sentences/passive/jargon hide actor. Split to <=25-word sentences, expand acronyms first use, define Twenty/forward arrow/idempotent/quarantine/props/arms, active voice.

## NITS (9) N-01 to N-09

N-01 importe vs ingresos/resultados/metas glossary mapping. N-02 ephemeral /tmp ref vendor or drop. N-03 Status/Dates without exit criteria, add Definition of Ready. N-04 Overview 130-word paragraph to 4 sentences. N-05 capitalisation drift glossary + find-replace. N-06 code-span label inconsistency. N-07 ambiguous pronouns/weasels/fragments. N-08 Future single-bullet dump one per item. N-09 separator pseudo-structure delete ---.

## Synthesis note

Design B-02 + style money-words merged to B-02; B-07 + retention merged to B-07; orphan-contexto + glossary + terminology merged to B-11; B-05 + stage-names merged to B-05; duplication absorbed to M-11; long-sentences/jargon/passive grouped to M-13.
