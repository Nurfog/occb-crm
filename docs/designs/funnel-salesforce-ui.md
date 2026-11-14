# OCCB Funnel and States Salesforce Lookalike (UI Only)

* **Author:** Nurfog
* **Status:** Draft
* **Date:** 2026-11-12

--------------------------------------------------------------------------------

## Overview

OCCB is a school CRM with colegios, alumnos, apoderados, admission funnel, tasks, notes, calendar, mail, search, favorites, views, timeline, automations, chat, keys, webhooks, and import. Users who saw Salesforce demo videos expect a familiar funnel: a stage path on top, dense cards, highlight headers, and related lists. This design adds that appearance in the Dioxus frontend only. No tables, endpoints, or state machines change. Spanish stays default with ES/EN toggle. Deletes stay soft with admin-only anonymize. The 122 E2E checks stay green.

## The Problem

New users orient by funnel position and record context. Today DetalleOportunidad shows no stage path, so position is visible only by scrolling kanban. Colegios, alumno, and apoderado fichas bury key facts in the body with no pinned summary. Rule and view configs read as codes. The gap is orientation, not function: handlers, validation, RUT checks, and audit already work and are E2E pinned. The risk is copying Salesforce behavior that would lie: hard delete, merge, drag reorder, forecasts, or sharing have no backend and would break invariants.

Proposed solution: clickable Path strip derived from etapa (display plus same mover_etapa, permits non-linear jump; arrow remains forward-only canonical), highlights headers from existing fields, CSS-only grouping and tints, sentence previews composed from stored JSON, and label chips. All reuse current handlers and selectors and add only wrapping markup.

## Goals

* Admission detalle shows a clickable Path strip with five school stages and guidance text, clicking a stage emits EtapaOportunidad converted to DB string and calls mover_etapa with PATCH /api/oportunidades/{id}/etapa {etapa:<db-string>}; Path permits non-linear jump; arrow remains forward-only canonical.
* Colegios, alumno, and apoderado fichas show a pinned highlights header built from struct fields plus relations already fetched on that route (colegio nombre,RBD,comuna plus opps_rel.len from existing listar_oportunidades; alumno nombre,RUT display,estado plus vinculados.len; apoderado nombre,RUT plus alumnos.len); reuse existing listar (no new params); counts are active only.
* Kanban keeps arrow as canonical advance, cards keep nombre, importe display, responsable, fecha, colegio, contacto with CSS-only urgency dot from actualizado via hace_tiempo (red if not Matriculado and >7d, amber if >3d); per-column counts reuse existing span.count list length, no new scope.
* Tasks group client-side (Rust date compare) from vencimiento date in local TZ via a_campo_local: Overdue date<today, Today ==, Upcoming >, Sin fecha None always last; headers ES/EN paired; calendar keeps month view with static UTC suffix only in title and detail line, never in data-dia or cal-punto; search keeps grouped results with ChipVistaActiva showing Vista and Busqueda plus clear reusing on_cargar; timeline shows hace_tiempo plus fmt_dia_hora as title, never leaks RUT, phone, or mail body.
* Automations show one sentence per rule Cuando disparador, Si campo operador valor, Entonces accion from stored rule JSON (empty Sin condicion siempre, unknown raw key) in ES and EN; chat shows hardcoded static limit hint above chat-compose in ES and EN; Ajustes keeps Claves, Webhooks, Import plus space form with no tree, secret-once and csv-result classes unchanged.
* Every new label adds paired fields to Textos plus both Idioma arms in i18n.rs before S1; no E2E selector or asserted text breaks (opp-card, ghost-new, record-panel, tbl, cal-grid, search-dialog, automation-card, chat-layout, secret-once, csv-result, data-dia, data-correo plus opp-name, col, stage, search-result, star-favorite, saved-view-select, automation-form, chat-history, Deshabilitar, Anonimizar, Confirmar, Crear, Guardar); cargo test --workspace plus cargo clippy --workspace -- -D warnings plus node e2e/prueba.mjs stay green with 122/122 and no stacking on red.

## Non-Goals

* No drag and drop for kanban or calendar; no week or day views; no threading for correos without hilo_id.
* No Flow canvas, no column configurator, no list sharing, no Chatter comments, no merge or duplicate engine.
* No multi-space, no backup scheduling, no RLS per-row change, no new tables, columns, endpoints, or state values.
* No hard deletes, no English-only labels, no amount forecasting, no domain change away from schools.
* No new frontend dependencies, no framework change, no main.rs split in this design (deferred).

