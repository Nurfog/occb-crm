-- Esquema MVP espejo de twenty-server (simplificado).
create extension if not exists "pgcrypto";

create table if not exists companies (
  id uuid primary key default gen_random_uuid(),
  name text not null,
  domain text not null default '',
  industry text not null default '',
  employees integer not null default 0,
  city text not null default '',
  created_at timestamptz not null default now()
);

create table if not exists people (
  id uuid primary key default gen_random_uuid(),
  first_name text not null,
  last_name text not null,
  email text not null unique,
  phone text not null default '',
  job_title text not null default '',
  company_id uuid references companies(id) on delete set null,
  city text not null default '',
  created_at timestamptz not null default now()
);

create type opportunity_stage as enum ('new','screening','meeting','proposal','customer');

create table if not exists opportunities (
  id uuid primary key default gen_random_uuid(),
  name text not null,
  amount bigint not null default 0,
  stage opportunity_stage not null default 'new',
  company_id uuid references companies(id) on delete set null,
  point_of_contact_id uuid references people(id) on delete set null,
  close_date timestamptz,
  created_at timestamptz not null default now()
);
