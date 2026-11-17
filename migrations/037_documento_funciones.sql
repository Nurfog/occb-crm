-- 037 — Alta/edición con documento fiscal por país (multi-país, aditivo).
-- Las funciones actuales (crear_alumno, crear_apoderado, ...) siguen intactas
-- para Chile y sus llamadas. Estas variantes aceptan país/tipo/documento:
--   - CL + RUT: normaliza, valida módulo 11 y guarda rut + documento_numero.
--   - Otro país: valida formato, guarda documento_numero y rut NULL.
-- Duplicado por (país, tipo, documento) => P0003 con mensaje en español.

create or replace function crear_alumno_documento(
  p_pais text, p_tipo text, p_documento text,
  p_nombres text, p_apellidos text,
  p_fecha_nacimiento date default null, p_curso text default '',
  p_colegio_rbd text default null, p_estado text default 'postulante',
  p_grupo_sanguineo text default '', p_alergias text default '',
  p_medicamentos text default '', p_antecedentes_medicos text default '',
  p_contacto_emergencia text default ''
)
returns alumnos
language plpgsql security definer set search_path = public as $$
declare
  v_pais text := upper(btrim(coalesce(nullif(p_pais, ''), 'CL')));
  v_tipo text := lower(btrim(coalesce(nullif(p_tipo, ''), 'rut')));
  v_doc text := upper(btrim(coalesce(p_documento, '')));
  v_rut text;
  v_alumno alumnos%rowtype;
begin
  if not exists (select 1 from paises where iso2 = v_pais) then
    raise exception 'País no soportado: %.', v_pais using errcode = 'P0001';
  end if;
  if v_pais = 'CL' and v_tipo = 'rut' then
    v_rut := rut_normalizado(v_doc);
    if not es_rut_valido(v_rut) then
      raise exception 'El RUT no es válido (revise el dígito verificador).' using errcode = 'P0001';
    end if;
    if v_rut = '00000000-0' then
      raise exception 'El RUT está reservado para anonimización.' using errcode = 'P0001';
    end if;
    if exists (select 1 from alumnos where pais_iso = 'CL' and documento_tipo = 'rut' and documento_numero = v_rut) then
      raise exception 'Ya existe un alumno con ese RUT.' using errcode = 'P0003';
    end if;
    return crear_alumno(v_rut, p_nombres, p_apellidos, p_fecha_nacimiento, p_curso,
      p_colegio_rbd, p_estado, p_grupo_sanguineo, p_alergias, p_medicamentos,
      p_antecedentes_medicos, p_contacto_emergencia);
  end if;
  if not documento_valido(v_pais, v_tipo, v_doc) then
    raise exception 'El documento fiscal no es válido para %.', v_pais using errcode = 'P0001';
  end if;
  if exists (select 1 from alumnos where pais_iso = v_pais and documento_tipo = v_tipo and upper(documento_numero) = v_doc) then
    raise exception 'Ya existe un alumno con ese documento.' using errcode = 'P0003';
  end if;
  if btrim(coalesce(p_nombres, '')) = '' then raise exception 'Los nombres son obligatorios.' using errcode = 'P0001'; end if;
  if btrim(coalesce(p_apellidos, '')) = '' then raise exception 'Los apellidos son obligatorios.' using errcode = 'P0001'; end if;
  if lower(btrim(coalesce(p_estado, 'postulante'))) not in ('postulante', 'matriculado', 'trasladado', 'retirado') then
    raise exception 'Estado no válido.' using errcode = 'P0001';
  end if;
  if p_colegio_rbd is not null and not exists (select 1 from colegios where rbd = p_colegio_rbd) then
    raise exception 'El colegio indicado no existe.' using errcode = 'P0001';
  end if;
  insert into alumnos (rut, pais_iso, documento_tipo, documento_numero, nombres, apellidos,
    fecha_nacimiento, curso, colegio_rbd, estado, grupo_sanguineo, alergias, medicamentos,
    antecedentes_medicos, contacto_emergencia)
  values (null, v_pais, v_tipo, v_doc, btrim(p_nombres), btrim(p_apellidos),
    p_fecha_nacimiento, btrim(coalesce(p_curso, '')), p_colegio_rbd,
    lower(btrim(coalesce(p_estado, 'postulante'))), '', '', '', '', '')
  returning * into v_alumno;
  return v_alumno;
