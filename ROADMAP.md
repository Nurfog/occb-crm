# Hoja de ruta — OCCB CRM

Marcas: `⬜` pendiente · `🚧` en proceso · `✅` terminada (también en las fases).
Cómo leer este archivo: cada fase lista exactamente qué se implementa en base de
datos, API, frontal y pruebas. Una fase solo se marca hecha con su E2E en verde,
clippy sin avisos, tests en verde y push al remoto.
Idioma por defecto español con i18n ES/EN en todo lo visible.

## Completado

### ✅ Fase 0 — Base del workspace
- ✅ Workspace Cargo Rust 2024 con `core` (dominio + validación + i18n + semilla),
  `api` (Axum) y `app` (Dioxus 0.7 + Router).
- ✅ Docker: PostgreSQL 18 con healthcheck, Redis 8, volúmenes nombrados.
- ✅ Dominio con validación pura y 6+ tests (`cargo test`), errores en español con
  formato único `{ codigo, mensaje }`.
- ✅ Convención de seguridad: la API nunca escribe con SQL directo; todo el CRUD
  pasa por funciones `SECURITY DEFINER` con validación y RLS activado.
- ✅ E2E en Chrome headless por CDP (los casos se describen en cada fase).

### ✅ Fase 1 — Núcleo funcional
- ✅ Base de datos (`migrations/001–006`):
  - ✅ Tablas `empresas`, `personas`, `oportunidades` con CHECK, índices y RLS.
  - ✅ Funciones: `listar_*`, `obtener_*`, `crear_*`, `actualizar_empresa`,
    `eliminar_*`, `mover_etapa_oportunidad`, `panel_resumen`, `sembrar_datos_demo`.
  - ✅ Permisos `EXECUTE` al público (en producción: rol `occb_app` mínimo).
- ✅ API (`crates/api` modular: `configuracion`, `errores`, `bd`, `rutas_*`):
  - ✅ `GET /api/salud`, `GET /api/panel` (resumen calculado en PG).
  - ✅ `GET/POST /api/empresas`, `GET/PATCH/DELETE /api/empresas/{id}`.
  - ✅ `GET/POST /api/personas`, `GET/DELETE /api/personas/{id}`.
  - ✅ `GET/POST /api/oportunidades`, `PATCH /api/oportunidades/{id}/etapa`,
    `DELETE /api/oportunidades/{id}`.
  - ✅ Alias en inglés (`/api/companies`, `/api/people`, `/api/opportunities`,
    `/api/dashboard`, `/api/health`).
  - ✅ Capas: `tracing`, CORS permisivo, compresión, timeout 10s con 408.
- ✅ Frontal (`crates/app` + `cliente.rs` con reqwest):
  - ✅ Rutas `/`, `/personas`, `/empresas`, `/oportunidades`, `/ajustes`, `/acceso`, 404.
  - ✅ Panel con tarjetas y salud de tubería; tablas con búsqueda; tarjetas de empresa;
    kanban de 5 etapas con ▶ que persiste; formularios de alta; bajas con ✕;
    selector de idioma ES/EN global; respaldo demo si la API no responde.
  - ✅ Contenedor multi-etapa (compilación WASM + nginx con fallback SPA), `:8080`.
- ✅ E2E 9/9 en verde (panel vivo, alta/baja por UI con persistencia en PG,
  kanban persistido, acceso en español). Cazó un bug real (204 sin cuerpo).

### ✅ Fase 1.5 — Completar lo construido
- ✅ Renombre de etapa «Cribado» → «Contacto» (migración 007 que actualiza
  restricción, filas, funciones y siembra).
- ✅ `actualizar_persona` + `actualizar_oportunidad` (migración 008) con
  `PATCH /api/personas/{id}` y `PATCH /api/oportunidades/{id}`.
- ✅ Formularios completos: empresas (nombre, dominio, sector, empleados, ciudad);
  personas (nombre, apellidos, correo, teléfono, puesto, ciudad, empresa);
  oportunidades (nombre, importe, empresa).
