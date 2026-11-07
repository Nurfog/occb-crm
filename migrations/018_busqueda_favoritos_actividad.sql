-- 018 — Búsqueda, favoritos, vistas y auditoría escolar.
-- 016/017 ya están aplicadas; empresas y CRM externo quedan fuera.

create extension if not exists pg_trgm;

alter table personas add column if not exists busqueda_vector tsvector generated always as
  (to_tsvector('pg_catalog.spanish', coalesce(nombre,'')||' '||coalesce(apellido,'')||' '||coalesce(correo,'')||' '||coalesce(telefono,'')||' '||coalesce(puesto,''))) stored;
alter table oportunidades add column if not exists busqueda_vector tsvector generated always as
  (to_tsvector('pg_catalog.spanish', coalesce(nombre,'')||' '||coalesce(etapa,'')||' '||coalesce(colegio_rbd,''))) stored;
alter table alumnos add column if not exists busqueda_vector tsvector generated always as
  (to_tsvector('pg_catalog.spanish', coalesce(nombres,'')||' '||coalesce(apellidos,'')||' '||coalesce(rut,'')||' '||coalesce(curso,'')||' '||coalesce(estado,''))) stored;
alter table apoderados add column if not exists busqueda_vector tsvector generated always as
  (to_tsvector('pg_catalog.spanish', coalesce(nombres,'')||' '||coalesce(apellidos,'')||' '||coalesce(rut,'')||' '||coalesce(correo,'')||' '||coalesce(telefono,''))) stored;
alter table tareas add column if not exists busqueda_vector tsvector generated always as
  (to_tsvector('pg_catalog.spanish', coalesce(titulo,'')||' '||coalesce(detalle,'')||' '||coalesce(estado,''))) stored;
alter table notas add column if not exists busqueda_vector tsvector generated always as
  (to_tsvector('pg_catalog.spanish', coalesce(titulo,'')||' '||coalesce(cuerpo,''))) stored;
alter table eventos add column if not exists busqueda_vector tsvector generated always as
  (to_tsvector('pg_catalog.spanish', coalesce(titulo,'')||' '||coalesce(lugar,'')||' '||coalesce(descripcion,''))) stored;
alter table correos add column if not exists busqueda_vector tsvector generated always as
  (to_tsvector('pg_catalog.spanish', coalesce(asunto,'')||' '||coalesce(remitente,'')||' '||coalesce(cuerpo,''))) stored;
alter table colegios add column if not exists busqueda_vector tsvector generated always as
  (to_tsvector('pg_catalog.spanish', coalesce(nombre,'')||' '||coalesce(rbd,'')||' '||coalesce(comuna,''))) stored;

create index if not exists idx_personas_busqueda on personas using gin(busqueda_vector);
create index if not exists idx_oportunidades_busqueda on oportunidades using gin(busqueda_vector);
create index if not exists idx_alumnos_busqueda on alumnos using gin(busqueda_vector);
create index if not exists idx_apoderados_busqueda on apoderados using gin(busqueda_vector);
create index if not exists idx_tareas_busqueda on tareas using gin(busqueda_vector);
create index if not exists idx_notas_busqueda on notas using gin(busqueda_vector);
create index if not exists idx_eventos_busqueda on eventos using gin(busqueda_vector);
create index if not exists idx_correos_busqueda on correos using gin(busqueda_vector);
create index if not exists idx_colegios_busqueda on colegios using gin(busqueda_vector);

