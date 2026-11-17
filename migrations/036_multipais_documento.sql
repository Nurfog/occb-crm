-- 036 — Multi-país: catálogo, documento fiscal genérico y geografía flexible.
-- Aditivo y compatible: Chile sigue funcionando igual (defaults 'CL'/'rut').
-- La PK sigue siendo uuid; el documento fiscal es dato de negocio único por
-- (país, tipo, número), no clave primaria.

create table if not exists monedas (
  iso text primary key,
  nombre text not null,
  decimales smallint not null default 2 check (decimales between 0 and 4)
);
insert into monedas (iso, nombre, decimales) values
  ('CLP', 'Peso chileno', 0),
  ('ARS', 'Peso argentino', 2),
  ('BOB', 'Boliviano', 2),
  ('BRL', 'Real brasileño', 2),
  ('COP', 'Peso colombiano', 2),
  ('CRC', 'Colón costarricense', 2),
  ('CUP', 'Peso cubano', 2),
  ('DOP', 'Peso dominicano', 2),
  ('GTQ', 'Quetzal', 2),
  ('HNL', 'Lempira', 2),
  ('MXN', 'Peso mexicano', 2),
  ('NIO', 'Córdoba', 2),
  ('PAB', 'Balboa', 2),
  ('PEN', 'Sol peruano', 2),
  ('PYG', 'Guaraní', 0),
  ('USD', 'Dólar estadounidense', 2),
  ('UYU', 'Peso uruguayo', 2),
  ('VES', 'Bolívar', 2)
on conflict (iso) do nothing;

create table if not exists paises (
  iso2 text primary key check (char_length(iso2) = 2),
  nombre text not null,
  moneda_iso text not null references monedas(iso),
  locale text not null default 'es',
  -- Etiqueta visible del documento fiscal ("RUT", "CUIT", "RFC", "CPF"...).
  documento_etiqueta text not null default 'Documento',
  creado_en timestamptz not null default now()
);
insert into paises (iso2, nombre, moneda_iso, locale, documento_etiqueta) values
  ('CL', 'Chile', 'CLP', 'es-CL', 'RUT'),
  ('AR', 'Argentina', 'ARS', 'es-AR', 'CUIT'),
  ('BO', 'Bolivia', 'BOB', 'es-BO', 'NIT'),
  ('BR', 'Brasil', 'BRL', 'pt-BR', 'CPF'),
  ('CO', 'Colombia', 'COP', 'es-CO', 'NIT'),
  ('CR', 'Costa Rica', 'CRC', 'es-CR', 'Cédula'),
  ('DO', 'República Dominicana', 'DOP', 'es-DO', 'RNC'),
  ('EC', 'Ecuador', 'USD', 'es-EC', 'RUC'),
  ('GT', 'Guatemala', 'GTQ', 'es-GT', 'NIT'),
  ('HN', 'Honduras', 'HNL', 'es-HN', 'RTN'),
  ('MX', 'México', 'MXN', 'es-MX', 'RFC'),
  ('NI', 'Nicaragua', 'NIO', 'es-NI', 'RUC'),
  ('PA', 'Panamá', 'PAB', 'es-PA', 'RUC'),
  ('PE', 'Perú', 'PEN', 'es-PE', 'RUC'),
  ('PY', 'Paraguay', 'PYG', 'es-PY', 'RUC'),
  ('SV', 'El Salvador', 'USD', 'es-SV', 'NIT'),
  ('UY', 'Uruguay', 'UYU', 'es-UY', 'RUT'),
  ('VE', 'Venezuela', 'VES', 'es-VE', 'RIF')
on conflict (iso2) do nothing;

alter table monedas enable row level security;
alter table paises enable row level security;
revoke all on monedas from public;
revoke all on paises from public;
grant select on monedas to public;
grant select on paises to public;

-- Dígito verificador módulo 11 genérico.
-- `p_pesos` se aplica de izquierda a derecha (orden de las fórmulas publicadas).
-- Devuelve `11 - (suma % 11)`; cada país interpreta 10/11.
create or replace function dv_mod11(p_numero text, p_pesos integer[])
returns integer
language plpgsql immutable as $$
declare
  v_suma integer := 0;
  v_i integer;
  v_largo integer := length(p_numero);
  v_peso integer;
begin
  for v_i in 1..v_largo loop
    v_peso := p_pesos[((v_i - 1) % array_length(p_pesos, 1)) + 1];
    v_suma := v_suma + substr(p_numero, v_i, 1)::integer * v_peso;
  end loop;
  return 11 - (v_suma % 11);
end; $$;
comment on function dv_mod11(text, integer[]) is 'Suma ponderada módulo 11; pesos de derecha a izquierda, el llamador interpreta 10/11';

