-- 028 — Borrado lógico + anonimización (protección de datos).
-- Nada se borra: desactivar oculta de listas; anonimizar reemplaza datos
-- personales por valores por defecto. Los UUID se conservan, así que informes
-- e historial siguen consistentes. Auditoría, vínculos y datos operativos
-- (sesiones, favoritos, vistas, conversaciones, automatizaciones, webhooks,
-- claves, traslados-historial) no se tocan.

alter table personas add column if not exists activo boolean not null default true;
alter table oportunidades add column if not exists activo boolean not null default true;
alter table alumnos add column if not exists activo boolean not null default true;
alter table apoderados add column if not exists activo boolean not null default true;
alter table tareas add column if not exists activo boolean not null default true;
alter table notas add column if not exists activo boolean not null default true;
alter table eventos add column if not exists activo boolean not null default true;
alter table correos add column if not exists activo boolean not null default true;
alter table ingresos_medicos add column if not exists activo boolean not null default true;
alter table colegios add column if not exists activo boolean not null default true;

-- Unicidad solo entre activos: los anonimizados comparten valores por defecto.
alter table alumnos drop constraint if exists alumnos_rut_key;
alter table apoderados drop constraint if exists apoderados_rut_key;
alter table personas drop constraint if exists personas_correo_key;
create unique index if not exists uq_alumnos_rut_activo on alumnos (rut) where activo;
create unique index if not exists uq_apoderados_rut_activo on apoderados (rut) where activo;
create unique index if not exists uq_personas_correo_activo on personas (correo) where activo;

-- Personas: listas y lectura solo activas; baja desactiva; edición exige activa.
create or replace function listar_personas(
  p_busqueda text default '', p_limite integer default 50, p_desplazamiento integer default 0
)
returns setof personas language plpgsql security definer set search_path = public as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 50), 1), 100);
  v_desplazamiento integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_busqueda text := btrim(coalesce(p_busqueda, ''));
begin
  return query select * from personas p
  where p.activo
    and (v_busqueda = ''
     or (p.nombre || ' ' || p.apellido) ilike '%' || v_busqueda || '%'
     or p.correo ilike '%' || v_busqueda || '%'
     or p.puesto ilike '%' || v_busqueda || '%')
  order by p.apellido asc, p.nombre asc
  limit v_limite offset v_desplazamiento;
end; $$;

create or replace function obtener_persona(p_id uuid)
returns personas language plpgsql security definer set search_path = public as $$
declare v_persona personas%rowtype;
begin
  if p_id is null then raise exception 'El identificador de la persona es obligatorio.' using errcode = 'P0001'; end if;
  select * into v_persona from personas where id = p_id and activo;
  if not found then raise exception 'Persona no encontrada.' using errcode = 'P0002'; end if;
  return v_persona;
end; $$;

create or replace function actualizar_persona(p_id uuid, p_nombre text, p_apellido text, p_correo text, p_telefono text default '', p_puesto text default '', p_ciudad text default '', p_region_codigo smallint default null, p_comuna_codigo text default null)
returns personas language plpgsql security definer set search_path = public as $$
declare v_nombre text := btrim(coalesce(p_nombre, '')); v_apellido text := btrim(coalesce(p_apellido, ''));
v_correo text := lower(btrim(coalesce(p_correo, ''))); v_persona personas%rowtype; v_comuna text := nullif(btrim(coalesce(p_comuna_codigo, '')), '');
begin
  if p_id is null then raise exception 'El identificador de la persona es obligatorio.' using errcode = 'P0001'; end if;
  if v_nombre = '' then raise exception 'El nombre es obligatorio.' using errcode = 'P0001'; end if;
  if v_apellido = '' then raise exception 'Los apellidos son obligatorios.' using errcode = 'P0001'; end if;
  if v_correo !~ '^[^@\s]+@[^@\s]+\.[^@\s]+$' then raise exception 'El correo no es válido.' using errcode = 'P0001'; end if;
  if p_region_codigo is not null and not exists (select 1 from regiones where codigo = p_region_codigo) then raise exception 'La región no es válida.' using errcode = 'P0001'; end if;
  if v_comuna is not null then
    if p_region_codigo is null then raise exception 'Elija la región de la comuna.' using errcode = 'P0001'; end if;
    if not exists (select 1 from comunas where codigo = v_comuna and region = p_region_codigo) then raise exception 'La comuna no pertenece a la región.' using errcode = 'P0001'; end if;
  end if;
  update personas set nombre = v_nombre, apellido = v_apellido, correo = v_correo, telefono = btrim(coalesce(p_telefono, '')), puesto = btrim(coalesce(p_puesto, '')), ciudad = btrim(coalesce(p_ciudad, '')), region_codigo = p_region_codigo, comuna_codigo = v_comuna
  where id = p_id and activo returning * into v_persona;
  if not found then raise exception 'Persona no encontrada.' using errcode = 'P0002'; end if;
  return v_persona;
exception when unique_violation then raise exception 'Ya existe una persona con ese correo.' using errcode = 'P0003';
end; $$;

create or replace function eliminar_persona(p_id uuid)
returns boolean language plpgsql security definer set search_path = public as $$
begin
  if p_id is null then raise exception 'El identificador de la persona es obligatorio.' using errcode = 'P0001'; end if;
  update personas set activo = false where id = p_id;
  if not found then raise exception 'Persona no encontrada.' using errcode = 'P0002'; end if;
  return true;
end; $$;

-- Postulaciones: listas y lectura solo activas; importes de inactivas no suman.
create or replace function listar_oportunidades(p_etapa text default '', p_limite integer default 100, p_desplazamiento integer default 0)
returns setof oportunidades language plpgsql security definer set search_path = public as $$
declare v_limite integer := least(greatest(coalesce(p_limite, 100), 1), 200);
v_desp integer := greatest(coalesce(p_desplazamiento, 0), 0);
v_etapa text := lower(btrim(coalesce(p_etapa, '')));
begin
  if v_etapa <> '' and v_etapa not in ('contacto', 'postulante', 'evaluado', 'aceptado', 'matriculado') then
    raise exception 'Etapa no válida. Use: contacto, postulante, evaluado, aceptado o matriculado.' using errcode = 'P0001';
  end if;
  return query select * from oportunidades o where o.activo and (v_etapa = '' or o.etapa = v_etapa)
  order by o.importe desc limit v_limite offset v_desp;
