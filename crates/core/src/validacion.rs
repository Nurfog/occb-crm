//! Validación pura del dominio con mensajes en español.
//! Se usa tanto en la API (antes de llamar a PostgreSQL) como en el frontal.

use super::dominio::{CrearAlumno, CrearApoderado, CrearColegio, CrearCorreo, CrearEvento, CrearIngreso, CrearNota, CrearOportunidad, CrearPersona, CrearTarea};
use thiserror::Error;

/// Error de dominio con mensaje listo para mostrar al usuario.
#[derive(Debug, Error, PartialEq)]
pub enum ErrorDominio {
    /// Un campo no cumple las reglas de negocio.
    #[error("Validación: {0}")]
    Validacion(String),
}

/// Recorta espacios de forma segura.
fn limpio(valor: &str) -> String {
    valor.trim().to_string()
}

/// Valida que un texto obligatorio no esté vacío.
fn exigir_texto(valor: &str, campo: &str, maximo: usize) -> Result<String, ErrorDominio> {
    let texto = limpio(valor);
    if texto.is_empty() {
        return Err(ErrorDominio::Validacion(format!(
            "El campo «{campo}» es obligatorio."
        )));
    }
    if texto.len() > maximo {
        return Err(ErrorDominio::Validacion(format!(
            "El campo «{campo}» no puede superar {maximo} caracteres."
        )));
    }
    Ok(texto)
}

/// Valida un colegio completo y devuelve la versión saneada.
pub fn validar_colegio(mut entrada: CrearColegio) -> Result<CrearColegio, ErrorDominio> {
    if limpio(&entrada.rbd).is_empty() {
        return Err(ErrorDominio::Validacion("El RBD es obligatorio.".into()));
    }
    entrada.rbd = limpio(&entrada.rbd);
    entrada.nombre = exigir_texto(&entrada.nombre, "nombre", 200)?;
    entrada.dependencia = limpio(&entrada.dependencia);
    entrada.dv = limpio(&entrada.dv);
    let mut region = Some(entrada.region);
    let mut comuna = Some(entrada.comuna.clone());
    validar_territorio_entrada(&mut region, &mut comuna)?;
    entrada.region = region.ok_or_else(|| ErrorDominio::Validacion("La región es obligatoria.".into()))?;
    entrada.comuna = comuna.ok_or_else(|| ErrorDominio::Validacion("La comuna es obligatoria.".into()))?;
    Ok(entrada)
}

/// Comprueba un correo con una regla simple pero suficiente para el MVP.
pub fn correo_valido(correo: &str) -> bool {
    let correo = correo.trim();
    let partes: Vec<&str> = correo.split('@').collect();
    if partes.len() != 2 {
        return false;
    }
    let (usuario, dominio) = (partes[0], partes[1]);
    !usuario.is_empty() && dominio.contains('.') && !dominio.starts_with('.')
}

/// Normaliza un RUT chileno a `12345678-5` (sin puntos, DV en mayúscula).
/// Acepta puntos, espacios y guion variados; devuelve `None` si no tiene forma válida.
pub fn normalizar_rut(rut: &str) -> Option<String> {
    let limpio: String = rut
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_uppercase();
    if limpio.len() < 2 {
        return None;
    }
    let (cuerpo, dv) = limpio.split_at(limpio.len() - 1);
    if cuerpo.is_empty() || !cuerpo.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if !(dv.chars().all(|c| c.is_ascii_digit()) || dv == "K") {
        return None;
    }
    // Cuerpo sin ceros a la izquierda sobrantes (conserva un cero si es todo ceros).
    let cuerpo = cuerpo.trim_start_matches('0');
    let cuerpo = if cuerpo.is_empty() { "0" } else { cuerpo };
    Some(format!("{cuerpo}-{dv}"))
}

