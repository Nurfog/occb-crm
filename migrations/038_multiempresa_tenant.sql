-- 038 — Multiempresa contable + columnas de tenant (aditivo).
-- `empresas` es la razón social emisora (factura, paga sueldos, declara IVA),
-- distinta de `holdings` (sostenedor que agrupa colegios para informes).
-- Un holding puede tener varias empresas; un colegio contratado pertenece a un
-- holding y sus documentos se emiten desde una empresa.
-- Las columnas holding_id/colegio_rbd en datos de negocio quedan NULLABLES:
-- NULL = dato de plataforma/legado. El aislamiento se activa en una migración
-- posterior, cuando se decida el modelo de identidad (admin global vs holding).

create table if not exists empresas (
  id uuid primary key default gen_random_uuid(),
  holding_id uuid not null references holdings(id) on delete cascade,
  pais_iso text not null references paises(iso2),
  documento_tipo text not null default 'rut',
  documento_numero text not null,
  razon_social text not null,
  nombre_fantasia text not null default '',
  moneda_funcional text not null references monedas(iso),
  regimen_tributario text not null default '',
  activa boolean not null default true,
  creado_en timestamptz not null default now(),
  unique (holding_id, pais_iso, documento_tipo, documento_numero)
);
comment on table empresas is 'Razón social emisora de documentos; varias por holding posible';
create index if not exists idx_empresas_holding on empresas (holding_id) where activa;

create table if not exists puntos_emision (
  id uuid primary key default gen_random_uuid(),
  empresa_id uuid not null references empresas(id) on delete cascade,
  pais_iso text not null references paises(iso2),
  tipo_documento text not null,
  serie text not null default '',
  folio_desde bigint not null default 1,
  folio_hasta bigint,
  vigente boolean not null default true,
  creado_en timestamptz not null default now(),
  unique (empresa_id, pais_iso, tipo_documento, serie)
);
comment on table puntos_emision is 'Series y rangos de folio autorizados por país y tipo de documento';
create index if not exists idx_puntos_empresa on puntos_emision (empresa_id) where vigente;

alter table empresas enable row level security;
alter table puntos_emision enable row level security;
revoke all on empresas from public;
revoke all on puntos_emision from public;
grant select on empresas to public;
grant select on puntos_emision to public;

-- Tenant en datos de negocio núcleo (nullable: legado/plataforma).
alter table alumnos
  add column if not exists holding_id uuid references holdings(id) on delete set null;
alter table apoderados
  add column if not exists holding_id uuid references holdings(id) on delete set null;
alter table oportunidades
  add column if not exists holding_id uuid references holdings(id) on delete set null;
create index if not exists idx_alumnos_holding on alumnos (holding_id) where holding_id is not null;
create index if not exists idx_apoderados_holding on apoderados (holding_id) where holding_id is not null;
create index if not exists idx_oportunidades_holding on oportunidades (holding_id) where holding_id is not null;

-- Herencia de holding desde el colegio contratado del RBD.
create or replace function holding_de_rbd(p_rbd text)
returns uuid
language sql stable security definer set search_path = public as $$
  select c.holding_id from colegios_contratados c
  where c.rbd = p_rbd order by c.creado_en limit 1;
$$;
comment on function holding_de_rbd(text) is 'Holding que tiene contratado un RBD (NULL si el colegio no está contratado)';
grant execute on function holding_de_rbd(text) to public;

create or replace function heredar_holding_dato()
returns trigger language plpgsql as $$
begin
  if NEW.holding_id is null and coalesce(NEW.colegio_rbd, '') <> '' then
    NEW.holding_id := holding_de_rbd(NEW.colegio_rbd);
  end if;
  return NEW;
end; $$;

drop trigger if exists trg_holding_alumno on alumnos;
create trigger trg_holding_alumno before insert or update on alumnos
  for each row execute function heredar_holding_dato();
drop trigger if exists trg_holding_oportunidad on oportunidades;
create trigger trg_holding_oportunidad before insert or update on oportunidades
  for each row execute function heredar_holding_dato();

-- Apoderados: el holding se asigna explícitamente (o queda NULL). No se
-- infiere por comuna: un apoderado puede tener hijos en colegios de holdings
-- distintos y una heurística geográfica asignaría el tenant equivocado.

-- Alcance de la petición (lo fija la API por transacción). Vacío = sin scope.
create or replace function scope_holding()
returns uuid language sql stable as $$
  select nullif(current_setting('app.holding_id', true), '')::uuid;
$$;
create or replace function scope_colegio()
returns text language sql stable as $$
  select nullif(current_setting('app.colegio_rbd', true), '');
$$;
comment on function scope_holding() is 'Holding activo de la petición (NULL si la petición no fijó alcance)';
comment on function scope_colegio() is 'Colegio activo de la petición (NULL si es alcance de todo el holding)';
grant execute on function scope_holding() to public;
grant execute on function scope_colegio() to public;
