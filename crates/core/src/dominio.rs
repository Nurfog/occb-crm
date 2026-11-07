//! Tipos de dominio en español: colegios, personas, admisión y panel.
//!
//! Los nombres de campos usan español (`nombre`, `correo`, `importe`, `etapa`)
//! para que la base de datos, la API y el frontal compartan vocabulario.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Colegio del directorio oficial (clave RBD).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Colegio {
    pub rbd: String,
    pub dv: String,
    pub nombre: String,
    pub dependencia: String,
    pub region: i16,
    pub comuna: String,
    pub latitud: Option<f64>,
    pub longitud: Option<f64>,
    pub estado: String,
}

/// Datos para registrar un colegio manualmente.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrearColegio {
    pub rbd: String,
    pub nombre: String,
    pub region: i16,
    pub comuna: String,
    pub dependencia: String,
    pub dv: String,
}

/// Estado de un alumno en el sistema escolar.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EstadoAlumno {
    Postulante,
    Matriculado,
    Trasladado,
    Retirado,
}

impl EstadoAlumno {
    /// Todos los estados en orden.
    pub fn todas() -> &'static [Self] {
        use EstadoAlumno::*;
        &[Postulante, Matriculado, Trasladado, Retirado]
    }

    /// Convierte el texto de PG al estado.
    pub fn desde_texto(valor: &str) -> Option<Self> {
        match valor {
            "postulante" => Some(Self::Postulante),
            "matriculado" => Some(Self::Matriculado),
            "trasladado" => Some(Self::Trasladado),
            "retirado" => Some(Self::Retirado),
            _ => None,
        }
    }

    /// Texto que se persiste en PostgreSQL.
    pub fn como_texto(&self) -> &'static str {
        match self {
            Self::Postulante => "postulante",
            Self::Matriculado => "matriculado",
            Self::Trasladado => "trasladado",
            Self::Retirado => "retirado",
        }
    }
}

/// Alumno del colegio (RUT único con dígito verificador).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Alumno {
    pub id: Uuid,
    pub rut: String,
    pub nombres: String,
    pub apellidos: String,
    pub fecha_nacimiento: Option<chrono::NaiveDate>,
    pub curso: String,
    pub colegio_rbd: Option<String>,
    pub estado: EstadoAlumno,
    pub creado_en: DateTime<Utc>,
}

/// Datos para matricular un alumno.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrearAlumno {
    pub rut: String,
    pub nombres: String,
    pub apellidos: String,
    pub fecha_nacimiento: Option<chrono::NaiveDate>,
    pub curso: String,
    pub colegio_rbd: Option<String>,
    pub estado: EstadoAlumno,
}

/// Apoderado (madre, padre, tutor u otro) con RUT único.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Apoderado {
    pub id: Uuid,
    pub rut: String,
    pub nombres: String,
    pub apellidos: String,
    pub telefono: String,
    pub correo: String,
    pub direccion: String,
    pub region_codigo: Option<i16>,
    pub comuna_codigo: Option<String>,
    pub creado_en: DateTime<Utc>,
}

/// Datos para registrar un apoderado.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrearApoderado {
    pub rut: String,
    pub nombres: String,
    pub apellidos: String,
    pub telefono: String,
    pub correo: String,
    pub direccion: String,
    pub region_codigo: Option<i16>,
    pub comuna_codigo: Option<String>,
}

/// Vínculo entre un alumno y su apoderado.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VinculoApoderado {
    pub id: Uuid,
    pub alumno_id: Uuid,
    pub apoderado_id: Uuid,
    pub vinculo: String,
    pub retiro_autorizado: bool,
    pub notificaciones: bool,
}

/// Traslado de un alumno entre colegios (historial).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Traslado {
    pub id: Uuid,
    pub alumno_id: Uuid,
    pub origen_rbd: Option<String>,
    pub destino_rbd: String,
    pub fecha: DateTime<Utc>,
    pub motivo: String,
}

