-- 040 — Colegio contratado identificado por RBD, perteneciente a una empresa.
-- 029 definió PK (holding_id, rbd) y holding_id NOT NULL: eso ataba todo
-- colegio a una corporación y rompía el caso "colegio independiente".
-- La unidad operativa es la EMPRESA; el holding queda como agrupación opcional.
-- Un RBD se contrata una sola vez en la instancia.

alter table colegios_contratados drop constraint if exists colegios_contratados_pkey;
alter table colegios_contratados alter column holding_id drop not null;
alter table colegios_contratados add primary key (rbd);
create index if not exists idx_contratados_holding on colegios_contratados (holding_id)
  where holding_id is not null;
comment on column colegios_contratados.empresa_id is 'Empresa que contrata el colegio (unidad fiscal y operativa); NULL = pendiente de asignar';
comment on column colegios_contratados.holding_id is 'Corporación del colegio, derivada de la empresa; NULL si la empresa es independiente';
