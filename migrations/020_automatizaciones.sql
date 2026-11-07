-- 020 — Automatizaciones acotadas al flujo escolar de admisión.
-- Disparadores sobre postulaciones; acciones por funciones SQL seguras.

create table if not exists automatizaciones (
  id uuid primary key default gen_random_uuid(),
  nombre text not null check (char_length(btrim(nombre)) between 1 and 120),
  activa boolean not null default true,
  disparador text not null check (disparador in ('al_crear','al_actualizar')),
  objeto text not null default 'postulacion' check (objeto = 'postulacion'),
  condiciones jsonb not null default '[]'::jsonb check (jsonb_typeof(condiciones)='array'),
  acciones jsonb not null default '[]'::jsonb check (jsonb_typeof(acciones)='array' and jsonb_array_length(acciones) between 1 and 10),
  creado_en timestamptz not null default now(),
  actualizado_en timestamptz not null default now()
);
comment on table automatizaciones is 'Reglas simples sobre postulaciones de admisión';
create index if not exists idx_automatizaciones_activas on automatizaciones(activa,disparador,objeto);

create table if not exists ejecuciones (
  id uuid primary key default gen_random_uuid(),
  automatizacion_id uuid not null references automatizaciones(id) on delete cascade,
  registro_id uuid,
  disparador text not null,
  ok boolean not null,
  detalle jsonb not null default '{}'::jsonb,
  usuario_id uuid references usuarios(id) on delete set null,
  creado_en timestamptz not null default now()
);
create index if not exists idx_ejecuciones_automatizacion on ejecuciones(automatizacion_id,creado_en desc);
create index if not exists idx_ejecuciones_registro on ejecuciones(registro_id,creado_en desc);
alter table automatizaciones enable row level security;
alter table ejecuciones enable row level security;
revoke all on automatizaciones,ejecuciones from public;
grant select on automatizaciones,ejecuciones to public;

