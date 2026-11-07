-- 021 — Conversaciones y mensajes personales del asistente.

create table if not exists conversaciones (
  id uuid primary key default gen_random_uuid(),
  usuario_id uuid not null references usuarios(id) on delete cascade,
  titulo text not null default 'Nueva conversación' check(char_length(btrim(titulo)) between 1 and 160),
  creado_en timestamptz not null default now(),
  actualizado_en timestamptz not null default now()
);
create index if not exists idx_conversaciones_usuario on conversaciones(usuario_id,actualizado_en desc);

create table if not exists mensajes (
  id uuid primary key default gen_random_uuid(),
  conversacion_id uuid not null references conversaciones(id) on delete cascade,
  rol text not null check(rol in ('usuario','agente')),
  contenido text not null check(char_length(btrim(contenido)) between 1 and 20000),
  creado_en timestamptz not null default now()
);
create index if not exists idx_mensajes_conversacion on mensajes(conversacion_id,creado_en);

alter table conversaciones enable row level security;
alter table mensajes enable row level security;
revoke all on conversaciones,mensajes from public;
grant select on conversaciones,mensajes to public;

create or replace function listar_conversaciones(p_usuario_id uuid,p_limite integer default 50)
returns setof conversaciones language sql security definer set search_path=public as $$
  select * from conversaciones where usuario_id=p_usuario_id
  order by actualizado_en desc limit least(greatest(coalesce(p_limite,50),1),100);
$$;

create or replace function crear_conversacion(p_usuario_id uuid,p_titulo text default '')
returns conversaciones language plpgsql security definer set search_path=public as $$
declare v conversaciones%rowtype; t text:=btrim(coalesce(p_titulo,''));
begin
  if p_usuario_id is null then raise exception 'El usuario es obligatorio.' using errcode='P0001'; end if;
  if t='' then t:='Nueva conversación'; end if;
  if char_length(t)>160 then raise exception 'El título no puede superar 160 caracteres.' using errcode='P0001'; end if;
  insert into conversaciones(usuario_id,titulo) values(p_usuario_id,t) returning * into v; return v;
end; $$;

create or replace function obtener_conversacion(p_usuario_id uuid,p_id uuid)
returns conversaciones language plpgsql security definer set search_path=public as $$
declare v conversaciones%rowtype;
begin
  select * into v from conversaciones where id=p_id and usuario_id=p_usuario_id;
  if not found then raise exception 'Conversación no encontrada.' using errcode='P0002'; end if;
  return v;
end; $$;

create or replace function listar_mensajes(p_usuario_id uuid,p_conversacion_id uuid,p_limite integer default 100)
returns setof mensajes language plpgsql security definer set search_path=public as $$
begin
  if not exists(select 1 from conversaciones where id=p_conversacion_id and usuario_id=p_usuario_id) then
    raise exception 'Conversación no encontrada.' using errcode='P0002';
  end if;
  return query select * from mensajes where conversacion_id=p_conversacion_id order by creado_en
    limit least(greatest(coalesce(p_limite,100),1),200);
end; $$;

create or replace function guardar_mensaje(p_usuario_id uuid,p_conversacion_id uuid,p_rol text,p_contenido text)
returns mensajes language plpgsql security definer set search_path=public as $$
declare v mensajes%rowtype; r text:=lower(btrim(coalesce(p_rol,''))); c text:=btrim(coalesce(p_contenido,''));
begin
  if not exists(select 1 from conversaciones where id=p_conversacion_id and usuario_id=p_usuario_id) then
    raise exception 'Conversación no encontrada.' using errcode='P0002';
  end if;
  if r not in ('usuario','agente') then raise exception 'Rol de mensaje no válido.' using errcode='P0001'; end if;
  if char_length(c) not between 1 and 20000 then raise exception 'El mensaje debe tener entre 1 y 20000 caracteres.' using errcode='P0001'; end if;
  insert into mensajes(conversacion_id,rol,contenido) values(p_conversacion_id,r,c) returning * into v;
  update conversaciones set actualizado_en=now() where id=p_conversacion_id;
  return v;
end; $$;

create or replace function actualizar_titulo_conversacion(p_usuario_id uuid,p_id uuid,p_titulo text)
returns conversaciones language plpgsql security definer set search_path=public as $$
declare v conversaciones%rowtype; t text:=btrim(coalesce(p_titulo,''));
begin
  if char_length(t) not between 1 and 160 then raise exception 'El título es obligatorio (máximo 160 caracteres).' using errcode='P0001'; end if;
  update conversaciones set titulo=t,actualizado_en=now() where id=p_id and usuario_id=p_usuario_id returning * into v;
  if not found then raise exception 'Conversación no encontrada.' using errcode='P0002'; end if;
  return v;
end; $$;

create or replace function eliminar_conversacion(p_usuario_id uuid,p_id uuid)
returns boolean language plpgsql security definer set search_path=public as $$
begin
  delete from conversaciones where id=p_id and usuario_id=p_usuario_id;
  if not found then raise exception 'Conversación no encontrada.' using errcode='P0002'; end if;
  return true;
end; $$;

grant execute on function listar_conversaciones(uuid,integer) to public;
grant execute on function crear_conversacion(uuid,text) to public;
grant execute on function obtener_conversacion(uuid,uuid) to public;
grant execute on function listar_mensajes(uuid,uuid,integer) to public;
grant execute on function guardar_mensaje(uuid,uuid,text,text) to public;
grant execute on function actualizar_titulo_conversacion(uuid,uuid,text) to public;
grant execute on function eliminar_conversacion(uuid,uuid) to public;