-- Validador por país. Estricto donde la regla es estable (CL, BR, AR);
-- formato + longitud donde el DV depende de casos especiales (PE, MX, CO, etc.).
-- Desconocido: longitud 3..30 (permisivo, se documenta como pendiente).
create or replace function documento_valido(p_pais text, p_tipo text, p_numero text)
returns boolean
language plpgsql immutable as $$
declare
  v text := upper(regexp_replace(coalesce(p_numero, ''), '[\s.\-]', '', 'g'));
  v_dv integer;
begin
  if length(v) < 3 or length(v) > 30 then return false; end if;
  case upper(coalesce(p_pais, 'CL'))
  when 'CL' then
    if upper(coalesce(p_tipo, 'rut')) <> 'RUT' then return length(v) >= 3; end if;
    return es_rut_valido(rut_normalizado(v));
  when 'BR' then
    -- CPF: 11 dígitos, dos DV módulo 11 (pesos 10..2 y 11..2).
    if length(v) <> 11 or v ~ '[^0-9]' then return false; end if;
    v_dv := dv_mod11(substr(v, 1, 9), array[10,9,8,7,6,5,4,3,2]);
    if v_dv >= 10 then v_dv := 0; end if;
    if substr(v, 10, 1)::integer <> v_dv then return false; end if;
    v_dv := dv_mod11(substr(v, 1, 10), array[11,10,9,8,7,6,5,4,3,2]);
    if v_dv >= 10 then v_dv := 0; end if;
    return substr(v, 11, 1)::integer = v_dv;
  when 'AR' then
    -- CUIT: 11 dígitos, peso 5 en unidades; 11->0 y 10->9.
    if length(v) <> 11 or v ~ '[^0-9]' then return false; end if;
    v_dv := dv_mod11(substr(v, 1, 10), array[5,4,3,2,7,6,5,4,3,2]);
    if v_dv = 11 then v_dv := 0; elsif v_dv = 10 then v_dv := 9; end if;
    return substr(v, 11, 1)::integer = v_dv;
  when 'PE' then
    return v ~ '^[0-9]{11}$' and substr(v, 1, 2) in ('10', '15', '17', '20');
  when 'MX' then
    return v ~ '^[A-ZÑ&]{3,4}[0-9]{6}[A-Z0-9]{3}$';
  when 'CO' then
    return v ~ '^[0-9]{9,10}$';
  when 'EC' then
    return v ~ '^[0-9]{10,13}$';
  when 'UY' then
    return v ~ '^[0-9]{11,12}$';
  else
    return true;
  end case;
end; $$;
comment on function documento_valido(text, text, text) is 'Valida documento fiscal por país; CL RUT módulo 11, BR CPF y AR CUIT estrictos, resto formato/longitud';
grant execute on function documento_valido(text, text, text) to public;

-- Alumnos: documento fiscal genérico. `rut` se conserva como alias chileno
-- (compatibilidad de funciones y UI) y pasa a permitir NULL para no-CL.
alter table alumnos
  add column if not exists pais_iso text not null default 'CL' references paises(iso2),
  add column if not exists documento_tipo text not null default 'rut',
  add column if not exists documento_numero text;
update alumnos set documento_numero = rut where documento_numero is null;
alter table alumnos alter column rut drop not null;
drop index if exists uq_alumnos_rut;
create unique index if not exists uq_alumnos_documento
  on alumnos (pais_iso, documento_tipo, upper(documento_numero))
  where documento_numero <> '00000000-0';

-- Apoderados: mismo tratamiento.
alter table apoderados
  add column if not exists pais_iso text not null default 'CL' references paises(iso2),
  add column if not exists documento_tipo text not null default 'rut',
  add column if not exists documento_numero text;
update apoderados set documento_numero = rut where documento_numero is null;
alter table apoderados alter column rut drop not null;
drop index if exists uq_apoderados_rut;
create unique index if not exists uq_apoderados_documento
  on apoderados (pais_iso, documento_tipo, upper(documento_numero))
  where documento_numero <> '00000000-0';

-- Colegios: país y división administrativa genérica; Chile conserva CUT.
alter table colegios
  add column if not exists pais_iso text not null default 'CL' references paises(iso2),
  add column if not exists division1 text,
  add column if not exists division2 text;
alter table colegios alter column region drop not null;
alter table colegios alter column comuna drop not null;

-- Tenant: país del holding y del colegio contratado.
alter table holdings
  add column if not exists pais_iso text not null default 'CL' references paises(iso2);
alter table colegios_contratados
  add column if not exists pais_iso text not null default 'CL' references paises(iso2);

comment on column alumnos.documento_numero is 'Documento fiscal único por (país, tipo); para CL es el RUT normalizado';
comment on column colegios.pais_iso is 'CL usa region/comuna CUT; otros países usan division1/division2';
