# Auditoría de esquema — condiciones por tabla (2026-11-18)

Estado real: 36 tablas en `public` (001–043), RLS activo en todas, **0 políticas**.
Inventario tomado de PostgreSQL, no de las migraciones. Filas entre paréntesis.

## Checklist aplicado

| Código | Condición |
|---|---|
| C1 | PK `id uuid`; catálogos con clave natural solo si es estable (ISO, CUT, RBD) |
| C2 | `empresa_id` en todo dato de negocio (NULL = plataforma/legado) |
| C3 | `holding_id` derivado de empresa, no fuente de verdad duplicada |
| C4 | País, moneda y documento en entidades fiscales/contables |
| C5 | Soft-delete `activo` en negocio; historial/log sin borrado |
| C6 | `creado_en` en toda fila; `actualizado_en` en editables |
| C7 | RLS con al menos una política, o excepción documentada |
| C8 | `REVOKE ALL` + `GRANT SELECT`; escritura solo por función |
| C9 | Unique de negocio por empresa/país (documento, correo, folio) |
| C10 | FK con `ON DELETE` explícito y coherente |
| C11 | Índice por tenant y por filtros usados |

## Matriz por grupo

### Catálogos
| Tabla | C1 | C4 | C5 | C6 | C7 | C11 |
|---|---|---|---|---|---|---|
| `paises` (18) | ISO2 natural ✔ | moneda + locale ✔ | n/a | `creado_en` ✔ | **sin política** | PK ✔ |
| `monedas` (18) | ISO natural ✔ | decimales ✔ | n/a | **falta** | **sin política** | PK ✔ |
| `regiones` (16) | CUT natural ✔ | país implícito CL | n/a | **falta** | **sin política** | PK ✔ |
| `comunas` (346) | CUT natural ✔ | país implícito CL | n/a | **falta** | **sin política** | PK ✔ |

Brecha: catálogos sin política RLS aunque son lectura pública; conviene política `select true` explícita o desactivar RLS y dejar grants.

### Directorio y tenant
| Tabla | Brechas |
|---|---|
| `colegios` (12.039) | PK RBD ✔, `pais_iso` ✔, `activo` ✔; RLS sin política; sin `actualizado_en`; `region/comuna` nullable (correcto para no-CL) |
| `holdings` | PK uuid ✔, país ✔; sin `activo` (usa borrado físico con cascada); sin `actualizado_en`; RLS sin política |
| `empresas` | PK uuid ✔, país/moneda/documento ✔, `activa` ✔; sin `actualizado_en`; RLS sin política; sin unique por `(pais, documento)` explícito en catálogo (sí índice en 039) |
| `colegios_contratados` | PK RBD ✔; `empresa_id` nullable (**debe ser obligatorio para contratar**); `holding_id` derivado ✔; sin `activo`; RLS sin política |
| `membresias_usuario` | PK uuid ✔; empresa/holding/colegio; unique natural ✔; RLS sin política |
| `puntos_emision` | PK uuid ✔; **falta `folio_actual`** para consumo; `vigente` en vez de `activo`; RLS sin política |
| `importaciones` | PK `job_id` uuid ✔; **falta `empresa_id` y `usuario_id`**; `errores` jsonb ✔; RLS sin política |
| `supresion` | PK `rut_hash` natural ✔; sin `empresa_id` (¿global o por empresa? decidir); RLS sin política |

### Datos núcleo con tenant
| Tabla | Brechas |
|---|---|
| `alumnos` (17) | `empresa_id`/`holding_id`/país/documento ✔; `activo` ✔; unique documento ✔; RLS sin política; `rut` nullable ✔ |
| `apoderados` (7) | Igual que alumnos; **sin `colegio_rbd`**, por eso no hereda empresa (solo si se asigna a mano) |
| `oportunidades` (12) | `empresa_id`/`holding_id` ✔; **sin `moneda_iso`** (importe sin moneda explícita); `activo` ✔; RLS sin política |

### Datos derivados sin tenant (brecha C2)
`tareas` (2), `notas` (1), `eventos` (2), `correos` (3), `ingresos_medicos` (3), `traslados`, `alumno_apoderado` (4):
- **Falta `empresa_id`** en todas. Debe heredarse del alumno/oportunidad y filtrarse por empresa.
- `alumno_apoderado` **sin `activo`**: un vínculo retirado se borra físicamente.
- `traslados` **sin `creado_en`** (solo `fecha` de negocio).
- `ingresos_medicos` con `activo` ✔.
- Todas con RLS sin política.