create or replace function buscar_espacio(p_texto text)
returns table(tipo text,id text,titulo text,detalle text,puntaje real)
language plpgsql security definer set search_path=public
as $$
declare q text:=left(btrim(coalesce(p_texto,'')),200); ts tsquery;
begin
  if q='' then return; end if;
  if char_length(q)>=2 then ts:=websearch_to_tsquery('pg_catalog.spanish',q); end if;
  return query
  with r as (
    select 'alumno'::text t,a.id::text rid,a.nombres||' '||a.apellidos titulo,a.curso||' · '||a.rut detalle,
      (case when char_length(q)>=2 and a.busqueda_vector@@ts then ts_rank(a.busqueda_vector,ts)+1 else .1 end)::real rank
    from alumnos a where (char_length(q)>=2 and a.busqueda_vector@@ts) or a.nombres ilike '%'||q||'%' or a.apellidos ilike '%'||q||'%' or a.rut ilike '%'||q||'%' or a.curso ilike '%'||q||'%'
    union all
    select 'apoderado',a.id::text,a.nombres||' '||a.apellidos,coalesce(nullif(a.correo,''),a.telefono),
      (case when char_length(q)>=2 and a.busqueda_vector@@ts then ts_rank(a.busqueda_vector,ts)+1 else .1 end)::real
    from apoderados a where (char_length(q)>=2 and a.busqueda_vector@@ts) or a.nombres ilike '%'||q||'%' or a.apellidos ilike '%'||q||'%' or a.rut ilike '%'||q||'%' or a.correo ilike '%'||q||'%' or a.telefono ilike '%'||q||'%'
    union all
    select 'persona',p.id::text,p.nombre||' '||p.apellido,coalesce(nullif(p.correo,''),p.puesto),
      (case when char_length(q)>=2 and p.busqueda_vector@@ts then ts_rank(p.busqueda_vector,ts)+1 else .1 end)::real
    from personas p where (char_length(q)>=2 and p.busqueda_vector@@ts) or p.nombre ilike '%'||q||'%' or p.apellido ilike '%'||q||'%' or p.correo ilike '%'||q||'%' or p.telefono ilike '%'||q||'%'
    union all
    select 'postulacion',o.id::text,o.nombre,o.etapa||' · '||coalesce(o.colegio_rbd,''),
      (case when char_length(q)>=2 and o.busqueda_vector@@ts then ts_rank(o.busqueda_vector,ts)+1 else .1 end)::real
    from oportunidades o where (char_length(q)>=2 and o.busqueda_vector@@ts) or o.nombre ilike '%'||q||'%' or o.etapa ilike '%'||q||'%' or coalesce(o.colegio_rbd,'') ilike '%'||q||'%'
    union all
    select 'tarea',t.id::text,t.titulo,t.estado||' · '||left(t.detalle,100),
      (case when char_length(q)>=2 and t.busqueda_vector@@ts then ts_rank(t.busqueda_vector,ts)+1 else .1 end)::real
    from tareas t where (char_length(q)>=2 and t.busqueda_vector@@ts) or t.titulo ilike '%'||q||'%' or t.detalle ilike '%'||q||'%'
    union all
    select 'nota',n.id::text,coalesce(nullif(n.titulo,''),left(n.cuerpo,80)),left(n.cuerpo,100),
      (case when char_length(q)>=2 and n.busqueda_vector@@ts then ts_rank(n.busqueda_vector,ts)+1 else .1 end)::real
    from notas n where (char_length(q)>=2 and n.busqueda_vector@@ts) or n.titulo ilike '%'||q||'%' or n.cuerpo ilike '%'||q||'%'
    union all
    select 'evento',e.id::text,e.titulo,e.lugar||' · '||to_char(e.inicio,'DD/MM/YYYY HH24:MI'),
      (case when char_length(q)>=2 and e.busqueda_vector@@ts then ts_rank(e.busqueda_vector,ts)+1 else .1 end)::real
    from eventos e where (char_length(q)>=2 and e.busqueda_vector@@ts) or e.titulo ilike '%'||q||'%' or e.lugar ilike '%'||q||'%' or e.descripcion ilike '%'||q||'%'
    union all
    select 'correo',c.id::text,coalesce(nullif(c.asunto,''),'(sin asunto)'),c.remitente,
      (case when char_length(q)>=2 and c.busqueda_vector@@ts then ts_rank(c.busqueda_vector,ts)+1 else .1 end)::real
    from correos c where (char_length(q)>=2 and c.busqueda_vector@@ts) or c.asunto ilike '%'||q||'%' or c.remitente ilike '%'||q||'%' or c.cuerpo ilike '%'||q||'%'
    union all
    select 'colegio',c.rbd,c.nombre,c.rbd||' · '||c.comuna,
      (case when char_length(q)>=2 and c.busqueda_vector@@ts then ts_rank(c.busqueda_vector,ts)+1 else .1 end)::real
    from colegios c where (char_length(q)>=2 and c.busqueda_vector@@ts) or c.nombre ilike '%'||q||'%' or c.rbd ilike '%'||q||'%'
  ) select r.t,r.rid,r.titulo,r.detalle,r.rank from r order by r.rank desc,r.titulo limit 20;
