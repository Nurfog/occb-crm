//! Validación pura del dominio con mensajes en español.
//! Se usa tanto en la API (antes de llamar a PostgreSQL) como en el frontal.

use super::dominio::{CrearEmpresa, CrearOportunidad, CrearPersona};
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

/// Valida una empresa completa y devuelve la versión saneada.
pub fn validar_empresa(mut entrada: CrearEmpresa) -> Result<CrearEmpresa, ErrorDominio> {
    entrada.nombre = exigir_texto(&entrada.nombre, "nombre", 200)?;
    entrada.dominio = limpio(&entrada.dominio);
    entrada.sector = limpio(&entrada.sector);
    entrada.ciudad = limpio(&entrada.ciudad);
    if !(0..=1_000_000).contains(&entrada.empleados) {
        return Err(ErrorDominio::Validacion(
            "El número de empleados debe estar entre 0 y 1 000 000.".into(),
        ));
    }
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
    use uuid::Uuid;

    fn empresa_ejemplo() -> CrearEmpresa {
        CrearEmpresa {
            nombre: "Acme".into(),
            dominio: "acme.com".into(),
            sector: "Software".into(),
            empleados: 10,
            ciudad: "Madrid".into(),
        }
    }

    #[test]
    fn rechaza_nombre_vacio() {
        let mut datos = empresa_ejemplo();
        datos.nombre = "   ".into();
        assert!(validar_empresa(datos).is_err());
    }

    #[test]
    fn rechaza_empleados_negativos() {
        let mut datos = empresa_ejemplo();
        datos.empleados = -1;
        assert!(validar_empresa(datos).is_err());
    }

    #[test]
    fn rechaza_correo_invalido() {
        let entrada = CrearPersona {
            nombre: "Ada".into(),
            apellido: "Lovelace".into(),
            correo: "no-es-correo".into(),
            telefono: "".into(),
            puesto: "CTO".into(),
            empresa_id: None,
            ciudad: "Madrid".into(),
        };
        assert!(validar_persona(entrada).is_err());
    }

    #[test]
    fn rechaza_importe_negativo() {
        let entrada = CrearOportunidad {
            nombre: "Trato".into(),
            importe: -5,
            etapa: crate::dominio::EtapaOportunidad::Nueva,
            empresa_id: Some(Uuid::new_v4()),
        };
        assert!(validar_oportunidad(entrada).is_err());
    }

    #[test]
    fn acepta_datos_correctos() {
        assert!(validar_empresa(empresa_ejemplo()).is_ok());
    }
}
