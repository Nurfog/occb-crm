-- 016 — Calendario y correos vinculados a la vida escolar.
-- Sin empresas (track escolar): los vínculos son alumno, apoderado y postulación.
-- CRUD seguro + eventos del mes + marcar correo leído.

create table if not exists eventos (
  id uuid primary key default gen_random_uuid(),
  titulo text not null check (char_length(btrim(titulo)) between 1 and 200),
  inicio timestamptz not null,
  fin timestamptz not null check (fin >= inicio),
  lugar text not null default '',
  descripcion text not null default '',
  alumno_id uuid references alumnos(id) on delete cascade,
  apoderado_id uuid references apoderados(id) on delete cascade,
  oportunidad_id uuid references oportunidades(id) on delete cascade,
  creado_en timestamptz not null default now()
);
comment on table eventos is 'Agenda escolar (reuniones, evaluaciones, entrevistas)';

create table if not exists correos (
  id uuid primary key default gen_random_uuid(),
  asunto text not null default '',
  cuerpo text not null check (char_length(btrim(cuerpo)) > 0),
  remitente text not null check (remitente ~ '^[^@\\s]+@[^@\\s]+\\.[^@\\s]+$'),
  alumno_id uuid references alumnos(id) on delete cascade,
  apoderado_id uuid references apoderados(id) on delete cascade,
  oportunidad_id uuid references oportunidades(id) on delete cascade,
  leido boolean not null default false,
  recibido_en timestamptz not null default now(),
  creado_en timestamptz not null default now()
);
comment on table correos is 'Bandeja vinculada a alumnos, apoderados y postulaciones';

create index if not exists idx_eventos_inicio on eventos (inicio);
create index if not exists idx_eventos_fin on eventos (fin);
create index if not exists idx_eventos_alumno on eventos (alumno_id);
create index if not exists idx_correos_recibido on correos (recibido_en desc);
create index if not exists idx_correos_leido on correos (leido);
create index if not exists idx_correos_alumno on correos (alumno_id);

alter table eventos enable row level security;
alter table correos enable row level security;
revoke all on eventos, correos from public;
grant select on eventos, correos to public;

create or replace function listar_eventos(
  p_busqueda text default '',
  p_limite integer default 50,
  p_desplazamiento integer default 0
)
returns setof eventos
language plpgsql security definer set search_path = public
as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 50), 1), 100);
  v_desp integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_busqueda text := btrim(coalesce(p_busqueda, ''));
begin
  return query select * from eventos e
  where v_busqueda = '' or e.titulo ilike '%' || v_busqueda || '%'
    or e.lugar ilike '%' || v_busqueda || '%' or e.descripcion ilike '%' || v_busqueda || '%'
  order by e.inicio, e.creado_en desc
  limit v_limite offset v_desp;
end;
$$;

create or replace function obtener_evento(p_id uuid)
returns eventos
language plpgsql security definer set search_path = public
as $$
declare v eventos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  select * into v from eventos where id = p_id;
  if not found then raise exception 'Evento no encontrado.' using errcode = 'P0002'; end if;
  return v;
end;
$$;

create or replace function crear_evento(
  p_titulo text, p_inicio timestamptz, p_fin timestamptz,
  p_lugar text default '', p_descripcion text default '',
  p_alumno_id uuid default null, p_apoderado_id uuid default null, p_oportunidad_id uuid default null
)
returns eventos
language plpgsql security definer set search_path = public
as $$
declare
  v_titulo text := btrim(coalesce(p_titulo, ''));
  v_evento eventos%rowtype;
begin
  if v_titulo = '' then raise exception 'El título es obligatorio.' using errcode = 'P0001'; end if;
  if char_length(v_titulo) > 200 then raise exception 'El título no puede superar 200 caracteres.' using errcode = 'P0001'; end if;
  if p_inicio is null or p_fin is null then raise exception 'El inicio y el fin son obligatorios.' using errcode = 'P0001'; end if;
  if p_fin < p_inicio then raise exception 'El fin no puede ser anterior al inicio.' using errcode = 'P0001'; end if;
  if p_alumno_id is not null and not exists (select 1 from alumnos where id = p_alumno_id) then
    raise exception 'El alumno indicado no existe.' using errcode = 'P0001';
  end if;
  if p_apoderado_id is not null and not exists (select 1 from apoderados where id = p_apoderado_id) then
    raise exception 'El apoderado indicado no existe.' using errcode = 'P0001';
  end if;
  if p_oportunidad_id is not null and not exists (select 1 from oportunidades where id = p_oportunidad_id) then
    raise exception 'La postulación indicada no existe.' using errcode = 'P0001';
  end if;
  insert into eventos (titulo, inicio, fin, lugar, descripcion, alumno_id, apoderado_id, oportunidad_id)
  values (v_titulo, p_inicio, p_fin, btrim(coalesce(p_lugar, '')), btrim(coalesce(p_descripcion, '')),
          p_alumno_id, p_apoderado_id, p_oportunidad_id)
  returning * into v_evento;
  return v_evento;
end;
$$;

create or replace function actualizar_evento(
  p_id uuid, p_titulo text, p_inicio timestamptz, p_fin timestamptz,
  p_lugar text default '', p_descripcion text default ''
)
returns eventos
language plpgsql security definer set search_path = public
as $$
declare
  v_titulo text := btrim(coalesce(p_titulo, ''));
  v_evento eventos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if v_titulo = '' then raise exception 'El título es obligatorio.' using errcode = 'P0001'; end if;
  if p_inicio is null or p_fin is null then raise exception 'El inicio y el fin son obligatorios.' using errcode = 'P0001'; end if;
  if p_fin < p_inicio then raise exception 'El fin no puede ser anterior al inicio.' using errcode = 'P0001'; end if;
  update eventos set titulo = v_titulo, inicio = p_inicio, fin = p_fin,
    lugar = btrim(coalesce(p_lugar, '')), descripcion = btrim(coalesce(p_descripcion, ''))
  where id = p_id returning * into v_evento;
  if not found then raise exception 'Evento no encontrado.' using errcode = 'P0002'; end if;
  return v_evento;