- ✅ Fichas detalle: `/empresas/:id` (edición + personas y oportunidades
  relacionadas con enlaces) y `/personas/:id` (edición + selector de empresa +
  enlace a su empresa).
- ✅ Edición inline en tarjetas del kanban (✎ nombre e importe).
- ✅ E2E 11/11 en verde (las 9 anteriores + edición en ficha de persona y de
  empresa con persistencia verificada en PG).

## Siguiente (por orden)

### ✅ Fase 1.6 — Territorio chileno (Región → Comuna)
- ✅ Dataset oficial (16 regiones + 346 comunas CUT) validado y generado a código.
- ✅ `core/territorio` con tests (regiones, comunas por región, validación cruzada).
- ✅ Migración 009: tablas `regiones`/`comunas` + seed + columnas en
  empresas/personas + CRUD actualizado + `listar_regiones`/`listar_comunas_region`.
- ✅ Endpoints `GET /api/regiones` y `GET /api/comunas?region=`.
- ✅ Selector Región→Comuna reutilizable en altas y fichas (filtra comunas,
  autocompleta ciudad, valida coherencia).
- ✅ E2E: cascada filtra + guardado persiste códigos (pruebas 12–13).

### ✅ Fase E1 — Colegios y Admisión (track escolar)
Objetivo: el CRM habla de colegios, no de empresas; el embudo es admisión.
Se ignoran las tablas `crm_sales_*` del sistema escolar externo: el vínculo con
personas es nuestro.

- ✅ Directorio oficial (12.038 vigentes, RBD como clave) validado: comunas
  calzan 100% con las 346 y regiones con las 16.
- ✅ Migración 010: tabla `colegios` + seed + `listar_colegios` (filtros por
  región/comuna/texto, ideal para traslados) + `obtener/crear/eliminar_colegio`.
- ✅ Retiro total de Empresas (tabla, funciones, API, UI, dominio).
- ✅ Oportunidades → Admisión: etapas contacto/postulante/evaluado/aceptado/
  matriculado + columna `colegio_rbd`; personas sin `empresa_id`.
- ✅ UI: Colegios en el menú (directorio filtrable + registro + ficha con
  postulaciones), kanban de admisión, panel cuenta colegios.
- ✅ E2E 15/15 en verde (colegios CRUD por UI, kanban de admisión, fichas, territorio).

### ✅ Fase E2 — Alumnos y Apoderados
- ✅ Tablas `alumnos` (RUT único con dígito verificador, nombres, curso/nivel,
  colegio RBD, estado: postulante/matriculado/trasladado/retirado) y `apoderados`
  (RUT único, nombres, contacto, territorio) + `alumno_apoderado` (vínculo,
  retiro autorizado, notificaciones).
- ✅ Validación RUT módulo 11 en núcleo + funciones PG.
- ✅ CRUD completo + fichas (alumno con apoderados y colegios; apoderado con alumnos).
- ✅ UI reemplaza Personas por Alumnos + Apoderados en el menú.
- ✅ E2E: alta con RUT válido/inválido, vínculo apoderado, fichas cruzadas.

### ✅ Fase E3 — Traslados
- ✅ Tabla `traslados` (alumno, origen RBD, destino RBD, fecha, motivo) +
  `trasladar_alumno()` (valida destino ≠ origen, actualiza colegio, registra historial).
- ✅ UI: desde la ficha del alumno, buscador de colegios destino por región/comuna.
- ✅ E2E: traslado persiste colegio nuevo + historial visible.

