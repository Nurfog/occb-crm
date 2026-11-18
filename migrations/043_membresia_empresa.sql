-- 043 — Membresía de empresa independiente (ajuste de 029/039).
-- Una empresa puede no tener holding (colegio independiente). La membresía
-- debe poder existir sin holding; el alcance efectivo es la empresa.

alter table membresias_usuario alter column holding_id drop not null;

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
grant execute on function asignar_membresia(uuid,uuid,uuid,text) to public;

comment on column membresias_usuario.holding_id is 'Corporación derivada de la empresa; NULL si la empresa es independiente';
