-- 029 — Fundación tenant SaaS (aditivo, no rompe E2E 122).
-- Holdings + colegios contratados + membresías + importaciones + supresión + es_miembro.
-- El directorio colegios (12k MINEDUC) queda intacto y público; el tenant vive aparte.

create table if not exists holdings (
  id uuid primary key default gen_random_uuid(),
  nombre text not null,
  plan text not null check (plan in ('small','group','holding')),
  creado_en timestamptz not null default now()
);
comment on table holdings is 'Holding/sostenedor que agrupa colegios contratados';

create table if not exists colegios_contratados (
  holding_id uuid not null references holdings(id) on delete cascade,
  rbd text not null references colegios(rbd) on delete restrict,
  licencia text not null check (licencia in ('small','group','holding')),
  dominio text not null default '',
  flags jsonb not null default '{}',
  creado_en timestamptz not null default now(),
  primary key (holding_id, rbd)
);
comment on table colegios_contratados is 'Colegios contratados por holding (tenant), RBD FK al directorio MINEDUC';

create table if not exists membresias_usuario (
  id uuid primary key default gen_random_uuid(),
  usuario_id uuid not null references usuarios(id) on delete cascade,
  holding_id uuid not null references holdings(id) on delete cascade,
  colegio_rbd text references colegios(rbd) on delete cascade,
  creado_en timestamptz not null default now()
);
create index if not exists idx_membresias_usuario on membresias_usuario (usuario_id);
create index if not exists idx_membresias_holding on membresias_usuario (holding_id);
comment on table membresias_usuario is 'Alcance por usuario: holding_id obligatorio, colegio_rbd nulo = todo el holding';

create table if not exists importaciones (
  job_id uuid primary key default gen_random_uuid(),
  objeto text not null,
  total integer not null default 0,
  creados integer not null default 0,
  errores jsonb not null default '[]',
  creado_en timestamptz not null default now()
);
comment on table importaciones is 'Historial de CSV cuarentena (job por importación)';

create table if not exists supresion (
  rut_hash text primary key,
  motivo text not null default '',
  creado_en timestamptz not null default now()
);
comment on table supresion is 'Lista de supresión marketing: hash RUT normalizado que bloquea re-contacto';

alter table holdings enable row level security;
alter table colegios_contratados enable row level security;
alter table membresias_usuario enable row level security;
alter table importaciones enable row level security;
alter table supresion enable row level security;
revoke all on holdings from public;
revoke all on colegios_contratados from public;
revoke all on membresias_usuario from public;
revoke all on importaciones from public;
revoke all on supresion from public;
grant select on holdings to public;
grant select on colegios_contratados to public;
grant select on membresias_usuario to public;
grant select on importaciones to public;
grant select on supresion to public;

create or replace function es_miembro(p_usuario_id uuid, p_holding_id uuid, p_colegio_rbd text)
returns boolean
language sql stable security definer
set search_path = public as $$
  select exists (
    select 1 from membresias_usuario
    where usuario_id = p_usuario_id
      and (p_holding_id is null or holding_id = p_holding_id)
      and (p_colegio_rbd is null or colegio_rbd is null or colegio_rbd = p_colegio_rbd)
  );
$$;
comment on function es_miembro(uuid, uuid, text) is 'Alcance tenant: true si el usuario pertenece al holding/colegio (colegio_rbd nulo = todo el holding)';
grant execute on function es_miembro(uuid, uuid, text) to public;
