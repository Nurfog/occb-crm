-- 009 — Territorio chileno: regiones y comunas con códigos CUT.
-- Tablas de referencia + seed (16 regiones, 346 comunas, fuente SUBDERE),
-- columnas en empresas/personas y CRUD actualizado con validación.

create table if not exists regiones (
  codigo smallint primary key,
  nombre text not null,
  romano text not null default '',
  sigla text not null default ''
);
comment on table regiones is 'Regiones de Chile (código numérico 1–16)';

create table if not exists comunas (
  codigo text primary key,
  nombre text not null,
  region smallint not null references regiones(codigo)
);
comment on table comunas is 'Comunas con CUT de 5 dígitos';

insert into regiones (codigo, nombre, romano, sigla) values
  (1, 'Tarapacá', 'I', 'TA'),
  (2, 'Antofagasta', 'II', 'AN'),
  (3, 'Atacama', 'III', 'AT'),
  (4, 'Coquimbo', 'IV', 'CO'),
  (5, 'Valparaíso', 'V', 'VA'),
  (6, 'Libertador Gral. Bernardo O''Higgins', 'VI', 'LI'),
  (7, 'Maule', 'VII', 'ML'),
  (8, 'Biobío', 'VIII', 'BI'),
  (9, 'Araucanía', 'IX', 'AR'),
  (10, 'Los Lagos', 'X', 'LL'),
  (11, 'Aisén del Gral. Carlos Ibáñez del Campo', 'XI', 'AI'),
  (12, 'Magallanes y de la Antártica Chilena', 'XII', 'MA'),
  (13, 'Metropolitana de Santiago', 'XIII', 'RM'),
  (14, 'Los Ríos', 'XIV', 'LR'),
  (15, 'Arica y Parinacota', 'XV', 'AP'),
  (16, 'Ñuble', 'XVI', 'NB')
on conflict (codigo) do update set nombre = excluded.nombre, romano = excluded.romano, sigla = excluded.sigla;

