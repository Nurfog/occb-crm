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
    Alumno, Apoderado, Colegio, CrearAlumno, CrearApoderado, CrearColegio, CrearOportunidad,
    CrearPersona, EstadoAlumno, EtapaOportunidad, Oportunidad, Persona, ResumenPanel, Traslado,
    VinculoApoderado, calcular_resumen,
};
pub use territorio::{Comuna, Region, comuna_por_cut, comunas_de_region, region};
pub use semilla::{
    datos_semilla_alumnos, datos_semilla_apoderados, datos_semilla_colegios,
    datos_semilla_oportunidades, datos_semilla_personas,
};
pub use i18n::{Idioma, Textos, textos};
pub use validacion::{
    ErrorDominio, correo_valido, normalizar_rut, validar_alumno, validar_apoderado, validar_colegio,
    validar_oportunidad, validar_persona, validar_rut,
};
