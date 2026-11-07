-- 011 — Alumnos y apoderados con RUT chileno validado (módulo 11).
-- Tablas + funciones seguras + RLS + siembra demo.

-- Validador RUT inmutable (forma + dígito verificador).
create or replace function es_rut_valido(p_rut text)
returns boolean
language plpgsql immutable
as $$
declare
  v_limpio text := upper(regexp_replace(coalesce(p_rut, ''), '[^0-9Kk]', '', 'g'));
  v_cuerpo text;
  v_dv text;
  v_suma integer := 0;
  v_factor integer := 2;
  v_i integer;
  v_resto integer;
  v_esperado text;
begin
  if char_length(v_limpio) < 2 then return false; end if;
  v_cuerpo := substring(v_limpio from 1 for char_length(v_limpio) - 1);
  v_dv := substring(v_limpio from char_length(v_limpio) for 1);
  if v_cuerpo !~ '^[0-9]+$' then return false; end if;
  if v_dv !~ '^[0-9K]$' then return false; end if;
  for v_i in reverse char_length(v_cuerpo)..1 loop
    v_suma := v_suma + (substring(v_cuerpo from v_i for 1))::integer * v_factor;
    v_factor := case when v_factor = 7 then 2 else v_factor + 1 end;
  end loop;
  v_resto := v_suma % 11;
  v_esperado := case (11 - v_resto) when 11 then '0' when 10 then 'K' else (11 - v_resto)::text end;
  return v_esperado = v_dv;
end;
$$;
comment on function es_rut_valido(text) is 'Valida RUT chileno (módulo 11)';

create table if not exists alumnos (
  id uuid primary key default gen_random_uuid(),
  rut text not null unique check (es_rut_valido(rut)),
  nombres text not null check (char_length(btrim(nombres)) between 1 and 120),
  apellidos text not null check (char_length(btrim(apellidos)) between 1 and 120),
  fecha_nacimiento date,
  curso text not null default '',
  colegio_rbd text references colegios(rbd) on delete set null,
  estado text not null default 'postulante'
    check (estado in ('postulante', 'matriculado', 'trasladado', 'retirado')),
  creado_en timestamptz not null default now()
);
comment on table alumnos is 'Alumnos con RUT único validado';

create table if not exists apoderados (
  id uuid primary key default gen_random_uuid(),
  rut text not null unique check (es_rut_valido(rut)),
  nombres text not null check (char_length(btrim(nombres)) between 1 and 120),
  apellidos text not null check (char_length(btrim(apellidos)) between 1 and 120),
  telefono text not null default '',
  correo text not null default '' check (correo = '' or correo ~ '^[^@\s]+@[^@\s]+\.[^@\s]+$'),
  direccion text not null default '',
  region_codigo smallint references regiones(codigo),
  comuna_codigo text references comunas(codigo),
  creado_en timestamptz not null default now()
);
comment on table apoderados is 'Apoderados con RUT único validado';

create table if not exists alumno_apoderado (
  id uuid primary key default gen_random_uuid(),
  alumno_id uuid not null references alumnos(id) on delete cascade,
  apoderado_id uuid not null references apoderados(id) on delete cascade,
  vinculo text not null default '',
  retiro_autorizado boolean not null default false,
  notificaciones boolean not null default true,
  creado_en timestamptz not null default now(),
  unique (alumno_id, apoderado_id)
);
comment on table alumno_apoderado is 'Vínculos alumno-apoderado';

create index if not exists idx_alumnos_rut on alumnos (rut);
create index if not exists idx_alumnos_colegio on alumnos (colegio_rbd);
create index if not exists idx_alumnos_estado on alumnos (estado);
create index if not exists idx_apoderados_rut on apoderados (rut);
create index if not exists idx_vinculo_alumno on alumno_apoderado (alumno_id);
create index if not exists idx_vinculo_apoderado on alumno_apoderado (apoderado_id);

alter table alumnos enable row level security;
alter table apoderados enable row level security;
alter table alumno_apoderado enable row level security;
revoke all on alumnos, apoderados, alumno_apoderado from public;
grant select on alumnos, apoderados, alumno_apoderado to public;

-- Normaliza a CANONICO cuerpo-DV.
create or replace function rut_normalizado(p_rut text)
returns text
language sql immutable
set search_path = public
as $$
  select regexp_replace(upper(regexp_replace(p_rut, '[^0-9Kk]', '', 'g')), '^(.+)(.)$', '\1-\2');
