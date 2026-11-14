# SITEMAP de diseños — OCCB CRM

Índice de RFC y documentos de diseño en `docs/designs/`.

- [Auditoría OCCB CRM](./auditoria-occb-crm.md) — auditoría por crate + transversal (seguridad, prácticas, bugs, mejoras), con fases de remediación. Estado: RFC publicado 2026-11-12, pendiente `egm-draft`.
- [Funnel Salesforce UI (solo UI)](./funnel-salesforce-ui.md) — 2026-11-12, Nurfog. Path solo lectura + highlights + chips/sentence-preview en Dioxus, sin backend, E2E 122 en verde. Componentes: `main.rs`, `i18n.rs`, `main.css`.
- [SaaS Colegios Twenty](./saas-colegios-twenty.md) — 2026-11-12, Nurfog. CRM admisión como módulo del core escolar, CSV inbox, matrícula atómica, pool marketing con consentimiento, licencias 1/2-5/holding, holdings con informes agregados, colegios aislados, UI Twenty mínima. Tenancy-first, 122 E2E en verde.
- [SaaS Tenancy Matriculación (diseño)](./saas-tenancy-matriculacion.md) — 2026-11-12, Nurfog. 029 tenant fundación + 030 matrícula y consentimiento, API con scope, CSV cuarentena, frontend Twenty mínimo, i18n pareado, 122 E2E por etapa.