insert into comunas (codigo, nombre, region) values
  ('01101', 'Iquique', 1),
  ('01107', 'Alto Hospicio', 1),
  ('01401', 'Pozo Almonte', 1),
  ('01402', 'Camiña', 1),
  ('01403', 'Colchane', 1),
  ('01404', 'Huara', 1),
  ('01405', 'Pica', 1),
  ('02101', 'Antofagasta', 2),
  ('02102', 'Mejillones', 2),
  ('02103', 'Sierra Gorda', 2),
  ('02104', 'Taltal', 2),
  ('02201', 'Calama', 2),
  ('02202', 'Ollagüe', 2),
  ('02203', 'San Pedro de Atacama', 2),
  ('02301', 'Tocopilla', 2),
  ('02302', 'María Elena', 2),
  ('03101', 'Copiapó', 3),
  ('03102', 'Caldera', 3),
  ('03103', 'Tierra Amarilla', 3),
  ('03201', 'Chañaral', 3),
  ('03202', 'Diego de Almagro', 3),
  ('03301', 'Vallenar', 3),
  ('03302', 'Alto del Carmen', 3),
  ('03303', 'Freirina', 3),
  ('03304', 'Huasco', 3),
  ('04101', 'La Serena', 4),
  ('04102', 'Coquimbo', 4),
  ('04103', 'Andacollo', 4),
  ('04104', 'La Higuera', 4),
  ('04105', 'Paihuano', 4),
  ('04106', 'Vicuña', 4),
  ('04201', 'Illapel', 4),
  ('04202', 'Canela', 4),
  ('04203', 'Los Vilos', 4),
  ('04204', 'Salamanca', 4),
  ('04301', 'Ovalle', 4),
  ('04302', 'Combarbalá', 4),
  ('04303', 'Monte Patria', 4),
  ('04304', 'Punitaqui', 4),
  ('04305', 'Río Hurtado', 4),
  ('05101', 'Valparaíso', 5),
  ('05102', 'Casablanca', 5),
  ('05103', 'Concón', 5),
  ('05104', 'Juan Fernández', 5),
  ('05105', 'Puchuncaví', 5),
  ('05107', 'Quintero', 5),
  ('05109', 'Viña del Mar', 5),
  ('05201', 'Isla de Pascua', 5),
  ('05301', 'Los Andes', 5),
  ('05302', 'Calle Larga', 5),
  ('05303', 'Rinconada', 5),
  ('05304', 'San Esteban', 5),
  ('05401', 'La Ligua', 5),
  ('05402', 'Cabildo', 5),
  ('05403', 'Papudo', 5),
  ('05404', 'Petorca', 5),
  ('05405', 'Zapallar', 5),
  ('05501', 'Quillota', 5),
  ('05502', 'Calera', 5),
  ('05503', 'Hijuelas', 5),
  ('05504', 'La Cruz', 5),
  ('05506', 'Nogales', 5),
  ('05601', 'San Antonio', 5),
  ('05602', 'Algarrobo', 5),
  ('05603', 'Cartagena', 5),
  ('05604', 'El Quisco', 5),
  ('05605', 'El Tabo', 5),
  ('05606', 'Santo Domingo', 5),
  ('05701', 'San Felipe', 5),
  ('05702', 'Catemu', 5),
  ('05703', 'Llaillay', 5),
  ('05704', 'Panquehue', 5),
  ('05705', 'Putaendo', 5),
  ('05706', 'Santa María', 5),
  ('05801', 'Quilpué', 5),
  ('05802', 'Limache', 5),
  ('05803', 'Olmué', 5),
  ('05804', 'Villa Alemana', 5),
  ('06101', 'Rancagua', 6),
  ('06102', 'Codegua', 6),
  ('06103', 'Coinco', 6),
  ('06104', 'Coltauco', 6),
  ('06105', 'Doñihue', 6),
  ('06106', 'Graneros', 6),
  ('06107', 'Las Cabras', 6),
  ('06108', 'Machalí', 6),
  ('06109', 'Malloa', 6),
  ('06110', 'Mostazal', 6),
  ('06111', 'Olivar', 6),
  ('06112', 'Peumo', 6),
  ('06113', 'Pichidegua', 6),
  ('06114', 'Quinta de Tilcoco', 6),
  ('06115', 'Rengo', 6),
  ('06116', 'Requínoa', 6),
  ('06117', 'San Vicente', 6),
  ('06201', 'Pichilemu', 6),
  ('06202', 'La Estrella', 6),
  ('06203', 'Litueche', 6),
  ('06204', 'Marchigüe', 6),
  ('06205', 'Navidad', 6),
  ('06206', 'Paredones', 6),
  ('06301', 'San Fernando', 6),
  ('06302', 'Chépica', 6),
  ('06303', 'Chimbarongo', 6),
  ('06304', 'Lolol', 6),
  ('06305', 'Nancagua', 6),
  ('06306', 'Palmilla', 6),
  ('06307', 'Peralillo', 6),
  ('06308', 'Placilla', 6),
  ('06309', 'Pumanque', 6),
  ('06310', 'Santa Cruz', 6),
  ('07101', 'Talca', 7),
  ('07102', 'Constitución', 7),
  ('07103', 'Curepto', 7),
  ('07104', 'Empedrado', 7),
  ('07105', 'Maule', 7),
  ('07106', 'Pelarco', 7),
  ('07107', 'Pencahue', 7),
  ('07108', 'Río Claro', 7),
  ('07109', 'San Clemente', 7),
  ('07110', 'San Rafael', 7),
  ('07201', 'Cauquenes', 7),
  ('07202', 'Chanco', 7),
  ('07203', 'Pelluhue', 7),
  ('07301', 'Curicó', 7),
  ('07302', 'Hualañé', 7),
  ('07303', 'Licantén', 7),
  ('07304', 'Molina', 7),
  ('07305', 'Rauco', 7),
  ('07306', 'Romeral', 7),
  ('07307', 'Sagrada Familia', 7),
  ('07308', 'Teno', 7),
  ('07309', 'Vichuquén', 7),
  ('07401', 'Linares', 7),
  ('07402', 'Colbún', 7),
  ('07403', 'Longaví', 7),
  ('07404', 'Parral', 7),
  ('07405', 'Retiro', 7),
  ('07406', 'San Javier', 7),
  ('07407', 'Villa Alegre', 7),
  ('07408', 'Yerbas Buenas', 7),
  ('08101', 'Concepción', 8),
  ('08102', 'Coronel', 8),
  ('08103', 'Chiguayante', 8),
  ('08104', 'Florida', 8),
  ('08105', 'Hualqui', 8),
  ('08106', 'Lota', 8),
  ('08107', 'Penco', 8),
  ('08108', 'San Pedro de la Paz', 8),
  ('08109', 'Santa Juana', 8),
  ('08110', 'Talcahuano', 8),
  ('08111', 'Tomé', 8),
  ('08112', 'Hualpén', 8),
  ('08201', 'Lebu', 8),
  ('08202', 'Arauco', 8),
  ('08203', 'Cañete', 8),
  ('08204', 'Contulmo', 8),
  ('08205', 'Curanilahue', 8),
  ('08206', 'Los Alamos', 8),
  ('08207', 'Tirúa', 8),
  ('08301', 'Los Angeles', 8),
  ('08302', 'Antuco', 8),
  ('08303', 'Cabrero', 8),
  ('08304', 'Laja', 8),
  ('08305', 'Mulchén', 8),
  ('08306', 'Nacimiento', 8),
  ('08307', 'Negrete', 8),
  ('08308', 'Quilaco', 8),
  ('08309', 'Quilleco', 8),
  ('08310', 'San Rosendo', 8),
  ('08311', 'Santa Bárbara', 8),
  ('08312', 'Tucapel', 8),
  ('08313', 'Yumbel', 8),
  ('08314', 'Alto Biobío', 8),
  ('09101', 'Temuco', 9),
  ('09102', 'Carahue', 9),
  ('09103', 'Cunco', 9),
  ('09104', 'Curarrehue', 9),
  ('09105', 'Freire', 9),
  ('09106', 'Galvarino', 9),
  ('09107', 'Gorbea', 9),
  ('09108', 'Lautaro', 9),
  ('09109', 'Loncoche', 9),
  ('09110', 'Melipeuco', 9),
  ('09111', 'Nueva Imperial', 9),
  ('09112', 'Padre Las Casas', 9),
  ('09113', 'Perquenco', 9),
  ('09114', 'Pitrufquén', 9),
  ('09115', 'Pucón', 9),
  ('09116', 'Saavedra', 9),
  ('09117', 'Teodoro Schmidt', 9),
  ('09118', 'Toltén', 9),
  ('09119', 'Vilcún', 9),
  ('09120', 'Villarrica', 9),
  ('09121', 'Cholchol', 9),
  ('09201', 'Angol', 9),
  ('09202', 'Collipulli', 9),
  ('09203', 'Curacautín', 9),
  ('09204', 'Ercilla', 9),
  ('09205', 'Lonquimay', 9),
  ('09206', 'Los Sauces', 9),
  ('09207', 'Lumaco', 9),
  ('09208', 'Purén', 9),
  ('09209', 'Renaico', 9),
  ('09210', 'Traiguén', 9),
  ('09211', 'Victoria', 9),
  ('10101', 'Puerto Montt', 10),
  ('10102', 'Calbuco', 10),
  ('10103', 'Cochamó', 10),
  ('10104', 'Fresia', 10),
  ('10105', 'Frutillar', 10),
  ('10106', 'Los Muermos', 10),
  ('10107', 'Llanquihue', 10),
  ('10108', 'Maullín', 10),
  ('10109', 'Puerto Varas', 10),
  ('10201', 'Castro', 10),
  ('10202', 'Ancud', 10),
  ('10203', 'Chonchi', 10),
  ('10204', 'Curaco de Vélez', 10),
  ('10205', 'Dalcahue', 10),
  ('10206', 'Puqueldón', 10),
  ('10207', 'Queilén', 10),
  ('10208', 'Quellón', 10),
  ('10209', 'Quemchi', 10),
  ('10210', 'Quinchao', 10),
  ('10301', 'Osorno', 10),
  ('10302', 'Puerto Octay', 10),
  ('10303', 'Purranque', 10),
  ('10304', 'Puyehue', 10),
  ('10305', 'Río Negro', 10),
  ('10306', 'San Juan de la Costa', 10),
  ('10307', 'San Pablo', 10),
  ('10401', 'Chaitén', 10),
  ('10402', 'Futaleufú', 10),
  ('10403', 'Hualaihué', 10),
  ('10404', 'Palena', 10),
  ('11101', 'Coyhaique', 11),
  ('11102', 'Lago Verde', 11),
  ('11201', 'Aysén', 11),
  ('11202', 'Cisnes', 11),
  ('11203', 'Guaitecas', 11),
  ('11301', 'Cochrane', 11),
  ('11302', 'O''Higgins', 11),
  ('11303', 'Tortel', 11),
  ('11401', 'Chile Chico', 11),
  ('11402', 'Río Ibáñez', 11),
  ('12101', 'Punta Arenas', 12),
  ('12102', 'Laguna Blanca', 12),
  ('12103', 'Río Verde', 12),
  ('12104', 'San Gregorio', 12),
  ('12201', 'Cabo de Hornos', 12),
  ('12202', 'Antártica', 12),
  ('12301', 'Porvenir', 12),
  ('12302', 'Primavera', 12),
  ('12303', 'Timaukel', 12),
  ('12401', 'Natales', 12),
  ('12402', 'Torres del Paine', 12),
  ('13101', 'Santiago', 13),
  ('13102', 'Cerrillos', 13),
  ('13103', 'Cerro Navia', 13),
  ('13104', 'Conchalí', 13),
  ('13105', 'El Bosque', 13),
  ('13106', 'Estación Central', 13),
  ('13107', 'Huechuraba', 13),
  ('13108', 'Independencia', 13),
  ('13109', 'La Cisterna', 13),
  ('13110', 'La Florida', 13),
  ('13111', 'La Granja', 13),
  ('13112', 'La Pintana', 13),
  ('13113', 'La Reina', 13),
  ('13114', 'Las Condes', 13),
  ('13115', 'Lo Barnechea', 13),
  ('13116', 'Lo Espejo', 13),
  ('13117', 'Lo Prado', 13),
  ('13118', 'Macul', 13),
  ('13119', 'Maipú', 13),
  ('13120', 'Ñuñoa', 13),
  ('13121', 'Pedro Aguirre Cerda', 13),
  ('13122', 'Peñalolén', 13),
  ('13123', 'Providencia', 13),
  ('13124', 'Pudahuel', 13),
  ('13125', 'Quilicura', 13),
  ('13126', 'Quinta Normal', 13),
  ('13127', 'Recoleta', 13),
  ('13128', 'Renca', 13),
  ('13129', 'San Joaquín', 13),
  ('13130', 'San Miguel', 13),
  ('13131', 'San Ramón', 13),
  ('13132', 'Vitacura', 13),
  ('13201', 'Puente Alto', 13),
  ('13202', 'Pirque', 13),
  ('13203', 'San José de Maipo', 13),
  ('13301', 'Colina', 13),
  ('13302', 'Lampa', 13),
  ('13303', 'Tiltil', 13),
  ('13401', 'San Bernardo', 13),
  ('13402', 'Buin', 13),
  ('13403', 'Calera de Tango', 13),
  ('13404', 'Paine', 13),
  ('13501', 'Melipilla', 13),
  ('13502', 'Alhué', 13),
  ('13503', 'Curacaví', 13),
  ('13504', 'María Pinto', 13),
  ('13505', 'San Pedro', 13),
  ('13601', 'Talagante', 13),
  ('13602', 'El Monte', 13),
  ('13603', 'Isla de Maipo', 13),
  ('13604', 'Padre Hurtado', 13),
  ('13605', 'Peñaflor', 13),
  ('14101', 'Valdivia', 14),
  ('14102', 'Corral', 14),
  ('14103', 'Lanco', 14),
  ('14104', 'Los Lagos', 14),
  ('14105', 'Máfil', 14),
  ('14106', 'Mariquina', 14),
  ('14107', 'Paillaco', 14),
  ('14108', 'Panguipulli', 14),
  ('14201', 'La Unión', 14),
  ('14202', 'Futrono', 14),
  ('14203', 'Lago Ranco', 14),
  ('14204', 'Río Bueno', 14),
  ('15101', 'Arica', 15),
  ('15102', 'Camarones', 15),
  ('15201', 'Putre', 15),
  ('15202', 'General Lagos', 15),
  ('16101', 'Chillán', 16),
  ('16102', 'Bulnes', 16),
  ('16103', 'Chillán Viejo', 16),
  ('16104', 'El Carmen', 16),
  ('16105', 'Pemuco', 16),
  ('16106', 'Pinto', 16),
  ('16107', 'Quillón', 16),
  ('16108', 'San Ignacio', 16),
  ('16109', 'Yungay', 16),
  ('16201', 'Quirihue', 16),
  ('16202', 'Cobquecura', 16),
  ('16203', 'Coelemu', 16),
  ('16204', 'Ninhue', 16),
  ('16205', 'Portezuelo', 16),
  ('16206', 'Ranquil', 16),
  ('16207', 'Treguaco', 16),
  ('16301', 'San Carlos', 16),
  ('16302', 'Coihueco', 16),
  ('16303', 'Ñiquén', 16),
  ('16304', 'San Fabián', 16),
  ('16305', 'San Nicolás', 16)
