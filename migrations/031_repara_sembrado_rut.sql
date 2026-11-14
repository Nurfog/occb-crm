-- 031 — Repara siembra escolar en BD limpias (post-028, aditivo).
-- 028 cambió UNIQUE global de RUT por UNIQUE parcial WHERE activo; el
-- `ON CONFLICT (rut)` de sembrar_escolar_demo() quedó sin árbitro válido
-- (42P10 en arranque limpio). Se reemplaza por ON CONFLICT sin árbitro.
-- No se edita 011 (migración aplicada): se re-define hacia adelante.

create or replace function sembrar_escolar_demo()
returns void
language plpgsql security definer set search_path = public
as $$
declare
  v_a1 uuid; v_a2 uuid; v_a3 uuid; v_p1 uuid; v_p2 uuid; v_p3 uuid;
begin
  if (select count(*) from alumnos) > 0 then return; end if;
  insert into alumnos (rut, nombres, apellidos, curso, estado) values
    ('12345678-5', 'Martín', 'González', '8° Básico A', 'postulante'),
    ('11111111-1', 'Sofía', 'Fernández', '7° Básico B', 'matriculado'),
    ('6123456-K', 'Diego', 'Muñoz', '1° Medio A', 'postulante')
  on conflict do nothing;
  insert into apoderados (rut, nombres, apellidos, telefono, region_codigo, comuna_codigo) values
    ('1-9', 'Carmen', 'González', '+56 9 5555 0101', 13, '13101'),
    ('23456789-6', 'Pedro', 'Fernández', '+56 9 5555 0102', 5, '05101'),
    ('76543210-3', 'Ana', 'Muñoz', '+56 9 5555 0103', 8, '08101')
  on conflict do nothing;
  select id into v_a1 from alumnos where rut = '12345678-5';
  select id into v_a2 from alumnos where rut = '11111111-1';
  select id into v_a3 from alumnos where rut = '6123456-K';
  select id into v_p1 from apoderados where rut = '1-9';
  select id into v_p2 from apoderados where rut = '23456789-6';
  select id into v_p3 from apoderados where rut = '76543210-3';
  insert into alumno_apoderado (alumno_id, apoderado_id, vinculo, retiro_autorizado) values
    (v_a1, v_p1, 'Madre', true),
    (v_a2, v_p2, 'Padre', true),
    (v_a3, v_p3, 'Madre', false)
  on conflict (alumno_id, apoderado_id) do nothing;
end;
$$;