end; $$;

create or replace function mover_etapa_oportunidad(p_id uuid, p_etapa text)
returns oportunidades language plpgsql security definer set search_path = public as $$
declare v_etapa text := lower(btrim(coalesce(p_etapa, ''))); v oportunidades%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if v_etapa not in ('contacto', 'postulante', 'evaluado', 'aceptado', 'matriculado') then raise exception 'Etapa no válida. Use: contacto, postulante, evaluado, aceptado o matriculado.' using errcode = 'P0001'; end if;
  update oportunidades set etapa = v_etapa where id = p_id and activo returning * into v;
  if not found then raise exception 'Oportunidad no encontrada.' using errcode = 'P0002'; end if;
  return v;
end; $$;

create or replace function actualizar_oportunidad(p_id uuid, p_nombre text, p_importe bigint default 0, p_etapa text default 'contacto', p_colegio_rbd text default null)
returns oportunidades language plpgsql security definer set search_path = public as $$
declare v_nombre text := btrim(coalesce(p_nombre, ''));
v_etapa text := lower(btrim(coalesce(p_etapa, 'contacto'))); v oportunidades%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if v_nombre = '' then raise exception 'El nombre es obligatorio.' using errcode = 'P0001'; end if;
  if coalesce(p_importe, 0) < 0 then raise exception 'El importe no puede ser negativo.' using errcode = 'P0001'; end if;
  if v_etapa not in ('contacto', 'postulante', 'evaluado', 'aceptado', 'matriculado') then raise exception 'Etapa no válida. Use: contacto, postulante, evaluado, aceptado o matriculado.' using errcode = 'P0001'; end if;
  if p_colegio_rbd is not null and not exists (select 1 from colegios where rbd = p_colegio_rbd) then
    raise exception 'El colegio indicado no existe.' using errcode = 'P0001';
  end if;
  update oportunidades set nombre = v_nombre, importe = coalesce(p_importe, 0), etapa = v_etapa, colegio_rbd = p_colegio_rbd where id = p_id and activo returning * into v;
  if not found then raise exception 'Oportunidad no encontrada.' using errcode = 'P0002'; end if;
  return v;
end; $$;

create or replace function eliminar_oportunidad(p_id uuid)
returns boolean language plpgsql security definer set search_path = public as $$
declare v boolean;
begin
  if p_id is null then raise exception 'El identificador de la oportunidad es obligatorio.' using errcode = 'P0001'; end if;
  update oportunidades set activo = false where id = p_id returning true into v;
  if not found then raise exception 'Oportunidad no encontrada.' using errcode = 'P0002'; end if;
  return true;
end; $$;

drop function if exists panel_resumen();
create function panel_resumen()
returns table (total_colegios bigint, total_personas bigint, total_oportunidades bigint, valor_tuberia bigint, valor_ganado bigint, tasa_exito numeric)
language sql security definer set search_path = public as $$
  select (select count(*) from colegios where activo), (select count(*) from personas where activo), (select count(*) from oportunidades where activo),
  (select coalesce(sum(importe), 0) from oportunidades where activo and etapa <> 'matriculado'),
  (select coalesce(sum(importe), 0) from oportunidades where activo and etapa = 'matriculado'),
  case when (select count(*) from oportunidades where activo) = 0 then 0 else round((select count(*) from oportunidades where activo and etapa = 'matriculado')::numeric / (select count(*) from oportunidades where activo)::numeric * 100, 1) end;
$$;
grant execute on function panel_resumen() to public;

-- Alumnos y apoderados: listas y lectura solo activas; baja desactiva.
create or replace function listar_alumnos(
  p_busqueda text default '', p_estado text default '', p_colegio_rbd text default '',
  p_limite integer default 50, p_desplazamiento integer default 0
)
returns setof alumnos language plpgsql security definer set search_path = public as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 50), 1), 100);
  v_desp integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_busqueda text := btrim(coalesce(p_busqueda, ''));
begin
  return query select * from alumnos a
  where a.activo
    and ((v_busqueda = '' or (a.nombres || ' ' || a.apellidos) ilike '%' || v_busqueda || '%'
         or a.rut ilike v_busqueda || '%' or a.curso ilike '%' || v_busqueda || '%')
    and (btrim(coalesce(p_estado, '')) = '' or a.estado = lower(btrim(p_estado)))
    and (btrim(coalesce(p_colegio_rbd, '')) = '' or a.colegio_rbd = btrim(p_colegio_rbd)))
  order by a.apellidos asc, a.nombres asc
  limit v_limite offset v_desp;
end; $$;

create or replace function obtener_alumno(p_id uuid)
returns alumnos language plpgsql security definer set search_path = public as $$
declare v alumnos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  select * into v from alumnos where id = p_id and activo;
  if not found then raise exception 'Alumno no encontrado.' using errcode = 'P0002'; end if;
  return v;
end; $$;

create or replace function eliminar_alumno(p_id uuid)
returns boolean language plpgsql security definer set search_path = public as $$
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update alumnos set activo = false where id = p_id;
  if not found then raise exception 'Alumno no encontrado.' using errcode = 'P0002'; end if;
  return true;
end; $$;

create or replace function listar_apoderados(
  p_busqueda text default '', p_limite integer default 50, p_desplazamiento integer default 0
)
returns setof apoderados language plpgsql security definer set search_path = public as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 50), 1), 100);
  v_desp integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_busqueda text := btrim(coalesce(p_busqueda, ''));
begin
  return query select * from apoderados a
  where a.activo
    and (v_busqueda = ''
     or (a.nombres || ' ' || a.apellidos) ilike '%' || v_busqueda || '%'
     or a.rut ilike v_busqueda || '%')
  order by a.apellidos asc, a.nombres asc
  limit v_limite offset v_desp;
end; $$;

create or replace function obtener_apoderado(p_id uuid)
returns apoderados language plpgsql security definer set search_path = public as $$
declare v apoderados%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  select * into v from apoderados where id = p_id and activo;
  if not found then raise exception 'Apoderado no encontrado.' using errcode = 'P0002'; end if;
  return v;
end; $$;