on conflict (codigo) do update set nombre = excluded.nombre, region = excluded.region;

alter table empresas
  add column if not exists region_codigo smallint references regiones(codigo),
  add column if not exists comuna_codigo text references comunas(codigo);
alter table personas
  add column if not exists region_codigo smallint references regiones(codigo),
  add column if not exists comuna_codigo text references comunas(codigo);

alter table regiones enable row level security;
alter table comunas enable row level security;
revoke all on regiones, comunas from public;
grant select on regiones, comunas to public;

-- Listados de referencia para los selectores del frontal.
create or replace function listar_regiones()
returns setof regiones
language sql security definer set search_path = public
as $$ select * from regiones order by codigo; $$;

create or replace function listar_comunas_region(p_region smallint)
returns setof comunas
language plpgsql security definer set search_path = public
as $$
begin
  if p_region is null then
    raise exception 'La región es obligatoria para listar comunas.' using errcode = 'P0001';
  end if;
  return query select * from comunas where region = p_region order by nombre;
end;
$$;

grant execute on function listar_regiones() to public;
grant execute on function listar_comunas_region(smallint) to public;

drop function if exists crear_empresa(text, text, text, integer, text);
create or replace function crear_empresa(
  p_nombre text, p_dominio text default '', p_sector text default '',
  p_empleados integer default 0, p_ciudad text default '',
  p_region_codigo smallint default null, p_comuna_codigo text default null)
