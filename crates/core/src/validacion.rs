//! Validación pura del dominio con mensajes en español.
//! Se usa tanto en la API (antes de llamar a PostgreSQL) como en el frontal.

use super::dominio::{CrearColegio, CrearOportunidad, CrearPersona};
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
}
