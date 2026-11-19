-- 044 — Tenant (empresa) en todo el dato de negocio (aditivo).
-- Faltaban 16 tablas: tareas, notas, eventos, correos, ingresos, traslados,
-- vínculos, actividad, favoritos, vistas, conversaciones, mensajes, claves,
-- webhooks, entregas e importaciones. Todas heredan la empresa del registro
-- del que cuelgan; NULL = plataforma/legado.
-- `personas` (legacy) y `supresion` (¿global?) quedan para la decisión 048.

-- 1) Columnas
alter table tareas add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table notas add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table eventos add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table correos add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table ingresos_medicos add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table traslados add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table alumno_apoderado add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table actividad add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table favoritos add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table vistas add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table conversaciones add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table mensajes add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table claves_api add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table webhooks add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table entregas_webhook add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table importaciones add column if not exists empresa_id uuid references empresas(id) on delete set null;
alter table importaciones add column if not exists usuario_id uuid references usuarios(id) on delete set null;

create index if not exists idx_tareas_empresa on tareas (empresa_id) where empresa_id is not null;
create index if not exists idx_notas_empresa on notas (empresa_id) where empresa_id is not null;
create index if not exists idx_eventos_empresa on eventos (empresa_id) where empresa_id is not null;
create index if not exists idx_correos_empresa on correos (empresa_id) where empresa_id is not null;
create index if not exists idx_ingresos_empresa on ingresos_medicos (empresa_id) where empresa_id is not null;
create index if not exists idx_traslados_empresa on traslados (empresa_id) where empresa_id is not null;
create index if not exists idx_vinculos_empresa on alumno_apoderado (empresa_id) where empresa_id is not null;
create index if not exists idx_actividad_empresa on actividad (empresa_id) where empresa_id is not null;
create index if not exists idx_favoritos_empresa on favoritos (empresa_id) where empresa_id is not null;
create index if not exists idx_vistas_empresa on vistas (empresa_id) where empresa_id is not null;
create index if not exists idx_conversaciones_empresa on conversaciones (empresa_id) where empresa_id is not null;
create index if not exists idx_mensajes_empresa on mensajes (empresa_id) where empresa_id is not null;
create index if not exists idx_claves_empresa on claves_api (empresa_id) where empresa_id is not null;
create index if not exists idx_webhooks_empresa on webhooks (empresa_id) where empresa_id is not null;
create index if not exists idx_entregas_empresa on entregas_webhook (empresa_id) where empresa_id is not null;
create index if not exists idx_importaciones_empresa on importaciones (empresa_id) where empresa_id is not null;

-- 2) Resolución de empresa por referencia
create or replace function empresa_de_dato(p_alumno uuid, p_apoderado uuid, p_oportunidad uuid)
returns uuid language sql stable security definer set search_path = public as $$
  select coalesce(
    (select a.empresa_id from alumnos a where a.id = p_alumno),
    (select a.empresa_id from apoderados a where a.id = p_apoderado),
    (select o.empresa_id from oportunidades o where o.id = p_oportunidad)
  );
$$;
comment on function empresa_de_dato(uuid,uuid,uuid) is 'Empresa del registro relacionado (alumno/apoderado/postulación), en ese orden';
grant execute on function empresa_de_dato(uuid,uuid,uuid) to public;

create or replace function empresa_de_registro(p_tipo text, p_id uuid)
returns uuid language sql stable security definer set search_path = public as $$
  select case lower(coalesce(p_tipo, ''))
    when 'alumno' then (select a.empresa_id from alumnos a where a.id = p_id)
    when 'apoderado' then (select a.empresa_id from apoderados a where a.id = p_id)
    when 'postulacion' then (select o.empresa_id from oportunidades o where o.id = p_id)
    when 'oportunidad' then (select o.empresa_id from oportunidades o where o.id = p_id)
    when 'tarea' then (select t.empresa_id from tareas t where t.id = p_id)
    when 'nota' then (select n.empresa_id from notas n where n.id = p_id)
    when 'evento' then (select e.empresa_id from eventos e where e.id = p_id)
    when 'correo' then (select c.empresa_id from correos c where c.id = p_id)
    when 'colegio' then null
    else null
  end;
$$;
comment on function empresa_de_registro(text, uuid) is 'Empresa del registro según tipo usado en actividad/favoritos';
grant execute on function empresa_de_registro(text, uuid) to public;

-- 3) Triggers de herencia
create or replace function heredar_empresa_tres()
returns trigger language plpgsql as $$
begin
  if NEW.empresa_id is null then
    NEW.empresa_id := empresa_de_dato(NEW.alumno_id, NEW.apoderado_id, NEW.oportunidad_id);
  end if;
  return NEW;
end; $$;

create or replace function heredar_empresa_alumno()
returns trigger language plpgsql as $$
begin
  if NEW.empresa_id is null then
    NEW.empresa_id := empresa_de_dato(NEW.alumno_id, null, null);
  end if;
  return NEW;
end; $$;