create or replace function validar_regla_automatizacion(p_condiciones jsonb,p_acciones jsonb)
returns boolean language plpgsql immutable set search_path=public as $$
declare c jsonb; a jsonb; campo text; op text; tipo_accion text;
begin
  if jsonb_typeof(coalesce(p_condiciones,'[]'))<>'array' or jsonb_array_length(coalesce(p_condiciones,'[]'))>10 then
    raise exception 'La regla admite hasta 10 condiciones.' using errcode='P0001';
  end if;
  if jsonb_typeof(coalesce(p_acciones,'[]'))<>'array' or jsonb_array_length(coalesce(p_acciones,'[]')) not between 1 and 10 then
    raise exception 'La regla debe tener entre 1 y 10 acciones.' using errcode='P0001';
  end if;
  for c in select value from jsonb_array_elements(coalesce(p_condiciones,'[]')) loop
    campo:=coalesce(c->>'campo',''); op:=coalesce(c->>'operador','');
    if campo not in ('nombre','importe','etapa','colegio_rbd') then raise exception 'Campo de condición no permitido.' using errcode='P0001'; end if;
    if op not in ('igual','contiene','mayor_que') then raise exception 'Operador de condición no permitido.' using errcode='P0001'; end if;
    if op='mayor_que' and campo<>'importe' then raise exception 'mayor_que solo aplica al importe.' using errcode='P0001'; end if;
    if coalesce(c->>'valor','')='' then raise exception 'Cada condición debe tener un valor.' using errcode='P0001'; end if;
  end loop;
  for a in select value from jsonb_array_elements(p_acciones) loop
    tipo_accion:=coalesce(a->>'tipo','');
    if tipo_accion not in ('mover_etapa','crear_tarea','crear_nota') then raise exception 'Tipo de acción no permitido.' using errcode='P0001'; end if;
    if tipo_accion='crear_tarea' and coalesce(a#>>'{params,titulo}','')='' then raise exception 'La acción crear_tarea requiere params.titulo.' using errcode='P0001'; end if;
    if tipo_accion='crear_nota' and (coalesce(a#>>'{params,cuerpo}','')='' or coalesce(a#>>'{params,titulo}','')='') then raise exception 'La acción crear_nota requiere título y cuerpo.' using errcode='P0001'; end if;
    if tipo_accion='mover_etapa' and coalesce(a#>>'{params,etapa}','') not in ('contacto','postulante','evaluado','aceptado','matriculado') then
      raise exception 'Etapa de la acción no válida.' using errcode='P0001';
    end if;
  end loop;
  return true;
end;
$$;

create or replace function listar_automatizaciones()
returns setof automatizaciones language sql security definer set search_path=public
as $$ select * from automatizaciones order by creado_en desc; $$;

create or replace function obtener_automatizacion(p_id uuid)
returns automatizaciones language plpgsql security definer set search_path=public as $$
declare v automatizaciones%rowtype;
begin
  select * into v from automatizaciones where id=p_id;
  if not found then raise exception 'Automatización no encontrada.' using errcode='P0002'; end if;
  return v;
end; $$;

create or replace function crear_automatizacion(
  p_nombre text,p_disparador text,p_condiciones jsonb,p_acciones jsonb
)
returns automatizaciones language plpgsql security definer set search_path=public as $$
declare v automatizaciones%rowtype; n text:=btrim(coalesce(p_nombre,'')); d text:=lower(btrim(coalesce(p_disparador,'')));
begin
  if char_length(n) not between 1 and 120 then raise exception 'El nombre debe tener entre 1 y 120 caracteres.' using errcode='P0001'; end if;
  if d not in ('al_crear','al_actualizar') then raise exception 'Disparador no válido.' using errcode='P0001'; end if;
  perform validar_regla_automatizacion(coalesce(p_condiciones,'[]'),coalesce(p_acciones,'[]'));
  insert into automatizaciones(nombre,disparador,condiciones,acciones)
  values(n,d,coalesce(p_condiciones,'[]'),coalesce(p_acciones,'[]')) returning * into v;
  return v;
end; $$;

create or replace function actualizar_automatizacion(
  p_id uuid,p_nombre text,p_disparador text,p_condiciones jsonb,p_acciones jsonb
)
returns automatizaciones language plpgsql security definer set search_path=public as $$
declare v automatizaciones%rowtype; n text:=btrim(coalesce(p_nombre,'')); d text:=lower(btrim(coalesce(p_disparador,'')));
begin
  if char_length(n) not between 1 and 120 then raise exception 'El nombre debe tener entre 1 y 120 caracteres.' using errcode='P0001'; end if;
  if d not in ('al_crear','al_actualizar') then raise exception 'Disparador no válido.' using errcode='P0001'; end if;
  perform validar_regla_automatizacion(coalesce(p_condiciones,'[]'),coalesce(p_acciones,'[]'));
  update automatizaciones set nombre=n,disparador=d,condiciones=coalesce(p_condiciones,'[]'),acciones=coalesce(p_acciones,'[]'),actualizado_en=now()
  where id=p_id returning * into v;
  if not found then raise exception 'Automatización no encontrada.' using errcode='P0002'; end if;
  return v;
end; $$;

create or replace function eliminar_automatizacion(p_id uuid)
returns boolean language plpgsql security definer set search_path=public as $$
begin delete from automatizaciones where id=p_id; if not found then raise exception 'Automatización no encontrada.' using errcode='P0002'; end if; return true; end; $$;

create or replace function alternar_automatizacion(p_id uuid,p_activa boolean)
returns automatizaciones language plpgsql security definer set search_path=public as $$
declare v automatizaciones%rowtype;
begin update automatizaciones set activa=coalesce(p_activa,false),actualizado_en=now() where id=p_id returning * into v;
  if not found then raise exception 'Automatización no encontrada.' using errcode='P0002'; end if; return v;
end; $$;

create or replace function listar_ejecuciones(p_id uuid,p_limite integer default 50)
returns setof ejecuciones language sql security definer set search_path=public as $$
select * from ejecuciones where automatizacion_id=p_id order by creado_en desc limit least(greatest(coalesce(p_limite,50),1),100);
$$;

-- El motor se invoca desde la API dentro de la misma transacción que la
-- operación original. Cada automatización tiene subtransacción propia: un
-- error revierte sus acciones, queda registrado, y no cancela la operación.
create or replace function ejecutar_automatizaciones(
  p_disparador text,p_objeto text,p_registro_id uuid,p_usuario_id uuid
)
returns integer language plpgsql security definer set search_path=public as $$
declare
  regla automatizaciones%rowtype; condicion jsonb; accion jsonb;
  registro jsonb; actual text; esperado text; coincide boolean; todas boolean;
  cantidad integer:=0; n_acciones integer; detalle_error text;
begin
  if p_usuario_id is not null then perform set_config('app.usuario_id',p_usuario_id::text,true); end if;
  if p_objeto<>'postulacion' or p_disparador not in ('al_crear','al_actualizar') then return 0; end if;
  select to_jsonb(o) into registro from oportunidades o where o.id=p_registro_id;
  if registro is null then return 0; end if;
  for regla in select * from automatizaciones a where a.activa and a.objeto=p_objeto and a.disparador=p_disparador order by a.creado_en loop
    todas:=true;
    for condicion in select value from jsonb_array_elements(regla.condiciones) loop
      actual:=coalesce(registro->>(condicion->>'campo'),''); esperado:=condicion->>'valor'; coincide:=false;
      case condicion->>'operador'
        when 'igual' then coincide:=lower(actual)=lower(esperado);
        when 'contiene' then coincide:=position(lower(esperado) in lower(actual))>0;
        when 'mayor_que' then
          if actual ~ '^-?[0-9]+(\.[0-9]+)?$' and esperado ~ '^-?[0-9]+(\.[0-9]+)?$' then coincide:=actual::numeric>esperado::numeric; end if;
        else coincide:=false;
      end case;
      if not coincide then todas:=false; exit; end if;
    end loop;
    if not todas then continue; end if;
    begin
      n_acciones:=0;
      for accion in select value from jsonb_array_elements(regla.acciones) loop
        if accion->>'tipo'='crear_tarea' then
          perform crear_tarea(accion#>>'{params,titulo}',coalesce(accion#>>'{params,detalle}',''),'pendiente',coalesce(accion#>>'{params,prioridad}','media'),null,null,null,p_registro_id);
        elsif accion->>'tipo'='crear_nota' then
          perform crear_nota(accion#>>'{params,titulo}',accion#>>'{params,cuerpo}',null,null,p_registro_id);
        elsif accion->>'tipo'='mover_etapa' then
          perform mover_etapa_oportunidad(p_registro_id,accion#>>'{params,etapa}');
        end if;
        n_acciones:=n_acciones+1;
      end loop;
      insert into ejecuciones(automatizacion_id,registro_id,disparador,ok,detalle,usuario_id)
      values(regla.id,p_registro_id,p_disparador,true,jsonb_build_object('acciones',n_acciones,'resultado','correcto'),p_usuario_id);
    exception when others then
      detalle_error:=left(SQLERRM,500);
      insert into ejecuciones(automatizacion_id,registro_id,disparador,ok,detalle,usuario_id)
      values(regla.id,p_registro_id,p_disparador,false,jsonb_build_object('error',detalle_error),p_usuario_id);
    end;
    cantidad:=cantidad+1;
  end loop;
  return cantidad;
end; $$;

grant execute on function validar_regla_automatizacion(jsonb,jsonb) to public;
grant execute on function listar_automatizaciones() to public;
grant execute on function obtener_automatizacion(uuid) to public;
grant execute on function crear_automatizacion(text,text,jsonb,jsonb) to public;
grant execute on function actualizar_automatizacion(uuid,text,text,jsonb,jsonb) to public;
grant execute on function eliminar_automatizacion(uuid) to public;
grant execute on function alternar_automatizacion(uuid,boolean) to public;
grant execute on function listar_ejecuciones(uuid,integer) to public;
grant execute on function ejecutar_automatizaciones(text,text,uuid,uuid) to public;
