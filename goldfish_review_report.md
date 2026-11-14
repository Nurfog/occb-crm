# Goldfish Annotated Review Report

Status: resolved

**Target:** `/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md` (`full file`)

## Prioritized Findings & AI Suggestions

### [Issue 1] [`egm-design`] - Severity: Blocker
*   **Affected Scope:** [Lines 67-67](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L67-L67) (Technical Plan diagram)
*   **Discrepancy Description:** Diagram claims `POST /api/oportunidades/{id}/etapa`, contradicts UI-only invariant and codebase which uses PATCH with DB-string.
*   **Rationale:** `cliente.rs:493-502` uses PATCH with `etapa` DB text contacto/postulante; E2E maps ORDEN_ES to A_TEXTO_DB; no POST exists.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid, checkable in repo. Diagram invents endpoint.
*   ***Proposed Edit:***
    *   Location: Line 67
    *   Change: Replace `POST /api/oportunidades/{id}/etapa` with `PATCH /api/oportunidades/{id}/etapa {etapa:<db-string>}` and add rule never send display label.

### [Issue 2] [`egm-design`] - Severity: Blocker
*   **Affected Scope:** [Lines 86-86](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L86-L86) (Failure modes)
*   **Discrepancy Description:** Claims missing i18n key falls back to ES string with build checklist.
*   **Rationale:** `i18n.rs:32-35` obligates translation, struct has exhaustive Es+En arms, no fallback fn; missing key is compile error; zero proposed keys exist.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid, checkable. Fallback would hide missing EN.
*   ***Proposed Edit:***
    *   Location: Line 86 + Line 143-144
    *   Change: Delete fallback sentence. Replace with Build fails if either Idioma arm omits a key; checklist adds paired Textos fields plus both arms before S1. No runtime fallback.

### [Issue 3] [`egm-design`] - Severity: Blocker
*   **Affected Scope:** [Lines 11-21](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L11-L21) (read-only Path)
*   **Discrepancy Description:** Read-only Path that clicks to persist plus unstated gating (free-jump vs forward-only).
*   **Rationale:** Read-only contradicts click-to-mover_etapa; backend allows any move but UI implies linear; E2E asserts one-step advance.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid terminology and interaction gap.
*   ***Proposed Edit:***
    *   Location: Lines 11,17,21,48,77
    *   Change: Rename to clickable Path strip (display plus same mover_etapa). Add Path permits non-linear jump; arrow remains forward-only canonical. Guidance must not imply gating.

### [Issue 4] [`egm-design`] - Severity: Blocker
*   **Affected Scope:** [Lines 24-24](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L24-L24) (Task buckets)
*   **Discrepancy Description:** Tasks group by overdue/today/upcoming with no boundaries, timezone, or Sin fecha definition.
*   **Rationale:** No bucket dates, TZ via a_campo_local unspecified, Sin fecha only in failure line; same task groups differently across TZ.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid testability gap.
*   ***Proposed Edit:***
    *   Location: Goals Line 24 + Components + Failure Line 86
    *   Change: Define buckets on vencimiento date in local TZ via a_campo_local: Overdue date<today, Today ==, Upcoming >, Sin fecha None always last. Headers ES/EN paired.

### [Issue 5] [`egm-structural`] - Severity: Blocker
*   **Affected Scope:** [Lines 22-78](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L22-L78) (Highlights no-fetch)
*   **Discrepancy Description:** Highlights built from already loaded fields with no fetch contradicts loaders needing separate listar calls for counts.
*   **Rationale:** DetalleColegio needs listar_oportunidades filter; DetalleAlumno needs apoderados/traslados/tareas; already loaded undefined per route.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid data-flow contradiction.
*   ***Proposed Edit:***
    *   Location: Lines 22,78,84
    *   Change: Freeze per ficha: colegio nombre,RBD,comuna plus opps_rel.len from existing listar_oportunidades; alumno nombre,RUT,estado plus vinculados.len; apoderado nombre,RUT plus alumnos.len. Reuse existing listar (no new params).