end; $$;

create table if not exists favoritos(
  usuario_id uuid not null references usuarios(id) on delete cascade,
  tipo text not null check(tipo in ('alumno','apoderado','persona','postulacion')),
  referencia_id uuid not null, creado_en timestamptz not null default now(),
  primary key(usuario_id,tipo,referencia_id)
);
create index if not exists idx_favoritos_usuario on favoritos(usuario_id,creado_en desc);
alter table favoritos enable row level security; revoke all on favoritos from public; grant select on favoritos to public;

create or replace function alternar_favorito(p_usuario_id uuid,p_tipo text,p_referencia_id uuid)
returns boolean language plpgsql security definer set search_path=public as $$
declare t text:=lower(btrim(coalesce(p_tipo,'')));
begin
  if p_usuario_id is null or p_referencia_id is null then raise exception 'Usuario y registro son obligatorios.' using errcode='P0001'; end if;
  if t not in ('alumno','apoderado','persona','postulacion') then raise exception 'Tipo de favorito no válido.' using errcode='P0001'; end if;
  if t='alumno' and not exists(select 1 from alumnos where id=p_referencia_id) then raise exception 'Alumno no encontrado.' using errcode='P0002'; end if;
  if t='apoderado' and not exists(select 1 from apoderados where id=p_referencia_id) then raise exception 'Apoderado no encontrado.' using errcode='P0002'; end if;
  if t='persona' and not exists(select 1 from personas where id=p_referencia_id) then raise exception 'Persona no encontrada.' using errcode='P0002'; end if;
  if t='postulacion' and not exists(select 1 from oportunidades where id=p_referencia_id) then raise exception 'Postulación no encontrada.' using errcode='P0002'; end if;
  delete from favoritos where usuario_id=p_usuario_id and tipo=t and referencia_id=p_referencia_id;
  if found then return false; end if;
  insert into favoritos(usuario_id,tipo,referencia_id) values(p_usuario_id,t,p_referencia_id); return true;
end; $$;

create or replace function listar_favoritos(p_usuario_id uuid)
returns table(tipo text,id uuid,titulo text,detalle text,creado_en timestamptz)
language sql security definer set search_path=public as $$
  select f.tipo,f.referencia_id,a.nombres||' '||a.apellidos,a.rut,f.creado_en from favoritos f join alumnos a on f.tipo='alumno' and a.id=f.referencia_id where f.usuario_id=p_usuario_id
  union all select f.tipo,f.referencia_id,a.nombres||' '||a.apellidos,coalesce(nullif(a.correo,''),a.telefono),f.creado_en from favoritos f join apoderados a on f.tipo='apoderado' and a.id=f.referencia_id where f.usuario_id=p_usuario_id
  union all select f.tipo,f.referencia_id,p.nombre||' '||p.apellido,p.correo,f.creado_en from favoritos f join personas p on f.tipo='persona' and p.id=f.referencia_id where f.usuario_id=p_usuario_id
  union all select f.tipo,f.referencia_id,o.nombre,o.etapa,f.creado_en from favoritos f join oportunidades o on f.tipo='postulacion' and o.id=f.referencia_id where f.usuario_id=p_usuario_id
  order by 5 desc;
$$;

