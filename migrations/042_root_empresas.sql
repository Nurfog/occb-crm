-- 042 — Root de plataforma y API de empresas (RBAC, aditivo).
-- Modelo decidido: existe un root que ve todas las empresas; los demás
-- usuarios ven solo las empresas donde tienen membresía. Este corte crea el
-- rol y las funciones de administración; los filtros de lectura por empresa
-- se aplican en 043 junto con la E2E adaptada.

alter table usuarios add column if not exists es_root boolean not null default false;

-- El primer usuario de la instancia es root (y admin).
update usuarios set es_root = true
where id = (select id from usuarios order by creado_en limit 1)
  and not exists (select 1 from usuarios u2 where u2.es_root);

create or replace function registrar_usuario(p_nombre text, p_correo text, p_clave_hash text)
returns usuarios
language plpgsql security definer set search_path = public
as $$
declare
  v_nombre text := btrim(coalesce(p_nombre, ''));
  v_correo text := lower(btrim(coalesce(p_correo, '')));
  v_usuario usuarios%rowtype;
  v_total integer;
begin
  if v_nombre = '' then raise exception 'El nombre es obligatorio.' using errcode = 'P0001'; end if;
  if v_correo !~ '^[^@\s]+@[^@\s]+\.[^@\s]+$' then
    raise exception 'El correo electrónico no es válido.' using errcode = 'P0001';
  end if;
  if coalesce(p_clave_hash, '') = '' then raise exception 'La clave es obligatoria.' using errcode = 'P0001'; end if;
  select count(*) into v_total from usuarios;
  insert into usuarios (nombre, correo, clave_hash, rol, es_root)
  values (v_nombre, v_correo, p_clave_hash,
          case when v_total = 0 then 'admin' else 'miembro' end,
          v_total = 0)
  returning * into v_usuario;
  return v_usuario;
exception when unique_violation then
  raise exception 'Ya existe un usuario con ese correo.' using errcode = 'P0003';
end;
$$;

create or replace function es_root_usuario(p_usuario_id uuid)
returns boolean language sql stable security definer set search_path = public as $$
  select coalesce((select u.es_root from usuarios u where u.id = p_usuario_id), false);
$$;
comment on function es_root_usuario(uuid) is 'True si el usuario es root de plataforma (ve todas las empresas)';
grant execute on function es_root_usuario(uuid) to public;

create or replace function empresa_del_usuario(p_usuario_id uuid)
returns uuid language sql stable security definer set search_path = public as $$
  select m.empresa_id from membresias_usuario m
  where m.usuario_id = p_usuario_id and m.empresa_id is not null
  order by m.creado_en limit 1;
$$;
comment on function empresa_del_usuario(uuid) is 'Empresa principal del usuario (primera membresía con empresa)';
grant execute on function empresa_del_usuario(uuid) to public;

-- Crear empresa: root, o admin de un holding al que la empresa pertenezca.
create or replace function crear_empresa(
  p_usuario_id uuid, p_holding_id uuid, p_pais text, p_tipo_documento text,
  p_documento text, p_razon_social text, p_moneda text, p_regimen text default ''
)
returns empresas
language plpgsql security definer set search_path = public as $$
declare
  v empresas%rowtype;
  v_pais text := upper(btrim(coalesce(p_pais, 'CL')));
  v_doc text := upper(btrim(coalesce(p_documento, '')));
begin
  if not es_root_usuario(p_usuario_id) then
    raise exception 'Solo el root de plataforma puede crear empresas.' using errcode = 'P0002';
  end if;
  if not exists (select 1 from paises where iso2 = v_pais) then
    raise exception 'País no soportado: %.', v_pais using errcode = 'P0001';
  end if;
  if not exists (select 1 from monedas where iso = upper(btrim(coalesce(p_moneda, 'CLP')))) then
    raise exception 'Moneda no soportada.' using errcode = 'P0001';
  end if;
  if not documento_valido(v_pais, p_tipo_documento, v_doc) then
    raise exception 'El documento fiscal no es válido para %.', v_pais using errcode = 'P0001';
  end if;
  insert into empresas (holding_id, pais_iso, documento_tipo, documento_numero,
    razon_social, moneda_funcional, regimen_tributario)
  values (p_holding_id, v_pais, lower(coalesce(p_tipo_documento, 'rut')), v_doc,
    btrim(coalesce(p_razon_social, '')), upper(btrim(coalesce(p_moneda, 'CLP'))), coalesce(p_regimen, ''))
  returning * into v;
  return v;
exception when unique_violation then
  raise exception 'Ya existe una empresa con ese documento en ese país.' using errcode = 'P0003';
end; $$;
comment on function crear_empresa(uuid,uuid,text,text,text,text,text,text) is 'Crea razón social emisora; solo root';
grant execute on function crear_empresa(uuid,uuid,text,text,text,text,text,text) to public;

create or replace function listar_empresas(p_usuario_id uuid)
returns table (id uuid, holding_id uuid, pais_iso text, documento_tipo text,
  documento_numero text, razon_social text, moneda_funcional text, activa boolean, es_mia boolean)
language plpgsql security definer set search_path = public as $$
declare v_root boolean := es_root_usuario(p_usuario_id);
begin
  return query select e.id, e.holding_id, e.pais_iso, e.documento_tipo, e.documento_numero,
    e.razon_social, e.moneda_funcional, e.activa,
    exists (select 1 from membresias_usuario m where m.usuario_id = p_usuario_id and m.empresa_id = e.id)
  from empresas e
  where v_root or exists (select 1 from membresias_usuario m where m.usuario_id = p_usuario_id and m.empresa_id = e.id)
  order by e.razon_social;
end; $$;
comment on function listar_empresas(uuid) is 'Root ve todas; el resto solo sus empresas';
grant execute on function listar_empresas(uuid) to public;

create or replace function asignar_membresia(
  p_root_id uuid, p_usuario_id uuid, p_empresa_id uuid, p_colegio_rbd text default null
)
returns void
language plpgsql security definer set search_path = public as $$
declare v_holding uuid;
begin
  if not es_root_usuario(p_root_id) then
    raise exception 'Solo el root de plataforma puede asignar membresías.' using errcode = 'P0002';
  end if;
  if not exists (select 1 from usuarios where id = p_usuario_id) then
    raise exception 'Usuario no encontrado.' using errcode = 'P0002';
  end if;
  select holding_id into v_holding from empresas where id = p_empresa_id;
  if not found then
    raise exception 'Empresa no encontrada.' using errcode = 'P0002';
  end if;
  insert into membresias_usuario (usuario_id, holding_id, colegio_rbd, empresa_id)
  values (p_usuario_id, v_holding, p_colegio_rbd, p_empresa_id)
  on conflict do nothing;
end; $$;
comment on function asignar_membresia(uuid,uuid,uuid,text) is 'Root asigna usuario a empresa (holding derivado)';
grant execute on function asignar_membresia(uuid,uuid,uuid,text) to public;

-- Miembros con su empresa principal y bandera root, para la UI de acceso.
drop function if exists listar_miembros();
create or replace function listar_miembros()
returns table (id uuid, nombre text, correo text, rol text, es_root boolean, empresa_id uuid, creado_en timestamptz)
language sql security definer set search_path = public as $$
  select u.id, u.nombre, u.correo, u.rol, u.es_root,
    (select m.empresa_id from membresias_usuario m
      where m.usuario_id = u.id and m.empresa_id is not null
      order by m.creado_en limit 1),
    u.creado_en
  from usuarios u order by u.creado_en;
$$;
grant execute on function listar_miembros() to public;
