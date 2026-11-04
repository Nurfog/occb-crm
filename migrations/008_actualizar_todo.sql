-- 008 — Actualización de personas y oportunidades (ciclo completo del registro).
-- La API ya tenía actualizar_empresa; aquí se agregan las dos que faltaban.

create or replace function actualizar_persona(
  p_id uuid,
  p_nombre text,
  p_apellido text,
  p_correo text,
  p_telefono text default '',
  p_puesto text default '',
  p_empresa_id uuid default null,
  p_ciudad text default ''
)
returns personas
language plpgsql
security definer
set search_path = public
as $$
declare
  v_nombre text := btrim(coalesce(p_nombre, ''));
  v_apellido text := btrim(coalesce(p_apellido, ''));
  v_correo text := lower(btrim(coalesce(p_correo, '')));
  v_persona personas%rowtype;
begin
  if p_id is null then
    raise exception 'El identificador de la persona es obligatorio.' using errcode = 'P0001';
  end if;
  if v_nombre = '' then
    raise exception 'El nombre es obligatorio.' using errcode = 'P0001';
  end if;
  if v_apellido = '' then
    raise exception 'Los apellidos son obligatorios.' using errcode = 'P0001';
  end if;
  if v_correo !~ '^[^@\s]+@[^@\s]+\.[^@\s]+$' then
    raise exception 'El correo electrónico no tiene un formato válido.' using errcode = 'P0001';
  end if;
  if p_empresa_id is not null and not exists (select 1 from empresas where id = p_empresa_id) then
    raise exception 'La empresa indicada no existe.' using errcode = 'P0001';
  end if;
  update personas
  set nombre = v_nombre,
      apellido = v_apellido,
      correo = v_correo,
      telefono = btrim(coalesce(p_telefono, '')),
      puesto = btrim(coalesce(p_puesto, '')),
      empresa_id = p_empresa_id,
      ciudad = btrim(coalesce(p_ciudad, ''))
  where id = p_id
  returning * into v_persona;
  if not found then
    raise exception 'Persona no encontrada.' using errcode = 'P0002';
  end if;
  return v_persona;
exception
  when unique_violation then
    raise exception 'Ya existe una persona con ese correo.' using errcode = 'P0003';
end;
$$;

create or replace function actualizar_oportunidad(
  p_id uuid,
  p_nombre text,
  p_importe bigint default 0,
  p_etapa text default 'nueva',
  p_empresa_id uuid default null
)
returns oportunidades
language plpgsql
security definer
set search_path = public
as $$
declare
  v_nombre text := btrim(coalesce(p_nombre, ''));
  v_etapa text := lower(btrim(coalesce(p_etapa, 'nueva')));
  v_oportunidad oportunidades%rowtype;
begin
  if p_id is null then
    raise exception 'El identificador de la oportunidad es obligatorio.' using errcode = 'P0001';
  end if;
  if v_nombre = '' then
    raise exception 'El nombre de la oportunidad es obligatorio.' using errcode = 'P0001';
  end if;
  if coalesce(p_importe, 0) < 0 then
    raise exception 'El importe no puede ser negativo.' using errcode = 'P0001';
  end if;
  if v_etapa not in ('nueva', 'contacto', 'reunion', 'propuesta', 'cliente') then
    raise exception 'Etapa no válida. Use: nueva, contacto, reunion, propuesta o cliente.' using errcode = 'P0001';
  end if;
  if p_empresa_id is not null and not exists (select 1 from empresas where id = p_empresa_id) then
    raise exception 'La empresa indicada no existe.' using errcode = 'P0001';
  end if;
  update oportunidades
  set nombre = v_nombre,
      importe = coalesce(p_importe, 0),
      etapa = v_etapa,
      empresa_id = p_empresa_id
  where id = p_id
  returning * into v_oportunidad;
  if not found then
    raise exception 'Oportunidad no encontrada.' using errcode = 'P0002';
  end if;
  return v_oportunidad;
end;
$$;

grant execute on function actualizar_persona(uuid, text, text, text, text, text, uuid, text) to public;
grant execute on function actualizar_oportunidad(uuid, text, bigint, text, uuid) to public;
