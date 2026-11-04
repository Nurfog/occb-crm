-- 005 — Permisos mínimos y datos de demostración.
-- La API solo necesita EXECUTE en funciones; las tablas quedan protegidas por RLS.

-- Concede ejecución de funciones al acceso público (en producción, al rol occb_app).
grant execute on function listar_empresas(text, integer, integer) to public;
grant execute on function obtener_empresa(uuid) to public;
grant execute on function crear_empresa(text, text, text, integer, text) to public;
grant execute on function actualizar_empresa(uuid, text, text, text, integer, text) to public;
grant execute on function eliminar_empresa(uuid) to public;
grant execute on function listar_personas(text, integer, integer) to public;
grant execute on function obtener_persona(uuid) to public;
grant execute on function crear_persona(text, text, text, text, text, uuid, text) to public;
grant execute on function eliminar_persona(uuid) to public;
grant execute on function listar_oportunidades(text, integer, integer) to public;
grant execute on function crear_oportunidad(text, bigint, text, uuid) to public;
grant execute on function mover_etapa_oportunidad(uuid, text) to public;
grant execute on function eliminar_oportunidad(uuid) to public;
grant execute on function panel_resumen() to public;

-- Siembra idempotente: solo inserta cuando las tablas están vacías.
create or replace function sembrar_datos_demo()
returns void
language plpgsql
security definer
set search_path = public
as $$
declare
  v_acme uuid; v_globex uuid; v_initech uuid; v_umbrella uuid; v_hooli uuid; v_stark uuid;
  v_ada uuid; v_alan uuid; v_grace uuid; v_linus uuid; v_marie uuid;
begin
  if (select count(*) from empresas) > 0 then
    return;
  end if;

  insert into empresas (nombre, dominio, sector, empleados, ciudad) values
    ('Acme Corp', 'acme.com', 'Software', 240, 'Madrid'),
    ('Globex', 'globex.io', 'Finanzas', 85, 'Barcelona'),
    ('Hooli', 'hooli.com', 'Nube', 1200, 'Sevilla'),
    ('Initech', 'initech.dev', 'Herramientas', 42, 'Valencia'),
    ('Stark Industries', 'stark.ind', 'Industria', 3000, 'Zaragoza'),
    ('Umbrella', 'umbrella.salud', 'Salud', 510, 'Bilbao');

  select id into v_acme from empresas where nombre = 'Acme Corp';
  select id into v_globex from empresas where nombre = 'Globex';
  select id into v_initech from empresas where nombre = 'Initech';
  select id into v_umbrella from empresas where nombre = 'Umbrella';
  select id into v_hooli from empresas where nombre = 'Hooli';
  select id into v_stark from empresas where nombre = 'Stark Industries';

  insert into personas (nombre, apellido, correo, telefono, puesto, empresa_id, ciudad) values
    ('Ada', 'Lovelace', 'ada@acme.com', '+34 555 0101', 'Directora técnica', v_acme, 'Madrid'),
    ('Alan', 'Turing', 'alan@globex.io', '+34 555 0102', 'Responsable de datos', v_globex, 'Barcelona'),
    ('Grace', 'Hopper', 'grace@initech.dev', '+34 555 0103', 'VP Ingeniería', v_initech, 'Valencia'),
    ('Linus', 'Torvalds', 'linus@hooli.com', '+34 555 0104', 'Ingeniero sénior', v_hooli, 'Sevilla'),
    ('Marie', 'Curie', 'marie@umbrella.salud', '+34 555 0105', 'Jefa de investigación', v_umbrella, 'Bilbao');

  select id into v_ada from personas where correo = 'ada@acme.com';
  select id into v_alan from personas where correo = 'alan@globex.io';
  select id into v_grace from personas where correo = 'grace@initech.dev';
  select id into v_linus from personas where correo = 'linus@hooli.com';
  select id into v_marie from personas where correo = 'marie@umbrella.salud';

  insert into oportunidades (nombre, importe, etapa, empresa_id, contacto_id) values
    ('Acme — Plan empresarial', 120000, 'nueva', v_acme, v_ada),
    ('Globex — Despliegue de pagos', 85000, 'cribado', v_globex, v_alan),
    ('Initech — Ampliación de licencias', 24000, 'reunion', v_initech, v_grace),
    ('Umbrella — Suite de investigación', 150000, 'reunion', v_umbrella, v_marie),
    ('Hooli — Migración a la nube', 310000, 'propuesta', v_hooli, v_linus),
    ('Stark — IoT para fábricas', 480000, 'cliente', v_stark, v_ada);
end;
$$;
comment on function sembrar_datos_demo is 'Inserta datos demo solo si las tablas están vacías (idempotente)';
grant execute on function sembrar_datos_demo() to public;