create or replace function heredar_empresa_usuario()
returns trigger language plpgsql as $$
begin
  if NEW.empresa_id is null then
    NEW.empresa_id := empresa_del_usuario(NEW.usuario_id);
  end if;
  return NEW;
end; $$;

create or replace function heredar_empresa_mensaje()
returns trigger language plpgsql as $$
begin
  if NEW.empresa_id is null then
    select c.empresa_id into NEW.empresa_id from conversaciones c where c.id = NEW.conversacion_id;
  end if;
  return NEW;
end; $$;

create or replace function heredar_empresa_entrega()
returns trigger language plpgsql as $$
begin
  if NEW.empresa_id is null then
    select w.empresa_id into NEW.empresa_id from webhooks w where w.id = NEW.webhook_id;
  end if;
  return NEW;
end; $$;

create or replace function heredar_empresa_actividad()
returns trigger language plpgsql as $$
begin
  if NEW.empresa_id is null then
    NEW.empresa_id := empresa_de_registro(NEW.tipo, NEW.referencia_id);
  end if;
  return NEW;
end; $$;

create or replace function heredar_empresa_favorito()
returns trigger language plpgsql as $$
begin
  if NEW.empresa_id is null then
    NEW.empresa_id := coalesce(empresa_de_registro(NEW.tipo, NEW.referencia_id),
                               empresa_del_usuario(NEW.usuario_id));
  end if;
  return NEW;
end; $$;

drop trigger if exists trg_empresa_tarea on tareas;
create trigger trg_empresa_tarea before insert or update on tareas for each row execute function heredar_empresa_tres();
drop trigger if exists trg_empresa_nota on notas;
create trigger trg_empresa_nota before insert or update on notas for each row execute function heredar_empresa_tres();
drop trigger if exists trg_empresa_evento on eventos;
create trigger trg_empresa_evento before insert or update on eventos for each row execute function heredar_empresa_tres();
drop trigger if exists trg_empresa_correo on correos;
create trigger trg_empresa_correo before insert or update on correos for each row execute function heredar_empresa_tres();
drop trigger if exists trg_empresa_ingreso on ingresos_medicos;
create trigger trg_empresa_ingreso before insert or update on ingresos_medicos for each row execute function heredar_empresa_alumno();
drop trigger if exists trg_empresa_traslado on traslados;
create trigger trg_empresa_traslado before insert or update on traslados for each row execute function heredar_empresa_alumno();
drop trigger if exists trg_empresa_vinculo on alumno_apoderado;
create trigger trg_empresa_vinculo before insert or update on alumno_apoderado for each row execute function heredar_empresa_alumno();
drop trigger if exists trg_empresa_actividad on actividad;
create trigger trg_empresa_actividad before insert or update on actividad for each row execute function heredar_empresa_actividad();
drop trigger if exists trg_empresa_favorito on favoritos;
create trigger trg_empresa_favorito before insert or update on favoritos for each row execute function heredar_empresa_favorito();
drop trigger if exists trg_empresa_vista on vistas;
create trigger trg_empresa_vista before insert or update on vistas for each row execute function heredar_empresa_usuario();
drop trigger if exists trg_empresa_conversacion on conversaciones;
create trigger trg_empresa_conversacion before insert or update on conversaciones for each row execute function heredar_empresa_usuario();
drop trigger if exists trg_empresa_mensaje on mensajes;
create trigger trg_empresa_mensaje before insert or update on mensajes for each row execute function heredar_empresa_mensaje();
drop trigger if exists trg_empresa_clave on claves_api;
create trigger trg_empresa_clave before insert or update on claves_api for each row execute function heredar_empresa_usuario();
drop trigger if exists trg_empresa_webhook on webhooks;
create trigger trg_empresa_webhook before insert or update on webhooks for each row execute function heredar_empresa_usuario();
drop trigger if exists trg_empresa_entrega on entregas_webhook;
create trigger trg_empresa_entrega before insert or update on entregas_webhook for each row execute function heredar_empresa_entrega();

-- 4) Backfill de filas existentes
update tareas t set empresa_id = empresa_de_dato(t.alumno_id, t.apoderado_id, t.oportunidad_id) where t.empresa_id is null;
update notas n set empresa_id = empresa_de_dato(n.alumno_id, n.apoderado_id, n.oportunidad_id) where n.empresa_id is null;
update eventos e set empresa_id = empresa_de_dato(e.alumno_id, e.apoderado_id, e.oportunidad_id) where e.empresa_id is null;
update correos c set empresa_id = empresa_de_dato(c.alumno_id, c.apoderado_id, c.oportunidad_id) where c.empresa_id is null;
update ingresos_medicos i set empresa_id = empresa_de_dato(i.alumno_id, null, null) where i.empresa_id is null;
update traslados t set empresa_id = empresa_de_dato(t.alumno_id, null, null) where t.empresa_id is null;
update alumno_apoderado v set empresa_id = empresa_de_dato(v.alumno_id, null, null) where v.empresa_id is null;
update actividad a set empresa_id = empresa_de_registro(a.tipo, a.referencia_id) where a.empresa_id is null;
update favoritos f set empresa_id = coalesce(empresa_de_registro(f.tipo, f.referencia_id), empresa_del_usuario(f.usuario_id)) where f.empresa_id is null;
update vistas v set empresa_id = empresa_del_usuario(v.usuario_id) where v.empresa_id is null;
update conversaciones c set empresa_id = empresa_del_usuario(c.usuario_id) where c.empresa_id is null;
update mensajes m set empresa_id = (select c.empresa_id from conversaciones c where c.id = m.conversacion_id) where m.empresa_id is null;
update claves_api k set empresa_id = empresa_del_usuario(k.usuario_id) where k.empresa_id is null;
update webhooks w set empresa_id = empresa_del_usuario(w.usuario_id) where w.empresa_id is null;
update entregas_webhook e set empresa_id = (select w.empresa_id from webhooks w where w.id = e.webhook_id) where e.empresa_id is null;

