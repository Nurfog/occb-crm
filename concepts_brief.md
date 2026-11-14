# OCCB Twenty-Style SaaS with Holdings and Modules

* **Author:** Nurfog
* **Date:** 2026-11-12

## Overview

Expand OCCB from single-school CRM into school-core SaaS with admission as an optional module, CSV fallback, and holding-level view. Small license covers one school, group covers two to five with combined totals, holding covers six or more with custom price. Prospect data fills alumno, apoderados, and salud at matriculation; non-matriculated stays as marketing pool with consent and expiry. Holdings see aggregated ingresos, resultados, and metas plus individual schools by grant; schools see only own contexto with dominio, accesos, and web link and no cross-compare. UI follows Twenty patterns for views, pipelines, command menu, search, favorites, filters, saved views, workflows, and import.

## Analysis

No tenancy primitive exists. The 28 migrations carry no colegio or holding scope, JWT carries no scope, listar, panel, buscar, and timeline read global state, and RLS has no policies behind SECURITY DEFINER functions. Holding per-grant view and school own-only view cannot be frontend filters. A tenant model with scope parameter and claim checks must come before any SaaS interface.

Directory and tenant are conflated. Colegios holds the 12k MINEDUC directory with RBD as key and public read for traslados. Adding licencia, dominio, contacto, representantes, and accesos onto it pollutes tenant queries and locks the seed. Contracted schools need a separate table with RBD foreign key, licencia, dominio, and flags plus user memberships; the directory stays read-only.

No matriculation transaction exists. Oportunidad has nombre, importe, etapa, colegio_rbd, and dead contacto reference with no RUT, apoderados, or salud. Crear alumno requires valid RUT. No atomic matricular desde postulacion function exists. Separate client calls risk partial matricula, duplicate RUT errors, and orphan prospects.

RUT scope breaks multi-school use. Unique indexes are global where activo, reusable after disable, and the 00000000-0 sentinel passes validation. The same RUT across two schools in a holding collides or leaks. Prospect has no RUT field so pre-matricula dedup is impossible. A choice is needed between global RUT with traslado and scoped RUT plus quarantine, with no merge engine planned.

Marketing pool without consent is liability. Perpetual retention of non-matriculated minors with auto-filled salud plus RUT and contact and no origen, consent, or TTL conflicts with the soft-delete law and data rules. Anonymized pool has no marketing value; non-anonymized pool carries risk. Opt-in per channel, 12 to 24 month auto-anonymize, no salud retention for non-matriculated, and suppression list are needed.

Core inversion is undefined. No school-core tables for cursos, matricula, asistencia, or calificaciones exist. What is mandatory at conversion, who validates SIGE and RBD authority, and what marketing data can legally enroll remain open. Prospect data is not enrollment.

Licensing has no encodable base. No planes, suscripciones, metas, or ingresos tables exist; panel sums global valor_tuberia and valor_ganado. The 1, 2 to 5 unified, and negotiable holding tiers have no value metric per alumno, matricula, or usuario. The ISP analogy confuses sostenedor buyer and open negotiable terms erode margin.

Aggregation semantics are missing. Ingresos, resultados, and metas per scope plus unified 2 to 5 results lack sum versus average, currency, academic period, etapa homologation, and importe versus real arancel rules. Holding drill-down against school no-compare is asymmetric trust; one mis-scoped query leaks personal and finance data.

CSV fallback becomes second truth. Import is fixed 5 MB, 1000 rows, preview five, per-row crear with fixed rut and nombres headers, no mapping, no salud or vinculo or colegio_rbd, no prospect import. CSV-direct alumno diverges from prospect to matriculado on validation, audit, and automation and becomes shadow CRM without sunset.

The test and code shape gates change. One E2E file pins selectors and text plus timing; main.rs is monolith with client-side fetch-then-filter and paired ES and EN that fails build if one arm is missing. Any scope switcher, view builder, mapper, or dominio editor without freeze, split, and sharding turns the suite red. Search, filters, views, workflows, per-school customization, language debt, client fetch patterns, auth fragility, observability gaps, migration safety, and market mismatch all raise cost if ignored. Schoolccb migration stays unscoped without RBD canonicalization and licencia clash handling; its CRM stays out, with explicit matriculado handoff, not shared tables.

