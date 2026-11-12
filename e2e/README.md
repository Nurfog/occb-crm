# `e2e` Blueprint

## System Overview

Suite de extremo a extremo del CRM escolar (`node e2e/prueba.mjs`, sin dependencias): levanta Chrome headless vía CDP, recorre el frontal y la API con usuarios y filas dedicados (sello `E2E-<base36>`, sin tocar datos de usuario) y cierra con `E2E: TODO CORRECTO` o `E2E: N FALLOS` (código de salida 0/1).

## Local Interface & Invariants

- Requisitos: frontal en `:8080`, API en `:3001` y Chrome en `/opt/google/chrome/chrome` (reescribibles vía `FRONTAL_URL`, `API_URL`, `CHROME_BIN`); el mock LLM vía `LLM_MOCK_API_URL` y webhooks vía `WEBHOOK_HOST` (`[prueba.mjs:FRONTAL]`, `[prueba.mjs:API]`, `[prueba.mjs:CHROME]`, `[prueba.mjs:LLM_MOCK_API_URL]`).
- Ayudantes CDP: `dormir`, `esperarUrl`, `enviar`, `evaluar`, `esperarCondicion`, `ir`, `loginUI`, `texto`, `afirma` (122 afirmaciones con `PASS`/`FAIL` por línea) (`[prueba.mjs:afirma]`).
- Aislamiento: `reservaPuerto` efímero para CDP, perfil Chrome en `/tmp/opencode/e2e-chrome-<puerto>`, receptor webhook en `:18766`, usuarios `e2e-<SELLO>@test.cl` y colegios/oportunidades `E2E-<SELLO>` dedicados (`[prueba.mjs:SELLO]`, `[prueba.mjs:reservaPuerto]`).
- Cobertura: acceso ES/EN, panel vivo, colegios (API + ficha), kanban persistido, edición en fichas, territorio chileno, tareas/notas, calendario, correos, miembros/roles, automatizaciones, asistente mock, claves, webhooks e importación CSV.

## Subdirectories & System Boundaries

- (sin subdirectorios: solo `prueba.mjs`, 122 afirmaciones).

## Global Invariants

- Verde exigido antes de cerrar cada fase; las filas E2E se crean y borran dentro de la propia corrida.

## Global Side Effects

- Lanza y mata Chrome headless; crea/modifica/borra registros dedicados vía API; abre un receptor HTTP temporal para webhooks.

## System Verification

- Run: `WEBHOOK_HOST=host.docker.internal LLM_MOCK_API_URL=http://127.0.0.1:3002 node e2e/prueba.mjs`
