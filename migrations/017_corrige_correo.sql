-- 017 — Corrige el regex de remitente de 016.
-- La 016 usó doble barra (`\\s`), que en POSIX excluye la letra "s"
-- y rechaza correos válidos. Se corrige a una sola barra como en 014.

alter table correos drop constraint if exists correos_remitente_check;
alter table correos
  add constraint correos_remitente_check
  check (remitente ~ '^[^@\s]+@[^@\s]+\.[^@\s]+$');

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
  if v_remitente !~ '^[^@\s]+@[^@\s]+\.[^@\s]+$' then
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
