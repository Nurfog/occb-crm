-- 022 — Claves API y webhooks de eventos escolares.

create table if not exists claves_api (
  id uuid primary key default gen_random_uuid(),
  usuario_id uuid not null references usuarios(id) on delete cascade,
  nombre text not null check (char_length(btrim(nombre)) between 1 and 80),
  clave_hash text not null unique check (clave_hash ~ '^[0-9a-f]{64}$'),
  alcances text[] not null default array['*']::text[],
  revocada boolean not null default false,
  creado_en timestamptz not null default now(),
  ultimo_uso_en timestamptz
);
alter table claves_api enable row level security;
revoke all on claves_api from public;
grant select on claves_api to public;
create index if not exists idx_claves_api_usuario on claves_api(usuario_id,creado_en desc);

create table if not exists webhooks (
  id uuid primary key default gen_random_uuid(),
  usuario_id uuid not null references usuarios(id) on delete cascade,
  nombre text not null check (char_length(btrim(nombre)) between 1 and 80),
  url text not null check(url ~ '^https://' or url like 'http://localhost%' or url like 'http://127.0.0.1%' or url like 'http://[::1]%'),
  eventos text[] not null check (cardinality(eventos) between 1 and 20),
  secreto text not null,
  activa boolean not null default true,
  creado_en timestamptz not null default now()
);
alter table webhooks enable row level security;
revoke all on webhooks from public;
grant select on webhooks to public;
create index if not exists idx_webhooks_usuario_activa on webhooks(usuario_id,activa);

create table if not exists entregas_webhook (
  id uuid primary key default gen_random_uuid(),
  webhook_id uuid not null references webhooks(id) on delete cascade,
  evento text not null,
  carga jsonb not null,
  intentos smallint not null default 0 check(intentos between 0 and 3),
  ok boolean,
  procesando boolean not null default false,
  proximo_intento_en timestamptz not null default now(),
  ultimo_error text not null default '',
  creado_en timestamptz not null default now(),
  ultimo_intento_en timestamptz
);
alter table entregas_webhook enable row level security;
revoke all on entregas_webhook from public;
grant select on entregas_webhook to public;
create index if not exists idx_entregas_pendientes on entregas_webhook(ok,procesando,proximo_intento_en,creado_en);
create index if not exists idx_entregas_webhook on entregas_webhook(webhook_id,creado_en desc);

create or replace function crear_clave_api(p_usuario_id uuid,p_nombre text,p_hash text,p_alcances text[] default array['*']::text[])
returns claves_api language plpgsql security definer set search_path=public as $$
declare v claves_api%rowtype; n text:=btrim(coalesce(p_nombre,'')); scopes text[]:=coalesce(p_alcances,array['*']::text[]);
begin
  if p_usuario_id is null then raise exception 'El usuario es obligatorio.' using errcode='P0001'; end if;
  if char_length(n) not between 1 and 80 then raise exception 'El nombre debe tener entre 1 y 80 caracteres.' using errcode='P0001'; end if;
  if coalesce(p_hash,'') !~ '^[0-9a-f]{64}$' then raise exception 'La clave no tiene un hash válido.' using errcode='P0001'; end if;
  if cardinality(scopes)=0 or exists(select 1 from unnest(scopes) s where s not in ('*','lectura','escritura')) then raise exception 'Alcance de clave no válido.' using errcode='P0001'; end if;
  insert into claves_api(usuario_id,nombre,clave_hash,alcances) values(p_usuario_id,n,p_hash,scopes) returning * into v;
  return v;
exception when unique_violation then raise exception 'La clave ya existe.' using errcode='P0003';
end; $$;

create or replace function listar_claves_api(p_usuario_id uuid)
returns table(id uuid,nombre text,alcances text[],revocada boolean,creado_en timestamptz,ultimo_uso_en timestamptz)
language sql security definer set search_path=public as $$
  select c.id,c.nombre,c.alcances,c.revocada,c.creado_en,c.ultimo_uso_en from claves_api c where c.usuario_id=p_usuario_id order by c.creado_en desc;
$$;

