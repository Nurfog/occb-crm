-- 025 — Permite host.docker.internal como destino de webhooks en desarrollo.
-- 022 ya está aplicada: se amplía el CHECK y la validación sin reescribirla.
-- Sigue siendo loopback (puerta del host); en producción se usa https.

alter table webhooks drop constraint if exists webhooks_url_check;
alter table webhooks
  add constraint webhooks_url_check check (
    url ~ '^https://'
    or url like 'http://localhost%'
    or url like 'http://127.0.0.1%'
    or url like 'http://[::1]%'
    or url like 'http://host.docker.internal%'
  );

create or replace function crear_webhook(p_usuario_id uuid,p_nombre text,p_url text,p_eventos text[])
returns webhooks language plpgsql security definer set search_path=public as $$
declare v webhooks%rowtype; n text:=btrim(coalesce(p_nombre,'')); u text:=btrim(coalesce(p_url,'')); ev text[]:=coalesce(p_eventos,'{}');
begin
  if p_usuario_id is null then raise exception 'El usuario es obligatorio.' using errcode='P0001'; end if;
  if char_length(n) not between 1 and 80 then raise exception 'El nombre debe tener entre 1 y 80 caracteres.' using errcode='P0001'; end if;
  if u !~ '^https://' and u not like 'http://localhost%' and u not like 'http://127.0.0.1%' and u not like 'http://[::1]%' and u not like 'http://host.docker.internal%' then
    raise exception 'El webhook debe usar HTTPS (HTTP solo en loopback local).' using errcode='P0001'; end if;
  if cardinality(ev) not between 1 and 20 then raise exception 'Selecciona entre 1 y 20 eventos.' using errcode='P0001'; end if;
  if exists(select 1 from unnest(ev) e where e not in ('registro.creado','registro.actualizado','registro.eliminado','postulacion.creado','postulacion.actualizado','registro.*','prueba')) then
    raise exception 'Uno de los eventos de webhook no está permitido.' using errcode='P0001'; end if;
  insert into webhooks(usuario_id,nombre,url,eventos,secreto) values(p_usuario_id,n,u,ev,replace(gen_random_uuid()::text,'-','')) returning * into v;
  return v;
end; $$;
