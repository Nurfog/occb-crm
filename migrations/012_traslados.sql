-- 012 — Traslados de alumnos entre colegios con historial.
-- `trasladar_alumno()` valida destino distinto, actualiza el colegio y registra.

create table if not exists traslados (
  id uuid primary key default gen_random_uuid(),
  alumno_id uuid not null references alumnos(id) on delete cascade,
  origen_rbd text references colegios(rbd) on delete set null,
  destino_rbd text not null references colegios(rbd) on delete restrict,
  fecha timestamptz not null default now(),
  motivo text not null default ''
);
comment on table traslados is 'Historial de traslados de alumnos entre colegios';

create index if not exists idx_traslados_alumno on traslados (alumno_id);
create index if not exists idx_traslados_fecha on traslados (fecha desc);

alter table traslados enable row level security;
revoke all on traslados from public;
grant select on traslados to public;

create or replace function trasladar_alumno(
  p_alumno_id uuid,
  p_destino_rbd text,
  p_motivo text default ''
)
returns alumnos
language plpgsql
security definer
set search_path = public
as $$
declare
  v_alumno alumnos%rowtype;
  v_destino text := btrim(coalesce(p_destino_rbd, ''));
begin
  if p_alumno_id is null then
    raise exception 'El alumno es obligatorio.' using errcode = 'P0001';
  end if;
  if v_destino = '' then
    raise exception 'El colegio destino es obligatorio.' using errcode = 'P0001';
  end if;
  select * into v_alumno from alumnos where id = p_alumno_id;
  if not found then
    raise exception 'Alumno no encontrado.' using errcode = 'P0002';
  end if;
  if not exists (select 1 from colegios where rbd = v_destino) then
    raise exception 'El colegio destino no existe.' using errcode = 'P0001';
  end if;
  if v_alumno.colegio_rbd is not distinct from v_destino then
    raise exception 'El colegio destino debe ser distinto al actual.' using errcode = 'P0001';
  end if;
  insert into traslados (alumno_id, origen_rbd, destino_rbd, motivo)
  values (p_alumno_id, v_alumno.colegio_rbd, v_destino, btrim(coalesce(p_motivo, '')));
  update alumnos set colegio_rbd = v_destino where id = p_alumno_id
  returning * into v_alumno;
  return v_alumno;
end;
$$;

create or replace function traslados_de_alumno(p_alumno_id uuid)
returns setof traslados
language sql
security definer
set search_path = public
as $$
  select * from traslados where alumno_id = p_alumno_id order by fecha desc;
$$;

grant execute on function trasladar_alumno(uuid, text, text) to public;
grant execute on function traslados_de_alumno(uuid) to public;
