# `src` — Núcleo del dominio OCCB

## Overview

Paquete de biblioteca pura (`occb-core`) con los tipos de dominio en español, la validación sin E/S, el territorio chileno oficial, los textos ES/EN y los datos de demostración. Lo usan la API (antes de llamar a PostgreSQL) y el frontal (formularios y modo sin conexión) para compartir el mismo vocabulario (`nombre`, `correo`, `importe`, `etapa`).

## Interface

Tipos y re-exportaciones (`lib.rs:dominio`, `lib.rs:i18n`, `lib.rs:semilla`, `lib.rs:territorio`, `lib.rs:validacion`):

- Dominio (`dominio.rs`): `Colegio`, `CrearColegio`, `Persona`, `CrearPersona`, `Persona::nombre_completo`, `Persona::iniciales`, `Alumno`, `CrearAlumno`, `EstadoAlumno` (`todas`, `desde_texto`, `como_texto`), `Apoderado`, `CrearApoderado`, `VinculoApoderado`, `Traslado`, `IngresoMedico`, `CrearIngreso`, `Tarea`, `CrearTarea`, `EstadoTarea` (`todas`, `desde_texto`, `como_texto`, `siguiente`), `Nota`, `CrearNota`, `Evento`, `CrearEvento`, `Correo`, `CrearCorreo`, `EtapaOportunidad` (`todas`, `desde_texto`, `como_texto`, `probabilidad`, `siguiente`), `Oportunidad`, `CrearOportunidad`, `ResultadoBusqueda`, `Favorito`, `VistaGuardada`, `Actividad`, `Automatizacion`, `Ejecucion`, `Conversacion`, `Mensaje`, `ResumenPanel`, `calcular_resumen`.
- Validación (`validacion.rs`): `ErrorDominio::Validacion`, `validar_colegio`, `validar_persona`, `validar_alumno`, `validar_apoderado`, `validar_ingreso`, `validar_tarea`, `validar_nota`, `validar_evento`, `validar_correo`, `validar_oportunidad`, `validar_rut`, `normalizar_rut`, `correo_valido`.
- Territorio (`territorio.rs`): `Region`, `Comuna`, `REGIONES`, `COMUNAS`, `region`, `comunas_de_region`, `comuna_por_cut`, `validar_territorio`.
- Textos (`i18n.rs`): `Idioma::Es`, `Idioma::En`, `Idioma::codigo`, `Idioma::alternar`, `Textos`, `textos` (incluye vocabulario SaaS y Twenty: `monto_ref`, `vencidas`, `hoy`, `proximas`, `sin_fecha`, `etiqueta_utc`, `limpiar_filtro`, `cuando`, `si`, `entonces`, `pista_limite`, `holding`, `sede`, `licencia`, `meta`, `dominio`, `ingresos`, `alcance`, `consentimiento`).
- Demostración (`semilla.rs`): `datos_semilla_colegios`, `datos_semilla_personas`, `datos_semilla_alumnos`, `datos_semilla_apoderados`, `datos_semilla_oportunidades`, `datos_semilla_tareas`, `datos_semilla_notas`.

## Invariants

- RUT chileno con dígito verificador módulo 11; se normaliza a `12345678-5` sin puntos y DV en mayúscula (`[validacion.rs:normalizar_rut]`, `[validacion.rs:validar_rut]`).
- Pareja región/comuna coherente: ambas ausentes es válido; con comuna se exige región coincidente y códigos existentes (`[territorio.rs:validar_territorio]`).
- Directorio fijo: 16 regiones y 346 comunas; la Región Metropolitana (código 13) tiene 52 comunas (`[territorio.rs:REGIONES]`, `[territorio.rs:COMUNAS]`). Fichero generado por script: no editar a mano (`[territorio.rs:1]`).
- Reglas de negocio con mensajes en español (`[validacion.rs:ErrorDominio]`): evento con `fin` anterior a `inicio` se rechaza (`[validacion.rs:validar_evento]`); prioridad de tarea solo `baja`/`media`/`alta` (`[validacion.rs:validar_tarea]`); importe de oportunidad nunca negativo (`[validacion.rs:validar_oportunidad]`); ingreso médico exige motivo (`[validacion.rs:validar_ingreso]`).
- Embudo de admisión en orden `Contacto → Postulante → Evaluado → Aceptado → Matriculado`; `Matriculado` es terminal en `siguiente()` (`[dominio.rs:EtapaOportunidad::siguiente]`). El resumen cuenta en tubería todo lo no matriculado y como ganado solo lo matriculado (`[dominio.rs:calcular_resumen]`).
- Validación pura y sin E/S (funciones sincronas `Result<_, ErrorDominio>`, sin `async` ni `sqlx`): el mismo código corre en API y frontal (`[validacion.rs:validar_colegio]`, `[lib.rs:1]`).
- Añadir un texto de interfaz obliga a traducirlo a ES y EN (`[i18n.rs:Textos]`, `[i18n.rs:textos]`).
- `semilla.rs` son solo datos de demostración (nombres `Demo`, correos `@test.cl`, RUT de ejemplo); el directorio oficial de colegios vive en PostgreSQL (`[semilla.rs:datos_semilla_colegios]`).

## Side Effects

- Ninguno en la biblioteca: sin red, sin disco, sin base de datos y sin bloqueos.
- Único no-determinismo: `Uuid::new_v4()` y `Utc::now()` al construir registros de demostración (`[semilla.rs:datos_semilla_personas]`, `[semilla.rs:datos_semilla_oportunidades]`).

## Verification

- Run: `cargo test -p occb-core`
