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

### [*] Fase 1.6 — Territorio chileno (Región → Comuna)
- ✅ Dataset oficial (16 regiones + 346 comunas CUT) validado y generado a código.
- ✅ `core/territorio` con tests (regiones, comunas por región, validación cruzada).
- ✅ Migración 009: tablas `regiones`/`comunas` + seed + columnas en
  empresas/personas + CRUD actualizado + `listar_regiones`/`listar_comunas_region`.
- ✅ Endpoints `GET /api/regiones` y `GET /api/comunas?region=`.
- ✅ Selector Región→Comuna reutilizable en altas y fichas (filtra comunas,
  autocompleta ciudad, valida coherencia).
- ✅ E2E: cascada filtra + guardado persiste códigos (pruebas 12–13).

### [*] Fase E1 — Colegios y Admisión (track escolar)
Objetivo: el CRM habla de colegios, no de empresas; el embudo es admisión.
Se ignoran las tablas `crm_sales_*` del sistema escolar externo: el vínculo con
personas es nuestro.

- [*] Directorio oficial (12.038 vigentes, RBD como clave) validado: comunas
  calzan 100% con las 346 y regiones con las 16.
- [*] Migración 010: tabla `colegios` + seed + `listar_colegios` (filtros por
  región/comuna/texto, ideal para traslados) + `obtener/crear/eliminar_colegio`.
- [*] Retiro total de Empresas (tabla, funciones, API, UI, dominio).
- [*] Oportunidades → Admisión: etapas contacto/postulante/evaluado/aceptado/
  matriculado + columna `colegio_rbd`; personas sin `empresa_id`.
- [*] UI: Colegios en el menú (directorio filtrable + registro + ficha con
  postulaciones), kanban de admisión, panel cuenta colegios.
- [] E2E 13/13 adaptada (colegios CRUD por UI, kanban de admisión, fichas).

### ⬜ Fase E2 — Alumnos y Apoderados
- ⬜ Tablas `alumnos` (RUT único con dígito verificador, nombres, curso/nivel,
  colegio RBD, estado: postulante/matriculado/trasladado/retirado) y `apoderados`
  (RUT único, nombres, contacto, territorio) + `alumno_apoderado` (vínculo,
  retiro autorizado, notificaciones).
- ⬜ Validación RUT módulo 11 en núcleo + funciones PG.
- ⬜ CRUD completo + fichas (alumno con apoderados y colegios; apoderado con alumnos).
- ⬜ UI reemplaza Personas por Alumnos + Apoderados en el menú.
- ⬜ E2E: alta con RUT válido/inválido, vínculo apoderado, fichas cruzadas.

### ⬜ Fase E3 — Traslados
- ⬜ Tabla `traslados` (alumno, origen RBD, destino RBD, fecha, motivo) +
  `trasladar_alumno()` (valida destino ≠ origen, actualiza colegio, registra historial).
- ⬜ UI: desde la ficha del alumno, buscador de colegios destino por región/comuna.
- ⬜ E2E: traslado persiste colegio nuevo + historial visible.

### ⬜ Fase 2 — Tareas + Notas
Objetivo: los dos objetos de trabajo diario, relacionados con todo lo demás.

- ⬜ Base de datos (migración 010):
  - ⬜ Tabla `tareas`: `id`, `titulo` (1–200, obligatorio), `detalle`, `estado`
    (`pendiente|en_curso|hecha`, por defecto `pendiente`), `prioridad`
    (`baja|media|alta`, por defecto `media`), `vencimiento` (anulable),
    `persona_id`, `empresa_id`, `oportunidad_id` (anulables, `on delete set null`),
    `creado_en`. Índices por estado, vencimiento y cada referencia. RLS.
  - ⬜ Tabla `notas`: `id`, `titulo`, `cuerpo` (obligatorio no vacío),
    `persona_id`, `empresa_id`, `oportunidad_id` (anulables), `creado_en`.
    Índice GIN trigram (`pg_trgm`) para búsqueda por texto. RLS.
  - ⬜ Funciones por objeto: `listar_*` (búsqueda + filtros por estado/empresa +
    paginación), `obtener_*`, `crear_*` (validan título y referencias),
    `actualizar_*`, `eliminar_*`, más `marcar_tarea(id, estado)`. Todas
    `SECURITY DEFINER` con mensajes en español y `GRANT EXECUTE`.