returns empresas language plpgsql security definer set search_path = public as $$
declare
  v_nombre text := btrim(coalesce(p_nombre, ''));
  v_empresa empresas%rowtype;
  v_comuna text := nullif(btrim(coalesce(p_comuna_codigo, '')), '');
begin
  if v_nombre = '' then
    raise exception 'El nombre de la empresa es obligatorio.' using errcode = 'P0001';
  end if;
  if char_length(v_nombre) > 200 then
    raise exception 'El nombre no puede superar 200 caracteres.' using errcode = 'P0001';
  end if;
  if coalesce(p_empleados, 0) < 0 or coalesce(p_empleados, 0) > 1000000 then
    raise exception 'El número de empleados debe estar entre 0 y 1000000.' using errcode = 'P0001';
  end if;
  -- Territorio chileno opcional pero coherente.
  if p_region_codigo is not null and not exists (select 1 from regiones where codigo = p_region_codigo) then
    raise exception 'La región no es válida.' using errcode = 'P0001';
  end if;
  if coalesce(btrim(p_comuna_codigo), '') <> '' then
    if p_region_codigo is null then
      raise exception 'Elija la región de la comuna.' using errcode = 'P0001';
    end if;
    if not exists (select 1 from comunas where codigo = btrim(p_comuna_codigo) and region = p_region_codigo) then
      raise exception 'La comuna no pertenece a la región elegida.' using errcode = 'P0001';
    end if;
  end if;
  insert into empresas (nombre, dominio, sector, empleados, ciudad, region_codigo, comuna_codigo)
  values (v_nombre, btrim(coalesce(p_dominio, '')), btrim(coalesce(p_sector, '')),
          coalesce(p_empleados, 0), btrim(coalesce(p_ciudad, '')), p_region_codigo, v_comuna)
  returning * into v_empresa;
  return v_empresa;