create table if not exists vistas(
  id uuid primary key default gen_random_uuid(), usuario_id uuid not null references usuarios(id) on delete cascade,
  objeto text not null check(objeto in ('alumnos','apoderados','personas','oportunidades')),
  nombre text not null check(char_length(btrim(nombre)) between 1 and 80),
  tipo text not null check(tipo in ('tabla','kanban')),
  filtros jsonb not null default '{}'::jsonb check(jsonb_typeof(filtros)='object'),
  orden jsonb not null default '{}'::jsonb check(jsonb_typeof(orden)='object'),
  creado_en timestamptz not null default now(), actualizado_en timestamptz not null default now(),
  unique(usuario_id,objeto,nombre), check(tipo<>'kanban' or objeto='oportunidades')
);
create index if not exists idx_vistas_usuario_objeto on vistas(usuario_id,objeto,nombre);
alter table vistas enable row level security; revoke all on vistas from public; grant select on vistas to public;

create or replace function listar_vistas(p_usuario_id uuid,p_objeto text)
returns setof vistas language sql security definer set search_path=public
as $$ select * from vistas where usuario_id=p_usuario_id and objeto=p_objeto order by nombre; $$;
create or replace function crear_vista(p_usuario_id uuid,p_objeto text,p_nombre text,p_tipo text,p_filtros jsonb default '{}',p_orden jsonb default '{}')
returns vistas language plpgsql security definer set search_path=public as $$
declare v vistas%rowtype; o text:=lower(btrim(coalesce(p_objeto,''))); t text:=lower(btrim(coalesce(p_tipo,''))); n text:=btrim(coalesce(p_nombre,''));
begin
  if p_usuario_id is null then raise exception 'El usuario es obligatorio.' using errcode='P0001'; end if;
  if o not in ('alumnos','apoderados','personas','oportunidades') then raise exception 'Objeto de vista no válido.' using errcode='P0001'; end if;
  if t not in ('tabla','kanban') or (t='kanban' and o<>'oportunidades') then raise exception 'Tipo de vista no válido para el objeto.' using errcode='P0001'; end if;
  if char_length(n) not between 1 and 80 then raise exception 'El nombre debe tener entre 1 y 80 caracteres.' using errcode='P0001'; end if;
  insert into vistas(usuario_id,objeto,nombre,tipo,filtros,orden) values(p_usuario_id,o,n,t,coalesce(p_filtros,'{}'),coalesce(p_orden,'{}')) returning * into v; return v;
exception when unique_violation then raise exception 'Ya existe una vista con ese nombre.' using errcode='P0003';
end; $$;
create or replace function actualizar_vista(p_usuario_id uuid,p_id uuid,p_nombre text,p_tipo text,p_filtros jsonb default '{}',p_orden jsonb default '{}')
returns vistas language plpgsql security definer set search_path=public as $$
declare v vistas%rowtype; t text:=lower(btrim(coalesce(p_tipo,''))); n text:=btrim(coalesce(p_nombre,''));
begin
  if t not in ('tabla','kanban') then raise exception 'Tipo de vista no válido.' using errcode='P0001'; end if;
  update vistas set nombre=n,tipo=t,filtros=coalesce(p_filtros,'{}'),orden=coalesce(p_orden,'{}'),actualizado_en=now()
    where id=p_id and usuario_id=p_usuario_id and (t<>'kanban' or objeto='oportunidades') returning * into v;
  if not found then raise exception 'Vista no encontrada o tipo incompatible.' using errcode='P0002'; end if; return v;
exception when unique_violation then raise exception 'Ya existe una vista con ese nombre.' using errcode='P0003';
end; $$;
create or replace function eliminar_vista(p_usuario_id uuid,p_id uuid)
returns boolean language plpgsql security definer set search_path=public as $$
begin delete from vistas where id=p_id and usuario_id=p_usuario_id; if not found then raise exception 'Vista no encontrada.' using errcode='P0002'; end if; return true; end; $$;

