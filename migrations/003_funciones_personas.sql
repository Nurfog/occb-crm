-- 003 — Funciones seguras para PERSONAS.

create or replace function listar_personas(
  p_busqueda text default '',
  p_limite integer default 50,
  p_desplazamiento integer default 0
)
returns setof personas
language plpgsql
security definer
set search_path = public
as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 50), 1), 100);
  v_desplazamiento integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_busqueda text := btrim(coalesce(p_busqueda, ''));
begin
  return query
  select *
  from personas p
  where v_busqueda = ''
     or (p.nombre || ' ' || p.apellido) ilike '%' || v_busqueda || '%'
     or p.correo ilike '%' || v_busqueda || '%'
     or p.puesto ilike '%' || v_busqueda || '%'
  order by p.apellido asc, p.nombre asc
  limit v_limite offset v_desplazamiento;
end;
$$;

create or replace function obtener_persona(p_id uuid)
returns personas
language plpgsql
security definer
set search_path = public
as $$
declare
  v_persona personas%rowtype;
begin
  if p_id is null then
    raise exception 'El identificador de la persona es obligatorio.' using errcode = 'P0001';
  end if;
  select * into v_persona from personas where id = p_id;
  if not found then
    raise exception 'Persona no encontrada.' using errcode = 'P0002';
  end if;
  return v_persona;
end;
$$;

create or replace function crear_persona(
  p_nombre text,
  p_apellido text,
  p_correo text,
  p_telefono text default '',
  p_puesto text default '',
  p_empresa_id uuid default null,
  p_ciudad text default ''
)
returns personas
language plpgsql
security definer
set search_path = public
as $$
declare
  v_nombre text := btrim(coalesce(p_nombre, ''));
  v_apellido text := btrim(coalesce(p_apellido, ''));
  v_correo text := lower(btrim(coalesce(p_correo, '')));
  v_persona personas%rowtype;
begin
  if v_nombre = '' then
    raise exception 'El nombre es obligatorio.' using errcode = 'P0001';
  end if;
  if v_apellido = '' then
    raise exception 'Los apellidos son obligatorios.' using errcode = 'P0001';
  end if;
  if v_correo !~ '^[^@\s]+@[^@\s]+\.[^@\s]+$' then
    raise exception 'El correo electrónico no tiene un formato válido.' using errcode = 'P0001';
  end if;
  if p_empresa_id is not null and not exists (select 1 from empresas where id = p_empresa_id) then
    raise exception 'La empresa indicada no existe.' using errcode = 'P0001';
  end if;
  insert into personas (nombre, apellido, correo, telefono, puesto, empresa_id, ciudad)
  values (v_nombre, v_apellido, v_correo, btrim(coalesce(p_telefono, '')),
          btrim(coalesce(p_puesto, '')), p_empresa_id, btrim(coalesce(p_ciudad, '')))
  returning * into v_persona;
  return v_persona;
exception
  when unique_violation then
    raise exception 'Ya existe una persona con ese correo.' using errcode = 'P0003';
end;
$$;

create or replace function eliminar_persona(p_id uuid)
returns boolean
language plpgsql
security definer
set search_path = public
as $$
begin
  if p_id is null then
    raise exception 'El identificador de la persona es obligatorio.' using errcode = 'P0001';
  end if;
  delete from personas where id = p_id;
  if not found then
    raise exception 'Persona no encontrada.' using errcode = 'P0002';
  end if;
  return true;
end;
$$;