### ✅ Fase 1.7 — Parecido visual
- [*] Tokens oficiales (fondos, bordes, grises, radios, acento) + Inter.
- [*] Barra lateral fiel (224px, secciones, kbd ⌘K, estados activos).
- [*] Tablas densas con checkbox, fila fantasma + Nuevo y formularios plegados.
- [*] Tabs Tabla/Kanban reales en Admisión.
- [*] Fichas como panel lateral derecho (500px) con cierre.
- [*] E2E versionada en `e2e/prueba.mjs` (se ejecuta con `node e2e/prueba.mjs`).
- ✅ E2E 13/13 en verde tras el rediseño + capturas de verificación.
- ✅ Fichas reactivas al pinchar varios registros (efecto suscrito al id de ruta).
- ✅ Fidelidad al original montado localmente: chips, footer de stats, kanban rico
  ($/responsable/fecha/colegio/contacto), tabs Tabla/Kanban, "+ Nuevo" por columna.

### ✅ Fase 2 — Tareas + Notas
Objetivo: los dos objetos de trabajo diario, relacionados con todo lo demás.

- ✅ Base de datos (migración 013):
  - ✅ Tabla `tareas`: `id`, `titulo` (1–200, obligatorio), `detalle`, `estado`
    (`pendiente|en_curso|hecha`, por defecto `pendiente`), `prioridad`
    (`baja|media|alta`, por defecto `media`), `vencimiento` (anulable),
    `persona_id`, `empresa_id`, `oportunidad_id` (anulables, `on delete set null`),
    `creado_en`. Índices por estado, vencimiento y cada referencia. RLS.
  - ✅ Tabla `notas`: `id`, `titulo`, `cuerpo` (obligatorio no vacío),
    `persona_id`, `empresa_id`, `oportunidad_id` (anulables), `creado_en`.
    Índice GIN trigram (`pg_trgm`) para búsqueda por texto. RLS.
  - ✅ Funciones por objeto: `listar_*` (búsqueda + filtros por estado/empresa +
    paginación), `obtener_*`, `crear_*` (validan título y referencias),
    `actualizar_*`, `eliminar_*`, más `marcar_tarea(id, estado)`. Todas
    `SECURITY DEFINER` con mensajes en español y `GRANT EXECUTE`.
- ✅ API:
  - ✅ `GET/POST /api/tareas`, `GET/PATCH/DELETE /api/tareas/{id}`,
    `PATCH /api/tareas/{id}/estado`.
  - ✅ `GET/POST /api/notas`, `GET/PATCH/DELETE /api/notas/{id}`.
  - ✅ Validación Rust espejo de la SQL (título obligatorio, estado/prioridad válidos).
- ✅ Frontal:
  - ✅ Entradas de menú Tareas y Notas (quitan el atenuado), rutas `/tareas`, `/notas`.
  - ✅ Vista Tareas: lista con filtro por estado, ▶ para avanzar estado,
    alta con vencimiento y prioridad, edición inline, baja.
  - ✅ Vista Notas: lista con búsqueda de texto, editor de alta/edición, baja.
  - ✅ Fichas de alumno muestran sus tareas y notas con enlaces.
  - ✅ Contador de pendientes en el menú.
- ✅ E2E (Chrome): alta de tarea por UI persiste; avance de estado persiste;
  alta y edición de nota persisten; aparecen en la ficha relacionada.
- ✅ Hecho cuando: E2E 25/25, clippy limpio, migración 013 aplicada en fresco
  (`down -v` + `up` + arranque API sin errores).

### ✅ Fase 3 — Acceso real + miembros
Objetivo: multiusuario de verdad; base para permisos por usuario.

- ✅ Base de datos (migraciones 014 + 015):
  - ✅ Tabla `usuarios`: `id`, `nombre`, `correo` único (formato válido,
    se guarda en minúsculas), `clave_hash` (argon2, solo lo escribe la
    función), `rol` (`admin|miembro`, por defecto `miembro`, el primero
    del espacio nace admin), `creado_en`. RLS.
  - ✅ Tabla `sesiones`: `id`, `usuario_id` (`on delete cascade`),
    `renovacion_hash` (sha256 del token de renovación), `expira_en`,
    `creado_en`. Limpieza de expiradas en cada uso.
  - ✅ Funciones: `registrar_usuario` (valida, rechaza duplicados P0003),
    `credencial_usuario` (devuelve hash para verificar en Rust, sin exponerlo),
    `crear_sesion`, `rotar_sesion` (rotación con reuso detectado → revoca todo;
    015 corrige que el borrado persista sin `RAISE`), `cerrar_sesion`.
    (`cerrar_todas_sesiones` no hizo falta: el reuso ya revoca todo.)