/// Dígito verificador por módulo 11 para el cuerpo numérico del RUT.
fn digito_verificador(cuerpo: &str) -> char {
    let mut suma = 0u32;
    let mut factor = 2u32;
    for digito in cuerpo.chars().rev() {
        suma += digito.to_digit(10).unwrap_or(0) * factor;
        factor = if factor == 7 { 2 } else { factor + 1 };
    }
    match 11 - (suma % 11) {
        11 => '0',
        10 => 'K',
        n => char::from_digit(n, 10).unwrap_or('0'),
    }
}

/// Valida un RUT chileno (forma + dígito verificador) y devuelve el normalizado.
pub fn validar_rut(rut: &str) -> Result<String, ErrorDominio> {
    let normalizado = normalizar_rut(rut).ok_or_else(|| {
        ErrorDominio::Validacion("El RUT no tiene un formato válido (ej: 12345678-5).".into())
    })?;
    let cuerpo = normalizado.split('-').next().unwrap_or("");
    let dv = normalizado.chars().last().unwrap_or(' ');
    if digito_verificador(cuerpo) != dv {
        return Err(ErrorDominio::Validacion(
            "El dígito verificador del RUT no cuadra.".into(),
        ));
    }
    Ok(normalizado)
}

/// Valida una persona completa.
pub fn validar_persona(mut entrada: CrearPersona) -> Result<CrearPersona, ErrorDominio> {
    entrada.nombre = exigir_texto(&entrada.nombre, "nombre", 120)?;
    entrada.apellido = exigir_texto(&entrada.apellido, "apellidos", 120)?;
    entrada.correo = limpio(&entrada.correo).to_lowercase();
    if !correo_valido(&entrada.correo) {
        return Err(ErrorDominio::Validacion(
            "El correo electrónico no tiene un formato válido.".into(),
        ));
    }
    entrada.telefono = limpio(&entrada.telefono);
    entrada.puesto = limpio(&entrada.puesto);
    entrada.ciudad = limpio(&entrada.ciudad);
    validar_territorio_entrada(&mut entrada.region_codigo, &mut entrada.comuna_codigo)?;
    Ok(entrada)
}

/// Normaliza y valida la pareja región/comuna (cadenas vacías = ausente).
fn validar_territorio_entrada(
    region: &mut Option<i16>,
    comuna: &mut Option<String>,
) -> Result<(), ErrorDominio> {
    let comuna_limpia = comuna.as_deref().map(limpio).filter(|s| !s.is_empty());
    *comuna = comuna_limpia;
    let region_u8 = match region {
        None => None,
        Some(r) if (1..=16).contains(r) => Some(*r as u8),
        Some(_) => {
            return Err(ErrorDominio::Validacion("La región no es válida.".into()));
        }
    };
    super::territorio::validar_territorio(region_u8, comuna.as_deref())
}

/// Valida un alumno completo y devuelve la versión saneada (RUT normalizado).
pub fn validar_alumno(mut entrada: CrearAlumno) -> Result<CrearAlumno, ErrorDominio> {
    entrada.rut = validar_rut(&entrada.rut)?;
    entrada.nombres = exigir_texto(&entrada.nombres, "nombres", 120)?;
    entrada.apellidos = exigir_texto(&entrada.apellidos, "apellidos", 120)?;
    entrada.curso = limpio(&entrada.curso);
    entrada.grupo_sanguineo = entrada.grupo_sanguineo.trim().to_uppercase();
    if !["", "O-", "O+", "A-", "A+", "B-", "B+", "AB-", "AB+"].contains(&entrada.grupo_sanguineo.as_str()) {
        return Err(ErrorDominio::Validacion("Grupo sanguíneo no válido.".into()));
    }
    entrada.alergias = exigir_largo(&entrada.alergias, "alergias", 500)?;
    entrada.medicamentos = exigir_largo(&entrada.medicamentos, "medicamentos", 500)?;
    entrada.antecedentes_medicos = exigir_largo(&entrada.antecedentes_medicos, "antecedentes médicos", 2000)?;
    entrada.contacto_emergencia = exigir_largo(&entrada.contacto_emergencia, "contacto de emergencia", 200)?;
    Ok(entrada)
}