## Open Questions

* **Question 1**: Resolved 2026-11-12 by author (Aceptar y cerrar Q1). Path stages reuse etiqueta_etapa: Contacto/Contact, Postulante/Applicant, Evaluado/Evaluated, Aceptado/Accepted, Matriculado/Enrolled. Guidance ES: Contacto registra primer contacto; Postulante completa ficha y RUT; Evaluado revisa antecedentes; Aceptado confirma cupo; Matriculado cierra matricula. Guidance EN mirror. Display importe as Monto ref./Amount ref. for field importe. Display Camino de matricula with route /oportunidades intact.
* **Question 2**: Resolved as loaded list length via existing count (no extra fetch).
* **Question 3**: Resolved as static UTC string, locale-neutral, single-path in fmt helpers, only in title and detail line.

## Critical User Journeys (CUJs)

### CUJ 1: Admission clerk orients on stage

* **Given**: Postulacion with etapa Evaluado, detalle loaded with panel data and related lists.
* **When**: Clerk opens DetalleOportunidad.
* **Then**: Clickable Path strip shows Contacto, Postulante, Evaluado (current), Aceptado, Matriculado with guidance line; clicking another stage emits EtapaOportunidad, converts to DB string contacto,postulante,evaluado,aceptado,matriculado, calls mover_etapa with PATCH and persists; backward jump allowed; kanban column and E2E kanban checks still pass.

### CUJ 2: School secretary finds context fast

* **Given**: Colegio with RBD, region, comuna, postulaciones; alumno with RUT, apoderados, salud, ingresos.
* **When**: Secretary opens any ficha.
* **Then**: Highlights header pins nombre plus RBD or RUT state plus counts from already fetched relations with no Delete, Merge, or Eliminar affordance, X keeps title Deshabilitar, anonymize stays admin-only two-step; related lists hide when empty except Ingresos médicos; traslado flow shows Destino nombre, rbd, comuna from selected row above motive and confirms Trasladar a rbd with no new endpoint.

### CUJ 3: Daily worker triages tasks without new queries

* **Given**: Tareas with vencimiento and prioridad, calendar events in UTC, search views with active filter.
* **When**: Worker opens Tareas, Calendario, or search.
* **Then**: Tasks group Overdue, Today, Upcoming, Sin fecha last from vencimiento date in local TZ; calendar shows static UTC suffix only in title and drawer; search shows ChipVistaActiva Vista nombre plus Busqueda q above search-dialog with Clear resetting busqueda and seleccionada None reusing on_cargar; wrapping div only outside search-result; no new API params.

## Technical Plan (Architecture)

UI-only veneer over current Axum plus PostgreSQL backend. No endpoint, table, or state value changes. All new visuals derive from props already in memory and reuse existing mutators.

```text
[DetalleOportunidad] --etapa--> [PathStrip] --click--> mover_etapa (cliente.rs) --> PATCH /api/oportunidades/{id}/etapa {etapa:<db-string>}
[Ficha colegio/alumno/apoderado] --loaded struct--> [HighlightsHeader] (pure render, no fetch)
[Kanban / Tareas / Calendario] --loaded vec--> [GroupHeader / StalenessTint] (client-side date compare, CSS class)
[Automatizacion rule JSON] --> [SentencePreview] (pure format: Cuando/Si/Entonces)
[i18n.rs Textos] --paired ES/EN--> [all new labels]
[main.css] <-- chevrons, chips, tints, sticky headers (no layout framework)
```

Components (all in `crates/app/src/main.rs` unless noted, pure render unless noted):