- ⬜ API:
  - ⬜ `GET/POST /api/tareas`, `GET/PATCH/DELETE /api/tareas/{id}`,
    `PATCH /api/tareas/{id}/estado`.
  - ⬜ `GET/POST /api/notas`, `GET/PATCH/DELETE /api/notas/{id}`.
  - ⬜ Validación Rust espejo de la SQL (título obligatorio, estado/prioridad válidos).
- ⬜ Frontal:
  - ⬜ Entradas de menú Tareas y Notas (quitan el atenuado), rutas `/tareas`, `/notas`.
  - ⬜ Vista Tareas: lista con filtro por estado, ▶ para avanzar estado,
    alta con vencimiento y prioridad, edición inline, baja.
  - ⬜ Vista Notas: lista con búsqueda de texto, editor de alta/edición, baja.
  - ⬜ Fichas de empresa/persona/oportunidad muestran sus tareas y notas con enlaces.
  - ⬜ Contador de pendientes en el menú.
- ⬜ E2E (Chrome): alta de tarea por UI persiste; avance de estado persiste;
  alta y edición de nota persisten; aparecen en la ficha relacionada.
- ⬜ Hecho cuando: E2E 15/15, clippy limpio, migración 010 aplicada en fresco
  (`down -v` + `up` + arranque API sin errores).

### ⬜ Fase 3 — Acceso real + miembros
Objetivo: multiusuario de verdad; base para permisos por usuario.

- ⬜ Base de datos (migración 011):
  - ⬜ Tabla `usuarios`: `id`, `nombre`, `correo` único (formato válido),
    `clave_hash` (argon2, solo lo escribe la función), `rol`
    (`admin|miembro`, por defecto `miembro`), `creado_en`. RLS.
  - ⬜ Tabla `sesiones`: `id`, `usuario_id` (`on delete cascade`),
    `renovacion_hash` (sha256 del token de renovación), `expira_en`,
    `creado_en`. Limpieza de expiradas en cada uso.
  - ⬜ Funciones: `registrar_usuario` (valida, rechaza duplicados P0003),
    `credencial_usuario` (devuelve hash para verificar en Rust, sin exponerlo),
    `crear_sesion`, `rotar_sesion` (rotación con reuso detectado → revoca todo),
    `cerrar_sesion`, `cerrar_todas_sesiones`.
- ⬜ API (nuevas dependencias: `argon2`, `jsonwebtoken`):
  - ⬜ `POST /api/acceso/registro`, `POST /api/acceso/login` (devuelve acceso corto
    + renovación en cookie `HttpOnly`), `POST /api/acceso/renovar`,
    `POST /api/acceso/salir`, `GET /api/acceso/yo`.
  - ⬜ Middleware que exige JWT en todo `/api` salvo salud y acceso; el resto
    exige miembro.
  - ⬜ `GET /api/miembros`, `PATCH /api/miembros/{id}/rol` (solo admin).
  - ⬜ RLS por usuario: políticas con `current_setting('app.usuario_id')` fijado por
    conexión desde el middleware; las funciones lo respetan.
- ⬜ Frontal:
  - ⬜ Pantallas `/acceso` (login real) y `/registro` (con casilla de privacidad),
    sesión solo en memoria (señal global, nunca almacenamiento local),
    renovación silenciosa, cierre en el menú, nombre del usuario en la barra.
  - ⬜ Vista `/miembros` (lista, cambio de rol para admin).
- ⬜ E2E: registro + login + ver yo; renovación sin re-login; rutas protegidas sin
  token devuelven 401 también desde el frontal (redirige a acceso); admin cambia rol.