### Auditoría y personal
| Tabla | Brechas |
|---|---|
| `actividad` (94) | **sin `empresa_id`**; sin `activo` (correcto: log inmutable); RLS sin política |
| `favoritos` | **PK compuesta `(usuario,tipo,referencia_id)`**, no `id uuid` (C1); sin `empresa_id`; RLS sin política |
| `vistas` | PK uuid ✔; sin `empresa_id`; `actualizado_en` ✔; RLS sin política |
| `conversaciones` | PK uuid ✔; **borrado físico** (falta `activo`); sin `empresa_id`; RLS sin política |
| `mensajes` | PK uuid ✔; **borrado físico**; sin `empresa_id` (heredar de conversación); RLS sin política |

### Integraciones y acceso
| Tabla | Brechas |
|---|---|
| `claves_api` (2) | PK uuid ✔; **sin `empresa_id`** (la clave hereda del usuario; conviene explícito); `revocada` en vez de `activo`; RLS sin política |
| `webhooks` | PK uuid ✔; **sin `empresa_id`**; `activa` en vez de `activo`; RLS sin política |
| `entregas_webhook` | Log; **sin `empresa_id`**; sin `activo` (correcto); RLS sin política |
| `usuarios` (3) | PK uuid ✔; `es_root` ✔; relación por membresía (correcto); RLS sin política |
| `sesiones` (42) | PK uuid ✔; empresa derivable del usuario (correcto no duplicar); RLS sin política |

### Legado
| Tabla | Brechas |
|---|---|
| `personas` (8) | **Legacy del CRM de empresas retirado**: sin país/documento/empresa; sigue en panel y búsqueda. Decidir retiro o migración a documento genérico. |

## Brechas por severidad

**Bloqueantes**
1. **C7: 0 políticas RLS en 36 tablas.** RLS activo sin políticas da falsa seguridad: cualquier `SECURITY DEFINER` lee todo y el aislamiento no existe.
2. **C2: `empresa_id` ausente en 19 tablas de negocio.** Tareas, notas, eventos, correos, ingresos, traslados, vínculos, actividad, favoritos, vistas, conversaciones, mensajes, claves, webhooks, entregas e importaciones no se pueden filtrar por empresa.
3. **`colegios_contratados.empresa_id` nullable**: hoy se puede contratar un colegio sin empresa, y entonces no hay tenant fiscal.

**Mayores**
4. `favoritos` sin `id uuid` (C1).
5. `conversaciones` y `mensajes` con borrado físico (C5).
6. `alumno_apoderado` sin `activo`; `traslados` sin `creado_en` (C5/C6).
7. `oportunidades.importe` sin `moneda_iso` (C4), aunque la empresa tenga moneda funcional.
8. `puntos_emision` sin `folio_actual`: los rangos existen pero nadie los consume.
9. `importaciones` sin `empresa_id`/`usuario_id`: no se puede auditar quién importó qué.
10. `personas` legacy sin decisión.

**Menores**
11. Sin `actualizado_en` en holdings, empresas, colegios, alumnos, apoderados, oportunidades, tareas, etc.
12. `activo` vs `activa` vs `vigente` vs `revocada`: tres nombres para el mismo concepto.
13. FKs con `ON DELETE` mezclados (`a` no action, `c` cascade, `n` set null) sin criterio escrito.
14. Catálogos con RLS sin política (lectura pública no declarada).

## Plan de normalización (estado)

- **044 aplicada (2026-11-18):** empresa_id en las 16 tablas restantes con triggers de herencia y backfill; importaciones registra usuario y empresa.
- **045 aplicada (2026-11-19):** favoritos con id uuid + unique natural; alumno_apoderado, conversaciones y mensajes con `activo`; traslados con `creado_en`.

## Plan de normalización propuesto

- **044 — Tenant en todo el negocio**: `empresa_id` en las 19 tablas, con triggers de herencia (tarea/nota/evento/correo/ingreso/traslado/vínculo ← alumno u oportunidad; actividad ← registro; favoritos/vistas/conversaciones/claves/webhooks/importaciones ← usuario). `colegios_contratados.empresa_id` obligatorio cuando hay contrato.
- **045 — Soft-delete y PK**: `favoritos` con `id uuid` + unique; `activo` en alumno_apoderado, conversaciones y mensajes; `creado_en` en traslados; unificar `activo`/`activa`/`vigente`/`revocada` con vistas de compatibilidad.
- **046 — Dinero explícito**: `oportunidades.moneda_iso` (default de la empresa), `puntos_emision.folio_actual` y `siguiente_folio(empresa, tipo)`; unique de folio por empresa.
- **047 — RLS real**: filtro explícito por `empresa_id` en `listar_*/panel/buscar` (root ve todo) **más** políticas RLS como defensa en profundidad; requiere fijar `app.empresa_id` por transacción.
- **048 — Legacy**: retirar `personas` o migrarla a documento genérico; decidir `supresion` global o por empresa.

No se propone tocar los 12.039 colegios del directorio: su PK RBD y su país son correctos y no son tenant.