* `BarraEtapas`: receives current EtapaOportunidad and renders five chevrons plus guidance line; on_select emits EtapaOportunidad, converts via enum to DB string, calls cliente::mover_etapa with PATCH; never sends etiqueta_etapa output; no drag.
* `CabeceraResumen`: props for colegio (nombre,RBD,comuna plus opps_rel.len), alumno (nombre,RUT display,estado plus vinculados.len), apoderado (nombre,RUT plus alumnos.len) from already fetched Detalle signals; sticky div top0 z-index opaque bg above existing .record-panel; reuse existing listar (no new params); adds no Delete, Merge, or Eliminar; counts active only.
* `TarjetaPostulacion`: existing opp-card plus urgency dot from actualizado via hace_tiempo (red if not Matriculado and >7d, amber if >3d) with title Sin movimiento hace X dias; per-column counts reuse existing span.count list length; CSS in main.css.
* `GrupoTareas`, `EtiquetaUTC`, `ChipVistaActiva`, `VistaPreviaRegla`, `PistaLimiteChat`: tiny pure fns under 50 lines reusing etiqueta_etapa, etiqueta_tarea, etiqueta_estado, hace_tiempo, a_campo_local, fmt_hora, fmt_dia_hora, fmt_fecha; client-side grouping uses Rust date compare plus CSS-only tints and chips; UTC suffix static UTC appended in fmt helpers only for title and detail line; ChipVistaActiva vista Option String,busqueda String,on_limpiar Callback above search-dialog; VistaPreviaRegla Cuando Si Entonces with empty Sin condicion siempre; PistaLimiteChat hardcoded ES and EN above chat-compose.
* `i18n.rs`: add paired Textos fields for Path guidance, display Monto ref. and Amount ref. for field importe (or Arancel ref. and Tuition ref. per Q1), Overdue, Today, Upcoming, Sin fecha, UTC label, filter clear, Cuando, Si, Entonces, limit hint; both Idioma arms required; no runtime fallback.
* `main.css`: add .path-strip clip-path polygon with .path-current and .path-done, .highlights sticky top0 z-index opaque bg, .stale tint with threshold actualizado>14d else drop to S2, .chip-pill; do not reuse .chip row for pills; ban style except width on bar-fill; keep DOM classes in main.rs and CSS rules in main.css; keep opp-card, ghost-new, record-panel, tbl, cal-grid, search-dialog, automation-card, chat-layout, secret-once, csv-result, data-dia, data-correo plus opp-name, col, stage, search-result, star-favorite, saved-view-select, automation-form, chat-history, Deshabilitar, Anonimizar, Confirmar, Crear, Guardar asserted copy intact with opp-name descendant of opp-card and stage descendant of col.

Data flow stays as today: Dioxus signals fetch once through `cliente::*`, render derives display strings, user action calls same mutator, signal refreshes. No new store, no new polling, no new dep.

Failure modes closed by construction: exhaustive match on EtapaOportunidad, invalid DB string rejected at deserialization, no UI fallback; if RBD is empty the frontend hides the RBD row; related lists hide when empty except Ingresos médicos; the frontend never renders an empty row; missing vencimiento puts task under Sin fecha None always last; Build fails if either Idioma arm omits a key with no runtime fallback; E2E text match preserved because new markup wraps outside asserted substrings and never renames asserted copy with UTC only in title and detail line.

## Alternatives Considered

### Alternative A: Pixel clone of Sales Cloud

* **Pros**: Familiar to ex-Salesforce users; covers forecasts, hierarchies, sharing.
* **Cons**: Needs new objects, sharing tables, forecast jobs, hierarchy queries; explodes main.rs monolith; breaks soft-delete law and 122 E2E; school users pay for unused sales surface.
* **Reason for Rejection**: Rejected by author: school domain has no use for forecasts or sharing, cost exceeds orientation value.

### Alternative B: Do nothing, document gaps only

* **Pros**: Zero E2E risk; zero i18n risk; keeps main.rs untouched.
* **Cons**: Leaves orientation gap; new clerks still scroll kanban to find stage; fichas stay dense without summary.
* **Reason for Rejection**: Rejected by author as sole outcome: Path strip plus headers are near-zero risk with clear orientation payoff, kept as baseline fallback if staging turns red.

### Alternative C: Full interaction upgrade with drag, week views, Flow canvas

* **Pros**: Closest feel to Salesforce power users; drag reorder and calendar drag feel direct.
* **Cons**: Needs order persistence, new drag state in WASM, headless timing flakes, backend for hilo threading and column config; requires main.rs split and E2E sharding first.
* **Reason for Rejection**: Rejected by author: deferred until main.rs split plus E2E sharding; arrow plus month view cover school flow today.

--------------------------------------------------------------------------------

## Non-Functional Requirements

* Security and privacy: no new data exposure; highlights reuse already-loaded fields; timeline keeps no RUT, phone, or mail body leak; anonymize display stays em dash with UUID history.
* Performance: no new fetch; grouping and tints compute from loaded vecs; CSS only; no new dep; WASM size change limited to markup and strings.
* Rollout and rollback: UI-only, no migration; each stage ships separately (S1 funnel, S2 record, S3 trust); revert is file revert plus rebuild; E2E green gates each stage with no stacking on red.

--------------------------------------------------------------------------------

### Additional Resources

