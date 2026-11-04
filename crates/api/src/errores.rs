//! Errores de la API con respuestas JSON en español.
//! Formato único: `{ "codigo": "...", "mensaje": "..." }`.

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use thiserror::Error;

/// Error unificado de la API.
#[derive(Debug, Error)]
pub enum ErrorApi {
    /// Recurso no encontrado (404).
    #[error("No encontrado: {0}")]
    NoEncontrado(String),
    /// Datos de entrada inválidos (400).
    #[error("Datos inválidos: {0}")]
    Validacion(String),
    /// Error de base de datos (400 si es regla de negocio, 500 si es interno).
    #[error("Error de base de datos: {0}")]
    BaseDatos(#[from] sqlx::Error),
}

impl IntoResponse for ErrorApi {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::NoEncontrado(detalle) => (
                StatusCode::NOT_FOUND,
                Json(json!({"codigo": "no_encontrado", "mensaje": detalle})),
            )
                .into_response(),
            Self::Validacion(detalle) => (
                StatusCode::BAD_REQUEST,
                Json(json!({"codigo": "datos_invalidos", "mensaje": detalle})),
            )
                .into_response(),
            Self::BaseDatos(error) => respuesta_bd(error),
        }
    }
}

/// Traduce errores de PostgreSQL a mensajes en español.
fn respuesta_bd(error: sqlx::Error) -> axum::response::Response {
    if let sqlx::Error::Database(base) = &error {
        let mensaje = base.message().to_string();
        let codigo = base.code().unwrap_or_default().to_string();
        // P0002 = no encontrado, P0001/P0003 = regla de negocio.
        if codigo == "P0002" {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"codigo": "no_encontrado", "mensaje": mensaje})),
            )
                .into_response();
        }
        if codigo == "P0001" || codigo == "P0003" {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"codigo": "datos_invalidos", "mensaje": mensaje})),
            )
                .into_response();
        }
        tracing::error!(?error, "Error de base de datos no controlado");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"codigo": "error_interno", "mensaje": "Error interno. Inténtelo de nuevo."})),
        )
            .into_response();
    }
    tracing::error!(?error, "Error de base de datos no controlado");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"codigo": "error_interno", "mensaje": "Error interno. Inténtelo de nuevo."})),
    )
        .into_response()
}

/// Convierte errores de dominio (validación pura) a errores de API.
impl From<occb_core::ErrorDominio> for ErrorApi {
    fn from(error: occb_core::ErrorDominio) -> Self {
        Self::Validacion(error.to_string())
    }
}
