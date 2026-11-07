-- 014 — Acceso real: usuarios con clave argon2, sesiones con rotación y roles.
-- La API verifica hashes en Rust; PG guarda hashes y valida unicidad.

create table if not exists usuarios (
  id uuid primary key default gen_random_uuid(),
  nombre text not null check (char_length(btrim(nombre)) between 1 and 120),
  correo text not null unique check (correo ~ '^[^@\s]+@[^@\s]+\.[^@\s]+$'),
  clave_hash text not null default '',
  rol text not null default 'miembro' check (rol in ('admin', 'miembro')),
  creado_en timestamptz not null default now()
);
comment on table usuarios is 'Usuarios del espacio (credenciales locales)';

create table if not exists sesiones (
  id uuid primary key default gen_random_uuid(),
  usuario_id uuid not null references usuarios(id) on delete cascade,
  renovacion_hash text not null,
  expira_en timestamptz not null,
  creado_en timestamptz not null default now()
);
comment on table sesiones is 'Sesiones con token de renovación hasheado';

create index if not exists idx_sesiones_usuario on sesiones (usuario_id);
create index if not exists idx_sesiones_expira on sesiones (expira_en);

alter table usuarios enable row level security;
alter table sesiones enable row level security;
revoke all on usuarios, sesiones from public;
grant select on usuarios, sesiones to public;

create or replace function registrar_usuario(p_nombre text, p_correo text, p_clave_hash text)
returns usuarios
language plpgsql security definer set search_path = public
as $$
declare
  v_nombre text := btrim(coalesce(p_nombre, ''));
  v_correo text := lower(btrim(coalesce(p_correo, '')));
  v_usuario usuarios%rowtype;
  v_total integer;
begin
  if v_nombre = '' then raise exception 'El nombre es obligatorio.' using errcode = 'P0001'; end if;
  if v_correo !~ '^[^@\s]+@[^@\s]+\.[^@\s]+$' then
    raise exception 'El correo electrónico no es válido.' using errcode = 'P0001';
  end if;
  if coalesce(p_clave_hash, '') = '' then raise exception 'La clave es obligatoria.' using errcode = 'P0001'; end if;
  -- El primer usuario del espacio nace admin.
  select count(*) into v_total from usuarios;
  insert into usuarios (nombre, correo, clave_hash, rol)
  values (v_nombre, v_correo, p_clave_hash, case when v_total = 0 then 'admin' else 'miembro' end)
  returning * into v_usuario;
  return v_usuario;
exception when unique_violation then
  raise exception 'Ya existe un usuario con ese correo.' using errcode = 'P0003';
end;
$$;

create or replace function credencial_usuario(p_correo text)
returns table (id uuid, nombre text, correo text, clave_hash text, rol text)
language sql security definer set search_path = public
as $$ select u.id, u.nombre, u.correo, u.clave_hash, u.rol from usuarios u where u.correo = lower(btrim(p_correo)); $$;

create or replace function crear_sesion(p_usuario_id uuid, p_renovacion_hash text, p_dias integer default 30)
returns sesiones
language plpgsql security definer set search_path = public
as $$
declare v sesiones%rowtype;
begin
  if p_usuario_id is null then raise exception 'El usuario es obligatorio.' using errcode = 'P0001'; end if;
  if coalesce(p_renovacion_hash, '') = '' then raise exception 'Token inválido.' using errcode = 'P0001'; end if;
  if not exists (select 1 from usuarios where id = p_usuario_id) then
    raise exception 'Usuario no encontrado.' using errcode = 'P0002';
  end if;
  delete from sesiones where expira_en < now();
  insert into sesiones (usuario_id, renovacion_hash, expira_en)
  values (p_usuario_id, p_renovacion_hash, now() + (coalesce(p_dias, 30) || ' days')::interval)
  returning * into v;
  return v;
end;
$$;

create or replace function rotar_sesion(p_sesion_id uuid, p_renovacion_hash text, p_nuevo_hash text)
returns sesiones
language plpgsql security definer set search_path = public
as $$
declare v sesiones%rowtype;
begin
  select * into v from sesiones where id = p_sesion_id;
  if not found then raise exception 'Sesión no encontrada.' using errcode = 'P0002'; end if;
  if v.expira_en < now() then
    delete from sesiones where id = p_sesion_id;
    raise exception 'Sesión expirada.' using errcode = 'P0002';
  end if;
  if v.renovacion_hash <> p_renovacion_hash then
    -- Reuso detectado: se revoca todo lo del usuario.
    delete from sesiones where usuario_id = v.usuario_id;
    raise exception 'Sesión revocada por seguridad.' using errcode = 'P0002';
  end if;
  update sesiones set renovacion_hash = p_nuevo_hash where id = p_sesion_id returning * into v;
  return v;
end;
$$;

create or replace function cerrar_sesion(p_sesion_id uuid)
returns boolean
language plpgsql security definer set search_path = public
as $$
begin
  delete from sesiones where id = p_sesion_id;
  if not found then raise exception 'Sesión no encontrada.' using errcode = 'P0002'; end if;
  return true;
end;
$$;

create or replace function listar_miembros()
returns table (id uuid, nombre text, correo text, rol text, creado_en timestamptz)
language sql security definer set search_path = public
as $$ select u.id, u.nombre, u.correo, u.rol, u.creado_en from usuarios u order by u.creado_en; $$;

create or replace function cambiar_rol(p_id uuid, p_rol text)
returns usuarios
language plpgsql security definer set search_path = public
as $$
declare
  v_rol text := lower(btrim(coalesce(p_rol, '')));
  v usuarios%rowtype;
begin
  if v_rol not in ('admin', 'miembro') then raise exception 'Rol no válido.' using errcode = 'P0001'; end if;
  update usuarios set rol = v_rol where id = p_id returning * into v;
  if not found then raise exception 'Usuario no encontrado.' using errcode = 'P0002'; end if;
  return v;
end;
$$;

grant execute on function registrar_usuario(text, text, text) to public;
grant execute on function credencial_usuario(text) to public;
grant execute on function crear_sesion(uuid, text, integer) to public;
grant execute on function rotar_sesion(uuid, text, text) to public;
grant execute on function cerrar_sesion(uuid) to public;
grant execute on function listar_miembros() to public;
grant execute on function cambiar_rol(uuid, text) to public;
