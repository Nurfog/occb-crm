-- 024 — Corrige el cálculo de espera en registrar_resultado_webhook.
-- 022 ya está aplicada: `make_interval(secs=)` se interpretaba como columna.
-- Se usa argumento nombrado con `=>` y segundos como double precision.

create or replace function registrar_resultado_webhook(p_id uuid,p_ok boolean,p_error text default '')
returns boolean language plpgsql security definer set search_path=public as $$
begin
  update entregas_webhook set procesando=false,ok=case when p_ok then true when intentos>=3 then false else null end,
    ultimo_error=case when p_ok then '' else left(coalesce(p_error,'Entrega fallida'),500) end,
    proximo_intento_en=case when p_ok or intentos>=3 then proximo_intento_en else now()+make_interval(secs => least(60,5*power(2,greatest(intentos-1,0)))::double precision) end
  where id=p_id and procesando;
  return found;
end; $$;
