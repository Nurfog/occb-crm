-- 026 — Ficha escolar ampliada: antecedentes de salud del alumno y ocupación
-- del apoderado. Columnas opcionales; las funciones viejas se retiran para
-- que solo exista una firma vigente por operación.

alter table alumnos
  add column if not exists grupo_sanguineo text not null default '' check (
    grupo_sanguineo in ('', 'O-', 'O+', 'A-', 'A+', 'B-', 'B+', 'AB-', 'AB+')),
  add column if not exists alergias text not null default '' check (char_length(alergias) <= 500),
  add column if not exists medicamentos text not null default '' check (char_length(medicamentos) <= 500),
  add column if not exists antecedentes_medicos text not null default '' check (char_length(antecedentes_medicos) <= 2000),
  add column if not exists contacto_emergencia text not null default '' check (char_length(contacto_emergencia) <= 200);

alter table apoderados
  add column if not exists ocupacion text not null default '' check (char_length(ocupacion) <= 120);

drop function if exists crear_alumno(text, text, text, date, text, text, text);
drop function if exists actualizar_alumno(uuid, text, text, text, date, text, text, text);
drop function if exists crear_apoderado(text, text, text, text, text, text, smallint, text);
drop function if exists actualizar_apoderado(uuid, text, text, text, text, text, text, smallint, text);

create or replace function crear_alumno(
  p_rut text, p_nombres text, p_apellidos text,
  p_fecha_nacimiento date default null, p_curso text default '',
  p_colegio_rbd text default null, p_estado text default 'postulante',
  p_grupo_sanguineo text default '', p_alergias text default '',
  p_medicamentos text default '', p_antecedentes_medicos text default '',
  p_contacto_emergencia text default ''
)
returns alumnos
language plpgsql security definer set search_path = public
as $$
declare
  v_rut text := rut_normalizado(p_rut);
  v_nombres text := btrim(coalesce(p_nombres, ''));
  v_apellidos text := btrim(coalesce(p_apellidos, ''));
  v_estado text := lower(btrim(coalesce(p_estado, 'postulante')));
  v_grupo text := upper(btrim(coalesce(p_grupo_sanguineo, '')));
  v_alumno alumnos%rowtype;
begin
  if not es_rut_valido(v_rut) then
    raise exception 'El RUT no es válido (revise el dígito verificador).' using errcode = 'P0001';
  end if;
  if v_nombres = '' then raise exception 'Los nombres son obligatorios.' using errcode = 'P0001'; end if;
  if v_apellidos = '' then raise exception 'Los apellidos son obligatorios.' using errcode = 'P0001'; end if;
  if v_estado not in ('postulante', 'matriculado', 'trasladado', 'retirado') then
    raise exception 'Estado no válido.' using errcode = 'P0001';
  end if;
  if v_grupo not in ('', 'O-', 'O+', 'A-', 'A+', 'B-', 'B+', 'AB-', 'AB+') then
    raise exception 'Grupo sanguíneo no válido.' using errcode = 'P0001';
  end if;
  if p_colegio_rbd is not null and not exists (select 1 from colegios where rbd = p_colegio_rbd) then
    raise exception 'El colegio indicado no existe.' using errcode = 'P0001';
  end if;
  insert into alumnos (rut, nombres, apellidos, fecha_nacimiento, curso, colegio_rbd, estado,
    grupo_sanguineo, alergias, medicamentos, antecedentes_medicos, contacto_emergencia)
  values (v_rut, v_nombres, v_apellidos, p_fecha_nacimiento, btrim(coalesce(p_curso, '')), p_colegio_rbd, v_estado,
    v_grupo, left(btrim(coalesce(p_alergias, '')), 500), left(btrim(coalesce(p_medicamentos, '')), 500),
    left(btrim(coalesce(p_antecedentes_medicos, '')), 2000), left(btrim(coalesce(p_contacto_emergencia, '')), 200))
  returning * into v_alumno;
  return v_alumno;
exception when unique_violation then
  raise exception 'Ya existe un alumno con ese RUT.' using errcode = 'P0003';
end;
$$;