create or replace function eliminar_apoderado(p_id uuid)
returns boolean language plpgsql security definer set search_path = public as $$
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update apoderados set activo = false where id = p_id;
  if not found then raise exception 'Apoderado no encontrado.' using errcode = 'P0002'; end if;
  return true;
end; $$;

-- Actualizar exige fila activa; crear valida vínculos activos.
create or replace function vincular_apoderado(
  p_alumno_id uuid, p_apoderado_id uuid,
  p_vinculo text default '', p_retiro boolean default false, p_notificaciones boolean default true
)
returns alumno_apoderado language plpgsql security definer set search_path = public as $$
declare v alumno_apoderado%rowtype;
begin
  if p_alumno_id is null or p_apoderado_id is null then
    raise exception 'Alumno y apoderado son obligatorios.' using errcode = 'P0001';
  end if;
  if not exists (select 1 from alumnos where id = p_alumno_id and activo) then
    raise exception 'El alumno indicado no existe.' using errcode = 'P0001';
  end if;
  if not exists (select 1 from apoderados where id = p_apoderado_id and activo) then
    raise exception 'El apoderado indicado no existe.' using errcode = 'P0001';
  end if;
  insert into alumno_apoderado (alumno_id, apoderado_id, vinculo, retiro_autorizado, notificaciones)
  values (p_alumno_id, p_apoderado_id, btrim(coalesce(p_vinculo, '')), coalesce(p_retiro, false), coalesce(p_notificaciones, true))
  returning * into v;
  return v;
exception when unique_violation then
  raise exception 'Ese apoderado ya está vinculado al alumno.' using errcode = 'P0003';
end; $$;

create or replace function apoderados_de_alumno(p_alumno_id uuid)
returns table (
  id uuid, rut text, nombres text, apellidos text, telefono text, correo text,
  vinculo text, retiro_autorizado boolean, notificaciones boolean
)
language sql security definer set search_path = public as $$
  select a.id, a.rut, a.nombres, a.apellidos, a.telefono, a.correo,
         v.vinculo, v.retiro_autorizado, v.notificaciones
  from alumno_apoderado v join apoderados a on a.id = v.apoderado_id
  where v.alumno_id = p_alumno_id and a.activo
  order by a.apellidos, a.nombres;
$$;

create or replace function alumnos_de_apoderado(p_apoderado_id uuid)
returns table (
  id uuid, rut text, nombres text, apellidos text, curso text, estado text, vinculo text
)
language sql security definer set search_path = public as $$
  select a.id, a.rut, a.nombres, a.apellidos, a.curso, a.estado, v.vinculo
  from alumno_apoderado v join alumnos a on a.id = v.alumno_id
  where v.apoderado_id = p_apoderado_id and a.activo
  order by a.apellidos, a.nombres;
$$;

create or replace function trasladar_alumno(p_alumno_id uuid, p_destino_rbd text, p_motivo text default '')
returns alumnos language plpgsql security definer set search_path = public as $$
declare v_alumno alumnos%rowtype; v_destino text := btrim(coalesce(p_destino_rbd, ''));
begin
  if p_alumno_id is null then raise exception 'El alumno es obligatorio.' using errcode = 'P0001'; end if;
  if v_destino = '' then raise exception 'El colegio destino es obligatorio.' using errcode = 'P0001'; end if;
  select * into v_alumno from alumnos where id = p_alumno_id and activo;
  if not found then raise exception 'Alumno no encontrado.' using errcode = 'P0002'; end if;
  if not exists (select 1 from colegios where rbd = v_destino) then
    raise exception 'El colegio destino no existe.' using errcode = 'P0001';
  end if;
  if v_alumno.colegio_rbd is not distinct from v_destino then
    raise exception 'El colegio destino debe ser distinto al actual.' using errcode = 'P0001';
  end if;
  insert into traslados (alumno_id, origen_rbd, destino_rbd, motivo)
  values (p_alumno_id, v_alumno.colegio_rbd, v_destino, btrim(coalesce(p_motivo, '')));
  update alumnos set colegio_rbd = v_destino where id = p_alumno_id returning * into v_alumno;
  return v_alumno;
end; $$;

-- Tareas y notas: listas y lectura solo activas; baja desactiva.
create or replace function listar_tareas(
  p_busqueda text default '', p_estado text default '',
  p_limite integer default 50, p_desplazamiento integer default 0
)
returns setof tareas language plpgsql security definer set search_path = public as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 50), 1), 100);
  v_desp integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_busqueda text := btrim(coalesce(p_busqueda, ''));
  v_estado text := lower(btrim(coalesce(p_estado, '')));
begin
  if v_estado <> '' and v_estado not in ('pendiente', 'en_curso', 'hecha') then
    raise exception 'Estado no válido.' using errcode = 'P0001';
  end if;
  return query select * from tareas t
  where t.activo
    and (v_busqueda = '' or t.titulo ilike '%' || v_busqueda || '%' or t.detalle ilike '%' || v_busqueda || '%')
    and (v_estado = '' or t.estado = v_estado)
  order by t.vencimiento nulls last, t.creado_en desc
  limit v_limite offset v_desp;
end; $$;

create or replace function obtener_tarea(p_id uuid)
returns tareas language plpgsql security definer set search_path = public as $$
declare v tareas%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  select * into v from tareas where id = p_id and activo;
  if not found then raise exception 'Tarea no encontrada.' using errcode = 'P0002'; end if;
  return v;
end; $$;

create or replace function actualizar_tarea(
  p_id uuid, p_titulo text, p_detalle text default '', p_estado text default 'pendiente',
  p_prioridad text default 'media', p_vencimiento date default null
)
returns tareas language plpgsql security definer set search_path = public as $$
declare
  v_titulo text := btrim(coalesce(p_titulo, ''));
  v_estado text := lower(btrim(coalesce(p_estado, 'pendiente')));
  v_prioridad text := lower(btrim(coalesce(p_prioridad, 'media')));
  v_tarea tareas%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if v_titulo = '' then raise exception 'El título es obligatorio.' using errcode = 'P0001'; end if;
  if v_estado not in ('pendiente', 'en_curso', 'hecha') then raise exception 'Estado no válido.' using errcode = 'P0001'; end if;
  if v_prioridad not in ('baja', 'media', 'alta') then raise exception 'Prioridad no válida.' using errcode = 'P0001'; end if;
  update tareas set titulo = v_titulo, detalle = btrim(coalesce(p_detalle, '')),
    estado = v_estado, prioridad = v_prioridad, vencimiento = p_vencimiento
  where id = p_id and activo returning * into v_tarea;
  if not found then raise exception 'Tarea no encontrada.' using errcode = 'P0002'; end if;
  return v_tarea;