$$;

create or replace function listar_alumnos(
  p_busqueda text default '',
  p_estado text default '',
  p_colegio_rbd text default '',
  p_limite integer default 50,
  p_desplazamiento integer default 0
)
returns setof alumnos
language plpgsql security definer set search_path = public
as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 50), 1), 100);
  v_desp integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_busqueda text := btrim(coalesce(p_busqueda, ''));
begin
  return query select * from alumnos a
  where (v_busqueda = '' or (a.nombres || ' ' || a.apellidos) ilike '%' || v_busqueda || '%'
         or a.rut ilike v_busqueda || '%' or a.curso ilike '%' || v_busqueda || '%')
    and (btrim(coalesce(p_estado, '')) = '' or a.estado = lower(btrim(p_estado)))
    and (btrim(coalesce(p_colegio_rbd, '')) = '' or a.colegio_rbd = btrim(p_colegio_rbd))
  order by a.apellidos asc, a.nombres asc
  limit v_limite offset v_desp;
end;
$$;

create or replace function obtener_alumno(p_id uuid)
returns alumnos
language plpgsql security definer set search_path = public
as $$
declare v alumnos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  select * into v from alumnos where id = p_id;
  if not found then raise exception 'Alumno no encontrado.' using errcode = 'P0002'; end if;
  return v;
end;
$$;

create or replace function crear_alumno(
  p_rut text, p_nombres text, p_apellidos text,
  p_fecha_nacimiento date default null, p_curso text default '',
  p_colegio_rbd text default null, p_estado text default 'postulante'
)
returns alumnos
language plpgsql security definer set search_path = public
as $$
declare
  v_rut text := rut_normalizado(p_rut);
  v_nombres text := btrim(coalesce(p_nombres, ''));
  v_apellidos text := btrim(coalesce(p_apellidos, ''));
  v_estado text := lower(btrim(coalesce(p_estado, 'postulante')));
  v_alumno alumnos%rowtype;
begin
  if not es_rut_valido(v_rut) then
    raise exception 'El RUT no es válido (revise el dígito verificador).' using errcode = 'P0001';
  end if;
  if v_nombres = '' then raise exception 'Los nombres son obligatorios.' using errcode = 'P0001'; end if;
  if v_apellidos = '' then raise exception 'Los apellidos son obligatorios.' using errcode = 'P0001'; end if;
  if v_estado not in ('postulante', 'matriculado', 'trasladado', 'retirado') then
    raise exception 'Estado no válido.' using errcode = 'P0001';
  end if;
  if p_colegio_rbd is not null and not exists (select 1 from colegios where rbd = p_colegio_rbd) then
    raise exception 'El colegio indicado no existe.' using errcode = 'P0001';
  end if;
  insert into alumnos (rut, nombres, apellidos, fecha_nacimiento, curso, colegio_rbd, estado)
  values (v_rut, v_nombres, v_apellidos, p_fecha_nacimiento, btrim(coalesce(p_curso, '')), p_colegio_rbd, v_estado)
  returning * into v_alumno;
  return v_alumno;
exception when unique_violation then
  raise exception 'Ya existe un alumno con ese RUT.' using errcode = 'P0003';
end;
$$;

create or replace function actualizar_alumno(
  p_id uuid, p_rut text, p_nombres text, p_apellidos text,
  p_fecha_nacimiento date default null, p_curso text default '',
  p_colegio_rbd text default null, p_estado text default 'postulante'
)
returns alumnos
language plpgsql security definer set search_path = public
as $$
declare
  v_rut text := rut_normalizado(p_rut);
  v_nombres text := btrim(coalesce(p_nombres, ''));
  v_apellidos text := btrim(coalesce(p_apellidos, ''));
  v_estado text := lower(btrim(coalesce(p_estado, 'postulante')));
  v_alumno alumnos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if not es_rut_valido(v_rut) then
    raise exception 'El RUT no es válido (revise el dígito verificador).' using errcode = 'P0001';
  end if;
  if v_nombres = '' then raise exception 'Los nombres son obligatorios.' using errcode = 'P0001'; end if;
  if v_apellidos = '' then raise exception 'Los apellidos son obligatorios.' using errcode = 'P0001'; end if;
  if v_estado not in ('postulante', 'matriculado', 'trasladado', 'retirado') then
    raise exception 'Estado no válido.' using errcode = 'P0001';
  end if;
  if p_colegio_rbd is not null and not exists (select 1 from colegios where rbd = p_colegio_rbd) then
    raise exception 'El colegio indicado no existe.' using errcode = 'P0001';
  end if;
  update alumnos set rut = v_rut, nombres = v_nombres, apellidos = v_apellidos,
    fecha_nacimiento = p_fecha_nacimiento, curso = btrim(coalesce(p_curso, '')),
    colegio_rbd = p_colegio_rbd, estado = v_estado
  where id = p_id returning * into v_alumno;
  if not found then raise exception 'Alumno no encontrado.' using errcode = 'P0002'; end if;
  return v_alumno;