-- 5) Importaciones: registrar actor además de empresa
create or replace function importar_alumnos_cuarentena(p_usuario_id uuid, p_filas jsonb)
returns jsonb
language plpgsql security definer set search_path = public as $$
declare
  fila jsonb;
  n integer := 0;
  creados integer := 0;
  errores jsonb := '[]'::jsonb;
  v_rut text;
  v_hash text;
  v_rbd text;
  v_cons text;
  v_job uuid := gen_random_uuid();
  v_nombre text;
  v_empresa uuid;
begin
  if p_usuario_id is null then raise exception 'El usuario es obligatorio.' using errcode = 'P0001'; end if;
  if jsonb_typeof(p_filas) <> 'array' or jsonb_array_length(p_filas) > 1000 then raise exception 'La carga debe ser una lista de hasta 1000 alumnos.' using errcode = 'P0001'; end if;
  perform set_config('app.usuario_id', p_usuario_id::text, true);
  v_empresa := empresa_del_usuario(p_usuario_id);
  for fila in select value from jsonb_array_elements(p_filas) loop
    n := n + 1;
    begin
      v_rut := rut_normalizado(fila->>'rut');
      if v_rut is null or not es_rut_valido(v_rut) then
        raise exception 'RUT inválido.' using errcode = 'P0001';
      end if;
      if v_rut = '00000000-0' then
        raise exception 'RUT reservado.' using errcode = 'P0001';
      end if;
      v_hash := encode(digest(v_rut, 'sha256'), 'hex');
      if exists (select 1 from supresion where rut_hash = v_hash) then
        raise exception 'RUT en lista de supresión.' using errcode = 'P0003';
      end if;
      v_rbd := nullif(btrim(coalesce(fila->>'colegio_rbd', '')), '');
      if v_rbd is null or not exists (select 1 from colegios where rbd = v_rbd) then
        raise exception 'RBD desconocido (revise el directorio MINEDUC).' using errcode = 'P0001';
      end if;
      v_cons := lower(btrim(coalesce(fila->>'consentimiento', '')));
      if v_cons not in ('si', 'sí', 'yes', '1') then
        raise exception 'Falta consentimiento explícito.' using errcode = 'P0001';
      end if;
      v_nombre := btrim(coalesce(fila->>'nombres', '')) || ' ' || btrim(coalesce(fila->>'apellidos', ''));
      if btrim(coalesce(fila->>'nombres', '')) = '' then
        raise exception 'El nombre es obligatorio.' using errcode = 'P0001';
      end if;
      insert into oportunidades (nombre, importe, etapa, colegio_rbd, prospecto_rut, origen, consentimiento, ttl, empresa_id)
      values (v_nombre, 0, 'contacto', v_rbd, v_rut, 'csv',
        jsonb_build_object('canal', 'csv', 'fecha', now()::date, 'prueba', v_job),
        now()::date + 90, empresa_de_rbd(v_rbd));
      creados := creados + 1;
    exception when others then
      errores := errores || jsonb_build_array(jsonb_build_object(
        'fila', coalesce((fila->>'fila')::integer, n + 1),
        'codigo', case SQLSTATE
          when 'P0003' then 'SUPRIMIDO'
          else case
            when SQLERRM like '%RUT%' then 'RUT_INVALIDO'
            when SQLERRM like '%RBD%' then 'RBD_DESCONOCIDO'
            when SQLERRM like '%onsentimiento%' then 'CONSENTIMIENTO_FALTANTE'
            else 'FILA_INVALIDA'
          end
        end,
        'mensaje', left(SQLERRM, 300)));
    end;
  end loop;
  insert into importaciones (job_id, objeto, total, creados, errores, usuario_id, empresa_id)
  values (v_job, 'alumnos', n, creados, errores, p_usuario_id, v_empresa);
  return jsonb_build_object('job_id', v_job, 'total', n, 'creados', creados, 'errores', errores);
end; $$;
grant execute on function importar_alumnos_cuarentena(uuid, jsonb) to public;