end;
$$;

create or replace function eliminar_evento(p_id uuid)
returns boolean
language plpgsql security definer set search_path = public
as $$
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  delete from eventos where id = p_id;
  if not found then raise exception 'Evento no encontrado.' using errcode = 'P0002'; end if;
  return true;
end;
$$;

create or replace function listar_eventos_mes(p_ano integer, p_mes integer)
returns setof eventos
language plpgsql security definer set search_path = public
as $$
declare
  v_inicio date;
  v_fin date;
begin
  if coalesce(p_ano, 0) < 2000 or coalesce(p_ano, 0) > 2100 then
    raise exception 'Año no válido.' using errcode = 'P0001';
  end if;
  if coalesce(p_mes, 0) not between 1 and 12 then
    raise exception 'Mes no válido (1 a 12).' using errcode = 'P0001';
  end if;
  v_inicio := make_date(p_ano, p_mes, 1);
  v_fin := (v_inicio + interval '1 month')::date;
  return query select * from eventos e
  where e.inicio < v_fin and e.fin >= v_inicio
  order by e.inicio;
end;
$$;

create or replace function listar_correos(
  p_busqueda text default '',
  p_solo_no_leidos boolean default false,
  p_limite integer default 50,
  p_desplazamiento integer default 0
)
returns setof correos
language plpgsql security definer set search_path = public
as $$
declare
  v_limite integer := least(greatest(coalesce(p_limite, 50), 1), 100);
  v_desp integer := greatest(coalesce(p_desplazamiento, 0), 0);
  v_busqueda text := btrim(coalesce(p_busqueda, ''));
begin
  return query select * from correos c
  where (v_busqueda = '' or c.asunto ilike '%' || v_busqueda || '%'
      or c.cuerpo ilike '%' || v_busqueda || '%' or c.remitente ilike '%' || v_busqueda || '%')
    and (not coalesce(p_solo_no_leidos, false) or not c.leido)
  order by c.recibido_en desc
  limit v_limite offset v_desp;
end;
$$;

create or replace function obtener_correo(p_id uuid)
returns correos
language plpgsql security definer set search_path = public
as $$
declare v correos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  select * into v from correos where id = p_id;
  if not found then raise exception 'Correo no encontrado.' using errcode = 'P0002'; end if;
  return v;
end;
$$;

create or replace function crear_correo(
  p_asunto text default '', p_cuerpo text default '', p_remitente text default '',
  p_alumno_id uuid default null, p_apoderado_id uuid default null, p_oportunidad_id uuid default null
)
returns correos
language plpgsql security definer set search_path = public
as $$
declare
  v_remitente text := lower(btrim(coalesce(p_remitente, '')));
  v_correo correos%rowtype;
begin
  if btrim(coalesce(p_cuerpo, '')) = '' then raise exception 'El contenido es obligatorio.' using errcode = 'P0001'; end if;
  if v_remitente !~ '^[^@\\s]+@[^@\\s]+\\.[^@\\s]+$' then
    raise exception 'El remitente no es un correo válido.' using errcode = 'P0001';
  end if;
  if p_alumno_id is not null and not exists (select 1 from alumnos where id = p_alumno_id) then
    raise exception 'El alumno indicado no existe.' using errcode = 'P0001';
  end if;
  if p_apoderado_id is not null and not exists (select 1 from apoderados where id = p_apoderado_id) then
    raise exception 'El apoderado indicado no existe.' using errcode = 'P0001';
  end if;
  if p_oportunidad_id is not null and not exists (select 1 from oportunidades where id = p_oportunidad_id) then
    raise exception 'La postulación indicada no existe.' using errcode = 'P0001';
  end if;
  insert into correos (asunto, cuerpo, remitente, alumno_id, apoderado_id, oportunidad_id)
  values (btrim(coalesce(p_asunto, '')), btrim(p_cuerpo), v_remitente,
          p_alumno_id, p_apoderado_id, p_oportunidad_id)
  returning * into v_correo;
  return v_correo;
end;
$$;

create or replace function marcar_correo_leido(p_id uuid, p_leido boolean default true)
returns correos
language plpgsql security definer set search_path = public
as $$
declare v correos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update correos set leido = coalesce(p_leido, true) where id = p_id returning * into v;
  if not found then raise exception 'Correo no encontrado.' using errcode = 'P0002'; end if;
  return v;
end;
$$;

create or replace function eliminar_correo(p_id uuid)
returns boolean
language plpgsql security definer set search_path = public
as $$
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  delete from correos where id = p_id;
  if not found then raise exception 'Correo no encontrado.' using errcode = 'P0002'; end if;
  return true;
end;
$$;

grant execute on function listar_eventos(text, integer, integer) to public;
grant execute on function obtener_evento(uuid) to public;
grant execute on function crear_evento(text, timestamptz, timestamptz, text, text, uuid, uuid, uuid) to public;
grant execute on function actualizar_evento(uuid, text, timestamptz, timestamptz, text, text) to public;
grant execute on function eliminar_evento(uuid) to public;
grant execute on function listar_eventos_mes(integer, integer) to public;
grant execute on function listar_correos(text, boolean, integer, integer) to public;
grant execute on function obtener_correo(uuid) to public;
grant execute on function crear_correo(text, text, text, uuid, uuid, uuid) to public;
grant execute on function marcar_correo_leido(uuid, boolean) to public;
grant execute on function eliminar_correo(uuid) to public;
