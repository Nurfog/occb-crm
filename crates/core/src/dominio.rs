//! Tipos de dominio en español: empresas, personas, oportunidades y panel.
//!
//! Los nombres de campos usan español (`nombre`, `correo`, `importe`, `etapa`)
//! para que la base de datos, la API y el frontal compartan vocabulario.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Empresa cliente.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Empresa {
    pub id: Uuid,
    pub nombre: String,
    pub dominio: String,
    pub sector: String,
    pub empleados: i32,
    pub ciudad: String,
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
    pub empresa_id: Option<Uuid>,
    pub ciudad: String,
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

/// Etapa del embudo comercial (kanban de oportunidades).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EtapaOportunidad {
    Nueva,
    Cribado,
    Reunion,
    Propuesta,
    Cliente,
}

impl EtapaOportunidad {
    /// Todas las etapas en orden del embudo.
    pub fn todas() -> &'static [Self] {
        use EtapaOportunidad::*;
        &[Nueva, Cribado, Reunion, Propuesta, Cliente]
    }

    /// Convierte el texto que guarda PostgreSQL a la etapa.
    pub fn desde_texto(valor: &str) -> Option<Self> {
        match valor {
            "nueva" => Some(Self::Nueva),
            "cribado" => Some(Self::Cribado),
            "reunion" => Some(Self::Reunion),
            "propuesta" => Some(Self::Propuesta),
            "cliente" => Some(Self::Cliente),
            _ => None,
        }
    }

    /// Texto que se persiste en PostgreSQL.
    pub fn como_texto(&self) -> &'static str {
        match self {
            Self::Nueva => "nueva",
            Self::Cribado => "cribado",
            Self::Reunion => "reunion",
            Self::Propuesta => "propuesta",
            Self::Cliente => "cliente",
        }
    }

    /// Probabilidad estimada de cierre por etapa.
    pub fn probabilidad(&self) -> u8 {
        match self {
            Self::Nueva => 10,
            Self::Cribado => 25,
            Self::Reunion => 50,
            Self::Propuesta => 75,
            Self::Cliente => 100,
        }
    }

    /// Avanza una etapa (el kanban usa este orden).
    pub fn siguiente(&self) -> Self {
        match self {
            Self::Nueva => Self::Cribado,
            Self::Cribado => Self::Reunion,
            Self::Reunion => Self::Propuesta,
            Self::Propuesta => Self::Cliente,
            Self::Cliente => Self::Cliente,
        }
    }
}

/// Oportunidad comercial.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Oportunidad {
    pub id: Uuid,
    pub nombre: String,
    pub importe: i64,
    pub etapa: EtapaOportunidad,
    pub empresa_id: Option<Uuid>,
    pub contacto_id: Option<Uuid>,
    pub fecha_cierre: Option<DateTime<Utc>>,
    pub creado_en: DateTime<Utc>,
}

/// Resumen numérico del panel principal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResumenPanel {
    pub total_empresas: i64,
    pub total_personas: i64,
    pub total_oportunidades: i64,
    pub valor_tuberia: i64,
    pub valor_ganado: i64,
    pub tasa_exito: f32,
}

/// Datos para crear una empresa (entrada de API y formularios).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrearEmpresa {
    pub nombre: String,
    pub dominio: String,
    pub sector: String,
    pub empleados: i32,
    pub ciudad: String,
}

/// Datos para crear una persona.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrearPersona {
    pub nombre: String,
    pub apellido: String,
    pub correo: String,
    pub telefono: String,
    pub puesto: String,
    pub empresa_id: Option<Uuid>,
    pub ciudad: String,
}

/// Datos para crear una oportunidad.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrearOportunidad {
    pub nombre: String,
    pub importe: i64,
    pub etapa: EtapaOportunidad,
    pub empresa_id: Option<Uuid>,
}

/// Calcula el resumen del panel a partir de listas en memoria.
/// La API en producción lo calcula en PostgreSQL con `panel_resumen()`.
pub fn calcular_resumen(
    empresas: &[Empresa],
    personas: &[Persona],
    oportunidades: &[Oportunidad],
) -> ResumenPanel {
    let valor_tuberia: i64 = oportunidades
        .iter()
        .filter(|o| o.etapa != EtapaOportunidad::Cliente)
        .map(|o| o.importe)
        .sum();
    let valor_ganado: i64 = oportunidades
        .iter()
        .filter(|o| o.etapa == EtapaOportunidad::Cliente)
        .map(|o| o.importe)
        .sum();
    let total = oportunidades.len().max(1) as f32;
    let ganadas = oportunidades
        .iter()
        .filter(|o| o.etapa == EtapaOportunidad::Cliente)
        .count() as f32;
    ResumenPanel {
        total_empresas: empresas.len() as i64,
        total_personas: personas.len() as i64,
        total_oportunidades: oportunidades.len() as i64,
        valor_tuberia,
        valor_ganado,
        tasa_exito: ganadas / total * 100.0,
    }
}