exception when unique_violation then
  raise exception 'Ya existe un alumno con ese RUT.' using errcode = 'P0003';
end;
$$;

create or replace function eliminar_alumno(p_id uuid)
returns boolean
language plpgsql security definer set search_path = public
as $$
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  delete from alumnos where id = p_id;
  if not found then raise exception 'Alumno no encontrado.' using errcode = 'P0002'; end if;
  return true;
end;
$$;

create or replace function listar_apoderados(
  p_busqueda text default '',
  p_limite integer default 50,
  p_desplazamiento integer default 0
)
returns setof apoderados
language plpgsql security definer set search_path = public
as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 50), 1), 100);
  v_desp integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_busqueda text := btrim(coalesce(p_busqueda, ''));
begin
  return query select * from apoderados a
  where v_busqueda = ''
     or (a.nombres || ' ' || a.apellidos) ilike '%' || v_busqueda || '%'
     or a.rut ilike v_busqueda || '%'
  order by a.apellidos asc, a.nombres asc
  limit v_limite offset v_desp;
end;
$$;

create or replace function obtener_apoderado(p_id uuid)
returns apoderados
language plpgsql security definer set search_path = public
as $$
declare v apoderados%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  select * into v from apoderados where id = p_id;
  if not found then raise exception 'Apoderado no encontrado.' using errcode = 'P0002'; end if;
  return v;
end;
$$;

create or replace function crear_apoderado(
  p_rut text, p_nombres text, p_apellidos text,
  p_telefono text default '', p_correo text default '', p_direccion text default '',
  p_region_codigo smallint default null, p_comuna_codigo text default null
)
returns apoderados
language plpgsql security definer set search_path = public
as $$
declare
  v_rut text := rut_normalizado(p_rut);
  v_nombres text := btrim(coalesce(p_nombres, ''));
  v_apellidos text := btrim(coalesce(p_apellidos, ''));
  v_correo text := lower(btrim(coalesce(p_correo, '')));
  v_comuna text := nullif(btrim(coalesce(p_comuna_codigo, '')), '');
  v_apoderado apoderados%rowtype;
begin
  if not es_rut_valido(v_rut) then
    raise exception 'El RUT no es válido (revise el dígito verificador).' using errcode = 'P0001';
  end if;
  if v_nombres = '' then raise exception 'Los nombres son obligatorios.' using errcode = 'P0001'; end if;
  if v_apellidos = '' then raise exception 'Los apellidos son obligatorios.' using errcode = 'P0001'; end if;
  if v_correo <> '' and v_correo !~ '^[^@\s]+@[^@\s]+\.[^@\s]+$' then
    raise exception 'El correo electrónico no es válido.' using errcode = 'P0001';
  end if;
  if p_region_codigo is not null and not exists (select 1 from regiones where codigo = p_region_codigo) then
    raise exception 'La región no es válida.' using errcode = 'P0001';
  end if;
  if v_comuna is not null then
    if p_region_codigo is null then raise exception 'Elija la región de la comuna.' using errcode = 'P0001'; end if;
    if not exists (select 1 from comunas where codigo = v_comuna and region = p_region_codigo) then
      raise exception 'La comuna no pertenece a la región elegida.' using errcode = 'P0001';
    end if;
  end if;
  insert into apoderados (rut, nombres, apellidos, telefono, correo, direccion, region_codigo, comuna_codigo)
  values (v_rut, v_nombres, v_apellidos, btrim(coalesce(p_telefono, '')), v_correo,
          btrim(coalesce(p_direccion, '')), p_region_codigo, v_comuna)
  returning * into v_apoderado;
  return v_apoderado;