create or replace function actualizar_alumno(
  p_id uuid, p_rut text, p_nombres text, p_apellidos text,
  p_fecha_nacimiento date default null, p_curso text default '',
  p_colegio_rbd text default null, p_estado text default 'postulante',
  p_grupo_sanguineo text default '', p_alergias text default '',
  p_medicamentos text default '', p_antecedentes_medicos text default '',
  p_contacto_emergencia text default ''
)
returns alumnos
language plpgsql security definer set search_path = public
as $$
declare
  v_rut text := rut_normalizado(p_rut);
  v_nombres text := btrim(coalesce(p_nombres, ''));
  v_apellidos text := btrim(coalesce(p_apellidos, ''));
  v_estado text := lower(btrim(coalesce(p_estado, 'postulante')));
  v_grupo text := upper(btrim(coalesce(p_grupo_sanguineo, '')));
  v_alumno alumnos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if not es_rut_valido(v_rut) then
    raise exception 'El RUT no es válido (revise el dígito verificador).' using errcode = 'P0001';
  end if;
  if v_nombres = '' then raise exception 'Los nombres son obligatorios.' using errcode = 'P0001'; end if;
  if v_apellidos = '' then raise exception 'Los apellidos son obligatorios.' using errcode = 'P0001'; end if;
  if v_estado not in ('postulante', 'matriculado', 'trasladado', 'retirado') then
    raise exception 'Estado no válido.' using errcode = 'P0001';
  end if;
  if v_grupo not in ('', 'O-', 'O+', 'A-', 'A+', 'B-', 'B+', 'AB-', 'AB+') then
    raise exception 'Grupo sanguíneo no válido.' using errcode = 'P0001';
  end if;
  if p_colegio_rbd is not null and not exists (select 1 from colegios where rbd = p_colegio_rbd) then
    raise exception 'El colegio indicado no existe.' using errcode = 'P0001';
  end if;
  update alumnos set rut = v_rut, nombres = v_nombres, apellidos = v_apellidos,
    fecha_nacimiento = p_fecha_nacimiento, curso = btrim(coalesce(p_curso, '')),
    colegio_rbd = p_colegio_rbd, estado = v_estado, grupo_sanguineo = v_grupo,
    alergias = left(btrim(coalesce(p_alergias, '')), 500),
    medicamentos = left(btrim(coalesce(p_medicamentos, '')), 500),
    antecedentes_medicos = left(btrim(coalesce(p_antecedentes_medicos, '')), 2000),
    contacto_emergencia = left(btrim(coalesce(p_contacto_emergencia, '')), 200)
  where id = p_id returning * into v_alumno;
  if not found then raise exception 'Alumno no encontrado.' using errcode = 'P0002'; end if;
  return v_alumno;
exception when unique_violation then
  raise exception 'Ya existe un alumno con ese RUT.' using errcode = 'P0003';
end;
$$;

create or replace function crear_apoderado(
  p_rut text, p_nombres text, p_apellidos text,
  p_telefono text default '', p_correo text default '', p_direccion text default '',
  p_region_codigo smallint default null, p_comuna_codigo text default null,
  p_ocupacion text default ''
)
returns apoderados
language plpgsql security definer set search_path = public
as $$
declare
  v_rut text := rut_normalizado(p_rut);
  v_nombres text := btrim(coalesce(p_nombres, ''));
  v_apellidos text := btrim(coalesce(p_apellidos, ''));
  v_correo text := lower(btrim(coalesce(p_correo, '')));
  v_comuna text := nullif(btrim(coalesce(p_comuna_codigo, '')), '');
  v_apoderado apoderados%rowtype;