* `concepts_brief.md` (Priority 1 intake, Nurfog 2026-11-12)
* `EGM.md` root blueprint plus `crates/app/src/README.md` frontend contract
* `e2e/prueba.mjs` 122 checks plus selectors list
* `docs/designs/auditoria-occb-crm.md` transversal constraints (soft-delete law, RUT, 122 green)

### Future Improvements

* Drag and drop as progressive enhancement after main.rs split; week views only if schools request scheduling density; Flow canvas only with backend ordering support.

--------------------------------------------------------------------------------

## Detailed Implementation Plan

### Target Files to Create/Modify

#### 1. `crates/app/src/main.rs` (Modify)

* **Type / Scope**: Dioxus frontend views and tiny pure components.
* **Proposed Changes**: [F1 BarraEtapas, F2 CabeceraResumen, F3 Staleness, F4 GrupoTareas, F5 UTC, F6 ChipVista, F7 VistaPreviaRegla, F8 PistaLimite] Add BarraEtapas used in DetalleOportunidad with etapa EtapaOportunidad prop emitting EtapaOportunidad converted to DB string to existing mover_etapa with PATCH; add CabeceraResumen used in DetalleColegio, DetalleAlumno, DetalleApoderado with loaded struct props from already fetched Detalle signals; extend opp-card markup with urgency dot from actualizado via hace_tiempo; add GrupoTareas headers from vencimiento date in local TZ with Sin fecha None last; append static UTC suffix only in title and detail line through fmt helpers; add ChipVistaActiva vista Option String,busqueda String,on_limpiar Callback above search-dialog outside search-result with Clear resetting busqueda and seleccionada None reusing on_cargar; add sentence template Cuando disparador Si campo operador valor Entonces accion with empty Sin condicion siempre; add hardcoded static limit hint above chat-compose; add destination summary Destino nombre rbd comuna from selected tr_resultados row above motive; wrap outside asserted substrings, never rename asserted copy; no drag, no week/day, no threading, no Flow, no column-config, no sharing, no Chatter, no merge, no new tables, columns, endpoints, state, no new dep, tiny pure fns under 50 lines.

#### 2. `crates/core/src/i18n.rs` (Modify)

* **Type / Scope**: Shared ES and EN interface strings.
* **Proposed Changes**: Add paired Textos fields for Path stage names plus one-line guidance, display Monto ref. and Amount ref. for field importe (or Arancel ref. and Tuition ref. per Q1), Overdue, Today, Upcoming, Sin fecha, UTC label, filter clear, Cuando, Si, Entonces, limit hint; both Idioma arms required; no runtime fallback. Trace CUJ1 to F1 plus Goal plus E2E 343-387; CUJ2 to F2; CUJ3 to F4 plus F5 plus F6.

#### 3. `crates/app/assets/main.css` (Modify)

* **Type / Scope**: Global stylesheet with long cache in nginx.
* **Proposed Changes**: Add .path-strip clip-path polygon with .path-current and .path-done, .highlights sticky top0 z-index opaque bg with preserved scroll, .stale tint with threshold actualizado>14d else drop to S2, .chip-pill; do not reuse .chip row for pills; ban style except width on bar-fill; keep DOM classes automation-card in main.rs 1762 and attrs data-dia 4700 data-correo 5020 plus CSS rules opp-card, ghost-new, record-panel fixed overflow, tbl, cal-grid, search-dialog, chat-layout, csv-result in main.css; verify 1280x900 plus 100vw.

### Verification Plan

* **Compilation/Build**: `cargo build -p occb-app`.
* **Unit Tests**: `cargo test --workspace` requires zero failures.
* **Static Analysis & Linting**: `cargo clippy --workspace -- -D warnings`.
* **Test Coverage Requirement**: WASM delta under 5 KB, Cargo.toml unchanged, revert is git revert plus rebuild plus E2E green, S1, S2, S3 each gated with no stacking on red.
* **E2E/Integration**: `node e2e/prueba.mjs` with FRONTAL http://127.0.0.1:8080 API http://127.0.0.1:3001 Chrome headless; 122/122 green; grep opp-card, ghost-new, record-panel, tbl, cal-grid, search-dialog, automation-card, chat-layout, secret-once, csv-result, data-dia, data-correo plus opp-name, col, stage, search-result, star-favorite, saved-view-select, automation-form, chat-history, Deshabilitar, Anonimizar, Confirmar, Crear, Guardar before merge with opp-name descendant of opp-card and stage descendant of col and UTC never in asserted cells; dx build --web local-optional in crates/app.

--------------------------------------------------------------------------------

*Drafted with EGM*
