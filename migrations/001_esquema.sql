-- 001 — Esquema base en español (PostgreSQL 18).
-- Tablas: empresas, personas, oportunidades.
-- Buenas prácticas: restricciones CHECK, índices, RLS activado y comentarios.

create extension if not exists "pgcrypto";

-- Limpieza idempotente solo para desarrollo (en producción usar migraciones hacia adelante).
-- No se borran datos aquí; las tablas se crean si no existen.

create table if not exists empresas (
  id uuid primary key default gen_random_uuid(),
  nombre text not null check (char_length(btrim(nombre)) between 1 and 200),
  dominio text not null default '',
  sector text not null default '',
  empleados integer not null default 0 check (empleados between 0 and 1000000),
  ciudad text not null default '',
  creado_en timestamptz not null default now()
);
comment on table empresas is 'Empresas cliente (clon de Companies en Twenty)';
comment on column empresas.nombre is 'Nombre comercial, obligatorio';

create table if not exists personas (
  id uuid primary key default gen_random_uuid(),
  nombre text not null check (char_length(btrim(nombre)) between 1 and 120),
  apellido text not null check (char_length(btrim(apellido)) between 1 and 120),
  correo text not null unique check (correo ~ '^[^@\s]+@[^@\s]+\.[^@\s]+$'),
  telefono text not null default '',
  puesto text not null default '',
  empresa_id uuid references empresas(id) on delete set null,
  ciudad text not null default '',
  creado_en timestamptz not null default now()
);
comment on table personas is 'Personas de contacto (clon de People en Twenty)';

create table if not exists oportunidades (
  id uuid primary key default gen_random_uuid(),
  nombre text not null check (char_length(btrim(nombre)) between 1 and 200),
  importe bigint not null default 0 check (importe >= 0),
  etapa text not null default 'nueva'
    check (etapa in ('nueva', 'cribado', 'reunion', 'propuesta', 'cliente')),
  empresa_id uuid references empresas(id) on delete set null,
  contacto_id uuid references personas(id) on delete set null,
  fecha_cierre timestamptz,
  creado_en timestamptz not null default now()
);
comment on table oportunidades is 'Oportunidades del embudo (clon de Opportunities en Twenty)';

create index if not exists idx_empresas_nombre on empresas (nombre);
create index if not exists idx_personas_correo on personas (correo);
create index if not exists idx_personas_empresa on personas (empresa_id);
create index if not exists idx_oportunidades_etapa on oportunidades (etapa);
create index if not exists idx_oportunidades_empresa on oportunidades (empresa_id);

-- Seguridad: activar RLS. Sin políticas permisivas, solo el propietario y las
-- funciones SECURITY DEFINER pueden leer/escribir. La API solo llama a funciones.
alter table empresas enable row level security;
alter table personas enable row level security;
alter table oportunidades enable row level security;

-- Denegar acceso directo al rol público; el acceso se hace vía funciones.
revoke all on empresas, personas, oportunidades from public;
grant select on empresas, personas, oportunidades to public;