create or replace function validar_clave_api(p_hash text)
returns table(id uuid,usuario_id uuid,nombre text,rol text,alcances text[])
language plpgsql security definer set search_path=public as $$
begin
  return query update claves_api c set ultimo_uso_en=now()
    from usuarios u where c.usuario_id=u.id and c.clave_hash=p_hash and not c.revocada
    returning c.id,c.usuario_id,c.nombre,u.rol,c.alcances;
end; $$;

create or replace function revocar_clave_api(p_usuario_id uuid,p_id uuid)
returns boolean language plpgsql security definer set search_path=public as $$
begin
  update claves_api set revocada=true where id=p_id and usuario_id=p_usuario_id;
  if not found then raise exception 'Clave de API no encontrada.' using errcode='P0002'; end if;
  return true;
end; $$;

create or replace function crear_webhook(p_usuario_id uuid,p_nombre text,p_url text,p_eventos text[])
returns webhooks language plpgsql security definer set search_path=public as $$
declare v webhooks%rowtype; n text:=btrim(coalesce(p_nombre,'')); u text:=btrim(coalesce(p_url,'')); ev text[]:=coalesce(p_eventos,'{}');
begin
  if p_usuario_id is null then raise exception 'El usuario es obligatorio.' using errcode='P0001'; end if;
  if char_length(n) not between 1 and 80 then raise exception 'El nombre debe tener entre 1 y 80 caracteres.' using errcode='P0001'; end if;
  if u !~ '^https://' and u not like 'http://localhost%' and u not like 'http://127.0.0.1%' and u not like 'http://[::1]%' then
    raise exception 'El webhook debe usar HTTPS (HTTP solo en localhost).' using errcode='P0001'; end if;
  if cardinality(ev) not between 1 and 20 then raise exception 'Selecciona entre 1 y 20 eventos.' using errcode='P0001'; end if;
  if exists(select 1 from unnest(ev) e where e not in ('registro.creado','registro.actualizado','registro.eliminado','postulacion.creado','postulacion.actualizado','registro.*','prueba')) then
    raise exception 'Uno de los eventos de webhook no está permitido.' using errcode='P0001'; end if;
  insert into webhooks(usuario_id,nombre,url,eventos,secreto) values(p_usuario_id,n,u,ev,replace(gen_random_uuid()::text,'-','')) returning * into v;
  return v;
end; $$;

create or replace function listar_webhooks(p_usuario_id uuid)
returns table(id uuid,nombre text,url text,eventos text[],activa boolean,creado_en timestamptz)
language sql security definer set search_path=public as $$
  select w.id,w.nombre,w.url,w.eventos,w.activa,w.creado_en from webhooks w where w.usuario_id=p_usuario_id order by w.creado_en desc;
$$;

create or replace function alternar_webhook(p_usuario_id uuid,p_id uuid,p_activa boolean)
returns webhooks language plpgsql security definer set search_path=public as $$
declare v webhooks%rowtype;
begin update webhooks set activa=coalesce(p_activa,false) where id=p_id and usuario_id=p_usuario_id returning * into v;
  if not found then raise exception 'Webhook no encontrado.' using errcode='P0002'; end if; return v;
end; $$;

create or replace function eliminar_webhook(p_usuario_id uuid,p_id uuid)
returns boolean language plpgsql security definer set search_path=public as $$
begin delete from webhooks where id=p_id and usuario_id=p_usuario_id;
  if not found then raise exception 'Webhook no encontrado.' using errcode='P0002'; end if; return true;
end; $$;

create or replace function probar_webhook(p_usuario_id uuid,p_id uuid)
returns uuid language plpgsql security definer set search_path=public as $$
declare v_id uuid;
begin
  insert into entregas_webhook(webhook_id,evento,carga)
  select w.id,'prueba',jsonb_build_object('evento','prueba','mensaje','Entrega de prueba OCCB','creado_en',now())
  from webhooks w where w.id=p_id and w.usuario_id=p_usuario_id and w.activa returning id into v_id;
  if v_id is null then raise exception 'Webhook no encontrado o inactivo.' using errcode='P0002'; end if;
  return v_id;