## Goals vs. Non-Goals

* **In-Scope Goals:** Define core as enrolled alumnos plus colegios; admission as optional funnel feeding core; without it CSV inbox only; add tenant foundation with contracted schools plus memberships plus scoped listar, panel, buscar; holding totals plus drill-down with school own-only enforced in SQL; CSV as quarantined inbox with RUT check plus RBD resolve plus consent flag leading to contacto postulacion, not direct loader; single matricular postulacion transaction idempotent reusing forward arrow; non-matriculated stays activo with origen prospecto plus consent plus TTL; keep plain language Small one school, Group two to five combined totals, Holding six plus custom price; Embudo as postulantes per etapa not money; RUT with DV unique per activo; ES default plus paired EN before ship; Deshabilitar plus Anonimizar confirm preserved; 122 green gated per stage.
* **Out-of-Scope Non-Goals:** No custom dominio hosting, no pagina-web builder or CMS, no custom permission matrix, no ISP discount engine or billing metering, no unified BI or forecasting, no Flow canvas, no drag kanban or calendar, no week or day views, no threading, no column configurator or sharing, no merge engine, no cross-colegio compare for colegio role, no backup-scheduling UI.

## Alternatives Weighed

* Keep CRM standalone plus handoff — Recommended baseline. OCCB stays admission CRM and emits matriculado event or API and webhook to SIS core. CSV as 90-day sunset migration tool with RUT quarantine. Least scope, preserves 122 checks, avoids SIS absorb. Rejected as sole outcome only because holding asks for combined view; kept as fallback if tenancy turns red.
* Invert with CRM as core — Rejected. Admission kernel is tenant and asistencia, notas, arancel subscribe to matriculado. Kills dual source with one RUT namespace and one audit. Cost is schools wanting only notas pay for CRM; schoolccb admin becomes adapter.
* Pool as consent asset — Rejected. Non-convertidos anonymized with hashed RUT plus comuna and nivel only form intra-holding demand routing plus apoderado-controlled re-contact. Monetizes loss. Cost is hardest privacy lift needing consent ledger plus settlement.
* Per-matriculado pricing — Rejected. Zero base plus price per effective matriculado with mover_etapa to matriculado as billable ledger plus CSV free as funnel; holdings pay success. Destroys negotiation overhead and fits marzo cashflow. Cost is anti-gaming for reversos and traslados double-count, cobranza, variable-cost fear, and immutable ledger conflicting with soft-delete reuse.

## Key Decisions & Staging for Next Steps

* Tenancy before Twenty. Freeze selectors and text, gate on clippy plus test plus E2E, split main.rs per view with no behavior change. Then tenant plus licencia schema plus schoolccb admin read-only. Then CSV inbox idempotent per tenant. Then matriculation transaction plus retention policy. No SaaS interface until tenant green.
* Language stays school-first. Oportunidades route stays for compat with Admision display; importe shows Monto ref. and Amount ref. or Arancel ref. and Tuition ref. with both keys together; holding vocab for ingresos, resultados, metas defined as formulas before UI.
* Consent plus RUT scope gates work. Add nullable validated prospecto_rut not unique plus origen, consentimiento, TTL; salud staged only at secretaria confirmation as matricula draft, never direct insert from marketing form; history survives anonymize with contact scrubbed and re-import resurrects not duplicates.
* Visibility enforced server-side. Panel, buscar, and listar take scope; holding sees totals plus each school on grant, colegio sees own; benchmark as anonymized percentiles if comparison demanded, no named neighbor leak.
* Twenty minimal first. Path strip plus highlights plus urgency dot plus task grouping plus UTC suffix plus active-view chip plus rule sentence plus static chat limit hint. Defer AND/OR builder with guided chips first, mapper, workflows expansion, custom domains. Preserve all E2E classes and text with wrapping divs only, revert on first red, no stacking.
