-- 034 — CRUD mínimo de holdings SaaS (M1 UI, aditivo).
-- Sin enforcement global: lecturas filtradas por membresía o rol admin;
-- escrituras exigen admin (la API lo verifica además en Rust).
-- `es_miembro` vive en 029.

create or replace function listar_holdings(p_usuario_id uuid)
returns table (id uuid, nombre text, plan text, creado_en timestamptz, n_colegios bigint, es_miembro boolean)
language plpgsql security definer set search_path = public as $$
declare v_admin boolean;
begin
  if p_usuario_id is null then raise exception 'Sesión inválida.' using errcode = 'P0001'; end if;
  select (u.rol = 'admin') into v_admin from usuarios u where u.id = p_usuario_id;
  return query select h.id, h.nombre, h.plan, h.creado_en,
    (select count(*) from colegios_contratados c where c.holding_id = h.id),
    (es_miembro(p_usuario_id, h.id, null) or coalesce(v_admin, false))
    from holdings h
    where coalesce(v_admin, false) or es_miembro(p_usuario_id, h.id, null)
    order by h.creado_en desc;
end; $$;
comment on function listar_holdings(uuid) is 'Holdings visibles: todos si admin, solo miembros si no';
grant execute on function listar_holdings(uuid) to public;

create or replace function crear_holding(p_usuario_id uuid, p_nombre text, p_plan text)
returns holdings
language plpgsql security definer set search_path = public as $$
declare v holdings%rowtype;
begin
  if p_usuario_id is null then raise exception 'Sesión inválida.' using errcode = 'P0001'; end if;
  if btrim(coalesce(p_nombre, '')) = '' then raise exception 'El nombre del holding es obligatorio.' using errcode = 'P0001'; end if;
  if lower(btrim(coalesce(p_plan, ''))) not in ('small', 'group', 'holding') then raise exception 'Plan no válido.' using errcode = 'P0001'; end if;
  insert into holdings (nombre, plan) values (btrim(p_nombre), lower(btrim(p_plan)))
  returning * into v;
  insert into membresias_usuario (usuario_id, holding_id, colegio_rbd)
  values (p_usuario_id, v.id, null);
  return v;
end; $$;
comment on function crear_holding(uuid, text, text) is 'Crea holding y da de alta al creador como miembro global';
grant execute on function crear_holding(uuid, text, text) to public;

create or replace function listar_colegios_contratados(p_usuario_id uuid, p_holding_id uuid)
returns table (holding_id uuid, rbd text, nombre text, licencia text, dominio text, creado_en timestamptz)
language plpgsql security definer set search_path = public as $$
declare v_admin boolean;
begin
  if p_usuario_id is null then raise exception 'Sesión inválida.' using errcode = 'P0001'; end if;
  if p_holding_id is null then raise exception 'Falta el holding.' using errcode = 'P0001'; end if;
  select (u.rol = 'admin') into v_admin from usuarios u where u.id = p_usuario_id;
  if not (coalesce(v_admin, false) or es_miembro(p_usuario_id, p_holding_id, null)) then
    raise exception 'Sin autorización para este holding.' using errcode = 'P0002';
  end if;
  return query select c.holding_id, c.rbd, g.nombre, c.licencia, c.dominio, c.creado_en
    from colegios_contratados c join colegios g on g.rbd = c.rbd
    where c.holding_id = p_holding_id order by g.nombre;
end; $$;
comment on function listar_colegios_contratados(uuid, uuid) is 'Colegios contratados de un holding (miembros o admin)';
grant execute on function listar_colegios_contratados(uuid, uuid) to public;

create or replace function vincular_colegio(p_holding_id uuid, p_rbd text, p_licencia text, p_dominio text default '')
returns colegios_contratados
language plpgsql security definer set search_path = public as $$
declare v colegios_contratados%rowtype;
begin
  if p_holding_id is null then raise exception 'Falta el holding.' using errcode = 'P0001'; end if;
  if not exists (select 1 from holdings where id = p_holding_id) then raise exception 'Holding no encontrado.' using errcode = 'P0002'; end if;
  if not exists (select 1 from colegios where rbd = nullif(btrim(coalesce(p_rbd, '')), '')) then raise exception 'RBD desconocido (revise el directorio MINEDUC).' using errcode = 'P0001'; end if;
  if lower(btrim(coalesce(p_licencia))) not in ('small', 'group', 'holding') then raise exception 'Licencia no válida.' using errcode = 'P0001'; end if;
  insert into colegios_contratados (holding_id, rbd, licencia, dominio)
  values (p_holding_id, btrim(p_rbd), lower(btrim(p_licencia)), coalesce(p_dominio, ''))
  on conflict (holding_id, rbd) do update set licencia = excluded.licencia, dominio = excluded.dominio
  returning * into v;
  return v;
end; $$;
comment on function vincular_colegio(uuid, text, text, text) is 'Vincula un RBD del directorio a un holding (upsert de licencia)';
grant execute on function vincular_colegio(uuid, text, text, text) to public;

create or replace function eliminar_holding(p_holding_id uuid)
returns boolean
language plpgsql security definer set search_path = public as $$
begin
  if p_holding_id is null then raise exception 'Falta el holding.' using errcode = 'P0001'; end if;
  delete from holdings where id = p_holding_id;
  if not found then raise exception 'Holding no encontrado.' using errcode = 'P0002'; end if;
  return true;
end; $$;
comment on function eliminar_holding(uuid) is 'Elimina un holding (cascada a contratados y membresías)';
grant execute on function eliminar_holding(uuid) to public;