end; $$;

create or replace function marcar_tarea(p_id uuid, p_estado text)
returns tareas language plpgsql security definer set search_path = public as $$
declare v_estado text := lower(btrim(coalesce(p_estado, ''))); v_tarea tareas%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if v_estado not in ('pendiente', 'en_curso', 'hecha') then raise exception 'Estado no válido.' using errcode = 'P0001'; end if;
  update tareas set estado = v_estado where id = p_id and activo returning * into v_tarea;
  if not found then raise exception 'Tarea no encontrada.' using errcode = 'P0002'; end if;
  return v_tarea;
end; $$;

create or replace function eliminar_tarea(p_id uuid)
returns boolean language plpgsql security definer set search_path = public as $$
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update tareas set activo = false where id = p_id;
  if not found then raise exception 'Tarea no encontrada.' using errcode = 'P0002'; end if;
  return true;
end; $$;

create or replace function listar_notas(
  p_busqueda text default '', p_limite integer default 50, p_desplazamiento integer default 0
)
returns setof notas language plpgsql security definer set search_path = public as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 50), 1), 100);
  v_desp integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_busqueda text := btrim(coalesce(p_busqueda, ''));
begin
  return query select * from notas n
  where n.activo
    and (v_busqueda = '' or n.titulo ilike '%' || v_busqueda || '%' or n.cuerpo ilike '%' || v_busqueda || '%')
  order by n.creado_en desc
  limit v_limite offset v_desp;
end; $$;

create or replace function obtener_nota(p_id uuid)
returns notas language plpgsql security definer set search_path = public as $$
declare v notas%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  select * into v from notas where id = p_id and activo;
  if not found then raise exception 'Nota no encontrada.' using errcode = 'P0002'; end if;
  return v;
end; $$;

create or replace function actualizar_nota(p_id uuid, p_titulo text default '', p_cuerpo text default '')
returns notas language plpgsql security definer set search_path = public as $$
declare v_nota notas%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if btrim(coalesce(p_cuerpo, '')) = '' then raise exception 'El contenido es obligatorio.' using errcode = 'P0001'; end if;
  update notas set titulo = btrim(coalesce(p_titulo, '')), cuerpo = btrim(p_cuerpo)
  where id = p_id and activo returning * into v_nota;
  if not found then raise exception 'Nota no encontrada.' using errcode = 'P0002'; end if;
  return v_nota;
end; $$;

create or replace function eliminar_nota(p_id uuid)
returns boolean language plpgsql security definer set search_path = public as $$
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update notas set activo = false where id = p_id;
  if not found then raise exception 'Nota no encontrada.' using errcode = 'P0002'; end if;
  return true;
end; $$;

create or replace function tareas_de_alumno(p_alumno_id uuid)
returns setof tareas language sql security definer set search_path = public
as $$ select * from tareas where alumno_id = p_alumno_id and activo order by vencimiento nulls last, creado_en desc; $$;

create or replace function notas_de_alumno(p_alumno_id uuid)
returns setof notas language sql security definer set search_path = public
as $$ select * from notas where alumno_id = p_alumno_id and activo order by creado_en desc; $$;

-- Eventos y correos: listas y lectura solo activas; baja desactiva.
create or replace function listar_eventos(
  p_busqueda text default '', p_limite integer default 50, p_desplazamiento integer default 0
)
returns setof eventos language plpgsql security definer set search_path = public as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 50), 1), 100);
  v_desp integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_busqueda text := btrim(coalesce(p_busqueda, ''));
begin
  return query select * from eventos e
  where e.activo
    and (v_busqueda = '' or e.titulo ilike '%' || v_busqueda || '%'
    or e.lugar ilike '%' || v_busqueda || '%' or e.descripcion ilike '%' || v_busqueda || '%')
  order by e.inicio, e.creado_en desc
  limit v_limite offset v_desp;
end; $$;

create or replace function obtener_evento(p_id uuid)
returns eventos language plpgsql security definer set search_path = public as $$
declare v eventos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  select * into v from eventos where id = p_id and activo;
  if not found then raise exception 'Evento no encontrado.' using errcode = 'P0002'; end if;
  return v;
end; $$;

create or replace function actualizar_evento(
  p_id uuid, p_titulo text, p_inicio timestamptz, p_fin timestamptz,
  p_lugar text default '', p_descripcion text default ''
)
returns eventos language plpgsql security definer set search_path = public as $$
declare v_titulo text := btrim(coalesce(p_titulo, '')); v_evento eventos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if v_titulo = '' then raise exception 'El título es obligatorio.' using errcode = 'P0001'; end if;
  if p_inicio is null or p_fin is null then raise exception 'El inicio y el fin son obligatorios.' using errcode = 'P0001'; end if;
  if p_fin < p_inicio then raise exception 'El fin no puede ser anterior al inicio.' using errcode = 'P0001'; end if;
  update eventos set titulo = v_titulo, inicio = p_inicio, fin = p_fin,
    lugar = btrim(coalesce(p_lugar, '')), descripcion = btrim(coalesce(p_descripcion, ''))
  where id = p_id and activo returning * into v_evento;
  if not found then raise exception 'Evento no encontrado.' using errcode = 'P0002'; end if;
  return v_evento;
end; $$;

create or replace function eliminar_evento(p_id uuid)
returns boolean language plpgsql security definer set search_path = public as $$
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update eventos set activo = false where id = p_id;
  if not found then raise exception 'Evento no encontrado.' using errcode = 'P0002'; end if;
  return true;
end; $$;

create or replace function listar_eventos_mes(p_ano integer, p_mes integer)
returns setof eventos language plpgsql security definer set search_path = public as $$
declare v_inicio date; v_fin date;
begin
  if coalesce(p_ano, 0) < 2000 or coalesce(p_ano, 0) > 2100 then raise exception 'Año no válido.' using errcode = 'P0001'; end if;
  if coalesce(p_mes, 0) not between 1 and 12 then raise exception 'Mes no válido (1 a 12).' using errcode = 'P0001'; end if;
  v_inicio := make_date(p_ano, p_mes, 1);
  v_fin := (v_inicio + interval '1 month')::date;
  return query select * from eventos e
  where e.activo and e.inicio < v_fin and e.fin >= v_inicio
  order by e.inicio;
