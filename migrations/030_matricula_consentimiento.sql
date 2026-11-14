-- 030 — Matrícula y consentimiento (aditivo, no rompe E2E 122).
-- Prospecto en oportunidad + supresión por hash + matricular_postulacion idempotente.
-- Salud queda en payload de la transacción (no se inserta ingreso hasta Matriculado).

alter table oportunidades
  add column if not exists prospecto_rut text,
  add column if not exists origen text check (origen in ('contacto','csv','traslado')),
  add column if not exists consentimiento jsonb,
  add column if not exists ttl date;
create index if not exists idx_oportunidades_prospecto on oportunidades (prospecto_rut) where activo;

create or replace function matricular_postulacion(
  p_oportunidad_id uuid,
  p_alumno_rut text,
  p_nombres text,
  p_apellidos text,
  p_curso text default '1 básico',
  p_colegio_rbd text default null,
  p_apoderados_rut text[] default '{}',
  p_idempotency_key text default null
)
returns uuid
language plpgsql security definer
set search_path = public as $$
declare
  v_opp oportunidades%rowtype;
  v_rut text;
  v_hash text;
  v_alumno_id uuid;
  v_colegio text;
  v_nombre text;
  v_apellidos text;
begin
  select * into v_opp from oportunidades where id = p_oportunidad_id for update;
  if not found then
    raise exception using errcode = 'P0002', message = 'no_encontrado: postulación inexistente';
  end if;

  v_rut := rut_normalizado(p_alumno_rut);
  if v_rut is null or not es_rut_valido(v_rut) then
    raise exception using errcode = 'P0001', message = 'datos_invalidos: RUT inválido';
  end if;
  if v_rut = '00000000-0' then
    raise exception using errcode = 'P0001', message = 'datos_invalidos: RUT reservado';
  end if;

  v_hash := encode(digest(v_rut, 'sha256'), 'hex');
  if exists (select 1 from supresion where rut_hash = v_hash) then
    raise exception using errcode = 'P0003', message = 'suprimido: RUT en lista de supresión';
  end if;

  -- Idempotencia natural: misma oportunidad + mismo RUT activo devuelve el alumno.
  select id into v_alumno_id from alumnos where rut = v_rut and activo limit 1;
  if v_alumno_id is not null then
    perform mover_etapa_oportunidad(p_oportunidad_id, 'matriculado');
    return v_alumno_id;
  end if;

  v_nombre := nullif(trim(p_nombres), '');
  v_apellidos := nullif(trim(p_apellidos), '');
  if v_nombre is null then
    -- Deriva nombres desde oportunidad.nombre "Nombres Apellidos ..." (primer token a nombres).
    v_nombre := split_part(trim(v_opp.nombre), ' ', 1);
    v_apellidos := nullif(trim(substr(trim(v_opp.nombre), length(split_part(trim(v_opp.nombre), ' ', 1)) + 1)), '');
  end if;
  if v_nombre is null or v_nombre = '' then
    raise exception using errcode = 'P0001', message = 'datos_invalidos: nombre obligatorio';
  end if;

  v_colegio := coalesce(p_colegio_rbd, v_opp.colegio_rbd);
  if v_colegio is not null and not exists (select 1 from colegios where rbd = v_colegio) then
    raise exception using errcode = 'P0001', message = 'datos_invalidos: colegio inexistente';
  end if;

  insert into alumnos (rut, nombres, apellidos, curso, colegio_rbd, estado)
  values (v_rut, v_nombre, coalesce(v_apellidos, ''), coalesce(nullif(trim(p_curso), ''), '1 básico'), v_colegio, 'matriculado')
  returning id into v_alumno_id;

  -- Vincula apoderados existentes por RUT normalizado (los inexistentes se ignoran y se reportan en API).
  if p_apoderados_rut is not null then
    declare r text; vr text; ap_id uuid; begin
      foreach r in array p_apoderados_rut loop
        vr := rut_normalizado(r);
        if vr is not null and es_rut_valido(vr) then
          select id into ap_id from apoderados where rut = vr and activo limit 1;
          if ap_id is not null then
            insert into alumno_apoderado (alumno_id, apoderado_id) values (v_alumno_id, ap_id)
            on conflict do nothing;
          end if;
        end if;
      end loop;
    end;
  end if;

  perform mover_etapa_oportunidad(p_oportunidad_id, 'matriculado');
  return v_alumno_id;
end;
$$;
comment on function matricular_postulacion(uuid, text, text, text, text, text, text[], text) is 'Matricula atómica idempotente desde postulación (oportunidad + RUT) a alumno matriculado';
grant execute on function matricular_postulacion(uuid, text, text, text, text, text, text[], text) to public;
