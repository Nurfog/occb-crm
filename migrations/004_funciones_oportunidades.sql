-- 004 — Funciones seguras para OPORTUNIDADES y PANEL.

create or replace function listar_oportunidades(
  p_etapa text default '',
  p_limite integer default 100,
  p_desplazamiento integer default 0
)
returns setof oportunidades
language plpgsql
security definer
set search_path = public
as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 100), 1), 200);
  v_desplazamiento integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_etapa text := lower(btrim(coalesce(p_etapa, '')));
begin
  if v_etapa <> '' and v_etapa not in ('nueva', 'cribado', 'reunion', 'propuesta', 'cliente') then
    raise exception 'Etapa no válida. Use: nueva, cribado, reunion, propuesta o cliente.' using errcode = 'P0001';
  end if;
  return query
  select *
  from oportunidades o
  where v_etapa = '' or o.etapa = v_etapa
  order by o.importe desc
  limit v_limite offset v_desplazamiento;
end;
$$;

create or replace function crear_oportunidad(
  p_nombre text,
  p_importe bigint default 0,
  p_etapa text default 'nueva',
  p_empresa_id uuid default null
)
returns oportunidades
language plpgsql
security definer
set search_path = public
as $$
declare
  v_nombre text := btrim(coalesce(p_nombre, ''));
  v_etapa text := lower(btrim(coalesce(p_etapa, 'nueva')));
  v_oportunidad oportunidades%rowtype;
begin
  if v_nombre = '' then
    raise exception 'El nombre de la oportunidad es obligatorio.' using errcode = 'P0001';
  end if;
  if coalesce(p_importe, 0) < 0 then
    raise exception 'El importe no puede ser negativo.' using errcode = 'P0001';
  end if;
  if v_etapa not in ('nueva', 'cribado', 'reunion', 'propuesta', 'cliente') then
    raise exception 'Etapa no válida. Use: nueva, cribado, reunion, propuesta o cliente.' using errcode = 'P0001';
  end if;
  if p_empresa_id is not null and not exists (select 1 from empresas where id = p_empresa_id) then
    raise exception 'La empresa indicada no existe.' using errcode = 'P0001';
  end if;
  insert into oportunidades (nombre, importe, etapa, empresa_id)
  values (v_nombre, coalesce(p_importe, 0), v_etapa, p_empresa_id)
  returning * into v_oportunidad;
  return v_oportunidad;
end;
$$;

create or replace function mover_etapa_oportunidad(p_id uuid, p_etapa text)
returns oportunidades
language plpgsql
security definer
set search_path = public
as $$
declare
  v_etapa text := lower(btrim(coalesce(p_etapa, '')));
  v_oportunidad oportunidades%rowtype;
begin
  if p_id is null then
    raise exception 'El identificador de la oportunidad es obligatorio.' using errcode = 'P0001';
  end if;
  if v_etapa not in ('nueva', 'cribado', 'reunion', 'propuesta', 'cliente') then
    raise exception 'Etapa no válida. Use: nueva, cribado, reunion, propuesta o cliente.' using errcode = 'P0001';
  end if;
  update oportunidades set etapa = v_etapa where id = p_id returning * into v_oportunidad;
  if not found then
    raise exception 'Oportunidad no encontrada.' using errcode = 'P0002';
  end if;
  return v_oportunidad;
end;
$$;

create or replace function eliminar_oportunidad(p_id uuid)
returns boolean
language plpgsql
security definer
set search_path = public
as $$
begin
  if p_id is null then
    raise exception 'El identificador de la oportunidad es obligatorio.' using errcode = 'P0001';
  end if;
  delete from oportunidades where id = p_id;
  if not found then
    raise exception 'Oportunidad no encontrada.' using errcode = 'P0002';
  end if;
  return true;
end;
$$;

-- Resumen del panel: lo calcula la base de datos (una sola consulta).
create or replace function panel_resumen()
returns table (
  total_empresas bigint,
  total_personas bigint,
  total_oportunidades bigint,
  valor_tuberia bigint,
  valor_ganado bigint,
  tasa_exito numeric
)
language sql
security definer
set search_path = public
as $$
  select
    (select count(*) from empresas),
    (select count(*) from personas),
    (select count(*) from oportunidades),
    (select coalesce(sum(importe), 0) from oportunidades where etapa <> 'cliente'),
    (select coalesce(sum(importe), 0) from oportunidades where etapa = 'cliente'),
    case
      when (select count(*) from oportunidades) = 0 then 0
      else round(
        (select count(*) from oportunidades where etapa = 'cliente')::numeric
        / (select count(*) from oportunidades)::numeric * 100, 1)
    end;
$$;
comment on function panel_resumen is 'Resumen del panel principal en una sola llamada';