end; $$;

create or replace function encolar_webhooks_actividad()
returns trigger language plpgsql security definer set search_path=public as $$
declare ev text;
begin
  if NEW.usuario_id is null then return NEW; end if;
  ev:=case when NEW.tipo='postulacion' then 'postulacion.'||NEW.accion else 'registro.'||NEW.accion end;
  insert into entregas_webhook(webhook_id,evento,carga)
  select w.id,ev,jsonb_build_object('id',NEW.referencia_id,'tipo',NEW.tipo,'accion',NEW.accion,'detalle',NEW.detalle,'creado_en',NEW.creado_en)
  from webhooks w where w.usuario_id=NEW.usuario_id and w.activa and (ev=any(w.eventos) or 'registro.*'=any(w.eventos));
  return NEW;
end; $$;
create trigger encolar_webhooks_por_actividad after insert on actividad for each row execute function encolar_webhooks_actividad();

create or replace function reclamar_entregas_webhook(p_limite integer default 10)
returns table(id uuid,url text,secreto text,evento text,carga jsonb,intentos smallint)
language plpgsql security definer set search_path=public as $$
begin
  return query with candidatos as (
    select e.id from entregas_webhook e join webhooks w on w.id=e.webhook_id
    where w.activa and e.ok is distinct from true and not e.procesando and e.intentos<3 and e.proximo_intento_en<=now()
    order by e.creado_en limit least(greatest(coalesce(p_limite,10),1),50) for update of e skip locked
  ), reclamadas as (
    update entregas_webhook e set procesando=true,intentos=e.intentos+1,ultimo_intento_en=now()
    from candidatos c where e.id=c.id returning e.id,e.webhook_id,e.evento,e.carga,e.intentos
  ) select r.id,w.url,w.secreto,r.evento,r.carga,r.intentos from reclamadas r join webhooks w on w.id=r.webhook_id;
end; $$;

create or replace function registrar_resultado_webhook(p_id uuid,p_ok boolean,p_error text default '')
returns boolean language plpgsql security definer set search_path=public as $$
begin
  update entregas_webhook set procesando=false,ok=case when p_ok then true when intentos>=3 then false else null end,
    ultimo_error=case when p_ok then '' else left(coalesce(p_error,'Entrega fallida'),500) end,
    proximo_intento_en=case when p_ok or intentos>=3 then proximo_intento_en else now()+make_interval(secs=least(60,5*power(2,greatest(intentos-1,0))::integer)) end
  where id=p_id and procesando;
  return found;
end; $$;

create or replace function listar_entregas_webhook(p_usuario_id uuid,p_webhook_id uuid,p_limite integer default 50)
returns table(id uuid,evento text,intentos smallint,ok boolean,ultimo_error text,creado_en timestamptz,ultimo_intento_en timestamptz)
language sql security definer set search_path=public as $$
  select e.id,e.evento,e.intentos,e.ok,e.ultimo_error,e.creado_en,e.ultimo_intento_en
  from entregas_webhook e join webhooks w on w.id=e.webhook_id where w.usuario_id=p_usuario_id and w.id=p_webhook_id
  order by e.creado_en desc limit least(greatest(coalesce(p_limite,50),1),100);
$$;

grant execute on function crear_clave_api(uuid,text,text,text[]) to public;
grant execute on function listar_claves_api(uuid) to public;
grant execute on function validar_clave_api(text) to public;
grant execute on function revocar_clave_api(uuid,uuid) to public;
grant execute on function crear_webhook(uuid,text,text,text[]) to public;
grant execute on function listar_webhooks(uuid) to public;
grant execute on function alternar_webhook(uuid,uuid,boolean) to public;
grant execute on function eliminar_webhook(uuid,uuid) to public;
grant execute on function probar_webhook(uuid,uuid) to public;
grant execute on function reclamar_entregas_webhook(integer) to public;
grant execute on function registrar_resultado_webhook(uuid,boolean,text) to public;
grant execute on function listar_entregas_webhook(uuid,uuid,integer) to public;