/// Recorta un texto opcional verificando su largo máximo.
fn exigir_largo(valor: &str, campo: &str, maximo: usize) -> Result<String, ErrorDominio> {
    let texto = limpio(valor);
    if texto.len() > maximo {
        return Err(ErrorDominio::Validacion(format!(
            "El campo «{campo}» no puede superar {maximo} caracteres."
        )));
    }
    Ok(texto)
}

/// Valida un apoderado completo y devuelve la versión saneada.
pub fn validar_apoderado(mut entrada: CrearApoderado) -> Result<CrearApoderado, ErrorDominio> {
    entrada.rut = validar_rut(&entrada.rut)?;
    entrada.nombres = exigir_texto(&entrada.nombres, "nombres", 120)?;
    entrada.apellidos = exigir_texto(&entrada.apellidos, "apellidos", 120)?;
    entrada.telefono = limpio(&entrada.telefono);
    entrada.direccion = limpio(&entrada.direccion);
    if !entrada.correo.trim().is_empty() {
        entrada.correo = limpio(&entrada.correo).to_lowercase();
        if !correo_valido(&entrada.correo) {
            return Err(ErrorDominio::Validacion(
                "El correo electrónico no tiene un formato válido.".into(),
            ));
        }
    }
    validar_territorio_entrada(&mut entrada.region_codigo, &mut entrada.comuna_codigo)?;
    entrada.ocupacion = exigir_largo(&entrada.ocupacion, "ocupación", 120)?;
    Ok(entrada)
}

/// Valida un ingreso médico (motivo obligatorio, largos acotados).
pub fn validar_ingreso(mut entrada: CrearIngreso) -> Result<CrearIngreso, ErrorDominio> {
    entrada.motivo = exigir_texto(&entrada.motivo, "motivo", 200)?;
    entrada.detalle = exigir_largo(&entrada.detalle, "detalle", 2000)?;
    entrada.profesional = exigir_largo(&entrada.profesional, "profesional", 120)?;
    Ok(entrada)
}

/// Valida una tarea completa.
pub fn validar_tarea(mut entrada: CrearTarea) -> Result<CrearTarea, ErrorDominio> {
    entrada.titulo = exigir_texto(&entrada.titulo, "título", 200)?;
    entrada.detalle = limpio(&entrada.detalle);
    match entrada.prioridad.to_lowercase().as_str() {
        "baja" | "media" | "alta" => {}
        _ => {
            return Err(ErrorDominio::Validacion(
                "La prioridad debe ser baja, media o alta.".into(),
            ));
        }
    }
    Ok(entrada)
}

/// Valida una nota completa.
pub fn validar_nota(mut entrada: CrearNota) -> Result<CrearNota, ErrorDominio> {
    entrada.titulo = limpio(&entrada.titulo);
    entrada.cuerpo = exigir_texto(&entrada.cuerpo, "contenido", 5000)?;
    Ok(entrada)
}

/// Valida un evento completo (título y fin posterior al inicio).
pub fn validar_evento(mut entrada: CrearEvento) -> Result<CrearEvento, ErrorDominio> {
    entrada.titulo = exigir_texto(&entrada.titulo, "título", 200)?;
    entrada.lugar = limpio(&entrada.lugar);
    entrada.descripcion = limpio(&entrada.descripcion);
    if entrada.fin < entrada.inicio {
        return Err(ErrorDominio::Validacion(
            "El fin no puede ser anterior al inicio.".into(),
        ));
    }
    Ok(entrada)
}

/// Valida un correo completo (remitente válido y contenido obligatorio).
pub fn validar_correo(mut entrada: CrearCorreo) -> Result<CrearCorreo, ErrorDominio> {
    entrada.asunto = limpio(&entrada.asunto);
    entrada.cuerpo = exigir_texto(&entrada.cuerpo, "contenido", 5000)?;
    entrada.remitente = limpio(&entrada.remitente).to_lowercase();
    if !correo_valido(&entrada.remitente) {
        return Err(ErrorDominio::Validacion(
            "El remitente no es un correo válido.".into(),
        ));
    }
    Ok(entrada)
}

