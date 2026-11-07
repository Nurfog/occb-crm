-- 019 — Normaliza JSON null enviado al guardar vistas.
-- 018 ya está aplicada: no se modifica su checksum.

create or replace function crear_vista(
  p_usuario_id uuid,p_objeto text,p_nombre text,p_tipo text,
  p_filtros jsonb default '{}'::jsonb,p_orden jsonb default '{}'::jsonb
)
returns vistas language plpgsql security definer set search_path=public as $$
declare
  v vistas%rowtype;
  o text:=lower(btrim(coalesce(p_objeto,'')));
  t text:=lower(btrim(coalesce(p_tipo,'')));
  n text:=btrim(coalesce(p_nombre,''));
  f jsonb:=case when jsonb_typeof(p_filtros)='object' then p_filtros else '{}'::jsonb end;
  ord jsonb:=case when jsonb_typeof(p_orden)='object' then p_orden else '{}'::jsonb end;
begin
  if p_usuario_id is null then raise exception 'El usuario es obligatorio.' using errcode='P0001'; end if;
  if o not in ('alumnos','apoderados','personas','oportunidades') then raise exception 'Objeto de vista no válido.' using errcode='P0001'; end if;
  if t not in ('tabla','kanban') or (t='kanban' and o<>'oportunidades') then raise exception 'Tipo de vista no válido para el objeto.' using errcode='P0001'; end if;
  if char_length(n) not between 1 and 80 then raise exception 'El nombre debe tener entre 1 y 80 caracteres.' using errcode='P0001'; end if;
  insert into vistas(usuario_id,objeto,nombre,tipo,filtros,orden) values(p_usuario_id,o,n,t,f,ord) returning * into v;
  return v;
exception when unique_violation then raise exception 'Ya existe una vista con ese nombre.' using errcode='P0003';
end; $$;

create or replace function actualizar_vista(
  p_usuario_id uuid,p_id uuid,p_nombre text,p_tipo text,
  p_filtros jsonb default '{}'::jsonb,p_orden jsonb default '{}'::jsonb
)
returns vistas language plpgsql security definer set search_path=public as $$
declare
  v vistas%rowtype;
  t text:=lower(btrim(coalesce(p_tipo,'')));
  n text:=btrim(coalesce(p_nombre,''));
  f jsonb:=case when jsonb_typeof(p_filtros)='object' then p_filtros else '{}'::jsonb end;
  ord jsonb:=case when jsonb_typeof(p_orden)='object' then p_orden else '{}'::jsonb end;
begin
  if t not in ('tabla','kanban') then raise exception 'Tipo de vista no válido.' using errcode='P0001'; end if;
  update vistas set nombre=n,tipo=t,filtros=f,orden=ord,actualizado_en=now()
    where id=p_id and usuario_id=p_usuario_id and (t<>'kanban' or objeto='oportunidades') returning * into v;
  if not found then raise exception 'Vista no encontrada o tipo incompatible.' using errcode='P0002'; end if;
  return v;
exception when unique_violation then raise exception 'Ya existe una vista con ese nombre.' using errcode='P0003';
end; $$;
