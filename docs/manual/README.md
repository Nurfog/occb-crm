# `manual` — Manual de uso OCCB CRM

## Overview

Manual de usuario en español (`MANUAL.md`, 14 secciones): acceso, panel, alumnos + ficha (salud e ingresos 1:N), apoderados, admisión, tareas, notas, calendario, correos, miembros, automatizaciones, asistente, ajustes (claves, webhooks, CSV) y búsqueda/favoritos/protección de datos. Medios excluidos de revisión por `.egmignore`: 15 capturas (`img/01–15`) y `video/recorrido.mp4` (37 s, 1280×720 H.264).

## Interface

- `MANUAL.md`: 14 secciones con enlaces relativos a `img/*.png` y `video/recorrido.mp4` (`[MANUAL.md:1. Entrar al sistema]` … `[MANUAL.md:14. Búsqueda, favoritos, vistas y protección de datos]`).
- `img/`: `01-acceso.png` … `15-busqueda.png` (1280×900, generadas con tour automatizado y filas demo temporales).
- `video/`: `recorrido.mp4` (paseo completo generado con `ffmpeg`, framerate 0.4).

## Invariants

- Los enlaces del manual son relativos (`img/…`, `video/…`); si se renombra un medio, el `MANUAL.md` debe actualizarse.
- Las fotos se regeneran con filas demo temporales que el propio tour crea y borra.

## Side Effects

- Ninguno (documentación estática; el tour de generación es el único que escribe, fuera de este subárbol).

## Verification

- Run: `ls docs/manual/img/*.png docs/manual/video/recorrido.mp4` (15 + 1 ficheros)
