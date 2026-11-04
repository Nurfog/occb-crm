-- 002 — Funciones seguras para EMPRESAS.
-- Patrón: la API nunca hace INSERT/UPDATE/DELETE directos; solo llama a estas
-- funciones SECURITY DEFINER con validación en español.

-- Lista con búsqueda opcional y paginación segura.
create or replace function listar_empresas(
  p_busqueda text default '',
  p_limite integer default 50,
  p_desplazamiento integer default 0
)
returns setof empresas
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
  from empresas e
  where v_busqueda = ''
     or e.nombre ilike '%' || v_busqueda || '%'
     or e.dominio ilike '%' || v_busqueda || '%'
     or e.sector ilike '%' || v_busqueda || '%'
  order by e.nombre asc
  limit v_limite offset v_desplazamiento;
end;
$$;
comment on function listar_empresas is 'Lista empresas con búsqueda y paginación. Uso: SELECT * FROM listar_empresas(''acme'', 50, 0)';

create or replace function obtener_empresa(p_id uuid)
returns empresas
language plpgsql
security definer
set search_path = public
as $$
declare
  v_empresa empresas%rowtype;
begin
  if p_id is null then
    raise exception 'El identificador de la empresa es obligatorio.' using errcode = 'P0001';
  end if;
  select * into v_empresa from empresas where id = p_id;
  if not found then
    raise exception 'Empresa no encontrada.' using errcode = 'P0002';
  end if;
  return v_empresa;
end;
$$;

create or replace function crear_empresa(
  p_nombre text,
  p_dominio text default '',
  p_sector text default '',
  p_empleados integer default 0,
  p_ciudad text default ''
)
returns empresas
language plpgsql
security definer
set search_path = public
as $$
declare
  v_nombre text := btrim(coalesce(p_nombre, ''));
  v_empresa empresas%rowtype;
begin
  if v_nombre = '' then
    raise exception 'El nombre de la empresa es obligatorio.' using errcode = 'P0001';
  end if;
  if char_length(v_nombre) > 200 then
    raise exception 'El nombre no puede superar 200 caracteres.' using errcode = 'P0001';
  end if;
  if coalesce(p_empleados, 0) < 0 or coalesce(p_empleados, 0) > 1000000 then
    raise exception 'El número de empleados debe estar entre 0 y 1000000.' using errcode = 'P0001';
  end if;
  insert into empresas (nombre, dominio, sector, empleados, ciudad)
  values (v_nombre, btrim(coalesce(p_dominio, '')), btrim(coalesce(p_sector, '')),
          coalesce(p_empleados, 0), btrim(coalesce(p_ciudad, '')))
  returning * into v_empresa;
  return v_empresa;
end;
$$;

create or replace function actualizar_empresa(
  p_id uuid,
  p_nombre text,
  p_dominio text default '',
  p_sector text default '',
  p_empleados integer default 0,
  p_ciudad text default ''
)
returns empresas
language plpgsql
security definer
set search_path = public
as $$
declare
  v_nombre text := btrim(coalesce(p_nombre, ''));
  v_empresa empresas%rowtype;
begin
  if p_id is null then
    raise exception 'El identificador de la empresa es obligatorio.' using errcode = 'P0001';
  end if;
  if v_nombre = '' then
    raise exception 'El nombre de la empresa es obligatorio.' using errcode = 'P0001';
  end if;
  if coalesce(p_empleados, 0) < 0 then
    raise exception 'El número de empleados no puede ser negativo.' using errcode = 'P0001';
  end if;
  update empresas
  set nombre = v_nombre,
      dominio = btrim(coalesce(p_dominio, '')),
      sector = btrim(coalesce(p_sector, '')),
      empleados = coalesce(p_empleados, 0),
      ciudad = btrim(coalesce(p_ciudad, ''))
  where id = p_id
  returning * into v_empresa;
  if not found then
    raise exception 'Empresa no encontrada.' using errcode = 'P0002';
  end if;
  return v_empresa;
end;
$$;

create or replace function eliminar_empresa(p_id uuid)
returns boolean
language plpgsql
security definer
set search_path = public
as $$
begin
  if p_id is null then
    raise exception 'El identificador de la empresa es obligatorio.' using errcode = 'P0001';
  end if;
  -- Las personas y oportunidades quedan con empresa NULL (ON DELETE SET NULL).
  delete from empresas where id = p_id;
  if not found then
    raise exception 'Empresa no encontrada.' using errcode = 'P0002';
  end if;
  return true;
end;
$$;