### [Issue 6] [`egm-design`] - Severity: Blocker
*   **Affected Scope:** [Lines 82-82](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L82-L82) (E2E safelist)
*   **Discrepancy Description:** Safelist omits ~15 pinned selectors; wrap-only unenforceable; UTC suffix in fmt helpers unsafe in asserted cells.
*   **Rationale:** e2e asserts opp-name/col/stage, panel form Crear/Guardar, search-result, star-favorite, saved-view-select, automation-form, chat-history, secret-once, Deshabilitar/Anonimizar/Confirmar, data-dia includes text; suffix in data-dia truncates.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid, checkable in prueba.mjs.
*   ***Proposed Edit:***
    *   Location: Lines 82,156 + Failure Line 86
    *   Change: Expand safelist to full set; add opp-name stays descendant of opp-card, stage stays descendant of col, never rename asserted substrings. UTC suffix only in title/detail line, never in data-dia, opp-name, stage, ghost-new, data-correo.

### [Issue 7] [`egm-structural`] - Severity: Blocker
*   **Affected Scope:** [Lines 38-38](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L38-L38) (Q1 wording)
*   **Discrepancy Description:** Q1 leaves all copy unsigned (Path stages, guidance, Monto ref) while Goals promise ES+EN.
*   **Rationale:** Zero strings drafted; i18n precedent requires both arms together; cannot test unwritten copy. Actual wording needs author knowledge outside repo.
*   ***AI Suggestion:*** ESCALATE
*   ***Justification:*** Structure can be fixed, but exact ES/EN strings need author sign-off, not inferable from code.
*   ***Proposed Edit:***
    *   Location: Line 38
    *   Change: Keep Q1 as S1 merge blocker; add empty decision table skeleton (5 stages x ES/EN plus guidance plus Monto/Amount) for author to fill. No i18n diff without both arms plus sign-off.

### [Issue 8] [`egm-structural`] - Severity: Blocker
*   **Affected Scope:** [Lines 1-130](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L1-L130) (Traceability)
*   **Discrepancy Description:** Zero numbered features F1..Fn, no trace matrix CUJ-Goal-E2E.
*   **Rationale:** No ID to pass/fail; Goals and CUJs unlinked; antagonistic test Feature 47 fails.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid spec hygiene, human reader needs it, not only reviewer.
*   ***Proposed Edit:***
    *   Location: Goals + CUJs + Technical Plan intro
    *   Change: Restructure into F1 BarraEtapas, F2 CabeceraResumen, F3 Staleness, F4 GrupoTareas, F5 UTC, F6 ChipVista, F7 VistaPreviaRegla, F8 PistaLimite with ID, Description, Data contract, i18n keys, Selectors preserved, Acceptance Given-When-Then. Add trace CUJ1 to F1 plus E2E 343-387.

### [Issue 9] [`egm-structural`] - Severity: Blocker
*   **Affected Scope:** [Lines 64-86](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L64-L86) (Altitude leak)
*   **Discrepancy Description:** Components list with props, CSS classes, fmt helpers is zoom-in detail inside feature range; required per-feature fields absent.
*   **Rationale:** Comprehension and zoom-out confirm L64-86 belongs in appendix; unfielded prose is not spec; i18n and DB-string contracts missing per feature.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid altitude gap for human implementer.
*   ***Proposed Edit:***
    *   Location: Lines 64-86
    *   Change: Lift to capabilities (derivation rule, data contract, frozen selectors); move props and class names to Detailed appendix. Enforce per-feature template per Issue 8.

### [Issue 10] [`egm-codebase-details`] - Severity: Blocker
*   **Affected Scope:** [Lines 138-138](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L138-L138) (Vista shell)
*   **Discrepancy Description:** CabeceraResumen used in DetalleColegio, DetalleAlumno, DetalleApoderado, Vista shell with loaded struct props; Vista shell has zero hits in main.rs.
*   **Rationale:** Exist DetalleColegio 1374, DetalleAlumno 2523, DetalleApoderado 3377; Vista shell 0 hits; candidates DisposicionApp 145, BarraLateral 251.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid hallucination, checkable.
*   ***Proposed Edit:***
    *   Location: Line 138
    *   Change: Delete Vista shell or replace with DisposicionApp:145 with justification. Keep DetalleColegio, DetalleAlumno, DetalleApoderado only.