- ✅ API (nuevas dependencias: `argon2`, `password-hash`, `sha2`, `jsonwebtoken`):
  - ✅ `POST /api/acceso/registro`, `POST /api/acceso/login` (devuelve acceso
    corto 15 min con `jti` único + `sesion_id` + renovación en el cuerpo JSON
    para la SPA, además de cookie `HttpOnly`), `POST /api/acceso/renovar`
    (acepta cuerpo o cookie), `POST /api/acceso/salir`, `GET /api/acceso/yo`.
  - ✅ Middleware que exige JWT: públicas solo salud, acceso, regiones/comunas,
    colegios y panel; el resto exige sesión (401 `sin_sesion`).
  - ✅ `GET /api/miembros`, `PATCH /api/miembros/{id}/rol` (solo admin,
    403 `prohibido`).
  - ⬜ RLS por usuario → se movió a Fase 8 (las funciones `SECURITY DEFINER`
    + RLS base ya aíslan el acceso; el JWT lleva el rol hasta 15 min).
- ✅ Frontal:
  - ✅ Pantalla `/acceso` con pestañas de login y registro (sin ruta separada
    ni casilla de privacidad: no se pide nada opcional), sesión solo en memoria
    (señal global, nunca almacenamiento local), renovación silenciosa con
    reintento ante 401, cierre en el menú, nombre y rol en la barra, guardia
    que redirige a acceso recordando el destino (VOLVER).
  - ✅ Vista `/miembros` (lista, cambio de rol para admin).
- ✅ E2E: registro + login + ver yo; renovación sin re-login; reuso que revoca;
  rutas protegidas sin token devuelven 401 y el frontal redirige a acceso;
  admin cambia rol (403 para miembro); registro y guardia por UI.
- ✅ Hecho cuando: E2E 40/40, sin secretos en logs ni respuestas, clippy limpio.
- ⬜ Hecho cuando: E2E 20/20, sin secretos en logs ni respuestas, `down -v` limpio.

### ✅ Fase 4 — Calendario + Correos
Objetivo: agenda y bandeja vinculadas a los registros.

- ✅ Base de datos (migraciones 016 + 017):
  - ✅ Tabla `eventos`: `id`, `titulo` (1–200), `inicio`/`fin` (`fin ≥ inicio`),
    `lugar`, `descripcion`, `alumno_id`, `apoderado_id`, `oportunidad_id`
    (anulables, en cascada), `creado_en`. Índices por rango de fechas. RLS.
    (Adaptación escolar: sin `empresa_id`; `persona_id` se abrió en
    alumno/apoderado.)
  - ✅ Tabla `correos`: `id`, `asunto`, `cuerpo` obligatorio, `remitente`
    (formato correo), mismos tres vínculos, `leido` (falso por defecto),
    `recibido_en`, `creado_en`. RLS.
  - ✅ Funciones CRUD completas + `listar_eventos_mes(año, mes)` +
    `marcar_correo_leido`. Todo `SECURITY DEFINER` en español.
  - ✅ 017 corrige el regex de remitente de 016 (doble barra excluía la "s").
- ✅ API:
  - ✅ `GET/POST /api/eventos`, `GET /api/eventos/mes?año=&mes=`,
    `GET/PATCH/DELETE /api/eventos/{id}`.
  - ✅ `GET/POST /api/correos` (filtro `no_leidos`), `GET /api/correos/{id}`,
    `PATCH /api/correos/{id}/leido`, `DELETE /api/correos/{id}` (extra para
    higiene de filas E2E).