exception when unique_violation then
  raise exception 'Ya existe un apoderado con ese RUT.' using errcode = 'P0003';
end;
$$;

create or replace function actualizar_apoderado(
  p_id uuid, p_rut text, p_nombres text, p_apellidos text,
  p_telefono text default '', p_correo text default '', p_direccion text default '',
  p_region_codigo smallint default null, p_comuna_codigo text default null
)
returns apoderados
language plpgsql security definer set search_path = public
as $$
declare
  v_rut text := rut_normalizado(p_rut);
  v_nombres text := btrim(coalesce(p_nombres, ''));
  v_apellidos text := btrim(coalesce(p_apellidos, ''));
  v_correo text := lower(btrim(coalesce(p_correo, '')));
  v_comuna text := nullif(btrim(coalesce(p_comuna_codigo, '')), '');
  v_apoderado apoderados%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if not es_rut_valido(v_rut) then
    raise exception 'El RUT no es válido (revise el dígito verificador).' using errcode = 'P0001';
  end if;
  if v_nombres = '' then raise exception 'Los nombres son obligatorios.' using errcode = 'P0001'; end if;
  if v_apellidos = '' then raise exception 'Los apellidos son obligatorios.' using errcode = 'P0001'; end if;
  if v_correo <> '' and v_correo !~ '^[^@\s]+@[^@\s]+\.[^@\s]+$' then
    raise exception 'El correo electrónico no es válido.' using errcode = 'P0001';
  end if;
  if p_region_codigo is not null and not exists (select 1 from regiones where codigo = p_region_codigo) then
    raise exception 'La región no es válida.' using errcode = 'P0001';
  end if;
  if v_comuna is not null then
    if p_region_codigo is null then raise exception 'Elija la región de la comuna.' using errcode = 'P0001'; end if;
    if not exists (select 1 from comunas where codigo = v_comuna and region = p_region_codigo) then
      raise exception 'La comuna no pertenece a la región elegida.' using errcode = 'P0001';
    end if;
  end if;
  update apoderados set rut = v_rut, nombres = v_nombres, apellidos = v_apellidos,
    telefono = btrim(coalesce(p_telefono, '')), correo = v_correo,
    direccion = btrim(coalesce(p_direccion, '')), region_codigo = p_region_codigo, comuna_codigo = v_comuna
  where id = p_id returning * into v_apoderado;
  if not found then raise exception 'Apoderado no encontrado.' using errcode = 'P0002'; end if;
  return v_apoderado;
exception when unique_violation then
  raise exception 'Ya existe un apoderado con ese RUT.' using errcode = 'P0003';
end;
$$;

create or replace function eliminar_apoderado(p_id uuid)
returns boolean
language plpgsql security definer set search_path = public
as $$
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  delete from apoderados where id = p_id;
  if not found then raise exception 'Apoderado no encontrado.' using errcode = 'P0002'; end if;
  return true;
end;
$$;

create or replace function vincular_apoderado(
  p_alumno_id uuid, p_apoderado_id uuid,
  p_vinculo text default '', p_retiro boolean default false, p_notificaciones boolean default true
)
returns alumno_apoderado
language plpgsql security definer set search_path = public
as $$
declare v alumno_apoderado%rowtype;
begin
  if p_alumno_id is null or p_apoderado_id is null then
    raise exception 'Alumno y apoderado son obligatorios.' using errcode = 'P0001';
  end if;
  if not exists (select 1 from alumnos where id = p_alumno_id) then
    raise exception 'El alumno indicado no existe.' using errcode = 'P0001';
  end if;
  if not exists (select 1 from apoderados where id = p_apoderado_id) then
    raise exception 'El apoderado indicado no existe.' using errcode = 'P0001';
  end if;
  insert into alumno_apoderado (alumno_id, apoderado_id, vinculo, retiro_autorizado, notificaciones)
  values (p_alumno_id, p_apoderado_id, btrim(coalesce(p_vinculo, '')), coalesce(p_retiro, false), coalesce(p_notificaciones, true))
  returning * into v;
  return v;
exception when unique_violation then
  raise exception 'Ese apoderado ya está vinculado al alumno.' using errcode = 'P0003';
end;
$$;

create or replace function desvincular_apoderado(p_alumno_id uuid, p_apoderado_id uuid)
returns boolean
language plpgsql security definer set search_path = public
as $$
begin
  delete from alumno_apoderado where alumno_id = p_alumno_id and apoderado_id = p_apoderado_id;
  if not found then raise exception 'Vínculo no encontrado.' using errcode = 'P0002'; end if;
  return true;