- ⬜ Hecho cuando: E2E 20/20, sin secretos en logs ni respuestas, `down -v` limpio.

### ⬜ Fase 4 — Calendario + Correos
Objetivo: agenda y bandeja vinculadas a los registros.

- ⬜ Base de datos (migración 012):
  - ⬜ Tabla `eventos`: `id`, `titulo` obligatorio, `inicio`/`fin` (fin ≥ inicio),
    `lugar`, `descripcion`, `persona_id`, `empresa_id`, `oportunidad_id`
    (anulables), `creado_en`. Índice por rango de fechas. RLS.
  - ⬜ Tabla `correos`: `id`, `asunto`, `cuerpo`, `remitente` (formato correo),
    `persona_id` (anulable), `leido` (por defecto falso), `recibido_en`,
    `creado_en`. RLS.
  - ⬜ Funciones CRUD completas + `listar_eventos_mes(año, mes)` +
    `marcar_correo_leido`. Todo `SECURITY DEFINER` en español.
- ⬜ API:
  - ⬜ `GET/POST /api/eventos`, `GET/PATCH/DELETE /api/eventos/{id}`,
    `GET /api/eventos/mes?año=&mes=`.
  - ⬜ `GET/POST /api/correos`, `GET /api/correos/{id}`,
    `PATCH /api/correos/{id}/leido`.
- ⬜ Frontal:
  - ⬜ Ruta `/calendario`: rejilla mensual navegable, día seleccionado con sus
    eventos, alta/edición/baja de eventos, puntos en días con eventos.
  - ⬜ Ruta `/correos`: lista con no-leídos destacados, lector, marcar leído,
    redacción vinculada a persona, contador en el menú.
  - ⬜ Fichas muestran eventos y correos relacionados.
- ⬜ E2E: crear evento por UI aparece en su día; crear correo vinculado aparece en
  la ficha de la persona; leído persiste.
- ⬜ Hecho cuando: E2E 24/24 y mes con eventos límite (31 días) renderiza sin errores.

### ⬜ Fase 5 — Búsqueda global + Favoritos + Vistas + Timeline
Objetivo: moverse por el espacio como en un CRM grande.

- ⬜ Base de datos (migración 013):
  - ⬜ Búsqueda: columnas `tsvector` en español (`pg_catalog.spanish`) para
    empresas/personas/oportunidades/notas/tareas + función `buscar_espacio(texto)`
    que devuelve `(tipo, id, titulo, detalle)` rankeada, límite 20.
  - ⬜ Tabla `favoritos`: `usuario_id`, `tipo` (empresa|persona|oportunidad),
    `referencia_id`, único por tríada. Funciones `alternar_favorito`, `listar_favoritos`.
  - ⬜ Tabla `vistas`: `id`, `usuario_id`, `objeto` (empresas|personas|oportunidades),
    `nombre`, `tipo` (tabla|kanban), `filtros jsonb`, `orden jsonb`.
    CRUD + `listar_vistas(objeto)`.
  - ⬜ Tabla `actividad`: `id`, `tipo`, `referencia_id`, `accion`, `detalle jsonb`,
    `usuario_id`, `creado_en`. Triggers que registran altas/cambios/bajas de los
    objetos núcleo. Función `timeline(tipo, id, limite)`.
- ⬜ API:
  - ⬜ `GET /api/buscar?q=`, `GET /api/favoritos`, `POST /api/favoritos/alternar`.
  - ⬜ `GET/POST /api/vistas?objeto=`, `PATCH/DELETE /api/vistas/{id}`.
  - ⬜ `GET /api/actividad?tipo=&id=` para el timeline de cada ficha.
- ⬜ Frontal:
  - ⬜ Buscador ⌘K en la barra lateral (atajo real, resultados agrupados, Enter abre).
  - ⬜ Estrellas ★ funcionales en listas y fichas; sección Favoritos viva.
  - ⬜ Selector de vista por objeto (tabla/kanban donde aplique) con guardar vista.
  - ⬜ Timeline en cada ficha (actividad ordenada con autor y fecha).