- ✅ Frontal:
  - ✅ Ruta `/calendario`: rejilla mensual navegable (lunes primero), día
    seleccionado con sus eventos, alta/edición/baja, puntos en días con
    eventos. Horas en UTC de ida y vuelta (un solo huso).
  - ✅ Ruta `/correos`: lista con no-leídos destacados, lector, marcar
    leído/no-leído, redacción vinculada a alumno/apoderado/postulación,
    contador en el menú (se abre marcando leído).
  - ✅ Fichas de alumno y apoderado muestran eventos y correos relacionados.
- ✅ E2E: evento por UI aparece en su día; correo vinculado aparece en la
  ficha del apoderado; leído persiste; contador en menú; validaciones.
- ✅ Hecho cuando: E2E 55/55, clippy limpio, mes de 31 días renderiza bien.

### ✅ Fase 5 — Búsqueda global + Favoritos + Vistas + Timeline
Objetivo: moverse por el espacio como en un CRM grande.

-- ✅ Base de datos (migraciones 018 + 019):
  - ✅ Búsqueda FTS española + respaldo para prefijos/consultas de 1 carácter en
    colegios, alumnos, apoderados, personas, postulaciones, tareas, notas,
    eventos y correos. `buscar_espacio(texto)` ordena por relevancia y limita 20.
  - ✅ Favoritos personales para alumno/apoderado/persona/postulación;
    vistas personales para alumnos/apoderados/personas/postulaciones
    (tabla o Kanban de postulaciones). 019 normaliza `null` JSON a `{}`.
  - ✅ Actividad y triggers acotados (no guardan RUT, teléfono ni cuerpo de
    correo); las escrituras se ejecutan con transacción y `app.usuario_id` local,
    que atribuye el autor desde el JWT sin aceptar IDs del cliente.
- ✅ API protegida: búsqueda, listar/alternar favoritos, CRUD vistas y timeline.
  Las colecciones personales se filtran por el usuario autenticado.
- ✅ Frontal:
  - ✅ ⌘K/Ctrl+K abre búsqueda agrupada; Enter abre el primer resultado.
  - ✅ Estrellas en listas y fichas; favoritos vivos en la barra lateral.
  - ✅ Guardar/aplicar/eliminar vistas personales en listas; Kanban donde aplica.
  - ✅ Timeline con autor, acción y fecha en fichas de persona, alumno,
    apoderado y postulación.
- ✅ E2E: encuentra por nombre/correo, tolera un carácter, favoritos y vistas
  quedan aislados por usuario, favoritos sobreviven navegación, cambio de persona
  aparece en timeline con el autor real.
- ✅ Hecho cuando: E2E 69/69 y clippy limpio.

### ✅ Fase 6 — Automatizaciones
Objetivo: reglas sencillas sin programar.

- ✅ Base de datos (migración 020): reglas limitadas a postulaciones escolares,
  condiciones guiadas (`igual|contiene|mayor_que`) y acciones seguras
  (`mover_etapa|crear_tarea|crear_nota`), con historial de ejecuciones.
- ✅ API:
  - ✅ `GET/POST /api/automatizaciones`, `GET/PATCH/DELETE /api/automatizaciones/{id}`,
    `PATCH /api/automatizaciones/{id}/activa`, `GET /api/automatizaciones/{id}/ejecuciones`.
  - ✅ Motor al crear, actualizar o mover postulaciones. Una subtransacción por
    regla revierte sus acciones si falla, registra el error y conserva la operación
    original; las acciones usan funciones SQL seguras existentes.
- ✅ Frontal:
  - ✅ Ruta `/automatizaciones`: lista, activar/desactivar, editor guiado y
    ejecuciones por regla.
- ✅ E2E: «al crear postulación con importe > 100000 → crear tarea» genera tarea
  vinculada; desactivarla detiene nuevas tareas; error de regla queda registrado
  sin revertir la postulación.