end; $$;

create or replace function listar_correos(
  p_busqueda text default '', p_solo_no_leidos boolean default false,
  p_limite integer default 50, p_desplazamiento integer default 0
)
returns setof correos language plpgsql security definer set search_path = public as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 50), 1), 100);
  v_desp integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_busqueda text := btrim(coalesce(p_busqueda, ''));
begin
  return query select * from correos c
  where c.activo
    and (v_busqueda = '' or c.asunto ilike '%' || v_busqueda || '%'
      or c.cuerpo ilike '%' || v_busqueda || '%' or c.remitente ilike '%' || v_busqueda || '%')
    and (not coalesce(p_solo_no_leidos, false) or not c.leido)
  order by c.recibido_en desc
  limit v_limite offset v_desp;
end; $$;

create or replace function obtener_correo(p_id uuid)
returns correos language plpgsql security definer set search_path = public as $$
declare v correos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  select * into v from correos where id = p_id and activo;
  if not found then raise exception 'Correo no encontrado.' using errcode = 'P0002'; end if;
  return v;
end; $$;

create or replace function marcar_correo_leido(p_id uuid, p_leido boolean default true)
returns correos language plpgsql security definer set search_path = public as $$
declare v correos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update correos set leido = coalesce(p_leido, true) where id = p_id and activo returning * into v;
  if not found then raise exception 'Correo no encontrado.' using errcode = 'P0002'; end if;
  return v;
end; $$;

create or replace function eliminar_correo(p_id uuid)
returns boolean language plpgsql security definer set search_path = public as $$
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update correos set activo = false where id = p_id;
  if not found then raise exception 'Correo no encontrado.' using errcode = 'P0002'; end if;
  return true;
end; $$;

-- Vínculos en crear_* solo aceptan registros activos.
create or replace function crear_tarea(
  p_titulo text, p_detalle text default '', p_estado text default 'pendiente',
  p_prioridad text default 'media', p_vencimiento date default null,
  p_alumno_id uuid default null, p_apoderado_id uuid default null, p_oportunidad_id uuid default null
)
returns tareas language plpgsql security definer set search_path = public as $$
declare
  v_titulo text := btrim(coalesce(p_titulo, ''));
  v_estado text := lower(btrim(coalesce(p_estado, 'pendiente')));
  v_prioridad text := lower(btrim(coalesce(p_prioridad, 'media')));
  v_tarea tareas%rowtype;
begin
  if v_titulo = '' then raise exception 'El título es obligatorio.' using errcode = 'P0001'; end if;
  if char_length(v_titulo) > 200 then raise exception 'El título no puede superar 200 caracteres.' using errcode = 'P0001'; end if;
  if v_estado not in ('pendiente', 'en_curso', 'hecha') then raise exception 'Estado no válido.' using errcode = 'P0001'; end if;
  if v_prioridad not in ('baja', 'media', 'alta') then raise exception 'Prioridad no válida.' using errcode = 'P0001'; end if;
  if p_alumno_id is not null and not exists (select 1 from alumnos where id = p_alumno_id and activo) then
    raise exception 'El alumno indicado no existe.' using errcode = 'P0001';
  end if;
  if p_apoderado_id is not null and not exists (select 1 from apoderados where id = p_apoderado_id and activo) then
    raise exception 'El apoderado indicado no existe.' using errcode = 'P0001';
  end if;
  if p_oportunidad_id is not null and not exists (select 1 from oportunidades where id = p_oportunidad_id and activo) then
    raise exception 'La postulación indicada no existe.' using errcode = 'P0001';
  end if;
  insert into tareas (titulo, detalle, estado, prioridad, vencimiento, alumno_id, apoderado_id, oportunidad_id)
  values (v_titulo, btrim(coalesce(p_detalle, '')), v_estado, v_prioridad, p_vencimiento,
          p_alumno_id, p_apoderado_id, p_oportunidad_id)
  returning * into v_tarea;
  return v_tarea;
end; $$;

create or replace function crear_nota(
  p_titulo text default '', p_cuerpo text default '',
  p_alumno_id uuid default null, p_apoderado_id uuid default null, p_oportunidad_id uuid default null
)
returns notas language plpgsql security definer set search_path = public as $$
declare v_nota notas%rowtype;
begin
  if btrim(coalesce(p_cuerpo, '')) = '' then raise exception 'El contenido es obligatorio.' using errcode = 'P0001'; end if;
  if p_alumno_id is not null and not exists (select 1 from alumnos where id = p_alumno_id and activo) then
    raise exception 'El alumno indicado no existe.' using errcode = 'P0001';
  end if;
  if p_apoderado_id is not null and not exists (select 1 from apoderados where id = p_apoderado_id and activo) then
    raise exception 'El apoderado indicado no existe.' using errcode = 'P0001';
  end if;
  if p_oportunidad_id is not null and not exists (select 1 from oportunidades where id = p_oportunidad_id and activo) then
    raise exception 'La postulación indicada no existe.' using errcode = 'P0001';
  end if;
  insert into notas (titulo, cuerpo, alumno_id, apoderado_id, oportunidad_id)
  values (btrim(coalesce(p_titulo, '')), btrim(p_cuerpo), p_alumno_id, p_apoderado_id, p_oportunidad_id)
  returning * into v_nota;
  return v_nota;
end; $$;

