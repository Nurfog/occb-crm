# `src` — Frontal Dioxus OCCB

## Overview

Código del frontal web (`occb-app`, Dioxus 0.7 → WASM + `nginx`): enrutador con disposición común (`DisposicionApp` con barra lateral y superior), vistas por recurso y un cliente HTTP central (`cliente.rs`) contra la API. La sesión vive solo en memoria (`SESION`); sin sesión el guardia redirige a `VistaAcceso` y vuelve al destino pendiente (`VOLVER`). Los textos salen de `occb-core::textos(idioma)` con `IDIOMA` global (ES por defecto, conmutable a EN).

## Interface

- Arranque y rutas (`main.rs:main`, `main.rs:Aplicacion`, `main.rs:Ruta`, `main.rs:DisposicionApp`): `dioxus::launch(Aplicacion)`; `Ruta` con `/acceso`, `/`, `/alumnos`, `/alumnos/:id`, `/apoderados`, `/apoderados/:id`, `/tareas`, `/notas`, `/personas`, `/personas/:id`, `/colegios/:rbd`, `/oportunidades`, `/oportunidades/:id`, `/miembros`, `/calendario`, `/correos`, `/automatizaciones`, `/agentes`, `/ajustes` y `/:..segmentos` (`NoEncontrada`); `VistaAcceso` va antes del `layout` para que quede fuera de la disposición.
- Cliente (`cliente.rs`): `Sesion`, `SESION`, `Miembro`, `NO_LEIDOS`, `FAVORITOS`; `registrar`, `iniciar_sesion`, `cerrar_sesion` (limpia `SESION`, `FAVORITOS` y `NO_LEIDOS`), `guardar_sesion`, `renovar_sesion`, `ejecutar` (añade `Authorization: Bearer` y ante 401 renueva una vez y reintenta, bucle `0..2`), `leer_cuerpo` (éxito exige `is_success`, error desde campo `mensaje`), `base` (`API_BASE_URL` inyectada en compilación, defecto `http://127.0.0.1:3001`); recursos `obtener_panel`, `listar_*`, `obtener_*`, `crear_*`, `actualizar_*`, `marcar_*`, `eliminar_*`, `anonimizar_*`, `mover_etapa`, `trasladar_alumno`, `vincular/desvincular_apoderado`, `buscar_espacio`, `listar_favoritos`, `alternar_favorito`, `timeline`, `eventos_mes`, `importar_csv` (multipart), claves/webhooks y conversaciones (`preguntar`).
- Vistas (`main.rs`): `Panel`, `VistaPersonas`, `DetallePersona`, `VistaOportunidades`, `DetalleOportunidad`, `DetalleColegio`, `VistaAlumnos`, `DetalleAlumno`, `VistaApoderados`, `DetalleApoderado`, `VistaTareas`, `VistaNotas`, `VistaCalendario`, `VistaCorreos`, `VistaAutomatizaciones`, `VistaAgentes`, `VistaAjustes`, `VistaMiembros`, `VistaAcceso`, `Timeline`, `SelectorVistas`, `SelectorTerritorio`, `EstrellaFavorito`, `BarraLateral`, `BarraSuperior`; helpers `hace_tiempo`, `etiqueta_etapa`, `etiqueta_estado`, `etiqueta_tarea`, `iniciales_de`, `campos_salud`, `panel_salud`, `a_campo_local`, `desde_campo_local`, `fmt_hora`, `fmt_dia_hora`, `fmt_fecha`, `recargar_mes`, `mover_mes`, `nombre_mes`, `recargar_personas`, `recargar_oportunidades`, `alternar_seleccion`, `alternar_todos`.

## Invariants

- Sesión solo en memoria: `None` = sin entrar; recargar pierde la sesión y el guardia manda a acceso (`[cliente.rs:SESION]`, `[main.rs:VistaAcceso]`, `[main.rs:VistaMiembros]`).
- Cada petición lleva `Bearer` si hay sesión y ante 401 se renueva una sola vez (`[cliente.rs:ejecutar]`).
- Fechas del calendario en UTC de ida y vuelta con formato `%Y-%m-%dT%H:%M` (`[main.rs:a_campo_local]`, `[main.rs:desde_campo_local]`); rejilla con lunes primero y día 1 seleccionado al mover mes (`[main.rs:VistaCalendario]`, `[main.rs:mover_mes]`).
- Selección masiva coherente: la cabecera marca todo lo visible (`[main.rs:alternar_todos]`).
- Anonimización solo admin (la API responde 403 si no lo es) (`[main.rs:DetalleAlumno]`, `[main.rs:DetalleApoderado]`).

## Side Effects

- Red vía `reqwest` en cada vista (`spawn` + `cliente::*` + `*.set(...)` sobre signals); renovación que muta o limpia `SESION`; importación CSV por `multipart/form-data` (`[cliente.rs:ejecutar]`, `[cliente.rs:renovar_sesion]`, `[cliente.rs:importar_csv]`).
- Estado global en memoria: `IDIOMA`, `VOLVER`, `NOMBRE_ESPACIO`, `BUSQUEDA_ABIERTA`, `FAVORITOS`, `NO_LEIDOS` (`[main.rs:IDIOMA]`, `[main.rs:VOLVER]`).
- Navegación: `nav.replace`/`nav.push` a acceso o al destino pendiente (`[main.rs:VistaAcceso]`, `[main.rs:BarraLateral]`).

## Verification

- Run: `cargo test -p occb-app`