end;
$$;

create or replace function apoderados_de_alumno(p_alumno_id uuid)
returns table (
  id uuid, rut text, nombres text, apellidos text, telefono text, correo text,
  vinculo text, retiro_autorizado boolean, notificaciones boolean
)
language sql security definer set search_path = public
as $$
  select a.id, a.rut, a.nombres, a.apellidos, a.telefono, a.correo,
         v.vinculo, v.retiro_autorizado, v.notificaciones
  from alumno_apoderado v join apoderados a on a.id = v.apoderado_id
  where v.alumno_id = p_alumno_id
  order by a.apellidos, a.nombres;
$$;

create or replace function alumnos_de_apoderado(p_apoderado_id uuid)
returns table (
  id uuid, rut text, nombres text, apellidos text, curso text, estado text, vinculo text
)
language sql security definer set search_path = public
as $$
  select a.id, a.rut, a.nombres, a.apellidos, a.curso, a.estado, v.vinculo
  from alumno_apoderado v join alumnos a on a.id = v.alumno_id
  where v.apoderado_id = p_apoderado_id
  order by a.apellidos, a.nombres;
$$;

create or replace function sembrar_escolar_demo()
returns void
language plpgsql security definer set search_path = public
as $$
declare
  v_a1 uuid; v_a2 uuid; v_a3 uuid; v_p1 uuid; v_p2 uuid; v_p3 uuid;
begin
  if (select count(*) from alumnos) > 0 then return; end if;
  insert into alumnos (rut, nombres, apellidos, curso, estado) values
    ('12345678-5', 'Martín', 'González', '8° Básico A', 'postulante'),
    ('11111111-1', 'Sofía', 'Fernández', '7° Básico B', 'matriculado'),
    ('6123456-K', 'Diego', 'Muñoz', '1° Medio A', 'postulante')
  on conflict (rut) do nothing;
  insert into apoderados (rut, nombres, apellidos, telefono, region_codigo, comuna_codigo) values
    ('1-9', 'Carmen', 'González', '+56 9 5555 0101', 13, '13101'),
    ('23456789-6', 'Pedro', 'Fernández', '+56 9 5555 0102', 5, '05101'),
    ('76543210-3', 'Ana', 'Muñoz', '+56 9 5555 0103', 8, '08101')
  on conflict (rut) do nothing;
  select id into v_a1 from alumnos where rut = '12345678-5';
  select id into v_a2 from alumnos where rut = '11111111-1';
  select id into v_a3 from alumnos where rut = '6123456-K';
  select id into v_p1 from apoderados where rut = '1-9';
  select id into v_p2 from apoderados where rut = '23456789-6';
  select id into v_p3 from apoderados where rut = '76543210-3';
  insert into alumno_apoderado (alumno_id, apoderado_id, vinculo, retiro_autorizado) values
    (v_a1, v_p1, 'Madre', true),
    (v_a2, v_p2, 'Padre', true),
    (v_a3, v_p3, 'Madre', false)
  on conflict (alumno_id, apoderado_id) do nothing;
end;
$$;

grant execute on function es_rut_valido(text) to public;
grant execute on function rut_normalizado(text) to public;
grant execute on function listar_alumnos(text, text, text, integer, integer) to public;
grant execute on function obtener_alumno(uuid) to public;
grant execute on function crear_alumno(text, text, text, date, text, text, text) to public;
grant execute on function actualizar_alumno(uuid, text, text, text, date, text, text, text) to public;
grant execute on function eliminar_alumno(uuid) to public;
grant execute on function listar_apoderados(text, integer, integer) to public;
grant execute on function obtener_apoderado(uuid) to public;
grant execute on function crear_apoderado(text, text, text, text, text, text, smallint, text) to public;
grant execute on function actualizar_apoderado(uuid, text, text, text, text, text, text, smallint, text) to public;
grant execute on function eliminar_apoderado(uuid) to public;
grant execute on function vincular_apoderado(uuid, uuid, text, boolean, boolean) to public;
grant execute on function desvincular_apoderado(uuid, uuid) to public;
grant execute on function apoderados_de_alumno(uuid) to public;
grant execute on function alumnos_de_apoderado(uuid) to public;
grant execute on function sembrar_escolar_demo() to public;
