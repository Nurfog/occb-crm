-- 039 — Empresa como unidad de tenant (ajuste de 038).
-- Cada colegio/razón social es una EMPRESA con su propio documento y país,
-- aunque pertenezca a una corporación. `holdings` pasa a ser agrupación
-- OPCIONAL para consolidar informes.
--   empresa: unidad operativa y fiscal (nombre, RUT/RUC/CUIT/RFC/CPF, país)
--   holding: corporación que agrupa empresas (NULL = independiente)
--   colegio_rbd: establecimiento del directorio, ligado a una empresa
-- Sin RLS todavía: el enforcement se activa cuando se decida el modo de
-- acceso global (queda pendiente y documentado).

-- 1) Empresa puede ser independiente.
alter table empresas alter column holding_id drop not null;

-- 1b) El documento fiscal es único por país, no por holding: una razón social
-- no puede repetirse aunque esté en corporaciones distintas.
do $$ declare c text; begin
  for c in select conname from pg_constraint
    where conrelid = 'empresas'::regclass and contype = 'u' loop
    execute format('alter table empresas drop constraint %I', c);
  end loop;
end $$;
create unique index if not exists uq_empresas_documento
  on empresas (pais_iso, documento_tipo, upper(documento_numero));

-- 2) Colegio contratado pertenece a una empresa; el holding es derivado.
alter table colegios_contratados
  add column if not exists empresa_id uuid references empresas(id) on delete set null;
create index if not exists idx_contratados_empresa on colegios_contratados (empresa_id);

-- 3) Membresía por empresa (el holding queda como agrupación de lectura).
alter table membresias_usuario
  add column if not exists empresa_id uuid references empresas(id) on delete cascade;
create index if not exists idx_membresias_empresa on membresias_usuario (empresa_id);

-- 4) Tenant en datos de negocio: empresa además de holding.
alter table alumnos
  add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table apoderados
  add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table oportunidades
  add column if not exists empresa_id uuid references empresas(id) on delete set null;
create index if not exists idx_alumnos_empresa on alumnos (empresa_id) where empresa_id is not null;
create index if not exists idx_apoderados_empresa on apoderados (empresa_id) where empresa_id is not null;
create index if not exists idx_oportunidades_empresa on oportunidades (empresa_id) where empresa_id is not null;

-- 5) Derivaciones: RBD -> empresa -> holding.
create or replace function empresa_de_rbd(p_rbd text)
returns uuid language sql stable security definer set search_path = public as $$
  select c.empresa_id from colegios_contratados c
  where c.rbd = p_rbd and c.empresa_id is not null
  order by c.creado_en limit 1;
$$;
comment on function empresa_de_rbd(text) is 'Empresa que tiene contratado un RBD (NULL si no está asignada)';
grant execute on function empresa_de_rbd(text) to public;

create or replace function holding_de_empresa(p_empresa uuid)
returns uuid language sql stable security definer set search_path = public as $$
  select holding_id from empresas where id = p_empresa;
$$;
comment on function holding_de_empresa(uuid) is 'Holding (corporación) de una empresa; NULL si es independiente';
grant execute on function holding_de_empresa(uuid) to public;

-- Hereda empresa y holding desde el colegio del dato (sin heurísticas).
create or replace function heredar_tenant_dato()
returns trigger language plpgsql as $$
declare v_empresa uuid;
begin
  if coalesce(NEW.colegio_rbd, '') <> '' then
    v_empresa := empresa_de_rbd(NEW.colegio_rbd);
    if NEW.empresa_id is null then NEW.empresa_id := v_empresa; end if;
    if NEW.holding_id is null and NEW.empresa_id is not null then
      NEW.holding_id := holding_de_empresa(NEW.empresa_id);
    end if;
  end if;
  return NEW;
end; $$;

drop trigger if exists trg_holding_alumno on alumnos;
drop trigger if exists trg_holding_oportunidad on oportunidades;
create trigger trg_tenant_alumno before insert or update on alumnos
  for each row execute function heredar_tenant_dato();
create trigger trg_tenant_oportunidad before insert or update on oportunidades
  for each row execute function heredar_tenant_dato();

-- 6) Alcance de la petición por empresa.
create or replace function scope_empresa()
returns uuid language sql stable as $$
  select nullif(current_setting('app.empresa_id', true), '')::uuid;
$$;
comment on function scope_empresa() is 'Empresa activa de la petición (NULL si no se fijó alcance)';
grant execute on function scope_empresa() to public;
