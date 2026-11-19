-- 045 — Soft-delete y PK de favoritos (aditivo).
-- 1) `favoritos` pasa de PK compuesta a `id uuid` + unique natural.
-- 2) `alumno_apoderado` gana `activo`: desvincular desactiva, no borra.
-- 3) `conversaciones` y `mensajes` ganan `activo`: eliminar desactiva en
--    cascada lógica; el historial queda.
-- 4) `traslados` gana `creado_en` (la `fecha` es de negocio, no de registro).
-- La unificación activo/activa/vigente/revocada queda para 048.

-- 1) Favoritos con id uuid
alter table favoritos add column if not exists id uuid;
update favoritos set id = gen_random_uuid() where id is null;
alter table favoritos alter column id set not null;
alter table favoritos alter column id set default gen_random_uuid();
alter table favoritos drop constraint if exists favoritos_pkey;
alter table favoritos add primary key (id);
create unique index if not exists uq_favoritos_natural
  on favoritos (usuario_id, tipo, referencia_id);
comment on column favoritos.id is 'PK uuid; el unique natural es (usuario, tipo, referencia)';

-- 2) Vínculo alumno-apoderado con baja lógica
alter table alumno_apoderado add column if not exists activo boolean not null default true;
create index if not exists idx_vinculos_activos on alumno_apoderado (alumno_id) where activo;

create or replace function desvincular_apoderado(p_alumno_id uuid, p_apoderado_id uuid)
returns boolean
language plpgsql security definer set search_path = public
as $$
begin
  update alumno_apoderado set activo = false
  where alumno_id = p_alumno_id and apoderado_id = p_apoderado_id and activo;
  if not found then raise exception 'Vínculo no encontrado.' using errcode = 'P0002'; end if;
  return true;
end;
$$;

drop function if exists vincular_apoderado(uuid, uuid, text, boolean, boolean);

-- Re-vincular reactiva el vínculo existente (el unique natural sigue igual).
create or replace function vincular_apoderado(
  p_alumno_id uuid, p_apoderado_id uuid, p_vinculo text default '',
  p_retiro_autorizado boolean default false, p_notificaciones boolean default true
)
returns boolean
language plpgsql security definer set search_path = public
as $$
begin
  if not exists (select 1 from alumnos where id = p_alumno_id and activo) then
    raise exception 'Alumno no encontrado.' using errcode = 'P0002';
  end if;
  if not exists (select 1 from apoderados where id = p_apoderado_id and activo) then
    raise exception 'Apoderado no encontrado.' using errcode = 'P0002';
  end if;
  insert into alumno_apoderado (alumno_id, apoderado_id, vinculo, retiro_autorizado, notificaciones, activo)
  values (p_alumno_id, p_apoderado_id, btrim(coalesce(p_vinculo, '')), p_retiro_autorizado, p_notificaciones, true)
  on conflict (alumno_id, apoderado_id) do update
    set activo = true,
        vinculo = excluded.vinculo,
        retiro_autorizado = excluded.retiro_autorizado,
        notificaciones = excluded.notificaciones;
  return true;
end;
$$;

-- 3) Conversaciones y mensajes con baja lógica
alter table conversaciones add column if not exists activo boolean not null default true;
alter table mensajes add column if not exists activo boolean not null default true;
create index if not exists idx_conversaciones_activas on conversaciones (usuario_id) where activo;


-- 4) Traslados con timestamp de registro
alter table traslados add column if not exists creado_en timestamptz not null default now();

-- 5a) El tipo de fila cambia al agregar `activo`: hay que recrear las
-- funciones que devuelven `conversaciones` o `mensajes` (no basta replace).
drop function if exists crear_conversacion(uuid, text);
drop function if exists obtener_conversacion(uuid, uuid);
drop function if exists actualizar_titulo_conversacion(uuid, uuid, text);
drop function if exists listar_conversaciones(uuid, integer);
drop function if exists listar_mensajes(uuid, uuid, integer);
drop function if exists guardar_mensaje(uuid, uuid, text, text);
drop function if exists eliminar_conversacion(uuid, uuid);

create or replace function crear_conversacion(p_usuario_id uuid,p_titulo text default '')
returns conversaciones language plpgsql security definer set search_path=public as $$
declare v conversaciones%rowtype; t text:=btrim(coalesce(p_titulo,''));
begin
  if p_usuario_id is null then raise exception 'El usuario es obligatorio.' using errcode='P0001'; end if;
  if t='' then t:='Nueva conversación'; end if;
  if char_length(t)>160 then raise exception 'El título no puede superar 160 caracteres.' using errcode='P0001'; end if;
  insert into conversaciones(usuario_id,titulo,activo) values(p_usuario_id,t,true) returning * into v; return v;
end; $$;

create or replace function obtener_conversacion(p_usuario_id uuid,p_id uuid)
returns conversaciones language plpgsql security definer set search_path=public as $$
declare v conversaciones%rowtype;
begin
  select * into v from conversaciones where id=p_id and usuario_id=p_usuario_id and activo;
  if not found then raise exception 'Conversación no encontrada.' using errcode='P0002'; end if;
  return v;
end; $$;

