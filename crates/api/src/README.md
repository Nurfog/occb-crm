# `src` — API HTTP OCCB (Axum + PostgreSQL)

## Overview

Paquete binario `occb-api`: expone la API HTTP del CRM escolar sobre Axum y PostgreSQL 18. Todo el acceso a datos pasa por funciones `SECURITY DEFINER` de `migrations/` (vía `sqlx`); la validación de negocio se reutiliza desde `occb-core` antes de tocar la base. El enrutado (`main.rs`) separa rutas públicas (salud, acceso, regiones/comunas, lectura de colegios y panel) de rutas protegidas por el middleware `exigir_sesion` (JWT Bearer o clave API). Incluye acceso con JWT + renovación opaca rotativa, claves API con alcances, webhooks HMAC con worker de reintentos, importación CSV, asistente con LLM y automatizaciones sobre postulaciones.

## Interface

- Núcleo (`main.rs`, `configuracion.rs`, `errores.rs`, `bd.rs`): `salud`, `Configuracion::desde_entorno` (lee `DATABASE_URL` y `PUERTO_API`, defectos `postgres://occb:occb@localhost:5432/occb` y `3001`), `ErrorApi` (`NoEncontrado`, `Validacion`, `Prohibido`, `DemasiadasSolicitudes`, `BaseDatos` con `IntoResponse` y `From<occb_core::ErrorDominio>`), `crear_pool` (máx. 10 conexiones), `transaccion_con_actor` (`SET LOCAL app.usuario_id`), `migrar_y_sembrar` (aplica `../../migrations` y siembra demo), `fila_a_*` (`fila_a_colegio`, `fila_a_persona`, `fila_a_oportunidad`, `fila_a_alumno`, `fila_a_apoderado`, `fila_a_traslado`, `fila_a_ingreso`, `fila_a_tarea`, `fila_a_nota`, `fila_a_evento`, `fila_a_correo`).
- Acceso (`auth.rs`): `UsuarioActual`, `CuerpoAcceso`, `UsuarioPublico`, `hash_clave` / `verificar_clave` (Argon2), `registro`, `login` (límite por IP), `renovar` (rota sesión), `salir`, `yo`, `miembros`, `cambiar_rol`, `exigir_sesion`.
- Directorio y territorio (`rutas_colegios.rs`, `rutas_territorio.rs`): `FiltrosColegios`, `CuerpoColegio`, `listar`, `obtener`, `crear`, `eliminar`; `RegionSalida`, `ComunaSalida`, `FiltroComunas`, `listar_regiones`, `listar_comunas`.
- Personas y escuela (`rutas_personas.rs`, `rutas_alumnos.rs`, `rutas_apoderados.rs`, `rutas_ingresos.rs`): `ParametrosLista`, `CuerpoPersona` / `CuerpoAlumno` / `CuerpoApoderado` / `CuerpoIngreso`, `listar`, `obtener`, `crear`, `actualizar`, `eliminar`, `anonimizar` (solo admin), más `apoderados_de`, `trasladar`, `historial`, `vincular`, `desvincular` (alumnos) y `alumnos_de` (apoderados).
- Admisión (`rutas_oportunidades.rs`): `FiltrosOportunidad`, `CuerpoOportunidad`, `CuerpoEtapa`, `listar`, `crear`, `actualizar`, `mover_etapa`, `eliminar`, `anonimizar`, `panel` (lee `panel_resumen()`).
- Trabajo (`rutas_tareas.rs`, `rutas_notas.rs`, `rutas_eventos.rs`, `rutas_correos.rs`): `CuerpoTarea` / `CuerpoEstado`, `CuerpoNota`, `CuerpoEvento`, `ParametrosMes`, `CuerpoCorreo` / `CuerpoLeido`, con `listar`, `obtener`, `crear`, `actualizar`, `marcar`, `eliminar` según el recurso.
- Espacio (`rutas_espacio.rs`): `ParametrosBusqueda`, `CuerpoFavorito`, `ParametrosVistas`, `CuerpoVista`, `ParametrosActividad`, `buscar`, `favoritos`, `alternar_favorito`, `listar_vistas`, `crear_vista`, `actualizar_vista`, `eliminar_vista`, `actividad`.
- Automatizaciones (`rutas_automatizaciones.rs`): `CuerpoAutomatizacion`, `CuerpoActiva`, `listar`, `obtener`, `crear`, `actualizar`, `eliminar`, `alternar`, `ejecuciones` (los fallos de regla quedan registrados sin revertir la operación origen).
- Asistente (`rutas_agentes.rs`): `CuerpoConversacion`, `CuerpoPregunta`, `RespuestaAgente`, `listar`, `crear`, `eliminar`, `mensajes`, `preguntar` (requiere `LLM_URL`, `LLM_CLAVE` y `LLM_MODELO`; `mock://occb-e2e` solo en `debug_assertions`).
- Integraciones (`rutas_integraciones.rs`, `rutas_importacion.rs`): `CuerpoClave`, `CuerpoWebhook`, `CuerpoActiva`, `listar_claves`, `crear_clave` (el secreto solo se muestra al crear), `revocar_clave`, `listar_webhooks`, `crear_webhook`, `alternar_webhook`, `eliminar_webhook`, `probar_webhook`, `entregas`, `ejecutar_worker_webhooks`; `ParametrosImportacion`, `importar` (CSV `alumnos`/`apoderados`, previsualiza y solo escribe si `aplicar=true`).

