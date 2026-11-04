//! Núcleo del dominio OCCB.
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
pub mod territorio;
pub mod validacion;

pub use dominio::{
    Colegio, CrearColegio, CrearOportunidad, CrearPersona, EtapaOportunidad, Oportunidad, Persona,
    ResumenPanel, calcular_resumen,
};
pub use territorio::{Comuna, Region, comuna_por_cut, comunas_de_region, region};
pub use semilla::{datos_semilla_colegios, datos_semilla_oportunidades, datos_semilla_personas};
pub use i18n::{Idioma, Textos, textos};
pub use validacion::{ErrorDominio, correo_valido, validar_colegio, validar_oportunidad, validar_persona};