end;
$$;

drop function if exists actualizar_empresa(uuid, text, text, text, integer, text);
create or replace function actualizar_empresa(
  p_id uuid, p_nombre text, p_dominio text default '', p_sector text default '',
  p_empleados integer default 0, p_ciudad text default '',
  p_region_codigo smallint default null, p_comuna_codigo text default null)
returns empresas language plpgsql security definer set search_path = public as $$
declare
  v_nombre text := btrim(coalesce(p_nombre, ''));
  v_empresa empresas%rowtype;
  v_comuna text := nullif(btrim(coalesce(p_comuna_codigo, '')), '');
begin
  if p_id is null then
    raise exception 'El identificador de la empresa es obligatorio.' using errcode = 'P0001';
  end if;
  if v_nombre = '' then
    raise exception 'El nombre de la empresa es obligatorio.' using errcode = 'P0001';
  end if;
  if coalesce(p_empleados, 0) < 0 then
    raise exception 'El número de empleados no puede ser negativo.' using errcode = 'P0001';
  end if;
  -- Territorio chileno opcional pero coherente.
  if p_region_codigo is not null and not exists (select 1 from regiones where codigo = p_region_codigo) then
    raise exception 'La región no es válida.' using errcode = 'P0001';
  end if;
  if coalesce(btrim(p_comuna_codigo), '') <> '' then
    if p_region_codigo is null then
      raise exception 'Elija la región de la comuna.' using errcode = 'P0001';
    end if;
    if not exists (select 1 from comunas where codigo = btrim(p_comuna_codigo) and region = p_region_codigo) then
      raise exception 'La comuna no pertenece a la región elegida.' using errcode = 'P0001';
    end if;
  end if;
  update empresas set nombre = v_nombre, dominio = btrim(coalesce(p_dominio, '')),
    sector = btrim(coalesce(p_sector, '')), empleados = coalesce(p_empleados, 0),
    ciudad = btrim(coalesce(p_ciudad, '')), region_codigo = p_region_codigo, comuna_codigo = v_comuna
  where id = p_id returning * into v_empresa;
  if not found then
    raise exception 'Empresa no encontrada.' using errcode = 'P0002';
  end if;
  return v_empresa;