- ✅ Hecho cuando: E2E 75/75, clippy limpio y fallo aislado por regla.

### ✅ Fase 7 — Agentes IA + chat
Objetivo: pedirle cosas al CRM conversando.

- ✅ Base de datos (migración 021): `conversaciones` y `mensajes`, aislamiento
  por usuario en todas las funciones `SECURITY DEFINER`, límites de historial.
- ✅ API: CRUD/listado personal y `POST /api/conversaciones/{id}/preguntar`.
  Cliente OpenAI-compatible configurable por `LLM_URL`, `LLM_CLAVE`, `LLM_MODELO`;
  sin configuración responde 503 `no_configurado` sin interrumpir el API.
  Herramientas cerradas y seguras: buscar registros, resumen real del panel,
  crear nota y crear tarea; conserva fuentes citables y las devuelve con enlaces.
  Proveedor `mock://occb-e2e` disponible solo en builds de desarrollo para pruebas.
- ✅ Frontal: ruta `/agentes`, historial por conversación, creación/baja,
  indicador de espera, manejo ES/EN y fuentes navegables.
- ✅ E2E: proveedor simulado consulta la cifra real de tubería y crea/cita nota;
  sin LLM configurado presenta el mensaje esperado y el resto del espacio sigue
  operativo; conversaciones aisladas por usuario.
- ✅ Hecho cuando: E2E total 82/82 y clippy limpio.

### ✅ Fase 8 — Plataforma y producción
Objetivo: que terceros integren y que operar sea aburrido (en el buen sentido).

- ✅ Integraciones (migraciones 022 + 024 + 025):
  - ✅ `claves_api`: hash sha256 único, secreto solo en la respuesta de creación,
    alcances (`*|lectura|escritura`), revocación; `X-Clave-API` autentica con
    permisos del dueño (solo lectura bloquea escrituras con 403).
  - ✅ `webhooks` + `entregas_webhook`: https obligatorio salvo loopback local
    (incluye `host.docker.internal`, migración 025); secreto de firma por
    webhook; worker con `SKIP LOCKED`, firma HMAC-SHA256 y 3 intentos con
    espera 5s/10s; el trigger encola actividad del propio autor.
  - ✅ UI en Ajustes: claves (crear/ver una vez/revocar) y webhooks
    (crear/probar/ver entregas).
- ✅ Importación CSV (migración 023):
  - ✅ `POST /api/importar/alumnos|apoderados` (multipart, ≤5 MB, ≤1000 filas):
    valida encabezados, previsualiza 5 filas con errores por fila y al aplicar
    crea las válidas reportando las malas por número de fila.
  - ✅ UI con archivo o pegado, previsualización y reporte.
- ✅ Operación:
  - ✅ Logs JSON con `LOG_FORMAT=json`; `GET /api/salud` extendido (versión,
    migración aplicada, latencia de PG, estado degradado si PG cae);
    límite 60 logins/min por IP (429 `demasiadas_solicitudes`).
  - ✅ Trazabilidad por usuario: `app.usuario_id` local por transacción en
    escrituras (autoría de actividad y webhooks); colecciones personales
    filtradas por JWT/clave.
  - ⬜ Respaldos programados y multi-espacio: fuera de alcance (ver nota).
- ✅ E2E: CSV con 1 fila mala importa 2/3 y la reporta; entrega firmada
  verificada punto a punto en local y ciclo con reintentos en docker;
  clave revocada devuelve 401; salud extendida verificada.
- ✅ Hecho cuando: E2E 95/95 en despliegue docker reconstruido.
- ⬜ Nota de alcance: respaldos `pg_dump` programados, RLS por fila con
  políticas y multi-espacio quedan para después del roadmap (la API ya
  aísla por usuario autenticado en funciones `SECURITY DEFINER`).

