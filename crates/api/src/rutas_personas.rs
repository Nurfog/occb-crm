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
    pub ciudad: Option<String>,
    pub region_codigo: Option<i16>,
    pub comuna_codigo: Option<String>,
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
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Json(cuerpo): Json<CuerpoPersona>,
) -> Result<(StatusCode, Json<occb_core::Persona>), ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    let entrada = occb_core::validar_persona(occb_core::CrearPersona {
        nombre: cuerpo.nombre.clone(),
        apellido: cuerpo.apellido.clone(),
        correo: cuerpo.correo.clone(),
        telefono: cuerpo.telefono.clone().unwrap_or_default(),
        puesto: cuerpo.puesto.clone().unwrap_or_default(),
        ciudad: cuerpo.ciudad.clone().unwrap_or_default(),
        region_codigo: cuerpo.region_codigo,
        comuna_codigo: cuerpo.comuna_codigo.clone(),
    })?;
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query(
        "select * from crear_persona($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&entrada.nombre)
    .bind(&entrada.apellido)
    .bind(&entrada.correo)
    .bind(&entrada.telefono)
    .bind(&entrada.puesto)
    .bind(&entrada.ciudad)
    .bind(entrada.region_codigo)
    .bind(&entrada.comuna_codigo)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(bd::fila_a_persona(&fila))))
}

/// DELETE /api/personas/:id
pub async fn eliminar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ErrorApi> {
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    sqlx::query("select eliminar_persona($1)")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Persona no encontrada.".into()))?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// PATCH /api/personas/:id — valida y llama a `actualizar_persona()`.
pub async fn actualizar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoPersona>,
) -> Result<Json<occb_core::Persona>, ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    let entrada = occb_core::validar_persona(occb_core::CrearPersona {
        nombre: cuerpo.nombre.clone(),
        apellido: cuerpo.apellido.clone(),
        correo: cuerpo.correo.clone(),
        telefono: cuerpo.telefono.clone().unwrap_or_default(),
        puesto: cuerpo.puesto.clone().unwrap_or_default(),
        ciudad: cuerpo.ciudad.clone().unwrap_or_default(),
        region_codigo: cuerpo.region_codigo,
        comuna_codigo: cuerpo.comuna_codigo.clone(),
    })?;
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query(
        "select * from actualizar_persona($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(id)
    .bind(&entrada.nombre)
    .bind(&entrada.apellido)
    .bind(&entrada.correo)
    .bind(&entrada.telefono)
    .bind(&entrada.puesto)
    .bind(&entrada.ciudad)
    .bind(entrada.region_codigo)
    .bind(&entrada.comuna_codigo)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ErrorApi::NoEncontrado("Persona no encontrada.".into()))?;
    tx.commit().await?;
    Ok(Json(bd::fila_a_persona(&fila)))
}
