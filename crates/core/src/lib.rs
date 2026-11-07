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
    Actividad, Alumno, Apoderado, Automatizacion, Colegio, Correo, CrearAlumno, CrearApoderado, CrearColegio,
    CrearCorreo, CrearEvento, CrearNota, CrearOportunidad, CrearPersona, CrearTarea, EstadoAlumno,
    Ejecucion, EstadoTarea, EtapaOportunidad, Evento, Favorito, Nota, Oportunidad, Persona,
    ResultadoBusqueda, ResumenPanel, Tarea, Traslado, VistaGuardada, VinculoApoderado,
    calcular_resumen,
};
pub use territorio::{Comuna, Region, comuna_por_cut, comunas_de_region, region};
pub use semilla::{
    datos_semilla_alumnos, datos_semilla_apoderados, datos_semilla_colegios, datos_semilla_notas,
    datos_semilla_oportunidades, datos_semilla_personas, datos_semilla_tareas,
};
pub use i18n::{Idioma, Textos, textos};
pub use validacion::{
    ErrorDominio, correo_valido, normalizar_rut, validar_alumno, validar_apoderado, validar_colegio,
    validar_correo, validar_evento, validar_nota, validar_oportunidad, validar_persona, validar_rut,
    validar_tarea,
};
