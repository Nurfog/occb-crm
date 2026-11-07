-- 015 — Corrige rotar_sesion: el reuso debe revocar de verdad.
-- El RAISE tras el DELETE abortaba la función y PostgreSQL revertía el borrado.
-- Ahora el reuso borra todas las sesiones del usuario y devuelve fila NULL
-- (la API lo interpreta como "Sesión revocada por seguridad.", 401).

create or replace function rotar_sesion(p_sesion_id uuid, p_renovacion_hash text, p_nuevo_hash text)
returns sesiones
language plpgsql security definer set search_path = public
as $$
declare v sesiones%rowtype;
begin
  select * into v from sesiones where id = p_sesion_id;
  if not found then raise exception 'Sesión no encontrada.' using errcode = 'P0002'; end if;
  if v.expira_en < now() then
    delete from sesiones where id = p_sesion_id;
    raise exception 'Sesión expirada.' using errcode = 'P0002';
  end if;
  if v.renovacion_hash <> p_renovacion_hash then
    -- Reuso detectado: se revoca todo lo del usuario. Sin RAISE para que
    -- el borrado persista; se devuelve NULL y la API responde 401.
    delete from sesiones where usuario_id = v.usuario_id;
    return null;
  end if;
  update sesiones set renovacion_hash = p_nuevo_hash where id = p_sesion_id returning * into v;
  return v;
end;
$$;
