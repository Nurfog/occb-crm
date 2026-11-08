-- 023 — Importación CSV escolar parcial: filas válidas sí, inválidas reportadas.

create or replace function importar_alumnos(p_usuario_id uuid,p_filas jsonb)
returns jsonb language plpgsql security definer set search_path=public as $$
declare fila jsonb; n integer:=0; creados integer:=0; errores jsonb:='[]'::jsonb;
begin
  if p_usuario_id is null then raise exception 'El usuario es obligatorio.' using errcode='P0001'; end if;
  if jsonb_typeof(p_filas)<>'array' or jsonb_array_length(p_filas)>1000 then raise exception 'La carga debe ser una lista de hasta 1000 alumnos.' using errcode='P0001'; end if;
  perform set_config('app.usuario_id',p_usuario_id::text,true);
  for fila in select value from jsonb_array_elements(p_filas) loop
    n:=n+1;
    begin
      perform crear_alumno(fila->>'rut',fila->>'nombres',fila->>'apellidos',null,coalesce(fila->>'curso',''),nullif(fila->>'colegio_rbd',''),coalesce(fila->>'estado','postulante'));
      creados:=creados+1;
    exception when others then
      errores:=errores||jsonb_build_array(jsonb_build_object('fila',coalesce((fila->>'fila')::integer,n+1),'mensaje',left(SQLERRM,300)));
    end;
  end loop;
  return jsonb_build_object('total',n,'creados',creados,'errores',errores);
end; $$;

create or replace function importar_apoderados(p_usuario_id uuid,p_filas jsonb)
returns jsonb language plpgsql security definer set search_path=public as $$
declare fila jsonb; n integer:=0; creados integer:=0; errores jsonb:='[]'::jsonb;
begin
  if p_usuario_id is null then raise exception 'El usuario es obligatorio.' using errcode='P0001'; end if;
  if jsonb_typeof(p_filas)<>'array' or jsonb_array_length(p_filas)>1000 then raise exception 'La carga debe ser una lista de hasta 1000 apoderados.' using errcode='P0001'; end if;
  perform set_config('app.usuario_id',p_usuario_id::text,true);
  for fila in select value from jsonb_array_elements(p_filas) loop
    n:=n+1;
    begin
      perform crear_apoderado(fila->>'rut',fila->>'nombres',fila->>'apellidos',coalesce(fila->>'telefono',''),coalesce(fila->>'correo',''),coalesce(fila->>'direccion',''),nullif(fila->>'region_codigo','')::smallint,nullif(fila->>'comuna_codigo',''));
      creados:=creados+1;
    exception when others then
      errores:=errores||jsonb_build_array(jsonb_build_object('fila',coalesce((fila->>'fila')::integer,n+1),'mensaje',left(SQLERRM,300)));
    end;
  end loop;
  return jsonb_build_object('total',n,'creados',creados,'errores',errores);
end; $$;

grant execute on function importar_alumnos(uuid,jsonb) to public;
grant execute on function importar_apoderados(uuid,jsonb) to public;