### [Issue 11] [`egm-codebase-details`] - Severity: Blocker
*   **Affected Scope:** [Lines 138-144](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L138-L144) (monto vs importe)
*   **Discrepancy Description:** Spec uses monto and Monto ref while codebase field is importe; i18n lists categories with zero identifiers.
*   **Rationale:** EGM and main.rs use importe (tbl th, detalle row, campo option); i18n has no monto; school-neutral would be Arancel ref.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid naming mismatch.
*   ***Proposed Edit:***
    *   Location: Lines 23,138,142-144
    *   Change: Use field name importe everywhere; decide display once in Q1 (Monto ref./Amount ref. or Arancel ref./Tuition ref.); list exact Textos fields for Es and En arms.

### [Issue 12] [`egm-codebase-details`] - Severity: Blocker
*   **Affected Scope:** [Lines 146-149](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L146-L149) (main.css)
*   **Discrepancy Description:** Generic chevron, sticky, stale, chip effects with no frozen classes or values; DOM vs CSS attribution wrong.
*   **Rationale:** main.css has opp-card, ghost-new, record-panel fixed+overflow, tbl, cal-grid, search-dialog, chat-layout, csv-result, chip row; missing highlights, stale, path-strip, clip-path. automation-card is DOM 1762, data-dia/data-correo are Dioxus attrs, not CSS.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid buildability gap.
*   ***Proposed Edit:***
    *   Location: Lines 146-149
    *   Change: Freeze Add .path-strip clip-path polygon, .path-current/.path-done, .highlights sticky top0 z-index opaque bg, .stale with threshold e.g. actualizado>14d else drop to S2, .chip-pill. Correct Keep DOM classes in main.rs and CSS rules in main.css. Ban style except width on bar-fill.

### [Issue 13] [`egm-codebase-details`] - Severity: Blocker
*   **Affected Scope:** [Lines 138-139](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L138-L139) (SelectorVistas chip)
*   **Discrepancy Description:** Active filter chip with clear reusing SelectorVistas; SelectorVistas has no chip, clear, or active-filter props.
*   **Rationale:** SelectorVistas renders select, delete, input, Guardar vista only; grouped search already exists 439-460; ChipVistaActiva zero hits.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid false reuse.
*   ***Proposed Edit:***
    *   Location: Line 138-139
    *   Change: Specify new ChipVistaActiva(vista:Option String,busqueda:String,on_limpiar:Callback) above .search-dialog, Clear resets busqueda plus seleccionada None reuses on_cargar, wrapping div only outside .search-result. Or drop reusing claim.

### [Issue 14] [`egm-style`] - Severity: Blocker
*   **Affected Scope:** [Lines 152-156](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L152-L156) (Verification toolchain)
*   **Discrepancy Description:** Disjunctive dx build vs cargo build per toolchain, duplicated cargo test, manual checks as gate, ports only, no S1/S2/S3 gates.
*   **Rationale:** Canonical EGM 36 is cargo test plus clippy plus node e2e; prueba needs frontal 8080 API 3001 Chrome; dx vs cargo ambiguous; manual not CI.
*   ***AI Suggestion:*** PROPOSE FIX
*   ***Justification:*** Valid untestable gate.
*   ***Proposed Edit:***
    *   Location: Lines 152-156
    *   Change: Pin cargo test --workspace plus cargo clippy --workspace -- -D warnings plus node e2e/prueba.mjs with FRONTAL 127.0.0.1:8080 API 127.0.0.1:3001, 122/122 green, no stacking on red, S1/S2/S3 each gated, dx build local-optional.

### [Issue 15] [`egm-style`] - Severity: Nit
*   **Affected Scope:** [Lines 86-86](/home/juan/dev/occb-crm/docs/designs/funnel-salesforce-ui.md#L86-L86) (Unknown etapa fallback)
*   **Discrepancy Description:** Unknown etapa falls back to first stage label is impossible with exhaustive enum plus DB constraint.
*   **Rationale:** EtapaOportunidad exhaustive, invalid DB rejected at deserialization.
*   **AI Suggestion:** DECLINE
*   **Justification:** Fact agreed but no contract break; fix only if editing nearby lines to avoid churn.
*   ***Proposed Edit:***
    *   Location: Line 86
    *   Change: No change now; if touched, replace with Exhaustive match, invalid rejected at deserialization.
