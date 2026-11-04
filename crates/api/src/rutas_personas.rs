//! Rutas de personas: handlers delgados que llaman solo a funciones SQL.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{bd, errores::ErrorApi};

/// Parámetros de listado.
#[derive(Debug, Deserialize)]
pub struct ParametrosLista {
    pub busqueda: Option<String>,
    pub limite: Option<i32>,
    pub desplazamiento: Option<i32>,
}

/// Cuerpo para crear persona, con validación.
#[derive(Debug, Deserialize, Validate)]
pub struct CuerpoPersona {
    #[validate(length(min = 1, message = "El nombre es obligatorio."))]
    pub nombre: String,
    #[validate(length(min = 1, message = "Los apellidos son obligatorios."))]
    pub apellido: String,
    #[validate(email(message = "El correo electrónico no tiene un formato válido."))]
    pub correo: String,
    pub telefono: Option<String>,
    pub puesto: Option<String>,
    pub empresa_id: Option<Uuid>,
    pub ciudad: Option<String>,
}

/// GET /api/personas
pub async fn listar(
    State(pool): State<PgPool>,
    Query(params): Query<ParametrosLista>,
) -> Result<Json<Vec<occb_core::Persona>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_personas($1, $2, $3)")
        .bind(params.busqueda.unwrap_or_default())
        .bind(params.limite.unwrap_or(50))
        .bind(params.desplazamiento.unwrap_or(0))
        .fetch_all(&pool)
        .await?;
    Ok(Json(filas.iter().map(bd::fila_a_persona).collect()))
}

/// GET /api/personas/:id
pub async fn obtener(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<occb_core::Persona>, ErrorApi> {
    let fila = sqlx::query("select * from obtener_persona($1)")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Persona no encontrada.".into()))?;
    Ok(Json(bd::fila_a_persona(&fila)))
}

/// POST /api/personas
pub async fn crear(
    State(pool): State<PgPool>,
    Json(cuerpo): Json<CuerpoPersona>,
) -> Result<(StatusCode, Json<occb_core::Persona>), ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    let entrada = occb_core::validar_persona(occb_core::CrearPersona {
        nombre: cuerpo.nombre,
        apellido: cuerpo.apellido,
        correo: cuerpo.correo,
        telefono: cuerpo.telefono.unwrap_or_default(),
        puesto: cuerpo.puesto.unwrap_or_default(),
        empresa_id: cuerpo.empresa_id,
        ciudad: cuerpo.ciudad.unwrap_or_default(),
    })?;
    let fila = sqlx::query(
        "select * from crear_persona($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&entrada.nombre)
    .bind(&entrada.apellido)
    .bind(&entrada.correo)
    .bind(&entrada.telefono)
    .bind(&entrada.puesto)
    .bind(entrada.empresa_id)
    .bind(&entrada.ciudad)
    .fetch_one(&pool)
    .await?;
    Ok((StatusCode::CREATED, Json(bd::fila_a_persona(&fila))))
}

/// DELETE /api/personas/:id
pub async fn eliminar(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ErrorApi> {
    sqlx::query("select eliminar_persona($1)")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Persona no encontrada.".into()))?;
    Ok(StatusCode::NO_CONTENT)
}