/// Valida una oportunidad completa.
pub fn validar_oportunidad(mut entrada: CrearOportunidad) -> Result<CrearOportunidad, ErrorDominio> {
    entrada.nombre = exigir_texto(&entrada.nombre, "nombre", 200)?;
    if entrada.importe < 0 {
        return Err(ErrorDominio::Validacion(
            "El importe no puede ser negativo.".into(),
        ));
    }
    Ok(entrada)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn colegio_ejemplo() -> CrearColegio {
        CrearColegio {
            rbd: "10001".into(),
            nombre: "Liceo Demo".into(),
            region: 13,
            comuna: "13101".into(),
            dependencia: "Municipal".into(),
            dv: "1".into(),
        }
    }

    #[test]
    fn rechaza_rbd_vacio() {
        let mut datos = colegio_ejemplo();
        datos.rbd = "   ".into();
        assert!(validar_colegio(datos).is_err());
    }

    #[test]
    fn rechaza_comuna_ajena() {
        let mut datos = colegio_ejemplo();
        datos.comuna = "01101".into();
        assert!(validar_colegio(datos).is_err());
    }

    #[test]
    fn rechaza_correo_invalido() {
        let entrada = CrearPersona {
            nombre: "Ada".into(),
            apellido: "Lovelace".into(),
            correo: "no-es-correo".into(),
            telefono: "".into(),
            puesto: "CTO".into(),
            ciudad: "Madrid".into(),
            region_codigo: None,
            comuna_codigo: None,
        };
        assert!(validar_persona(entrada).is_err());
    }

    #[test]
    fn rechaza_importe_negativo() {
        let entrada = CrearOportunidad {
            nombre: "Trato".into(),
            importe: -5,
            etapa: crate::dominio::EtapaOportunidad::Contacto,
            colegio_rbd: None,
        };
        assert!(validar_oportunidad(entrada).is_err());
    }

    #[test]
    fn acepta_datos_correctos() {
        assert!(validar_colegio(colegio_ejemplo()).is_ok());
    }

    #[test]
    fn valida_tarea_y_nota() {
        let tarea = CrearTarea {
            titulo: "Llamar apoderado".into(),
            detalle: "".into(),
            estado: crate::dominio::EstadoTarea::Pendiente,
            prioridad: "alta".into(),
            vencimiento: None,
            alumno_id: None,
            apoderado_id: None,
            oportunidad_id: None,
        };
        assert!(validar_tarea(tarea).is_ok());
        let mala = CrearTarea {
            titulo: "".into(),
            detalle: "".into(),
            estado: crate::dominio::EstadoTarea::Pendiente,
            prioridad: "urgente".into(),
            vencimiento: None,
            alumno_id: None,
            apoderado_id: None,
            oportunidad_id: None,
        };
        assert!(validar_tarea(mala).is_err());
        let nota = CrearNota {
            titulo: "Entrevista".into(),
            cuerpo: "Comprometido con el proceso.".into(),
            alumno_id: None,
            apoderado_id: None,
            oportunidad_id: None,
        };
        assert!(validar_nota(nota).is_ok());
    }

    #[test]
    fn valida_evento_y_correo() {
        let ahora = chrono::Utc::now();
        let evento = CrearEvento {
            titulo: "Entrevista familiar".into(),
            inicio: ahora,
            fin: ahora + chrono::Duration::hours(1),
            lugar: "Oficina".into(),
            descripcion: "".into(),
            alumno_id: None,
            apoderado_id: None,
            oportunidad_id: None,
        };
        assert!(validar_evento(evento.clone()).is_ok());
        let invertido = CrearEvento {
            fin: ahora - chrono::Duration::hours(1),
            ..evento
        };
        assert!(validar_evento(invertido).is_err());
        let correo = CrearCorreo {
            asunto: "Bienvenida".into(),
            cuerpo: "Hola.".into(),
            remitente: "colegio@test.cl".into(),
            alumno_id: None,
            apoderado_id: None,
            oportunidad_id: None,
        };
        assert!(validar_correo(correo).is_ok());
        let malo = CrearCorreo {
            asunto: "".into(),
            cuerpo: "Hola.".into(),
            remitente: "no-es-correo".into(),
            alumno_id: None,
            apoderado_id: None,
            oportunidad_id: None,
        };
        assert!(validar_correo(malo).is_err());
    }

    #[test]
    fn valida_alumno_con_rut() {
        let entrada = CrearAlumno {
            rut: "11.111.111-1".into(),
            nombres: "Juan".into(),
            apellidos: "Pérez".into(),
            fecha_nacimiento: None,
            curso: "8° Básico A".into(),
            colegio_rbd: None,
            estado: crate::dominio::EstadoAlumno::Postulante,
            grupo_sanguineo: "O+".into(),
            alergias: "Maní".into(),
            medicamentos: "".into(),
            antecedentes_medicos: "Asma leve.".into(),
            contacto_emergencia: "María +56912345678".into(),
        };
        let saneado = validar_alumno(entrada).unwrap();
        assert_eq!(saneado.rut, "11111111-1");
        assert_eq!(saneado.grupo_sanguineo, "O+");
    }

    #[test]
    fn rechaza_alumno_sin_nombres() {
        let entrada = CrearAlumno {
            rut: "12345678-5".into(),
            nombres: "   ".into(),
            apellidos: "Pérez".into(),
            fecha_nacimiento: None,
            curso: "".into(),
            colegio_rbd: None,
            estado: crate::dominio::EstadoAlumno::Postulante,
            grupo_sanguineo: "".into(),
            alergias: "".into(),
            medicamentos: "".into(),
            antecedentes_medicos: "".into(),
            contacto_emergencia: "".into(),
        };
        assert!(validar_alumno(entrada).is_err());
    }

    #[test]
    fn valida_ingreso_medico() {
        let bueno = CrearIngreso {
            alumno_id: uuid::Uuid::new_v4(),
            fecha: None,
            motivo: "Control".into(),
            detalle: "".into(),
            profesional: "Dra. Soto".into(),
        };
        assert!(validar_ingreso(bueno).is_ok());
        let malo = CrearIngreso {
            alumno_id: uuid::Uuid::new_v4(),
            fecha: None,
            motivo: "   ".into(),
            detalle: "".into(),
            profesional: "".into(),
        };
        assert!(validar_ingreso(malo).is_err());
    }

    #[test]
    fn rechaza_grupo_sanguineo_invalido() {        let entrada = CrearAlumno {
            rut: "12345678-5".into(),
            nombres: "Ana".into(),
            apellidos: "Pérez".into(),
            fecha_nacimiento: None,
            curso: "".into(),
            colegio_rbd: None,
            estado: crate::dominio::EstadoAlumno::Postulante,
            grupo_sanguineo: "Z+".into(),
            alergias: "".into(),
            medicamentos: "".into(),
            antecedentes_medicos: "".into(),
            contacto_emergencia: "".into(),
        };
        assert!(validar_alumno(entrada).is_err());
    }

    #[test]
    fn valida_rut_chileno() {
        assert_eq!(validar_rut("12.345.678-5").unwrap(), "12345678-5");
        assert_eq!(validar_rut("12345678-5").unwrap(), "12345678-5");
        assert_eq!(validar_rut(" 6.123.456-k ").unwrap(), "6123456-K");
        assert!(validar_rut("12345678-9").is_err());
        assert!(validar_rut("no-es-rut").is_err());
        assert!(validar_rut("1-9").is_ok());
    }
}