create or replace function crear_evento(
  p_titulo text, p_inicio timestamptz, p_fin timestamptz,
  p_lugar text default '', p_descripcion text default '',
  p_alumno_id uuid default null, p_apoderado_id uuid default null, p_oportunidad_id uuid default null
)
returns eventos language plpgsql security definer set search_path = public as $$
declare v_titulo text := btrim(coalesce(p_titulo, '')); v_evento eventos%rowtype;
begin
  if v_titulo = '' then raise exception 'El título es obligatorio.' using errcode = 'P0001'; end if;
  if char_length(v_titulo) > 200 then raise exception 'El título no puede superar 200 caracteres.' using errcode = 'P0001'; end if;
  if p_inicio is null or p_fin is null then raise exception 'El inicio y el fin son obligatorios.' using errcode = 'P0001'; end if;
  if p_fin < p_inicio then raise exception 'El fin no puede ser anterior al inicio.' using errcode = 'P0001'; end if;
  if p_alumno_id is not null and not exists (select 1 from alumnos where id = p_alumno_id and activo) then
    raise exception 'El alumno indicado no existe.' using errcode = 'P0001';
  end if;
  if p_apoderado_id is not null and not exists (select 1 from apoderados where id = p_apoderado_id and activo) then
    raise exception 'El apoderado indicado no existe.' using errcode = 'P0001';
  end if;
  if p_oportunidad_id is not null and not exists (select 1 from oportunidades where id = p_oportunidad_id and activo) then
    raise exception 'La postulación indicada no existe.' using errcode = 'P0001';
  end if;
  insert into eventos (titulo, inicio, fin, lugar, descripcion, alumno_id, apoderado_id, oportunidad_id)
  values (v_titulo, p_inicio, p_fin, btrim(coalesce(p_lugar, '')), btrim(coalesce(p_descripcion, '')),
          p_alumno_id, p_apoderado_id, p_oportunidad_id)
  returning * into v_evento;
  return v_evento;
end; $$;

create or replace function crear_correo(
  p_asunto text default '', p_cuerpo text default '', p_remitente text default '',
  p_alumno_id uuid default null, p_apoderado_id uuid default null, p_oportunidad_id uuid default null
)
returns correos language plpgsql security definer set search_path = public as $$
declare v_remitente text := lower(btrim(coalesce(p_remitente, ''))); v_correo correos%rowtype;
begin
  if btrim(coalesce(p_cuerpo, '')) = '' then raise exception 'El contenido es obligatorio.' using errcode = 'P0001'; end if;
  if v_remitente !~ '^[^@\s]+@[^@\s]+\.[^@\s]+$' then
    raise exception 'El remitente no es un correo válido.' using errcode = 'P0001';
  end if;
  if p_alumno_id is not null and not exists (select 1 from alumnos where id = p_alumno_id and activo) then
    raise exception 'El alumno indicado no existe.' using errcode = 'P0001';
  end if;
  if p_apoderado_id is not null and not exists (select 1 from apoderados where id = p_apoderado_id and activo) then
    raise exception 'El apoderado indicado no existe.' using errcode = 'P0001';
  end if;
  if p_oportunidad_id is not null and not exists (select 1 from oportunidades where id = p_oportunidad_id and activo) then
    raise exception 'La postulación indicada no existe.' using errcode = 'P0001';
  end if;
  insert into correos (asunto, cuerpo, remitente, alumno_id, apoderado_id, oportunidad_id)
  values (btrim(coalesce(p_asunto, '')), btrim(p_cuerpo), v_remitente,
          p_alumno_id, p_apoderado_id, p_oportunidad_id)
  returning * into v_correo;
  return v_correo;
end; $$;

-- Ingresos médicos: solo de alumnos activos; baja desactiva.
create or replace function listar_ingresos(p_alumno_id uuid)
returns setof ingresos_medicos language plpgsql security definer set search_path = public as $$
begin
  if p_alumno_id is null then raise exception 'El alumno es obligatorio.' using errcode = 'P0001'; end if;
  if not exists (select 1 from alumnos where id = p_alumno_id) then
    raise exception 'Alumno no encontrado.' using errcode = 'P0002';
  end if;
  if not exists (select 1 from alumnos where id = p_alumno_id and activo) then return; end if;
  return query select * from ingresos_medicos where alumno_id = p_alumno_id and activo order by fecha desc, creado_en desc;
end; $$;

create or replace function crear_ingreso(
  p_alumno_id uuid, p_fecha date default null,
  p_motivo text default '', p_detalle text default '', p_profesional text default ''
)
returns ingresos_medicos language plpgsql security definer set search_path = public as $$
declare v_motivo text := btrim(coalesce(p_motivo, '')); v_ingreso ingresos_medicos%rowtype;
begin
  if p_alumno_id is null then raise exception 'El alumno es obligatorio.' using errcode = 'P0001'; end if;
  if not exists (select 1 from alumnos where id = p_alumno_id and activo) then
    raise exception 'Alumno no encontrado.' using errcode = 'P0002';
  end if;
  if v_motivo = '' then raise exception 'El motivo es obligatorio.' using errcode = 'P0001'; end if;
  if char_length(v_motivo) > 200 then raise exception 'El motivo no puede superar 200 caracteres.' using errcode = 'P0001'; end if;
  insert into ingresos_medicos (alumno_id, fecha, motivo, detalle, profesional)
  values (p_alumno_id, coalesce(p_fecha, current_date), v_motivo,
    left(btrim(coalesce(p_detalle, '')), 2000), left(btrim(coalesce(p_profesional, '')), 120))
  returning * into v_ingreso;
  return v_ingreso;
end; $$;

create or replace function actualizar_ingreso(
  p_id uuid, p_fecha date default null,
  p_motivo text default '', p_detalle text default '', p_profesional text default ''
)
returns ingresos_medicos language plpgsql security definer set search_path = public as $$
declare v_motivo text := btrim(coalesce(p_motivo, '')); v_ingreso ingresos_medicos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if v_motivo = '' then raise exception 'El motivo es obligatorio.' using errcode = 'P0001'; end if;
  update ingresos_medicos set fecha = coalesce(p_fecha, fecha), motivo = v_motivo,
    detalle = left(btrim(coalesce(p_detalle, '')), 2000),
    profesional = left(btrim(coalesce(p_profesional, '')), 120)
  where id = p_id and activo returning * into v_ingreso;
  if not found then raise exception 'Ingreso no encontrado.' using errcode = 'P0002'; end if;
  return v_ingreso;
end; $$;

create or replace function eliminar_ingreso(p_id uuid)
returns boolean language plpgsql security definer set search_path = public as $$
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update ingresos_medicos set activo = false where id = p_id;
  if not found then raise exception 'Ingreso no encontrado.' using errcode = 'P0002'; end if;
  return true;
end; $$;