end; $$;
comment on function crear_alumno_documento(text,text,text,text,text,date,text,text,text,text,text,text,text,text) is 'Alta de alumno con documento fiscal por país (CL delega en crear_alumno)';
grant execute on function crear_alumno_documento(text,text,text,text,text,date,text,text,text,text,text,text,text,text) to public;

create or replace function crear_apoderado_documento(
  p_pais text, p_tipo text, p_documento text,
  p_nombres text, p_apellidos text,
  p_telefono text default '', p_correo text default '', p_direccion text default '',
  p_region_codigo smallint default null, p_comuna_codigo text default null,
  p_ocupacion text default ''
)
returns apoderados
language plpgsql security definer set search_path = public as $$
declare
  v_pais text := upper(btrim(coalesce(nullif(p_pais, ''), 'CL')));
  v_tipo text := lower(btrim(coalesce(nullif(p_tipo, ''), 'rut')));
  v_doc text := upper(btrim(coalesce(p_documento, '')));
  v_rut text;
  v_apoderado apoderados%rowtype;
begin
  if not exists (select 1 from paises where iso2 = v_pais) then
    raise exception 'País no soportado: %.', v_pais using errcode = 'P0001';
  end if;
  if v_pais = 'CL' and v_tipo = 'rut' then
    v_rut := rut_normalizado(v_doc);
    if not es_rut_valido(v_rut) then
      raise exception 'El RUT no es válido (revise el dígito verificador).' using errcode = 'P0001';
    end if;
    if exists (select 1 from apoderados where pais_iso = 'CL' and documento_tipo = 'rut' and documento_numero = v_rut) then
      raise exception 'Ya existe un apoderado con ese RUT.' using errcode = 'P0003';
    end if;
    return crear_apoderado(v_rut, p_nombres, p_apellidos, p_telefono, p_correo,
      p_direccion, p_region_codigo, p_comuna_codigo, p_ocupacion);
  end if;
  if not documento_valido(v_pais, v_tipo, v_doc) then
    raise exception 'El documento fiscal no es válido para %.', v_pais using errcode = 'P0001';
  end if;
  if exists (select 1 from apoderados where pais_iso = v_pais and documento_tipo = v_tipo and upper(documento_numero) = v_doc) then
    raise exception 'Ya existe un apoderado con ese documento.' using errcode = 'P0003';
  end if;
  insert into apoderados (rut, pais_iso, documento_tipo, documento_numero, nombres, apellidos,
    telefono, correo, direccion, region_codigo, comuna_codigo, ocupacion)
  values (null, v_pais, v_tipo, v_doc, btrim(p_nombres), btrim(p_apellidos),
    coalesce(p_telefono, ''), coalesce(p_correo, ''), coalesce(p_direccion, ''),
    p_region_codigo, p_comuna_codigo, coalesce(p_ocupacion, ''))
  returning * into v_apoderado;
  return v_apoderado;
end; $$;
comment on function crear_apoderado_documento(text,text,text,text,text,text,text,text,smallint,text,text) is 'Alta de apoderado con documento fiscal por país (CL delega en crear_apoderado)';
grant execute on function crear_apoderado_documento(text,text,text,text,text,text,text,text,smallint,text,text) to public;

