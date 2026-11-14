# Stage 9 Implementation Readiness — L136-205 ONLY — Consolidated

> Target: `/home/juan/dev/occb-crm/docs/designs/saas-tenancy-matriculacion.md:136-205`
> Mode: REVIEW. Standards inaccessible (permission.rejected external_directory). Fallback actionability/file/DDL precision/migration ordering/scope/auth/idempotency/testability/rollout gates + GoldfishContext grounding. Prior: zoom_out (4B+8M), critic (14B+13M+9N L1-135), intermediate (10B+7M). Target unchanged. All Blockers touching L136-205 persist.

## Verdict: NOT READY — 12 Blockers + 14 Majors + 6 Nits. Do not implement from L136-205.

Phantom summary: holdings, colegios_contratados, membresias_usuario, supresion, importaciones(job_id), prospecto_rut/origen/consentimiento/ttl on oportunidades, matricular_postulacion, es_miembro, p_scope, rutas_matricula.rs, SCOPE signal, ScopePicker/ChipVistaActiva, scope-CSS, Textos Small...Entonces do not exist.

## BLOCKERS B-01 to B-12

B-01 holdings entity missing; holding_id FK dangles; es_miembro unimplementable. Fix: add holdings + colegios_contratados(holding_id FK, rbd FK, licencia, dominio, flags) + membresias in 029 with FK ON DELETE + CHECK, or defer billing to flag-only.
B-02 JWT claims shape undefined; login enrichment missing. Reclamos{sub,nombre,rol,exp,jti} + UsuarioActual no scope; responder_sesion/renovar never query memberships. Fix: Reclamos + UsuarioActual add holding_id + colegio_rbd, emitir_acceso/login enrich from membresias, multi-holding rule.
B-03 es_miembro signature absent; scope guard not bound. No fn es_miembro, exigir_sesion no membership, main protegidas global, no rutas_matricula.rs. Fix: es_miembro(pool,usuario_id,holding_id,colegio_rbd) exists check + POST /api/oportunidades/:id/matricular in api main 197-205 zone with 403 prohibido.
B-04 transaccion_con_actor scope extension no signature; app.colegio_rbd omitted. bd.rs only app.usuario_id. Fix: transaccion_con_actor(pool,uid,hid,rbd) + SET LOCAL app.holding_id + app.colegio_rbd; update call sites.
B-05 p_scope type undefined; spoofable; RLS absent. listar/panel/buscar unscoped, SECURITY DEFINER no policies. Fix: freeze p_holding_id uuid + p_colegio_rbd text, 403 on null/null never global, enable RLS + policy using current_setting, bind to es_miembro.
B-06 /api/panel + /api/dashboard leak stays public. publicas includes panel/dashboard/colegios; 010 grant select public. Fix: delete panel/dashboard from publicas to protegidas requiring scope; listar_colegios directory public read-only; new listar_colegios_contratados requires es_miembro.
B-07 RUT scope undecided blocks 030; matricular + idempotency key missing. 030 assumes prospecto_rut nullable not unique while Q1 open; 028 uq global + sentinel; no matricular_postulacion; DB lowercase vs frontend capitalized. Fix: close Q1, full DDL alter + unique + matricular_postulacion(...,p_idempotency_key) + Idempotency-Key header, reject sentinel, freeze lowercase.
B-08 CSV quarantine vacuous; job_id + contacto postulacion undefined; sunset unowned. Import already 5MB/1000/preview5/headers rut,nombres,apellidos; 023 direct crear_alumno no job_id; e2e 3-col fixture. Fix: freeze headers [rut,nombres,apellidos,colegio_rbd,consentimiento_canal,fecha] + RBD contracted else hold + per-row fila/codigo/mensaje + importaciones(job_id) + oportunidades etapa contacto origen csv never alumnos + sunset 90d + owner.
B-09 Retention TTL + supresion + salud contradiction. 028 anonimizar immediate no TTL/cron; no supresion; no ultimo_contacto/origen/consent/ttl; crear_ingreso requires motivo. Fix: ttl=ultimo_contacto+18mo, supresion(rut_hash,motivo), cron nightly, re-import hash reject suprimido, salud draft tx payload only no crear_ingreso until Matriculado.
B-10 Licensing tiers unenforceable. No planes/suscripciones/metas/ingresos; no CHECK. Fix: CHECK + 403 EXCEDE_LICENCIA or display-only + no enforcement; define metric or defer.
B-11 i18n keys inexact; build-break risk + naming drift. Textos snake_case fixed, Es/En exhaustive; list mixes capitals/acronyms. Fix: exact snake_case identifiers + both ES/EN strings verbatim.
B-12 Verification unmeasurable. dx vs cargo ambiguous, cargo test duplicated, manual Path checks, WASM/Cargo/EXPLAIN/S-gates vague. Fix: pin cargo build -p occb-api, cargo test zero-fail, clippy -D warnings, WASM baseline + stat gate, Cargo diff exit-code, EXPLAIN Index Scan, grep filter 0, map S1=M1 S2=M2 S3=M3, E2E 122/122 + new asserts.

## MAJORS M-01 to M-14

M-01 Migration scope-carrier columns omitted + CONCURRENTLY runbook missing. M-02 p_scope join carriers unowned. M-03 rutas_matricula file/route/DTO missing + stage naming. M-04 CSV RBD resolve + consent schema missing. M-05 Frontend scope state + cliente signatures missing + SelectorVistas conflation. M-06 E2E fixtures no asserts/users/time-travel. M-07 Panel money-math undefined. M-08 main.css baseline + frozen selectors unlisted + WASM conflated. M-09 migrar_y_sembrar ordering vs seed/demo. M-10 Fail-closed secret + API-key scope absent. M-11 Rollout M vs S mapping + restore vague. M-12 Heading breaks traceability. M-13 One-assertion-per-bullet violated + duplication + missing IDs. M-14 Code-span + vague-verb inconsistency.

## NITS N-01 to N-06

N-01 Capitalization drift small/group/holding + CHECK. N-02 main.rs ambiguity api:: vs app::. N-03 preview five vs previa + 5MB/1000 keep as-is. N-04 Deshabilitar/Anonimizar wrap risk wrapping divs only. N-05 salud payload vs crear_ingreso motivo. N-06 plus chaining obscures actor, active voice + Tipo/Alcance + CREATE/ALTER ops.

## Shovel-ready checklist

1. holdings DDL + FKs + es_miembro + Reclamos JWT + transaccion_con_actor signatures. 2. p_scope as (p_holding_id,p_colegio_rbd) + /api/panel to protegidas + RLS. 3. Close Q1 + full matricular + Idempotency-Key. 4. Freeze CSV headers + importaciones(job_id) + origen csv + sunset owner. 5. Exact Textos + ES/EN + cliente + SCOPE + POST matricular. 6. Measurable verification WASM/EXPLAIN/Cargo/S=E2E asserts.