## Invariants

- Sesiones cortas y rotativas: acceso de 15 min (`[auth.rs:VIDA_ACCESO_SEGS]`), sesión de 30 días en cookie `occb_renovar` (`<sesion_id>.<token>`, `HttpOnly`, `SameSite=Lax`) (`[auth.rs:COOKIE_RENOVAR]`); `renovar` rota por huella `sha256` y revoca ante reuso (`[auth.rs:renovar]`).
- Registro abierto por diseño con clave mínima de 8 y nombre no vacío (`[auth.rs:registro]`); login limitado a 60 intentos por IP y 60 s (`[auth.rs:login_limitado]`); cambio de rol y anonimización solo admin (`[auth.rs:cambiar_rol]`, `[rutas_alumnos.rs:anonimizar]`, `[rutas_apoderados.rs:anonimizar]`, `[rutas_personas.rs:anonimizar]`, `[rutas_oportunidades.rs:anonimizar]`).
- Clave API sin alcance `*` ni escritura deniega todo lo distinto de `GET/HEAD/OPTIONS` con 403 (`[auth.rs:exigir_sesion]`); el secreto de `APP_SECRETO` avisa si cae al valor de desarrollo (`[auth.rs:secreto]`).
- Escritura de colegios protegida: lectura pública, `POST/DELETE` exigen sesión (`[main.rs:62]`, `[main.rs:78]`); territorio de referencia público (`[main.rs:70]`).
- Validación espejo de `occb-core` en cada escritura: RUT módulo 11, pareja región/comuna coherente, `fin >= inicio` en eventos, prioridad acotada, importe no negativo, motivo obligatorio en ingresos (`[validacion.rs:validar_rut]`, `[territorio.rs:validar_territorio]`, `[validacion.rs:validar_evento]`, `[validacion.rs:validar_tarea]`, `[validacion.rs:validar_oportunidad]`, `[validacion.rs:validar_ingreso]`).
- Códigos de BD mapeados: `P0002` → 404 `no_encontrado`, `P0001/P0003` → 400 `datos_invalidos` (`[errores.rs:respuesta_bd]`); webhooks exigen `https` (o `http` solo local) (`[rutas_integraciones.rs:url_webhook_valida]`).
- Pregunta al asistente acotada: no vacía, máx. 8000 caracteres, historial de 40, 5 iteraciones y 20000 de respuesta (`[rutas_agentes.rs:preguntar]`); importación CSV máx. 5 MB, 1000 filas y cabeceras `rut,nombres,apellidos` (`[rutas_importacion.rs:importar]`).

## Side Effects

- PostgreSQL en cada petición: lecturas `listar_*/obtener_*/panel_resumen()` y escrituras en transacción con actor (`transaccion_con_actor` + `commit`) que fijan `app.usuario_id` para autoría; migraciones y siembra demo al arrancar (`[bd.rs:migrar_y_sembrar]`).
- Cookies de sesión: `registro`/`login` crean sesión y emiten `SET_COOKIE`; `renovar` rota; `salir` borra (`[auth.rs:registro]`, `[auth.rs:renovar]`, `[auth.rs:salir]`).
- Red saliente: worker de webhooks en bucle (`reclamar_entregas_webhook(10)` → `POST` con `X-OCCB-Evento`/`X-OCCB-Firma` HMAC-SHA256, timeout 5 s → `registrar_resultado_webhook`, pausas 2 s/5 s/30 s) y proveedor LLM en `preguntar` (`reqwest`, timeout 30 s) (`[rutas_integraciones.rs:ejecutar_worker_webhooks]`, `[rutas_agentes.rs:preguntar]`).
- Observabilidad y límites: `tracing_subscriber`, capas `CompressionLayer`, `TimeoutLayer` 10 s, `CorsLayer` y `TraceLayer`; salud consulta `select 1` y `_sqlx_migrations` (`[main.rs:main]`, `[main.rs:salud]`).

## Verification

- Run: `cargo test -p occb-api`
