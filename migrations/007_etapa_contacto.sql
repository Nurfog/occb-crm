-- 007 — La etapa «cribado» pasa a llamarse «contacto» (más claro).
-- Actualiza restricción, filas existentes y funciones que validan la etapa.

-- Primero se suelta la restricción, se migran las filas y se crea la nueva.
alter table oportunidades drop constraint oportunidades_etapa_check;

update oportunidades set etapa = 'contacto' where etapa = 'cribado';

alter table oportunidades
  add constraint oportunidades_etapa_check
  check (etapa in ('nueva', 'contacto', 'reunion', 'propuesta', 'cliente'));

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
  if v_etapa <> '' and v_etapa not in ('nueva', 'contacto', 'reunion', 'propuesta', 'cliente') then
    raise exception 'Etapa no válida. Use: nueva, contacto, reunion, propuesta o cliente.' using errcode = 'P0001';
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
  if v_etapa not in ('nueva', 'contacto', 'reunion', 'propuesta', 'cliente') then
    raise exception 'Etapa no válida. Use: nueva, contacto, reunion, propuesta o cliente.' using errcode = 'P0001';
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
  if v_etapa not in ('nueva', 'contacto', 'reunion', 'propuesta', 'cliente') then
    raise exception 'Etapa no válida. Use: nueva, contacto, reunion, propuesta o cliente.' using errcode = 'P0001';
  end if;
  update oportunidades set etapa = v_etapa where id = p_id returning * into v_oportunidad;
  if not found then
    raise exception 'Oportunidad no encontrada.' using errcode = 'P0002';
  end if;
  return v_oportunidad;
end;
$$;

-- La siembra también usa el nombre nuevo (instalaciones frescas).
create or replace function sembrar_datos_demo()
returns void
language plpgsql
security definer
set search_path = public
as $$
declare
  v_acme uuid; v_globex uuid; v_initech uuid; v_umbrella uuid; v_hooli uuid; v_stark uuid;
  v_ada uuid; v_alan uuid; v_grace uuid; v_linus uuid; v_marie uuid;
begin
  if (select count(*) from empresas) > 0 then
    return;
  end if;

  insert into empresas (nombre, dominio, sector, empleados, ciudad) values
    ('Acme Corp', 'acme.com', 'Software', 240, 'Madrid'),
    ('Globex', 'globex.io', 'Finanzas', 85, 'Barcelona'),
    ('Hooli', 'hooli.com', 'Nube', 1200, 'Sevilla'),
    ('Initech', 'initech.dev', 'Herramientas', 42, 'Valencia'),
    ('Stark Industries', 'stark.ind', 'Industria', 3000, 'Zaragoza'),
    ('Umbrella', 'umbrella.salud', 'Salud', 510, 'Bilbao');

  select id into v_acme from empresas where nombre = 'Acme Corp';
  select id into v_globex from empresas where nombre = 'Globex';
  select id into v_initech from empresas where nombre = 'Initech';
  select id into v_umbrella from empresas where nombre = 'Umbrella';
  select id into v_hooli from empresas where nombre = 'Hooli';
  select id into v_stark from empresas where nombre = 'Stark Industries';

  insert into personas (nombre, apellido, correo, telefono, puesto, empresa_id, ciudad) values
    ('Ada', 'Lovelace', 'ada@acme.com', '+34 555 0101', 'Directora técnica', v_acme, 'Madrid'),
    ('Alan', 'Turing', 'alan@globex.io', '+34 555 0102', 'Responsable de datos', v_globex, 'Barcelona'),
    ('Grace', 'Hopper', 'grace@initech.dev', '+34 555 0103', 'VP Ingeniería', v_initech, 'Valencia'),
    ('Linus', 'Torvalds', 'linus@hooli.com', '+34 555 0104', 'Ingeniero sénior', v_hooli, 'Sevilla'),
    ('Marie', 'Curie', 'marie@umbrella.salud', '+34 555 0105', 'Jefa de investigación', v_umbrella, 'Bilbao');

  select id into v_ada from personas where correo = 'ada@acme.com';
  select id into v_alan from personas where correo = 'alan@globex.io';
  select id into v_grace from personas where correo = 'grace@initech.dev';
  select id into v_linus from personas where correo = 'linus@hooli.com';
  select id into v_marie from personas where correo = 'marie@umbrella.salud';

  insert into oportunidades (nombre, importe, etapa, empresa_id, contacto_id) values
    ('Acme — Plan empresarial', 120000, 'nueva', v_acme, v_ada),
    ('Globex — Despliegue de pagos', 85000, 'contacto', v_globex, v_alan),
    ('Initech — Ampliación de licencias', 24000, 'reunion', v_initech, v_grace),
    ('Umbrella — Suite de investigación', 150000, 'reunion', v_umbrella, v_marie),
    ('Hooli — Migración a la nube', 310000, 'propuesta', v_hooli, v_linus),
    ('Stark — IoT para fábricas', 480000, 'cliente', v_stark, v_ada);
end;
$$;