/// Estado de una tarea.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EstadoTarea {
    Pendiente,
    EnCurso,
    Hecha,
}

impl EstadoTarea {
    /// Todas en orden del tablero.
    pub fn todas() -> &'static [Self] {
        use EstadoTarea::*;
        &[Pendiente, EnCurso, Hecha]
    }

    /// Convierte el texto de PG al estado.
    pub fn desde_texto(valor: &str) -> Option<Self> {
        match valor {
            "pendiente" => Some(Self::Pendiente),
            "en_curso" => Some(Self::EnCurso),
            "hecha" => Some(Self::Hecha),
            _ => None,
        }
    }

    /// Texto que se persiste en PostgreSQL.
    pub fn como_texto(&self) -> &'static str {
        match self {
            Self::Pendiente => "pendiente",
            Self::EnCurso => "en_curso",
            Self::Hecha => "hecha",
        }
    }

    /// Avanza al siguiente estado (el tablero usa este orden).
    pub fn siguiente(&self) -> Self {
        match self {
            Self::Pendiente => Self::EnCurso,
            Self::EnCurso => Self::Hecha,
            Self::Hecha => Self::Hecha,
        }
    }
}

/// Tarea vinculada al quehacer escolar.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tarea {
    pub id: Uuid,
    pub titulo: String,
    pub detalle: String,
    pub estado: EstadoTarea,
    pub prioridad: String,
    pub vencimiento: Option<chrono::NaiveDate>,
    pub alumno_id: Option<Uuid>,
    pub apoderado_id: Option<Uuid>,
    pub oportunidad_id: Option<Uuid>,
    pub creado_en: DateTime<Utc>,
}

/// Datos para crear una tarea.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrearTarea {
    pub titulo: String,
    pub detalle: String,
    pub estado: EstadoTarea,
    pub prioridad: String,
    pub vencimiento: Option<chrono::NaiveDate>,
    pub alumno_id: Option<Uuid>,
    pub apoderado_id: Option<Uuid>,
    pub oportunidad_id: Option<Uuid>,
}

/// Nota libre vinculada a registros.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Nota {
    pub id: Uuid,
    pub titulo: String,
    pub cuerpo: String,
    pub alumno_id: Option<Uuid>,
    pub apoderado_id: Option<Uuid>,
    pub oportunidad_id: Option<Uuid>,
    pub creado_en: DateTime<Utc>,
}

/// Datos para crear una nota.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrearNota {
    pub titulo: String,
    pub cuerpo: String,
    pub alumno_id: Option<Uuid>,
    pub apoderado_id: Option<Uuid>,
    pub oportunidad_id: Option<Uuid>,
}

/// Evento de la agenda escolar (reunión, evaluación, entrevista).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evento {
    pub id: Uuid,
    pub titulo: String,
    pub inicio: DateTime<Utc>,
    pub fin: DateTime<Utc>,
    pub lugar: String,
    pub descripcion: String,
    pub alumno_id: Option<Uuid>,
    pub apoderado_id: Option<Uuid>,
    pub oportunidad_id: Option<Uuid>,
    pub creado_en: DateTime<Utc>,
}

/// Datos para crear un evento.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrearEvento {
    pub titulo: String,
    pub inicio: DateTime<Utc>,
    pub fin: DateTime<Utc>,
    pub lugar: String,
    pub descripcion: String,
    pub alumno_id: Option<Uuid>,
    pub apoderado_id: Option<Uuid>,
    pub oportunidad_id: Option<Uuid>,
}

/// Correo de la bandeja vinculada a la vida escolar.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Correo {
    pub id: Uuid,
    pub asunto: String,
    pub cuerpo: String,
    pub remitente: String,
    pub alumno_id: Option<Uuid>,
    pub apoderado_id: Option<Uuid>,
    pub oportunidad_id: Option<Uuid>,
    pub leido: bool,
    pub recibido_en: DateTime<Utc>,
    pub creado_en: DateTime<Utc>,
}