create or replace function actualizar_alumno_documento(
  p_id uuid, p_pais text, p_tipo text, p_documento text,
  p_nombres text, p_apellidos text,
  p_fecha_nacimiento date default null, p_curso text default '',
  p_colegio_rbd text default null, p_estado text default 'postulante',
  p_grupo_sanguineo text default '', p_alergias text default '',
  p_medicamentos text default '', p_antecedentes_medicos text default '',
  p_contacto_emergencia text default ''
)
returns alumnos
language plpgsql security definer set search_path = public as $$
declare
  v_pais text := upper(btrim(coalesce(nullif(p_pais, ''), 'CL')));
  v_tipo text := lower(btrim(coalesce(nullif(p_tipo, ''), 'rut')));
  v_doc text := upper(btrim(coalesce(p_documento, '')));
  v_rut text;
  v_alumno alumnos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if not exists (select 1 from paises where iso2 = v_pais) then
    raise exception 'País no soportado: %.', v_pais using errcode = 'P0001';
  end if;
  if v_pais = 'CL' and v_tipo = 'rut' then
    v_rut := rut_normalizado(v_doc);
    if not es_rut_valido(v_rut) then
      raise exception 'El RUT no es válido (revise el dígito verificador).' using errcode = 'P0001';
    end if;
    if exists (select 1 from alumnos where pais_iso = 'CL' and documento_tipo = 'rut' and documento_numero = v_rut and id <> p_id) then
      raise exception 'Ya existe un alumno con ese RUT.' using errcode = 'P0003';
    end if;
    return actualizar_alumno(p_id, v_rut, p_nombres, p_apellidos, p_fecha_nacimiento,
      p_curso, p_colegio_rbd, p_estado, p_grupo_sanguineo, p_alergias, p_medicamentos,
      p_antecedentes_medicos, p_contacto_emergencia);
  end if;
  if not documento_valido(v_pais, v_tipo, v_doc) then
    raise exception 'El documento fiscal no es válido para %.', v_pais using errcode = 'P0001';
  end if;
  if exists (select 1 from alumnos where pais_iso = v_pais and documento_tipo = v_tipo and upper(documento_numero) = v_doc and id <> p_id) then
    raise exception 'Ya existe un alumno con ese documento.' using errcode = 'P0003';
  end if;
  update alumnos set rut = null, pais_iso = v_pais, documento_tipo = v_tipo,
    documento_numero = v_doc, nombres = btrim(p_nombres), apellidos = btrim(p_apellidos),
    fecha_nacimiento = p_fecha_nacimiento, curso = btrim(coalesce(p_curso, '')),
    colegio_rbd = p_colegio_rbd, estado = lower(btrim(coalesce(p_estado, 'postulante')))
  where id = p_id and activo
  returning * into v_alumno;
  if not found then raise exception 'Alumno no encontrado.' using errcode = 'P0002'; end if;
  return v_alumno;
end; $$;
comment on function actualizar_alumno_documento(uuid,text,text,text,text,text,date,text,text,text,text,text,text,text,text) is 'Edición de alumno con documento fiscal por país (CL delega en actualizar_alumno)';
grant execute on function actualizar_alumno_documento(uuid,text,text,text,text,text,date,text,text,text,text,text,text,text,text) to public;

create or replace function actualizar_apoderado_documento(
  p_id uuid, p_pais text, p_tipo text, p_documento text,
  p_nombres text, p_apellidos text,
  p_telefono text default '', p_correo text default '', p_direccion text default '',
  p_region_codigo smallint default null, p_comuna_codigo text default null,
  p_ocupacion text default ''
)
returns apoderados
language plpgsql security definer set search_path = public as $$
declare
  v_pais text := upper(btrim(coalesce(nullif(p_pais, ''), 'CL')));
  v_tipo text := lower(btrim(coalesce(nullif(p_tipo, ''), 'rut')));
  v_doc text := upper(btrim(coalesce(p_documento, '')));
  v_rut text;
  v_apoderado apoderados%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  if not exists (select 1 from paises where iso2 = v_pais) then
    raise exception 'País no soportado: %.', v_pais using errcode = 'P0001';
  end if;
  if v_pais = 'CL' and v_tipo = 'rut' then
    v_rut := rut_normalizado(v_doc);
    if not es_rut_valido(v_rut) then
      raise exception 'El RUT no es válido (revise el dígito verificador).' using errcode = 'P0001';
    end if;
    if exists (select 1 from apoderados where pais_iso = 'CL' and documento_tipo = 'rut' and documento_numero = v_rut and id <> p_id) then
      raise exception 'Ya existe un apoderado con ese RUT.' using errcode = 'P0003';
    end if;
    return actualizar_apoderado(p_id, v_rut, p_nombres, p_apellidos, p_telefono, p_correo,
      p_direccion, p_region_codigo, p_comuna_codigo, p_ocupacion);
  end if;
  if not documento_valido(v_pais, v_tipo, v_doc) then
    raise exception 'El documento fiscal no es válido para %.', v_pais using errcode = 'P0001';
  end if;
  if exists (select 1 from apoderados where pais_iso = v_pais and documento_tipo = v_tipo and upper(documento_numero) = v_doc and id <> p_id) then
    raise exception 'Ya existe un apoderado con ese documento.' using errcode = 'P0003';
  end if;
  update apoderados set rut = null, pais_iso = v_pais, documento_tipo = v_tipo,
    documento_numero = v_doc, nombres = btrim(p_nombres), apellidos = btrim(p_apellidos),
    telefono = coalesce(p_telefono, ''), correo = coalesce(p_correo, ''),
    direccion = coalesce(p_direccion, ''), region_codigo = p_region_codigo,
    comuna_codigo = p_comuna_codigo, ocupacion = coalesce(p_ocupacion, '')
  where id = p_id and activo
  returning * into v_apoderado;
  if not found then raise exception 'Apoderado no encontrado.' using errcode = 'P0002'; end if;
  return v_apoderado;
