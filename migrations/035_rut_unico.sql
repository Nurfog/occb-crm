-- 035 — RUT/correo como dato único real (Q1 resuelto, aditivo).
-- Decisión: la PK sigue siendo `id uuid`; el RUT (y el correo de personas) es
-- dato único de negocio, no clave primaria. Se reemplaza la unicidad parcial
-- `WHERE activo` (que permitía reutilizar el RUT tras una baja) por unicidad
-- real, excluyendo solo el valor de anonimización:
--   - alumnos.rut / apoderados.rut: único salvo '00000000-0'
--   - personas.correo: único salvo ''
-- Así un registro deshabilitado conserva su RUT (no se puede reutilizar) y
-- solo la anonimización admin lo libera.

alter table alumnos drop constraint if exists alumnos_rut_key;
alter table apoderados drop constraint if exists apoderados_rut_key;
alter table personas drop constraint if exists personas_correo_key;
drop index if exists uq_alumnos_rut_activo;
drop index if exists uq_apoderados_rut_activo;
drop index if exists uq_personas_correo_activo;

create unique index if not exists uq_alumnos_rut
  on alumnos (rut) where rut <> '00000000-0';
create unique index if not exists uq_apoderados_rut
  on apoderados (rut) where rut <> '00000000-0';
create unique index if not exists uq_personas_correo
  on personas (lower(correo)) where correo <> '';

comment on index uq_alumnos_rut is 'RUT único de negocio (PK es id uuid); excluye el centinela de anonimización';
comment on index uq_apoderados_rut is 'RUT único de negocio (PK es id uuid); excluye el centinela de anonimización';
comment on index uq_personas_correo is 'Correo único de negocio (PK es id uuid); excluye el valor anonimizado vacío';