-- Colegios: directorio con baja lógica (los manuales se ocultan, el seed sigue).
create or replace function listar_colegios(
  p_region smallint default null, p_comuna text default '', p_busqueda text default '',
  p_limite integer default 50, p_desplazamiento integer default 0)
returns setof colegios language plpgsql security definer set search_path = public as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 50), 1), 100);
  v_desp integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_comuna text := btrim(coalesce(p_comuna, ''));
  v_busqueda text := btrim(coalesce(p_busqueda, ''));
begin
  return query select * from colegios c
  where c.activo
    and (p_region is null or c.region = p_region)
    and (v_comuna = '' or c.comuna = v_comuna)
    and (v_busqueda = '' or c.nombre ilike '%' || v_busqueda || '%' or c.rbd = v_busqueda)
  order by c.nombre limit v_limite offset v_desp;
end; $$;

create or replace function obtener_colegio(p_rbd text)
returns colegios language plpgsql security definer set search_path = public as $$
declare v colegios%rowtype;
begin
  if coalesce(btrim(p_rbd), '') = '' then
    raise exception 'El RBD del colegio es obligatorio.' using errcode = 'P0001';
  end if;
  select * into v from colegios where rbd = btrim(p_rbd) and activo;
  if not found then raise exception 'Colegio no encontrado.' using errcode = 'P0002'; end if;
  return v;
end; $$;

create or replace function eliminar_colegio(p_rbd text)
returns boolean language plpgsql security definer set search_path = public as $$
begin
  if coalesce(btrim(p_rbd), '') = '' then
    raise exception 'El RBD del colegio es obligatorio.' using errcode = 'P0001';
  end if;
  update colegios set activo = false where rbd = btrim(p_rbd);
  if not found then raise exception 'Colegio no encontrado.' using errcode = 'P0002'; end if;
  return true;
end; $$;
grant execute on function eliminar_colegio(text) to public;

-- Favoritos: nunca muestran registros desactivados; no se marca lo inactivo.
create or replace function alternar_favorito(p_usuario_id uuid, p_tipo text, p_referencia_id uuid)
returns boolean language plpgsql security definer set search_path = public as $$
declare t text := lower(btrim(coalesce(p_tipo, '')));
begin
  if p_usuario_id is null or p_referencia_id is null then raise exception 'Usuario y registro son obligatorios.' using errcode = 'P0001'; end if;
  if t not in ('alumno', 'apoderado', 'persona', 'postulacion') then raise exception 'Tipo de favorito no válido.' using errcode = 'P0001'; end if;
  if t = 'alumno' and not exists (select 1 from alumnos where id = p_referencia_id and activo) then raise exception 'Alumno no encontrado.' using errcode = 'P0002'; end if;
  if t = 'apoderado' and not exists (select 1 from apoderados where id = p_referencia_id and activo) then raise exception 'Apoderado no encontrado.' using errcode = 'P0002'; end if;
  if t = 'persona' and not exists (select 1 from personas where id = p_referencia_id and activo) then raise exception 'Persona no encontrada.' using errcode = 'P0002'; end if;
  if t = 'postulacion' and not exists (select 1 from oportunidades where id = p_referencia_id and activo) then raise exception 'Postulación no encontrada.' using errcode = 'P0002'; end if;
  delete from favoritos where usuario_id = p_usuario_id and tipo = t and referencia_id = p_referencia_id;
  if found then return false; end if;
  insert into favoritos (usuario_id, tipo, referencia_id) values (p_usuario_id, t, p_referencia_id);
  return true;
end; $$;

create or replace function listar_favoritos(p_usuario_id uuid)
returns table (tipo text, id uuid, titulo text, detalle text, creado_en timestamptz)
language sql security definer set search_path = public as $$
  select f.tipo, f.referencia_id, a.nombres || ' ' || a.apellidos, a.rut, f.creado_en
  from favoritos f join alumnos a on a.id = f.referencia_id
  where f.usuario_id = p_usuario_id and f.tipo = 'alumno' and a.activo
  union all
  select f.tipo, f.referencia_id, a.nombres || ' ' || a.apellidos, coalesce(nullif(a.correo, ''), a.telefono), f.creado_en
  from favoritos f join apoderados a on a.id = f.referencia_id
  where f.usuario_id = p_usuario_id and f.tipo = 'apoderado' and a.activo
  union all
  select f.tipo, f.referencia_id, p.nombre || ' ' || p.apellido, p.correo, f.creado_en
  from favoritos f join personas p on p.id = f.referencia_id
  where f.usuario_id = p_usuario_id and f.tipo = 'persona' and p.activo
  union all
  select f.tipo, f.referencia_id, o.nombre, o.etapa, f.creado_en
  from favoritos f join oportunidades o on o.id = f.referencia_id
  where f.usuario_id = p_usuario_id and f.tipo = 'postulacion' and o.activo
  order by 5 desc;
$$;