/// Datos para registrar un correo.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrearCorreo {
    pub asunto: String,
    pub cuerpo: String,
    pub remitente: String,
    pub alumno_id: Option<Uuid>,
    pub apoderado_id: Option<Uuid>,
    pub oportunidad_id: Option<Uuid>,
}

/// Resultado de la búsqueda global (el identificador del colegio es RBD).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResultadoBusqueda {
    pub tipo: String,
    pub id: String,
    pub titulo: String,
    pub detalle: String,
    pub puntaje: f32,
}

/// Registro marcado por el usuario para acceso rápido.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Favorito {
    pub tipo: String,
    pub id: Uuid,
    pub titulo: String,
    pub detalle: String,
    pub creado_en: DateTime<Utc>,
}

/// Vista guardada personal para una lista.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VistaGuardada {
    pub id: Uuid,
    pub objeto: String,
    pub nombre: String,
    pub tipo: String,
    pub filtros: serde_json::Value,
    pub orden: serde_json::Value,
    pub creado_en: DateTime<Utc>,
    pub actualizado_en: DateTime<Utc>,
}

/// Entrada de actividad con autor resuelto para timeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Actividad {
    pub id: Uuid,
    pub tipo: String,
    pub referencia_id: Uuid,
    pub accion: String,
    pub detalle: serde_json::Value,
    pub usuario_id: Option<Uuid>,
    pub autor: String,
    pub creado_en: DateTime<Utc>,
}

/// Regla ejecutable sobre altas o cambios de postulaciones.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Automatizacion {
    pub id: Uuid,
    pub nombre: String,
    pub activa: bool,
    pub disparador: String,
    pub objeto: String,
    pub condiciones: serde_json::Value,
    pub acciones: serde_json::Value,
    pub creado_en: DateTime<Utc>,
    pub actualizado_en: DateTime<Utc>,
}

/// Resultado de una regla disparada; los errores no revierten el registro origen.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ejecucion {
    pub id: Uuid,
    pub automatizacion_id: Uuid,
    pub registro_id: Option<Uuid>,
    pub disparador: String,
    pub ok: bool,
    pub detalle: serde_json::Value,
    pub usuario_id: Option<Uuid>,
    pub creado_en: DateTime<Utc>,
}

/// Conversación personal con el asistente del espacio escolar.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Conversacion {
    pub id: Uuid,
    pub usuario_id: Uuid,
    pub titulo: String,
    pub creado_en: DateTime<Utc>,
    pub actualizado_en: DateTime<Utc>,
}

/// Mensaje de una conversación.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mensaje {
    pub id: Uuid,
    pub conversacion_id: Uuid,
    pub rol: String,
    pub contenido: String,
    pub creado_en: DateTime<Utc>,
}

/// Persona de contacto.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Persona {
    pub id: Uuid,
    pub nombre: String,
    pub apellido: String,
    pub correo: String,
    pub telefono: String,
    pub puesto: String,
    pub ciudad: String,
    /// Código de región chilena (1–16), anulable.
    pub region_codigo: Option<i16>,
    /// CUT de comuna (5 dígitos), anulable.
    pub comuna_codigo: Option<String>,
    pub creado_en: DateTime<Utc>,
}

impl Persona {
    /// Nombre completo para tablas y tarjetas.
    pub fn nombre_completo(&self) -> String {
        format!("{} {}", self.nombre, self.apellido)
    }

    /// Iniciales para el avatar.
    pub fn iniciales(&self) -> String {
        let a = self.nombre.chars().next().unwrap_or('?');
        let b = self.apellido.chars().next().unwrap_or('?');
        format!("{a}{b}")
    }
}

/// Etapa del embudo de admisión (kanban).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EtapaOportunidad {
    Contacto,
    Postulante,
    Evaluado,
    Aceptado,
    Matriculado,
}