- ⬜ E2E: buscar encuentra por nombre/correo; favorito persiste tras recargar;
  editar un registro genera entrada de timeline visible en su ficha.
- ⬜ Hecho cuando: E2E 28/28 y búsqueda con 1 carácter no rompe nada.

### ⬜ Fase 6 — Automatizaciones
Objetivo: reglas sencillas sin programar.

- ⬜ Base de datos (migración 014):
  - ⬜ Tabla `automatizaciones`: `id`, `nombre` obligatorio, `activa` (defecto true),
    `disparador` (`al_crear|al_actualizar` + objeto), `condiciones jsonb`
    (lista de `{campo, operador, valor}` con operadores `igual|contiene|mayor_que`),
    `acciones jsonb` (lista de `{tipo: mover_etapa|crear_tarea|crear_nota, params}`),
    `creado_en`. RLS.
  - ⬜ Tabla `ejecuciones`: `id`, `automatizacion_id`, `ok`, `detalle`,
    `creado_en`. Función `registrar_ejecucion`.
  - ⬜ CRUD + `listar_automatizaciones` + `alternar_automatizacion`.
- ⬜ API:
  - ⬜ `GET/POST /api/automatizaciones`, `GET/PATCH/DELETE /api/automatizaciones/{id}`,
    `PATCH /api/automatizaciones/{id}/activa`, `GET /api/automatizaciones/{id}/ejecuciones`.
  - ⬜ Motor: tras cada crear/actualizar de oportunidades (y luego resto), evalúa
    reglas activas del objeto y ejecuta acciones con las mismas funciones seguras.
- ⬜ Frontal:
  - ⬜ Ruta `/automatizaciones`: lista con interruptor, editor (disparador +
    condiciones + acciones con formularios guiados), historial por regla.
- ⬜ E2E: regla «al crear oportunidad con importe > 100000 → crear tarea» genera la
  tarea visible; desactivarla deja de generar.
- ⬜ Hecho cuando: E2E 30/30 y una regla que falla registra ejecución con error sin
  romper la operación original.

### ⬜ Fase 7 — Agentes IA + chat
Objetivo: pedirle cosas al CRM conversando.

- ⬜ Base de datos (migración 015):
  - ⬜ Tabla `conversaciones`: `id`, `usuario_id`, `titulo`, `creado_en`.
  - ⬜ Tabla `mensajes`: `id`, `conversacion_id` (`on delete cascade`), `rol`
    (`usuario|agente`), `contenido`, `creado_en`.
  - ⬜ CRUD + `listar_mensajes(conversacion)`. RLS por usuario.
- ⬜ API (cliente LLM configurable por `LLM_URL`, `LLM_CLAVE`, `LLM_MODELO`;
  sin claves no arranca el módulo, el resto sigue igual):
  - ⬜ `GET/POST /api/conversaciones`, `GET /api/conversaciones/{id}/mensajes`,
    `POST /api/conversaciones/{id}/preguntar`.
  - ⬜ Herramientas del agente (solo vía funciones seguras existentes): buscar,
    listar y crear registros, resumen del panel. Cada llamada queda citada en la
    respuesta.
- ⬜ Frontal:
  - ⬜ Panel de chat lateral persistente (ruta `/agentes` + acceso rápido),
    historial por conversación, indicador de escritura, citas a registros con enlace.
- ⬜ E2E (con proveedor simulado en local): preguntar el total de la tubería
  responde con la cifra real; pedir crear una nota la crea y enlaza.
- ⬜ Hecho cuando: E2E 32/32 y sin `LLM_CLAVE` el módulo responde «no configurado»
  sin afectar al resto.

### ⬜ Fase 8 — Plataforma y producción
Objetivo: que terceros integren y que operar sea aburrido (en el buen sentido).