create or replace function actualizar_titulo_conversacion(p_usuario_id uuid,p_id uuid,p_titulo text)
returns conversaciones language plpgsql security definer set search_path=public as $$
declare v conversaciones%rowtype; t text:=btrim(coalesce(p_titulo,''));
begin
  if char_length(t) not between 1 and 160 then raise exception 'El título es obligatorio (máximo 160 caracteres).' using errcode='P0001'; end if;
  update conversaciones set titulo=t,actualizado_en=now()
  where id=p_id and usuario_id=p_usuario_id and activo returning * into v;
  if not found then raise exception 'Conversación no encontrada.' using errcode='P0002'; end if;
  return v;
end; $$;


-- 5) Lecturas que respetan la baja lógica
create or replace function apoderados_de_alumno(p_alumno_id uuid)
returns table (
  id uuid, rut text, nombres text, apellidos text, telefono text, correo text,
  vinculo text, retiro_autorizado boolean, notificaciones boolean
)
language sql security definer set search_path = public as $$
  select a.id, a.rut, a.nombres, a.apellidos, a.telefono, a.correo,
         v.vinculo, v.retiro_autorizado, v.notificaciones
  from alumno_apoderado v join apoderados a on a.id = v.apoderado_id
  where v.alumno_id = p_alumno_id and v.activo and a.activo
  order by a.apellidos, a.nombres;
$$;

create or replace function alumnos_de_apoderado(p_apoderado_id uuid)
returns table (
  id uuid, rut text, nombres text, apellidos text, curso text, estado text, vinculo text
)
language sql security definer set search_path = public as $$
  select a.id, a.rut, a.nombres, a.apellidos, a.curso, a.estado, v.vinculo
  from alumno_apoderado v join alumnos a on a.id = v.alumno_id
  where v.apoderado_id = p_apoderado_id and v.activo and a.activo
  order by a.apellidos, a.nombres;
$$;

create or replace function listar_mensajes(p_usuario_id uuid,p_conversacion_id uuid,p_limite integer default 100)
returns setof mensajes language plpgsql security definer set search_path=public as $$
begin
  if not exists(select 1 from conversaciones where id=p_conversacion_id and usuario_id=p_usuario_id and activo) then
    raise exception 'Conversación no encontrada.' using errcode='P0002';
  end if;
  return query select * from mensajes
    where conversacion_id=p_conversacion_id and activo
    order by creado_en
    limit least(greatest(coalesce(p_limite,100),1),200);
end; $$;

create or replace function guardar_mensaje(p_usuario_id uuid,p_conversacion_id uuid,p_rol text,p_contenido text)
returns mensajes language plpgsql security definer set search_path=public as $$
declare v mensajes%rowtype; r text:=lower(btrim(coalesce(p_rol,''))); c text:=btrim(coalesce(p_contenido,''));
begin
  if not exists(select 1 from conversaciones where id=p_conversacion_id and usuario_id=p_usuario_id and activo) then
    raise exception 'Conversación no encontrada.' using errcode='P0002';
  end if;
  if r not in ('usuario','agente') then raise exception 'Rol de mensaje no válido.' using errcode='P0001'; end if;
  if char_length(c) not between 1 and 20000 then raise exception 'El mensaje debe tener entre 1 y 20000 caracteres.' using errcode='P0001'; end if;
  insert into mensajes(conversacion_id,rol,contenido,activo) values(p_conversacion_id,r,c,true) returning * into v;
  update conversaciones set actualizado_en=now() where id=p_conversacion_id;
  return v;
end; $$;

create or replace function obtener_conversacion(p_usuario_id uuid,p_id uuid)
returns conversaciones language plpgsql security definer set search_path=public as $$
declare v conversaciones%rowtype;
begin
  select * into v from conversaciones where id=p_id and usuario_id=p_usuario_id and activo;
  if not found then raise exception 'Conversación no encontrada.' using errcode='P0002'; end if;
  return v;
end; $$;

-- listar_conversaciones: redefinir con filtro activo manteniendo su firma
create or replace function listar_conversaciones(p_usuario_id uuid, p_limite integer default 50)
returns setof conversaciones language sql security definer set search_path = public as $$
  select * from conversaciones
  where usuario_id = p_usuario_id and activo
  order by actualizado_en desc
  limit least(greatest(coalesce(p_limite, 50), 1), 100);
$$;

-- Permisos al final: todas las funciones ya existen
grant execute on function crear_conversacion(uuid, text) to public;
grant execute on function obtener_conversacion(uuid, uuid) to public;
grant execute on function actualizar_titulo_conversacion(uuid, uuid, text) to public;
grant execute on function listar_conversaciones(uuid, integer) to public;
grant execute on function listar_mensajes(uuid, uuid, integer) to public;
grant execute on function guardar_mensaje(uuid, uuid, text, text) to public;

create or replace function eliminar_conversacion(p_usuario_id uuid, p_id uuid)
returns boolean language plpgsql security definer set search_path = public as $$
begin
  update conversaciones set activo = false
  where id = p_id and usuario_id = p_usuario_id and activo;
  if not found then raise exception 'Conversación no encontrada.' using errcode = 'P0002'; end if;
  update mensajes set activo = false where conversacion_id = p_id;
  return true;
end; $$;
grant execute on function eliminar_conversacion(uuid, uuid) to public;