### ✅ Fase 9 — Ficha escolar ampliada
Objetivo: antecedentes de salud en la ficha del alumno y ocupación del
apoderado. El CRM de schoolccb queda reemplazado por este proyecto: no se
integra nada de ese CRM externo (tampoco `crm_sales_*`); la ficha escolar
vive aquí.

- ✅ Base de datos (migración 026): `alumnos.grupo_sanguineo` (lista cerrada),
  `alergias`, `medicamentos` (≤500), `antecedentes_medicos` (≤2000),
  `contacto_emergencia` (≤200); `apoderados.ocupacion` (≤120). Funciones
  `crear/actualizar` con una sola firma vigente (se retiran las anteriores).
- ✅ Base de datos (migración 027): `ingresos_medicos` 1:N por alumno
  (fecha, motivo obligatorio, detalle, profesional) con CRUD completo.
- ✅ API: cuerpos y mapeos extendidos; validación espejo en Rust.
- ✅ Frontal: sección Salud reutilizada en alta y edición de alumno, panel
  Salud en la ficha, ocupación en alta/ficha de apoderado (ES/EN).
- ✅ Frontal: panel Ingresos médicos en la ficha (alta/baja, historial
  ordenado por fecha).
- ✅ E2E: alta con salud por API y UI, grupo inválido rechazado, lectura
  en ambas fichas, limpieza de filas propias.
- ✅ E2E: dos ingresos por API, validación de motivo, alta/baja por UI,
  persistencia verificada.
- ✅ Hecho cuando: E2E 108/108, clippy limpio.

## Mapa de módulos → estado

| Módulo | Estado | Fase |
|---|---|---|
| Panel | ✅ Funcional | 1 |
| Personas | ✅ Funcional (CRUD + ficha) | 1 + 1.5 |
| Empresas | ✅ Funcional (CRUD + ficha) | 1 + 1.5 |
| Oportunidades | ✅ Funcional (CRUD + kanban + inline) | 1 + 1.5 |
| Fichas y relacionados | ✅ Funcional | 1.5 |
| Regiones y comunas | ✅ Funcional | 1.6 |
| Colegios y Admisión | ✅ Funcional | E1 |
| Alumnos y Apoderados | ✅ Funcional | E2 |
| Traslados | ✅ Funcional | E3 |
| Tareas | ✅ Funcional | 2 |
| Notas | ✅ Funcional | 2 |
| Usuarios y acceso | ✅ Funcional (JWT + rotación) | 3 |
| Miembros y roles | ✅ Funcional (solo admin cambia rol) | 3 |
| Calendario | ✅ Funcional | 4 |
| Correos | ✅ Funcional | 4 |
| Búsqueda global | ✅ FTS + ⌘K | 5 |
| Favoritos | ✅ Personal por usuario | 5 |
| Vistas guardadas | ✅ Filtros por usuario | 5 |
| Timeline | ✅ Autoría y trazabilidad | 5 |
| Automatizaciones | ✅ Reglas de admisión con historial | 6 |
| Agentes IA y chat | ✅ Chat escolar con herramientas seguras | 7 |
| Claves API y webhooks | ✅ Firmados, con reintentos | 8 |
| Importación CSV | ✅ Parcial con reporte | 8 |
| Respaldos y operación | ✅ Salud, ritmo y logs (sin programar) | 8 |
| Ajustes | ✅ Claves, webhooks e importación | 3 + 8 |
| Ficha escolar ampliada | ✅ Salud y ocupación | 9 |

## Reglas de cada fase (no negociables)
1. Primero la base de datos (tablas + funciones seguras + RLS + migración nueva;
   jamás editar migraciones aplicadas).
2. Después la API (handlers delgados, validación espejo, errores en español).
3. Después el frontal (ES/EN, conectado a la API, sin mocks salvo respaldo).
4. E2E en Chrome que verifique persistencia en PG antes de darla por hecha.
5. `cargo clippy -- -D warnings`, tests en verde, commit + push.
6. Actualizar este archivo (mover marcas y tachar pendientes).