create table if not exists actividad(
  id uuid primary key default gen_random_uuid(), tipo text not null, referencia_id uuid not null,
  accion text not null check(accion in ('creado','actualizado','eliminado')),
  detalle jsonb not null default '{}'::jsonb, usuario_id uuid references usuarios(id) on delete set null,
  creado_en timestamptz not null default now()
);
create index if not exists idx_actividad_referencia on actividad(tipo,referencia_id,creado_en desc);
create index if not exists idx_actividad_usuario on actividad(usuario_id,creado_en desc);
alter table actividad enable row level security; revoke all on actividad from public; grant select on actividad to public;

create or replace function registrar_actividad_cambio()
returns trigger language plpgsql security definer set search_path=public as $$
declare n jsonb; o jsonb; r jsonb; acc text; titulo text; actor uuid; campos jsonb;
begin
  n:=case when TG_OP='DELETE' then null else to_jsonb(NEW) end;
  o:=case when TG_OP='INSERT' then null else to_jsonb(OLD) end; r:=coalesce(n,o);
  acc:=case TG_OP when 'INSERT' then 'creado' when 'UPDATE' then 'actualizado' else 'eliminado' end;
  titulo:=coalesce(nullif(r->>'nombre',''),nullif(r->>'nombres','')||' '||nullif(r->>'apellidos',''),nullif(r->>'titulo',''),nullif(r->>'asunto',''),'Registro');
  actor:=nullif(current_setting('app.usuario_id',true),'')::uuid;
  if TG_OP='UPDATE' then
    select coalesce(jsonb_agg(k),'[]'::jsonb) into campos from (select key k from jsonb_each(n) where key not in ('busqueda_vector','creado_en') and n->key is distinct from o->key) c;
  else campos:='[]'::jsonb; end if;
  insert into actividad(tipo,referencia_id,accion,detalle,usuario_id) values(TG_ARGV[0],(r->>'id')::uuid,acc,jsonb_build_object('titulo',btrim(titulo),'campos',campos),actor);
  if TG_OP='DELETE' then return OLD; else return NEW; end if;
end; $$;

create trigger auditoria_personas after insert or update or delete on personas for each row execute function registrar_actividad_cambio('persona');
create trigger auditoria_oportunidades after insert or update or delete on oportunidades for each row execute function registrar_actividad_cambio('postulacion');
create trigger auditoria_alumnos after insert or update or delete on alumnos for each row execute function registrar_actividad_cambio('alumno');
create trigger auditoria_apoderados after insert or update or delete on apoderados for each row execute function registrar_actividad_cambio('apoderado');
create trigger auditoria_tareas after insert or update or delete on tareas for each row execute function registrar_actividad_cambio('tarea');
create trigger auditoria_notas after insert or update or delete on notas for each row execute function registrar_actividad_cambio('nota');
create trigger auditoria_eventos after insert or update or delete on eventos for each row execute function registrar_actividad_cambio('evento');
create trigger auditoria_correos after insert or update or delete on correos for each row execute function registrar_actividad_cambio('correo');

create or replace function timeline(p_tipo text,p_id uuid,p_limite integer default 50)
returns table(id uuid,tipo text,referencia_id uuid,accion text,detalle jsonb,usuario_id uuid,autor text,creado_en timestamptz)
language sql security definer set search_path=public as $$
  select a.id,a.tipo,a.referencia_id,a.accion,a.detalle,a.usuario_id,coalesce(u.nombre,'Sistema'),a.creado_en
  from actividad a left join usuarios u on u.id=a.usuario_id where a.tipo=p_tipo and a.referencia_id=p_id
  order by a.creado_en desc limit least(greatest(coalesce(p_limite,50),1),100);
$$;

grant execute on function buscar_espacio(text) to public;
grant execute on function alternar_favorito(uuid,text,uuid) to public;
grant execute on function listar_favoritos(uuid) to public;
grant execute on function listar_vistas(uuid,text) to public;
grant execute on function crear_vista(uuid,text,text,text,jsonb,jsonb) to public;
grant execute on function actualizar_vista(uuid,uuid,text,text,jsonb,jsonb) to public;
grant execute on function eliminar_vista(uuid,uuid) to public;
grant execute on function timeline(text,uuid,integer) to public;
