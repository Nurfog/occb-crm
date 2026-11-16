-- 032 — Credencial de sesión vía función (higiene, aditivo).
-- `renovar` leía `usuarios` con SQL directo, contra el invariante
-- "ningún SQL directo desde la app". Nueva función SECURITY DEFINER;
-- `renovar` la usa en vez del SELECT. Sin cambios de conducta.

create or replace function obtener_credencial_sesion(p_usuario_id uuid)
returns table(nombre text, rol text)
language sql stable security definer
set search_path = public as $$
  select u.nombre, u.rol from usuarios u where u.id = p_usuario_id;
$$;
comment on function obtener_credencial_sesion(uuid) is 'Nombre y rol para renovar sesión (evita SQL directo en auth.rs:renovar)';
grant execute on function obtener_credencial_sesion(uuid) to public;
