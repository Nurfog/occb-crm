-- 041 — vincular_colegio sobre el modelo empresa/RBD (ajuste de 034/040).
-- La PK de colegios_contratados es `rbd`; el holding es agrupación opcional.
-- Se conserva la firma de 034 (holding, rbd, licencia, dominio) para no romper
-- la API, pero el conflicto se resuelve por RBD y la empresa existente se
-- respeta. Nueva variante `vincular_colegio_empresa` para asignar empresa.

create or replace function vincular_colegio(
  p_holding_id uuid, p_rbd text, p_licencia text, p_dominio text default ''
)
returns colegios_contratados
language plpgsql security definer set search_path = public as $$
declare v colegios_contratados%rowtype;
begin
  if not exists (select 1 from colegios where rbd = nullif(btrim(coalesce(p_rbd, '')), '')) then
    raise exception 'RBD desconocido (revise el directorio MINEDUC).' using errcode = 'P0001';
  end if;
  if lower(btrim(coalesce(p_licencia))) not in ('small', 'group', 'holding') then
    raise exception 'Licencia no válida.' using errcode = 'P0001';
  end if;
  if p_holding_id is not null and not exists (select 1 from holdings where id = p_holding_id) then
    raise exception 'Holding no encontrado.' using errcode = 'P0002';
  end if;
  insert into colegios_contratados (rbd, holding_id, licencia, dominio)
  values (btrim(p_rbd), p_holding_id, lower(btrim(p_licencia)), coalesce(p_dominio, ''))
  on conflict (rbd) do update
    set holding_id = excluded.holding_id,
        licencia = excluded.licencia,
        dominio = excluded.dominio
  returning * into v;
  return v;
end; $$;
comment on function vincular_colegio(uuid, text, text, text) is 'Vincula/actualiza un RBD por su clave (holding opcional)';
grant execute on function vincular_colegio(uuid, text, text, text) to public;

create or replace function vincular_colegio_empresa(
  p_empresa_id uuid, p_rbd text, p_licencia text, p_dominio text default ''
)
returns colegios_contratados
language plpgsql security definer set search_path = public as $$
declare v colegios_contratados%rowtype;
begin
  if p_empresa_id is null or not exists (select 1 from empresas where id = p_empresa_id) then
    raise exception 'Empresa no encontrada.' using errcode = 'P0002';
  end if;
  if not exists (select 1 from colegios where rbd = nullif(btrim(coalesce(p_rbd, '')), '')) then
    raise exception 'RBD desconocido (revise el directorio MINEDUC).' using errcode = 'P0001';
  end if;
  if lower(btrim(coalesce(p_licencia))) not in ('small', 'group', 'holding') then
    raise exception 'Licencia no válida.' using errcode = 'P0001';
  end if;
  insert into colegios_contratados (rbd, empresa_id, holding_id, licencia, dominio)
  select btrim(p_rbd), p_empresa_id, e.holding_id, lower(btrim(p_licencia)), coalesce(p_dominio, '')
  from empresas e where e.id = p_empresa_id
  on conflict (rbd) do update
    set empresa_id = excluded.empresa_id,
        holding_id = excluded.holding_id,
        licencia = excluded.licencia,
        dominio = excluded.dominio
  returning * into v;
  return v;
end; $$;
comment on function vincular_colegio_empresa(uuid, text, text, text) is 'Vincula un RBD a una empresa; el holding se deriva de la empresa';
grant execute on function vincular_colegio_empresa(uuid, text, text, text) to public;
