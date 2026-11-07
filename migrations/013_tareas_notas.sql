-- 013 — Tareas y notas vinculadas a alumnos, apoderados y postulaciones.
-- CRUD seguro + marcar_tarea + búsqueda de texto en notas.

create extension if not exists pg_trgm;

create table if not exists tareas (
  id uuid primary key default gen_random_uuid(),
  titulo text not null check (char_length(btrim(titulo)) between 1 and 200),
  detalle text not null default '',
  estado text not null default 'pendiente'
    check (estado in ('pendiente', 'en_curso', 'hecha')),
  prioridad text not null default 'media'
    check (prioridad in ('baja', 'media', 'alta')),
  vencimiento date,
  alumno_id uuid references alumnos(id) on delete cascade,
  apoderado_id uuid references apoderados(id) on delete cascade,
  oportunidad_id uuid references oportunidades(id) on delete cascade,
  creado_en timestamptz not null default now()
);
comment on table tareas is 'Tareas vinculadas al quehacer escolar';

create table if not exists notas (
  id uuid primary key default gen_random_uuid(),
  titulo text not null default '',
  cuerpo text not null check (char_length(btrim(cuerpo)) > 0),
  alumno_id uuid references alumnos(id) on delete cascade,
  apoderado_id uuid references apoderados(id) on delete cascade,
  oportunidad_id uuid references oportunidades(id) on delete cascade,
  creado_en timestamptz not null default now()
);
comment on table notas is 'Notas libres vinculadas a registros';

create index if not exists idx_tareas_estado on tareas (estado);
create index if not exists idx_tareas_vencimiento on tareas (vencimiento);
create index if not exists idx_tareas_alumno on tareas (alumno_id);
create index if not exists idx_notas_alumno on notas (alumno_id);
create index if not exists idx_notas_cuerpo on notas using gin (cuerpo gin_trgm_ops);

alter table tareas enable row level security;
alter table notas enable row level security;
revoke all on tareas, notas from public;
grant select on tareas, notas to public;

create or replace function listar_tareas(
  p_busqueda text default '',
  p_estado text default '',
  p_limite integer default 50,
  p_desplazamiento integer default 0
)
returns setof tareas
language plpgsql security definer set search_path = public
as $$
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
  where (v_busqueda = '' or t.titulo ilike '%' || v_busqueda || '%' or t.detalle ilike '%' || v_busqueda || '%')
    and (v_estado = '' or t.estado = v_estado)
  order by t.vencimiento nulls last, t.creado_en desc
  limit v_limite offset v_desp;
end;
$$;

create or replace function obtener_tarea(p_id uuid)
returns tareas
language plpgsql security definer set search_path = public
as $$
declare v tareas%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  select * into v from tareas where id = p_id;
  if not found then raise exception 'Tarea no encontrada.' using errcode = 'P0002'; end if;
  return v;
end;
$$;

create or replace function crear_tarea(
  p_titulo text, p_detalle text default '', p_estado text default 'pendiente',
  p_prioridad text default 'media', p_vencimiento date default null,
  p_alumno_id uuid default null, p_apoderado_id uuid default null, p_oportunidad_id uuid default null
)
returns tareas
language plpgsql security definer set search_path = public
as $$
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
  if p_alumno_id is not null and not exists (select 1 from alumnos where id = p_alumno_id) then
    raise exception 'El alumno indicado no existe.' using errcode = 'P0001';
  end if;
  if p_apoderado_id is not null and not exists (select 1 from apoderados where id = p_apoderado_id) then
    raise exception 'El apoderado indicado no existe.' using errcode = 'P0001';
  end if;
  if p_oportunidad_id is not null and not exists (select 1 from oportunidades where id = p_oportunidad_id) then
    raise exception 'La postulación indicada no existe.' using errcode = 'P0001';
  end if;
  insert into tareas (titulo, detalle, estado, prioridad, vencimiento, alumno_id, apoderado_id, oportunidad_id)
  values (v_titulo, btrim(coalesce(p_detalle, '')), v_estado, v_prioridad, p_vencimiento,
          p_alumno_id, p_apoderado_id, p_oportunidad_id)
  returning * into v_tarea;
  return v_tarea;
end;
$$;

create or replace function actualizar_tarea(
  p_id uuid, p_titulo text, p_detalle text default '', p_estado text default 'pendiente',
  p_prioridad text default 'media', p_vencimiento date default null
)
returns tareas
language plpgsql security definer set search_path = public
as $$
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
  where id = p_id returning * into v_tarea;
  if not found then raise exception 'Tarea no encontrada.' using errcode = 'P0002'; end if;
  return v_tarea;
end;
$$;

create or replace function marcar_tarea(p_id uuid, p_estado text)
returns tareas
language plpgsql security definer set search_path = public
as $$
declare
  v_estado text := lower(btrim(coalesce(p_estado, '')));
  v_tarea tareas%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if v_estado not in ('pendiente', 'en_curso', 'hecha') then raise exception 'Estado no válido.' using errcode = 'P0001'; end if;
  update tareas set estado = v_estado where id = p_id returning * into v_tarea;
  if not found then raise exception 'Tarea no encontrada.' using errcode = 'P0002'; end if;
  return v_tarea;