impl EtapaOportunidad {
    /// Todas las etapas en orden del embudo.
    pub fn todas() -> &'static [Self] {
        use EtapaOportunidad::*;
        &[Contacto, Postulante, Evaluado, Aceptado, Matriculado]
    }

    /// Convierte el texto que guarda PostgreSQL a la etapa.
    pub fn desde_texto(valor: &str) -> Option<Self> {
        match valor {
            "contacto" => Some(Self::Contacto),
            "postulante" => Some(Self::Postulante),
            "evaluado" => Some(Self::Evaluado),
            "aceptado" => Some(Self::Aceptado),
            "matriculado" => Some(Self::Matriculado),
            _ => None,
        }
    }

    /// Texto que se persiste en PostgreSQL.
    pub fn como_texto(&self) -> &'static str {
        match self {
            Self::Contacto => "contacto",
            Self::Postulante => "postulante",
            Self::Evaluado => "evaluado",
            Self::Aceptado => "aceptado",
            Self::Matriculado => "matriculado",
        }
    }

    /// Probabilidad estimada de matrícula por etapa.
    pub fn probabilidad(&self) -> u8 {
        match self {
            Self::Contacto => 10,
            Self::Postulante => 30,
            Self::Evaluado => 55,
            Self::Aceptado => 80,
            Self::Matriculado => 100,
        }
    }

    /// Avanza una etapa (el kanban usa este orden).
    pub fn siguiente(&self) -> Self {
        match self {
            Self::Contacto => Self::Postulante,
            Self::Postulante => Self::Evaluado,
            Self::Evaluado => Self::Aceptado,
            Self::Aceptado => Self::Matriculado,
            Self::Matriculado => Self::Matriculado,
        }
    }
}

/// Oportunidad de admisión (postulación).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Oportunidad {
    pub id: Uuid,
    pub nombre: String,
    pub importe: i64,
    pub etapa: EtapaOportunidad,
    pub colegio_rbd: Option<String>,
    pub contacto_id: Option<Uuid>,
    pub fecha_cierre: Option<DateTime<Utc>>,
    pub creado_en: DateTime<Utc>,
}

/// Resumen numérico del panel principal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResumenPanel {
    pub total_colegios: i64,
    pub total_personas: i64,
    pub total_oportunidades: i64,
    pub valor_tuberia: i64,
    pub valor_ganado: i64,
    pub tasa_exito: f32,
}

/// Datos para crear una persona.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrearPersona {
    pub nombre: String,
    pub apellido: String,
    pub correo: String,
    pub telefono: String,
    pub puesto: String,
    pub ciudad: String,
    pub region_codigo: Option<i16>,
    pub comuna_codigo: Option<String>,
}

/// Datos para crear una oportunidad.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrearOportunidad {
    pub nombre: String,
    pub importe: i64,
    pub etapa: EtapaOportunidad,
    pub colegio_rbd: Option<String>,
}

/// Calcula el resumen del panel a partir de listas en memoria.
/// La API en producción lo calcula en PostgreSQL con `panel_resumen()`.
pub fn calcular_resumen(
    colegios: &[Colegio],
    personas: &[Persona],
    oportunidades: &[Oportunidad],
) -> ResumenPanel {
    let valor_tuberia: i64 = oportunidades
        .iter()
        .filter(|o| o.etapa != EtapaOportunidad::Matriculado)
        .map(|o| o.importe)
        .sum();
    let valor_ganado: i64 = oportunidades
        .iter()
        .filter(|o| o.etapa == EtapaOportunidad::Matriculado)
        .map(|o| o.importe)
        .sum();
    let total = oportunidades.len().max(1) as f32;
    let ganadas = oportunidades
        .iter()
        .filter(|o| o.etapa == EtapaOportunidad::Matriculado)
        .count() as f32;
    ResumenPanel {
        total_colegios: colegios.len() as i64,
        total_personas: personas.len() as i64,
        total_oportunidades: oportunidades.len() as i64,
        valor_tuberia,
        valor_ganado,
        tasa_exito: ganadas / total * 100.0,
    }
}
