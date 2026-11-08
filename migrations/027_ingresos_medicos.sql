-- 027 — Ingresos médicos del alumno (1 a muchos).
-- Cada ingreso es un evento fechado; los datos puntuales (grupo, alergias,
-- medicamentos, contacto) siguen como columnas de alumnos.

create table if not exists ingresos_medicos (
  id uuid primary key default gen_random_uuid(),
  alumno_id uuid not null references alumnos(id) on delete cascade,
  fecha date not null default current_date,
  motivo text not null check (char_length(btrim(motivo)) between 1 and 200),
  detalle text not null default '' check (char_length(detalle) <= 2000),
  profesional text not null default '' check (char_length(profesional) <= 120),
  creado_en timestamptz not null default now()
);
comment on table ingresos_medicos is 'Ingresos médicos fechados por alumno';

create index if not exists idx_ingresos_alumno on ingresos_medicos (alumno_id, fecha desc);

alter table ingresos_medicos enable row level security;
revoke all on ingresos_medicos from public;
grant select on ingresos_medicos to public;

create or replace function listar_ingresos(p_alumno_id uuid)
returns setof ingresos_medicos
language plpgsql security definer set search_path = public
as $$
begin
  if p_alumno_id is null then raise exception 'El alumno es obligatorio.' using errcode = 'P0001'; end if;
  if not exists (select 1 from alumnos where id = p_alumno_id) then
    raise exception 'Alumno no encontrado.' using errcode = 'P0002';
  end if;
  return query select * from ingresos_medicos where alumno_id = p_alumno_id order by fecha desc, creado_en desc;
end;
$$;

create or replace function crear_ingreso(
  p_alumno_id uuid, p_fecha date default null,
  p_motivo text default '', p_detalle text default '', p_profesional text default ''
)
returns ingresos_medicos
language plpgsql security definer set search_path = public
as $$
declare
  v_motivo text := btrim(coalesce(p_motivo, ''));
  v_ingreso ingresos_medicos%rowtype;
begin
  if p_alumno_id is null then raise exception 'El alumno es obligatorio.' using errcode = 'P0001'; end if;
  if not exists (select 1 from alumnos where id = p_alumno_id) then
    raise exception 'Alumno no encontrado.' using errcode = 'P0002';
  end if;
  if v_motivo = '' then raise exception 'El motivo es obligatorio.' using errcode = 'P0001'; end if;
  if char_length(v_motivo) > 200 then raise exception 'El motivo no puede superar 200 caracteres.' using errcode = 'P0001'; end if;
  insert into ingresos_medicos (alumno_id, fecha, motivo, detalle, profesional)
  values (p_alumno_id, coalesce(p_fecha, current_date), v_motivo,
    left(btrim(coalesce(p_detalle, '')), 2000), left(btrim(coalesce(p_profesional, '')), 120))
  returning * into v_ingreso;
  return v_ingreso;
end;
$$;

create or replace function actualizar_ingreso(
  p_id uuid, p_fecha date default null,
  p_motivo text default '', p_detalle text default '', p_profesional text default ''
)
returns ingresos_medicos
language plpgsql security definer set search_path = public
as $$
declare
  v_motivo text := btrim(coalesce(p_motivo, ''));
  v_ingreso ingresos_medicos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if v_motivo = '' then raise exception 'El motivo es obligatorio.' using errcode = 'P0001'; end if;
  update ingresos_medicos set fecha = coalesce(p_fecha, fecha), motivo = v_motivo,
    detalle = left(btrim(coalesce(p_detalle, '')), 2000),
    profesional = left(btrim(coalesce(p_profesional, '')), 120)
  where id = p_id returning * into v_ingreso;
  if not found then raise exception 'Ingreso no encontrado.' using errcode = 'P0002'; end if;
  return v_ingreso;
end;
$$;

create or replace function eliminar_ingreso(p_id uuid)
returns boolean
language plpgsql security definer set search_path = public
as $$
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  delete from ingresos_medicos where id = p_id;
  if not found then raise exception 'Ingreso no encontrado.' using errcode = 'P0002'; end if;
  return true;
end;
$$;

grant execute on function listar_ingresos(uuid) to public;
grant execute on function crear_ingreso(uuid, date, text, text, text) to public;
grant execute on function actualizar_ingreso(uuid, date, text, text, text) to public;
grant execute on function eliminar_ingreso(uuid) to public;
