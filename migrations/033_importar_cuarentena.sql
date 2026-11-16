-- 033 — CSV en cuarentena para alumnos (M2, opt-in, aditivo).
-- `modo=cuarentena` no crea alumnos: crea postulaciones
-- (`etapa='contacto'`, `origen='csv'`) con `prospecto_rut` normalizado,
-- consentimiento y TTL de 90 días, más fila en `importaciones` (job).
-- El modo directo (`importar_alumnos`) queda intacto.

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
begin
  if p_usuario_id is null then raise exception 'El usuario es obligatorio.' using errcode = 'P0001'; end if;
  if jsonb_typeof(p_filas) <> 'array' or jsonb_array_length(p_filas) > 1000 then raise exception 'La carga debe ser una lista de hasta 1000 alumnos.' using errcode = 'P0001'; end if;
  perform set_config('app.usuario_id', p_usuario_id::text, true);
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
      insert into oportunidades (nombre, importe, etapa, colegio_rbd, prospecto_rut, origen, consentimiento, ttl)
      values (v_nombre, 0, 'contacto', v_rbd, v_rut, 'csv',
        jsonb_build_object('canal', 'csv', 'fecha', now()::date, 'prueba', v_job),
        now()::date + 90);
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
  insert into importaciones (job_id, objeto, total, creados, errores)
  values (v_job, 'alumnos', n, creados, errores);
  return jsonb_build_object('job_id', v_job, 'total', n, 'creados', creados, 'errores', errores);
end; $$;
comment on function importar_alumnos_cuarentena(uuid, jsonb) is 'CSV a cuarentena: postulaciones contacto/origen csv con RUT validado, RBD de directorio, consentimiento y TTL 90d; nunca crea alumnos directos';
grant execute on function importar_alumnos_cuarentena(uuid, jsonb) to public;