end;
$$;

create or replace function eliminar_tarea(p_id uuid)
returns boolean
language plpgsql security definer set search_path = public
as $$
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  delete from tareas where id = p_id;
  if not found then raise exception 'Tarea no encontrada.' using errcode = 'P0002'; end if;
  return true;
end;
$$;

create or replace function listar_notas(
  p_busqueda text default '',
  p_limite integer default 50,
  p_desplazamiento integer default 0
)
returns setof notas
language plpgsql security definer set search_path = public
as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 50), 1), 100);
  v_desp integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_busqueda text := btrim(coalesce(p_busqueda, ''));
begin
  return query select * from notas n
  where v_busqueda = '' or n.titulo ilike '%' || v_busqueda || '%' or n.cuerpo ilike '%' || v_busqueda || '%'
  order by n.creado_en desc
  limit v_limite offset v_desp;
end;
$$;

create or replace function obtener_nota(p_id uuid)
returns notas
language plpgsql security definer set search_path = public
as $$
declare v notas%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  select * into v from notas where id = p_id;
  if not found then raise exception 'Nota no encontrada.' using errcode = 'P0002'; end if;
  return v;
end;
$$;

create or replace function crear_nota(
  p_titulo text default '', p_cuerpo text default '',
  p_alumno_id uuid default null, p_apoderado_id uuid default null, p_oportunidad_id uuid default null
)
returns notas
language plpgsql security definer set search_path = public
as $$
declare v_nota notas%rowtype;
begin
  if btrim(coalesce(p_cuerpo, '')) = '' then raise exception 'El contenido es obligatorio.' using errcode = 'P0001'; end if;
  if p_alumno_id is not null and not exists (select 1 from alumnos where id = p_alumno_id) then
    raise exception 'El alumno indicado no existe.' using errcode = 'P0001';
  end if;
  if p_apoderado_id is not null and not exists (select 1 from apoderados where id = p_apoderado_id) then
    raise exception 'El apoderado indicado no existe.' using errcode = 'P0001';
  end if;
  if p_oportunidad_id is not null and not exists (select 1 from oportunidades where id = p_oportunidad_id) then
    raise exception 'La postulación indicada no existe.' using errcode = 'P0001';
  end if;
  insert into notas (titulo, cuerpo, alumno_id, apoderado_id, oportunidad_id)
  values (btrim(coalesce(p_titulo, '')), btrim(p_cuerpo), p_alumno_id, p_apoderado_id, p_oportunidad_id)
  returning * into v_nota;
  return v_nota;
end;
$$;

create or replace function actualizar_nota(p_id uuid, p_titulo text default '', p_cuerpo text default '')
returns notas
language plpgsql security definer set search_path = public
as $$
declare v_nota notas%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if btrim(coalesce(p_cuerpo, '')) = '' then raise exception 'El contenido es obligatorio.' using errcode = 'P0001'; end if;
  update notas set titulo = btrim(coalesce(p_titulo, '')), cuerpo = btrim(p_cuerpo)
  where id = p_id returning * into v_nota;
  if not found then raise exception 'Nota no encontrada.' using errcode = 'P0002'; end if;
  return v_nota;
end;
$$;

create or replace function eliminar_nota(p_id uuid)
returns boolean
language plpgsql security definer set search_path = public
as $$
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  delete from notas where id = p_id;
  if not found then raise exception 'Nota no encontrada.' using errcode = 'P0002'; end if;
  return true;
end;
$$;

create or replace function tareas_de_alumno(p_alumno_id uuid)
returns setof tareas
language sql security definer set search_path = public
as $$ select * from tareas where alumno_id = p_alumno_id order by vencimiento nulls last, creado_en desc; $$;

create or replace function notas_de_alumno(p_alumno_id uuid)
returns setof notas
language sql security definer set search_path = public
as $$ select * from notas where alumno_id = p_alumno_id order by creado_en desc; $$;

grant execute on function listar_tareas(text, text, integer, integer) to public;
grant execute on function obtener_tarea(uuid) to public;
grant execute on function crear_tarea(text, text, text, text, date, uuid, uuid, uuid) to public;
grant execute on function actualizar_tarea(uuid, text, text, text, text, date) to public;
grant execute on function marcar_tarea(uuid, text) to public;
grant execute on function eliminar_tarea(uuid) to public;
grant execute on function listar_notas(text, integer, integer) to public;
grant execute on function obtener_nota(uuid) to public;
grant execute on function crear_nota(text, text, uuid, uuid, uuid) to public;
grant execute on function actualizar_nota(uuid, text, text) to public;
grant execute on function eliminar_nota(uuid) to public;
grant execute on function tareas_de_alumno(uuid) to public;
grant execute on function notas_de_alumno(uuid) to public;