end; $$;
grant execute on function actualizar_apoderado_documento(uuid,text,text,text,text,text,text,text,text,smallint,text,text) to public;

-- RUT nullable para otros países: el CHECK original exige un RUT válido y
-- `es_rut_valido(NULL)` es false. Se relaja a "null o válido".
alter table alumnos drop constraint if exists alumnos_rut_check;
alter table alumnos add constraint alumnos_rut_check check (rut is null or es_rut_valido(rut));
alter table apoderados drop constraint if exists apoderados_rut_check;
alter table apoderados add constraint apoderados_rut_check check (rut is null or es_rut_valido(rut));

-- Sincroniza el documento genérico desde el RUT chileno en cualquier vía de
-- escritura (crear_alumno, importar, matrícula, semilla). Para no-CL respeta
-- el documento ya informado y no toca el país.
create or replace function sincronizar_documento_alumno()
returns trigger language plpgsql as $$
begin
  if coalesce(NEW.pais_iso, '') = '' then NEW.pais_iso := 'CL'; end if;
  if coalesce(NEW.documento_tipo, '') = '' then NEW.documento_tipo := 'rut'; end if;
  if coalesce(NEW.documento_numero, '') = '' and coalesce(NEW.rut, '') <> '' then
    NEW.documento_numero := NEW.rut;
  end if;
  return NEW;
end; $$;

create or replace function sincronizar_documento_apoderado()
returns trigger language plpgsql as $$
begin
  if coalesce(NEW.pais_iso, '') = '' then NEW.pais_iso := 'CL'; end if;
  if coalesce(NEW.documento_tipo, '') = '' then NEW.documento_tipo := 'rut'; end if;
  if coalesce(NEW.documento_numero, '') = '' and coalesce(NEW.rut, '') <> '' then
    NEW.documento_numero := NEW.rut;
  end if;
  return NEW;
end; $$;

drop trigger if exists trg_documento_alumno on alumnos;
create trigger trg_documento_alumno
  before insert or update on alumnos
  for each row execute function sincronizar_documento_alumno();

drop trigger if exists trg_documento_apoderado on apoderados;
create trigger trg_documento_apoderado
  before insert or update on apoderados
  for each row execute function sincronizar_documento_apoderado();

-- Rellena filas creadas entre 036 y este trigger.
update alumnos set documento_numero = rut where coalesce(documento_numero, '') = '' and coalesce(rut, '') <> '';
update apoderados set documento_numero = rut where coalesce(documento_numero, '') = '' and coalesce(rut, '') <> '';

-- Anonimización: el RUT y el documento genérico pasan al centinela compartido
-- (excluido del índice único), de modo que el RUT queda liberado.
create or replace function anonimizar_alumno(p_id uuid)
returns alumnos language plpgsql security definer set search_path = public as $$
declare v alumnos%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update alumnos set activo = false, rut = '00000000-0', pais_iso = 'CL', documento_tipo = 'rut',
    documento_numero = '00000000-0', nombres = '—', apellidos = '—',
    fecha_nacimiento = null, curso = '', colegio_rbd = null,
    grupo_sanguineo = '', alergias = '', medicamentos = '',
    antecedentes_medicos = '', contacto_emergencia = ''
  where id = p_id returning * into v;
  if not found then raise exception 'Alumno no encontrado.' using errcode = 'P0002'; end if;
  return v;
end; $$;

create or replace function anonimizar_apoderado(p_id uuid)
returns apoderados language plpgsql security definer set search_path = public as $$
declare v apoderados%rowtype;
begin
  if p_id is null then raise exception 'El identificador es obligatorio.' using errcode = 'P0001'; end if;
  update apoderados set activo = false, rut = '00000000-0', pais_iso = 'CL', documento_tipo = 'rut',
    documento_numero = '00000000-0', nombres = '—', apellidos = '—',
    telefono = '', correo = '', direccion = '', region_codigo = null, comuna_codigo = null,
    ocupacion = ''
  where id = p_id returning * into v;
  if not found then raise exception 'Apoderado no encontrado.' using errcode = 'P0002'; end if;
  return v;
end; $$;
