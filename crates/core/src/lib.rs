//! Núcleo del dominio OCCB (clon de Twenty).
//! Todo en español por defecto, con internacionalización ES/EN para la interfaz.
//!
//! Buenas prácticas aplicadas:
//! - Tipos de dominio inmutables y serializables.
//! - Validación pura y testeable en [`validacion`].
//! - Textos de interfaz centralizados en [`i18n`].
//! - Datos de demostración aislados en [`semilla`].

pub mod dominio;
pub mod i18n;
pub mod semilla;
pub mod validacion;

pub use dominio::{
    CrearEmpresa, CrearOportunidad, CrearPersona, Empresa, EtapaOportunidad, Oportunidad, Persona,
    ResumenPanel, calcular_resumen,
};
pub use semilla::{datos_semilla_empresas, datos_semilla_oportunidades, datos_semilla_personas};
pub use i18n::{Idioma, Textos, textos};
pub use validacion::{ErrorDominio, correo_valido, validar_empresa, validar_oportunidad, validar_persona};
