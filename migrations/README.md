# `migrations` Blueprint

## System Overview

Historial lineal de esquema PostgreSQL 18 del CRM escolar (31 migraciones `sqlx`, `001–031`): tablas, funciones `SECURITY DEFINER` con `SET search_path = public`, RLS sin políticas permisivas (`REVOKE ALL` + `GRANT SELECT/EXECUTE` a `public`), índices y semillas. La API las aplica al arrancar (`bd::migrar_y_sembrar`); nunca se edita una migración aplicada (el `checksum` de `sqlx` lo impide), solo se añade la siguiente. `029` fundación tenant SaaS (holdings, colegios contratados, membresías, importaciones, supresión, `es_miembro`), `030` matrícula y consentimiento (`prospecto_rut/origen/consentimiento/ttl`, `matricular_postulacion` idempotente) y `031` repara siembra escolar en BD limpias son aditivas y no cambian el comportamiento legado.

## Local Interface & Invariants

Tablas (`CREATE TABLE`, minúsculas en los ficheros): `empresas` (legado, retirada del producto), `personas`, `oportunidades`, `regiones`, `comunas`, `colegios` (RBD como clave, 12 038 vigentes vía 36 `INSERT` multi-fila en `[010_colegios_admision.sql]`), `alumnos` (RUT único entre activos), `apoderados`, `alumno_apoderado`, `traslados`, `tareas`, `notas`, `eventos`, `correos`, `favoritos`, `vistas`, `actividad`, `automatizaciones`, `ejecuciones`, `conversaciones`, `mensajes`, `claves_api`, `webhooks`, `entregas_webhook`, `usuarios`, `sesiones`, `ingresos_medicos`, `holdings`, `colegios_contratados` (RBD FK al directorio), `membresias_usuario`, `importaciones` (jobs CSV), `supresion` (hash RUT).

Funciones por dominio (`CREATE OR REPLACE FUNCTION`, todas `SECURITY DEFINER`): `listar_*/obtener_*/crear_*/actualizar_*/eliminar_*` por entidad, `mover_etapa_oportunidad`, `panel_resumen`, `es_rut_valido`, `rut_normalizado`, `vincular/desvincular_apoderado`, `apoderados_de_alumno`, `alumnos_de_apoderado`, `trasladar_alumno`, `marcar_tarea`, `marcar_correo_leido`, `listar_eventos_mes`, `buscar_espacio` (FTS español + trigramas), `alternar_favorito`, `timeline`, `registrar_actividad_cambio` (disparadores de auditoría), `ejecutar_automatizaciones` y `validar_regla_automatizacion`, `registrar_usuario` (el primero admin), `credencial_usuario`, `crear/rotar/cerrar_sesion`, `listar_miembros`, `cambiar_rol`, `importar_alumnos/apoderados` (hasta 1000 filas, parcial con reporte), `crear/listar/validar/revocar_clave_api`, `crear/listar/alternar/eliminar/probar_webhook`, `reclamar_entregas_webhook`, `registrar_resultado_webhook` (backoff `least(60,5*2^(intentos-1))`), `anonimizar_persona/alumno/apoderado/oportunidad` y `eliminar_*` (borrado lógico).

Invariantes clave: RUT chileno con DV módulo 11 (`[011_alumnos_apoderados.sql:es_rut_valido]`); grupos sanguíneos cerrados y largos acotados en ficha escolar (`[026_ficha_escolar.sql:grupo_sanguineo]`); `fin >= inicio` en eventos; importes no negativos; etapas cerradas por dominio; `UNIQUE ... WHERE activo` tras el borrado lógico (`[028_borrado_logico.sql:uq_alumnos_rut_activo]`); claves API `sha256` de 64 hex y webhooks `https` (o `http` solo local); `rotar_sesion` con hash distinto revoca todo sin error; semillas idempotentes (`count > 0 → RETURN`, `ON CONFLICT DO NOTHING`).

## Subdirectories & System Boundaries

- (sin subdirectorios: los 31 ficheros `001–031_*.sql` viven directamente en esta carpeta y se aplican en orden lexicográfico).

## Global Invariants

- Todo el CRUD de la API pasa por estas funciones; ningún `INSERT/UPDATE/DELETE` directo desde la aplicación.
- RLS activo en todas las tablas de negocio; `public` solo lee vía funciones.
- Errores de negocio como excepciones `P0001/P0002/P0003` que la API mapea a 400/404 en español.

## Global Side Effects

- DDL + semillas al arrancar (extensiones `pgcrypto`, `pg_trgm`; vectores FTS `busqueda_vector` con índice GIN en 9 tablas); escrituras solo vía funciones RPC.

## System Verification

- Run: `docker compose up -d bd && cargo run -p occb-api` (aplica `001–031` al arrancar) o `node e2e/prueba.mjs` (E2E sobre esquema migrado).