end;
$$;

drop function if exists crear_persona(text, text, text, text, text, uuid, text);
create or replace function crear_persona(
  p_nombre text, p_apellido text, p_correo text, p_telefono text default '',
  p_puesto text default '', p_empresa_id uuid default null, p_ciudad text default '',
  p_region_codigo smallint default null, p_comuna_codigo text default null)
returns personas language plpgsql security definer set search_path = public as $$
declare
  v_nombre text := btrim(coalesce(p_nombre, ''));
  v_apellido text := btrim(coalesce(p_apellido, ''));
  v_correo text := lower(btrim(coalesce(p_correo, '')));
  v_persona personas%rowtype;
  v_comuna text := nullif(btrim(coalesce(p_comuna_codigo, '')), '');
begin
  if v_nombre = '' then
    raise exception 'El nombre es obligatorio.' using errcode = 'P0001';
  end if;
  if v_apellido = '' then
    raise exception 'Los apellidos son obligatorios.' using errcode = 'P0001';
  end if;
  if v_correo !~ '^[^@\s]+@[^@\s]+\.[^@\s]+$' then
    raise exception 'El correo electrónico no tiene un formato válido.' using errcode = 'P0001';
  end if;
  if p_empresa_id is not null and not exists (select 1 from empresas where id = p_empresa_id) then
    raise exception 'La empresa indicada no existe.' using errcode = 'P0001';
  end if;
  -- Territorio chileno opcional pero coherente.
  if p_region_codigo is not null and not exists (select 1 from regiones where codigo = p_region_codigo) then
    raise exception 'La región no es válida.' using errcode = 'P0001';
  end if;
  if coalesce(btrim(p_comuna_codigo), '') <> '' then
    if p_region_codigo is null then
      raise exception 'Elija la región de la comuna.' using errcode = 'P0001';
    end if;
    if not exists (select 1 from comunas where codigo = btrim(p_comuna_codigo) and region = p_region_codigo) then
      raise exception 'La comuna no pertenece a la región elegida.' using errcode = 'P0001';
    end if;
  end if;
  insert into personas (nombre, apellido, correo, telefono, puesto, empresa_id, ciudad, region_codigo, comuna_codigo)
  values (v_nombre, v_apellido, v_correo, btrim(coalesce(p_telefono, '')),
          btrim(coalesce(p_puesto, '')), p_empresa_id, btrim(coalesce(p_ciudad, '')), p_region_codigo, v_comuna)
  returning * into v_persona;
  return v_persona;