-- Búsqueda global: jamás devuelve registros desactivados.
create or replace function buscar_espacio(p_texto text)
returns table(tipo text,id text,titulo text,detalle text,puntaje real)
language plpgsql security definer set search_path=public
as $$
declare q text:=left(btrim(coalesce(p_texto,'')),200); ts tsquery;
begin
  if q='' then return; end if;
  if char_length(q)>=2 then ts:=websearch_to_tsquery('pg_catalog.spanish',q); end if;
  return query
  with r as (
    select 'alumno'::text t,a.id::text rid,a.nombres||' '||a.apellidos titulo,a.curso||' · '||a.rut detalle,
      (case when char_length(q)>=2 and a.busqueda_vector@@ts then ts_rank(a.busqueda_vector,ts)+1 else .1 end)::real rank
    from alumnos a where a.activo and ((char_length(q)>=2 and a.busqueda_vector@@ts) or a.nombres ilike '%'||q||'%' or a.apellidos ilike '%'||q||'%' or a.rut ilike '%'||q||'%' or a.curso ilike '%'||q||'%')
    union all
    select 'apoderado',a.id::text,a.nombres||' '||a.apellidos,coalesce(nullif(a.correo,''),a.telefono),
      (case when char_length(q)>=2 and a.busqueda_vector@@ts then ts_rank(a.busqueda_vector,ts)+1 else .1 end)::real
    from apoderados a where a.activo and ((char_length(q)>=2 and a.busqueda_vector@@ts) or a.nombres ilike '%'||q||'%' or a.apellidos ilike '%'||q||'%' or a.rut ilike '%'||q||'%' or a.correo ilike '%'||q||'%' or a.telefono ilike '%'||q||'%')
    union all
    select 'persona',p.id::text,p.nombre||' '||p.apellido,coalesce(nullif(p.correo,''),p.puesto),
      (case when char_length(q)>=2 and p.busqueda_vector@@ts then ts_rank(p.busqueda_vector,ts)+1 else .1 end)::real
    from personas p where p.activo and ((char_length(q)>=2 and p.busqueda_vector@@ts) or p.nombre ilike '%'||q||'%' or p.apellido ilike '%'||q||'%' or p.correo ilike '%'||q||'%' or p.telefono ilike '%'||q||'%')
    union all
    select 'postulacion',o.id::text,o.nombre,o.etapa||' · '||coalesce(o.colegio_rbd,''),
      (case when char_length(q)>=2 and o.busqueda_vector@@ts then ts_rank(o.busqueda_vector,ts)+1 else .1 end)::real
    from oportunidades o where o.activo and ((char_length(q)>=2 and o.busqueda_vector@@ts) or o.nombre ilike '%'||q||'%' or o.etapa ilike '%'||q||'%' or coalesce(o.colegio_rbd,'') ilike '%'||q||'%')
    union all
    select 'tarea',t.id::text,t.titulo,t.estado||' · '||left(t.detalle,100),
      (case when char_length(q)>=2 and t.busqueda_vector@@ts then ts_rank(t.busqueda_vector,ts)+1 else .1 end)::real
    from tareas t where t.activo and ((char_length(q)>=2 and t.busqueda_vector@@ts) or t.titulo ilike '%'||q||'%' or t.detalle ilike '%'||q||'%')
    union all
    select 'nota',n.id::text,coalesce(nullif(n.titulo,''),left(n.cuerpo,80)),left(n.cuerpo,100),
      (case when char_length(q)>=2 and n.busqueda_vector@@ts then ts_rank(n.busqueda_vector,ts)+1 else .1 end)::real
    from notas n where n.activo and ((char_length(q)>=2 and n.busqueda_vector@@ts) or n.titulo ilike '%'||q||'%' or n.cuerpo ilike '%'||q||'%')
    union all
    select 'evento',e.id::text,e.titulo,e.lugar||' · '||to_char(e.inicio,'DD/MM/YYYY HH24:MI'),
      (case when char_length(q)>=2 and e.busqueda_vector@@ts then ts_rank(e.busqueda_vector,ts)+1 else .1 end)::real
    from eventos e where e.activo and ((char_length(q)>=2 and e.busqueda_vector@@ts) or e.titulo ilike '%'||q||'%' or e.lugar ilike '%'||q||'%' or e.descripcion ilike '%'||q||'%')
    union all
    select 'correo',c.id::text,coalesce(nullif(c.asunto,''),'(sin asunto)'),c.remitente,
      (case when char_length(q)>=2 and c.busqueda_vector@@ts then ts_rank(c.busqueda_vector,ts)+1 else .1 end)::real
    from correos c where c.activo and ((char_length(q)>=2 and c.busqueda_vector@@ts) or c.asunto ilike '%'||q||'%' or c.remitente ilike '%'||q||'%' or c.cuerpo ilike '%'||q||'%')
    union all
    select 'colegio',c.rbd,c.nombre,c.rbd||' · '||c.comuna,
      (case when char_length(q)>=2 and c.busqueda_vector@@ts then ts_rank(c.busqueda_vector,ts)+1 else .1 end)::real
    from colegios c where c.activo and ((char_length(q)>=2 and c.busqueda_vector@@ts) or c.nombre ilike '%'||q||'%' or c.rbd ilike '%'||q||'%')
  ) select r.t,r.rid,r.titulo,r.detalle,r.rank from r order by r.rank desc,r.titulo limit 20;
end; $$;

-- Anonimización (derecho de supresión): desactiva y reemplaza datos
-- personales por valores por defecto. Los UUID e importes se conservan.
create or replace function anonimizar_persona(p_id uuid)
returns personas language plpgsql security definer set search_path = public as $$
declare v personas%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update personas set activo = false, nombre = '—', apellido = '—', correo = '',
    telefono = '', puesto = '', ciudad = '', region_codigo = null, comuna_codigo = null
  where id = p_id returning * into v;
  if not found then raise exception 'Persona no encontrada.' using errcode = 'P0002'; end if;
  return v;
end; $$;

create or replace function anonimizar_alumno(p_id uuid)
returns alumnos language plpgsql security definer set search_path = public as $$
declare v alumnos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update alumnos set activo = false, rut = '00000000-0', nombres = '—', apellidos = '—',
    fecha_nacimiento = null, curso = '', colegio_rbd = null,
    grupo_sanguineo = '', alergias = '', medicamentos = '',
    antecedentes_medicos = '', contacto_emergencia = ''
  where id = p_id returning * into v;
  if not found then raise exception 'Alumno no encontrado.' using errcode = 'P0002'; end if;
  return v;
end; $$;

create or replace function anonimizar_apoderado(p_id uuid)
returns apoderados language plpgsql security definer set search_path = public as $$
declare v apoderados%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update apoderados set activo = false, rut = '00000000-0', nombres = '—', apellidos = '—',
    telefono = '', correo = '', direccion = '', region_codigo = null, comuna_codigo = null,
    ocupacion = ''
  where id = p_id returning * into v;
  if not found then raise exception 'Apoderado no encontrado.' using errcode = 'P0002'; end if;
  return v;
end; $$;

create or replace function anonimizar_oportunidad(p_id uuid)
returns oportunidades language plpgsql security definer set search_path = public as $$
declare v oportunidades%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update oportunidades set activo = false, nombre = '—'
  where id = p_id returning * into v;
  if not found then raise exception 'Oportunidad no encontrada.' using errcode = 'P0002'; end if;
  return v;
end; $$;

grant execute on function anonimizar_persona(uuid) to public;
grant execute on function anonimizar_alumno(uuid) to public;
grant execute on function anonimizar_apoderado(uuid) to public;
grant execute on function anonimizar_oportunidad(uuid) to public;