- ⬜ Integraciones (migración 016):
  - ⬜ Tabla `claves_api`: `id`, `usuario_id`, `nombre`, `clave_hash` (sha256, se
    muestra una sola vez al crear), `alcances text[]`, `revocada`,
    `creado_en`. Auth por cabecera `X-Clave-API` con mismos permisos que el dueño.
  - ⬜ Tabla `webhooks`: `id`, `url` (https obligatoria salvo localhost),
    `eventos text[]`, `secreto`, `activa`, `creado_en`.
  - ⬜ Tabla `entregas_webhook`: `id`, `webhook_id`, `evento`, `carga jsonb`,
    `intentos`, `ok`, `creado_en`. Reintentos con espera exponencial (3 intentos).
  - ⬜ UI en Ajustes: claves (crear/ver una vez/revocar) y webhooks (crear/probar/ver).
- ⬜ Importación CSV:
  - ⬜ `POST /api/importar/empresas|personas` (multipart): valida cabecera, previsualiza
    5 filas con errores por fila, aplica el resto en una transacción, devuelve resumen.
  - ⬜ UI con subida, previsualización y reporte de errores descargable.
- ⬜ Operación:
  - ⬜ Respaldos: servicio `respaldos` (pg_dump diario a volumen + retención 7).
  - ⬜ Observabilidad: logs JSON en producción, `GET /api/salud` extendido
    (versión, migraciones, latencia PG) y límites de ritmo por IP en login.
  - ⬜ Multi-espacio: columna `espacio_id` (se diseña aquí, se ejecuta en Fase 9).
- ⬜ E2E: importar CSV con 1 fila mala importa 2/3 y reporta la mala; webhook de
  prueba llega a un receptor local; clave revocada devuelve 401.
- ⬜ Hecho cuando: E2E 35/35 y `docker compose up -d --build` en máquina limpia
  levanta todo sin intervención.

## Mapa de módulos → estado

| Módulo | Estado | Fase |
|---|---|---|
| Panel | ✅ Funcional | 1 |
| Personas | ✅ Funcional (CRUD + ficha) | 1 + 1.5 |
| Empresas | ✅ Funcional (CRUD + ficha) | 1 + 1.5 |
| Oportunidades | ✅ Funcional (CRUD + kanban + inline) | 1 + 1.5 |
| Fichas y relacionados | ✅ Funcional | 1.5 |
| Regiones y comunas | ✅ Funcional | 1.6 |
| Colegios y Admisión | [*] En curso | E1 |
| Alumnos y Apoderados | ⬜ Pendiente | E2 |
| Traslados | ⬜ Pendiente | E3 |
| Tareas | [] Pendiente | 2 |
| Notas | [] Pendiente | 2 |
| Usuarios y acceso | 🚧 Mock (real en 3) | 3 |
| Miembros y roles | [] Pendiente | 3 |
| Calendario | [] Pendiente | 4 |
| Correos | [] Pendiente | 4 |
| Búsqueda global | [] Pendiente | 5 |
| Favoritos | 🚧 Solo visual (funcional en 5) | 5 |
| Vistas guardadas | [] Pendiente | 5 |
| Timeline | [] Pendiente | 5 |
| Automatizaciones | 🚧 Solo visual (funcional en 6) | 6 |
| Agentes IA y chat | [] Pendiente | 7 |
| Claves API y webhooks | [] Pendiente | 8 |
| Importación CSV | [] Pendiente | 8 |
| Respaldos y operación | 🚧 Básico (completo en 8) | 8 |
| Ajustes | 🚧 Básico (completo en 3 + 8) | 3 + 8 |

## Reglas de cada fase (no negociables)
1. Primero la base de datos (tablas + funciones seguras + RLS + migración nueva;
   jamás editar migraciones aplicadas).
2. Después la API (handlers delgados, validación espejo, errores en español).
3. Después el frontal (ES/EN, conectado a la API, sin mocks salvo respaldo).
4. E2E en Chrome que verifique persistencia en PG antes de darla por hecha.
5. `cargo clippy -- -D warnings`, tests en verde, commit + push.
6. Actualizar este archivo (mover marcas y tachar pendientes).