exception when unique_violation then
  raise exception 'Ya existe una persona con ese correo.' using errcode = 'P0003';
end;
$$;

drop function if exists actualizar_persona(uuid, text, text, text, text, text, uuid, text);
create or replace function actualizar_persona(
  p_id uuid, p_nombre text, p_apellido text, p_correo text, p_telefono text default '',
  p_puesto text default '', p_empresa_id uuid default null, p_ciudad text default '',
  p_region_codigo smallint default null, p_comuna_codigo text default null)
returns personas language plpgsql security definer set search_path = public as $$
declare
  v_nombre text := btrim(coalesce(p_nombre, ''));
  v_apellido text := btrim(coalesce(p_apellido, ''));
  v_correo text := lower(btrim(coalesce(p_correo, '')));
  v_persona personas%rowtype;
  v_comuna text := nullif(btrim(coalesce(p_comuna_codigo, '')), '');
begin
  if p_id is null then
    raise exception 'El identificador de la persona es obligatorio.' using errcode = 'P0001';
  end if;
  if v_nombre = '' then
    raise exception 'El nombre es obligatorio.' using errcode = 'P0001';
  end if;
  if v_apellido = '' then
    raise exception 'Los apellidos son obligatorios.' using errcode = 'P0001';
  end if;
  if v_correo !~ '^[^@\s]+@[^@\s]+\.[^@\s]+$' then
    raise exception 'El correo electrónico no tiene un formato válido.' using errcode = 'P0001';
  end if;
  if p_empresa_id is not null and not exists (select 1 from empresas where id = p_empresa_id) then
    raise exception 'La empresa indicada no existe.' using errcode = 'P0001';
  end if;
  -- Territorio chileno opcional pero coherente.
  if p_region_codigo is not null and not exists (select 1 from regiones where codigo = p_region_codigo) then
    raise exception 'La región no es válida.' using errcode = 'P0001';
  end if;
  if coalesce(btrim(p_comuna_codigo), '') <> '' then
    if p_region_codigo is null then
      raise exception 'Elija la región de la comuna.' using errcode = 'P0001';
    end if;
    if not exists (select 1 from comunas where codigo = btrim(p_comuna_codigo) and region = p_region_codigo) then
      raise exception 'La comuna no pertenece a la región elegida.' using errcode = 'P0001';
    end if;
  end if;
  update personas set nombre = v_nombre, apellido = v_apellido, correo = v_correo,
    telefono = btrim(coalesce(p_telefono, '')), puesto = btrim(coalesce(p_puesto, '')),
    empresa_id = p_empresa_id, ciudad = btrim(coalesce(p_ciudad, '')),
    region_codigo = p_region_codigo, comuna_codigo = v_comuna
  where id = p_id returning * into v_persona;
  if not found then
    raise exception 'Persona no encontrada.' using errcode = 'P0002';
  end if;
  return v_persona;
exception when unique_violation then
  raise exception 'Ya existe una persona con ese correo.' using errcode = 'P0003';
end;
$$;

grant execute on function crear_empresa(text, text, text, integer, text, smallint, text) to public;
grant execute on function actualizar_empresa(uuid, text, text, text, integer, text, smallint, text) to public;
grant execute on function crear_persona(text, text, text, text, text, uuid, text, smallint, text) to public;
grant execute on function actualizar_persona(uuid, text, text, text, text, text, uuid, text, smallint, text) to public;