begin
  if not es_rut_valido(v_rut) then
    raise exception 'El RUT no es válido (revise el dígito verificador).' using errcode = 'P0001';
  end if;
  if v_nombres = '' then raise exception 'Los nombres son obligatorios.' using errcode = 'P0001'; end if;
  if v_apellidos = '' then raise exception 'Los apellidos son obligatorios.' using errcode = 'P0001'; end if;
  if v_correo <> '' and v_correo !~ '^[^@\s]+@[^@\s]+\.[^@\s]+$' then
    raise exception 'El correo electrónico no es válido.' using errcode = 'P0001';
  end if;
  if p_region_codigo is not null and not exists (select 1 from regiones where codigo = p_region_codigo) then
    raise exception 'La región no es válida.' using errcode = 'P0001';
  end if;
  if v_comuna is not null then
    if p_region_codigo is null then raise exception 'Elija la región de la comuna.' using errcode = 'P0001'; end if;
    if not exists (select 1 from comunas where codigo = v_comuna and region = p_region_codigo) then
      raise exception 'La comuna no pertenece a la región elegida.' using errcode = 'P0001';
    end if;
  end if;
  insert into apoderados (rut, nombres, apellidos, telefono, correo, direccion, region_codigo, comuna_codigo, ocupacion)
  values (v_rut, v_nombres, v_apellidos, btrim(coalesce(p_telefono, '')), v_correo,
          btrim(coalesce(p_direccion, '')), p_region_codigo, v_comuna, left(btrim(coalesce(p_ocupacion, '')), 120))
  returning * into v_apoderado;
  return v_apoderado;
exception when unique_violation then
  raise exception 'Ya existe un apoderado con ese RUT.' using errcode = 'P0003';
end;
$$;

create or replace function actualizar_apoderado(
  p_id uuid, p_rut text, p_nombres text, p_apellidos text,
  p_telefono text default '', p_correo text default '', p_direccion text default '',
  p_region_codigo smallint default null, p_comuna_codigo text default null,
  p_ocupacion text default ''
)
returns apoderados
language plpgsql security definer set search_path = public
as $$
declare
  v_rut text := rut_normalizado(p_rut);
  v_nombres text := btrim(coalesce(p_nombres, ''));
  v_apellidos text := btrim(coalesce(p_apellidos, ''));
  v_correo text := lower(btrim(coalesce(p_correo, '')));
  v_comuna text := nullif(btrim(coalesce(p_comuna_codigo, '')), '');
  v_apoderado apoderados%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if not es_rut_valido(v_rut) then
    raise exception 'El RUT no es válido (revise el dígito verificador).' using errcode = 'P0001';
  end if;
  if v_nombres = '' then raise exception 'Los nombres son obligatorios.' using errcode = 'P0001'; end if;
  if v_apellidos = '' then raise exception 'Los apellidos son obligatorios.' using errcode = 'P0001'; end if;
  if v_correo <> '' and v_correo !~ '^[^@\s]+@[^@\s]+\.[^@\s]+$' then
    raise exception 'El correo electrónico no es válido.' using errcode = 'P0001';
  end if;
  if p_region_codigo is not null and not exists (select 1 from regiones where codigo = p_region_codigo) then
    raise exception 'La región no es válida.' using errcode = 'P0001';
  end if;
  if v_comuna is not null then
    if p_region_codigo is null then raise exception 'Elija la región de la comuna.' using errcode = 'P0001'; end if;
    if not exists (select 1 from comunas where codigo = v_comuna and region = p_region_codigo) then
      raise exception 'La comuna no pertenece a la región elegida.' using errcode = 'P0001';
    end if;
  end if;
  update apoderados set rut = v_rut, nombres = v_nombres, apellidos = v_apellidos,
    telefono = btrim(coalesce(p_telefono, '')), correo = v_correo,
    direccion = btrim(coalesce(p_direccion, '')), region_codigo = p_region_codigo, comuna_codigo = v_comuna,
    ocupacion = left(btrim(coalesce(p_ocupacion, '')), 120)
  where id = p_id returning * into v_apoderado;
  if not found then raise exception 'Apoderado no encontrado.' using errcode = 'P0002'; end if;
  return v_apoderado;
exception when unique_violation then
  raise exception 'Ya existe un apoderado con ese RUT.' using errcode = 'P0003';
end;
$$;

grant execute on function crear_alumno(text, text, text, date, text, text, text, text, text, text, text, text) to public;
grant execute on function actualizar_alumno(uuid, text, text, text, date, text, text, text, text, text, text, text, text) to public;
grant execute on function crear_apoderado(text, text, text, text, text, text, smallint, text, text) to public;
grant execute on function actualizar_apoderado(uuid, text, text, text, text, text, text, smallint, text, text) to public;
