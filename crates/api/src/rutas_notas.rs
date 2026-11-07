//! Rutas de notas: handlers delgados que llaman solo a funciones SQL.

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

/// Parámetros de listado: búsqueda y paginación.
#[derive(Debug, Deserialize)]
pub struct ParametrosLista {
    pub busqueda: Option<String>,
    pub limite: Option<i32>,
    pub desplazamiento: Option<i32>,
}

/// Cuerpo para crear/actualizar nota, con validación.
#[derive(Debug, Deserialize, Validate)]
pub struct CuerpoNota {
    pub titulo: Option<String>,
    #[validate(length(min = 1, max = 5000, message = "El contenido es obligatorio."))]
    pub cuerpo: String,
    pub alumno_id: Option<Uuid>,
    pub apoderado_id: Option<Uuid>,
    pub oportunidad_id: Option<Uuid>,
}

/// GET /api/notas
pub async fn listar(
    State(pool): State<PgPool>,
    Query(params): Query<ParametrosLista>,
) -> Result<Json<Vec<occb_core::Nota>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_notas($1, $2, $3)")
        .bind(params.busqueda.unwrap_or_default())
        .bind(params.limite.unwrap_or(50))
        .bind(params.desplazamiento.unwrap_or(0))
        .fetch_all(&pool)
        .await?;
    Ok(Json(filas.iter().map(bd::fila_a_nota).collect()))
}

/// GET /api/notas/:id
pub async fn obtener(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<occb_core::Nota>, ErrorApi> {
    let fila = sqlx::query("select * from obtener_nota($1)")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Nota no encontrada.".into()))?;
    Ok(Json(bd::fila_a_nota(&fila)))
}

fn a_dominio(cuerpo: &CuerpoNota) -> Result<occb_core::CrearNota, ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    Ok(occb_core::validar_nota(occb_core::CrearNota {
        titulo: cuerpo.titulo.clone().unwrap_or_default(),
        cuerpo: cuerpo.cuerpo.clone(),
        alumno_id: cuerpo.alumno_id,
        apoderado_id: cuerpo.apoderado_id,
        oportunidad_id: cuerpo.oportunidad_id,
    })?)
}

/// POST /api/notas
pub async fn crear(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Json(cuerpo): Json<CuerpoNota>,
) -> Result<(StatusCode, Json<occb_core::Nota>), ErrorApi> {
    let entrada = a_dominio(&cuerpo)?;
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query("select * from crear_nota($1, $2, $3, $4, $5)")
        .bind(&entrada.titulo)
        .bind(&entrada.cuerpo)
        .bind(entrada.alumno_id)
        .bind(entrada.apoderado_id)
        .bind(entrada.oportunidad_id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(bd::fila_a_nota(&fila))))
}

/// PATCH /api/notas/:id
pub async fn actualizar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoNota>,
) -> Result<Json<occb_core::Nota>, ErrorApi> {
    let entrada = a_dominio(&cuerpo)?;
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query("select * from actualizar_nota($1, $2, $3)")
        .bind(id)
        .bind(&entrada.titulo)
        .bind(&entrada.cuerpo)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Nota no encontrada.".into()))?;
    tx.commit().await?;
    Ok(Json(bd::fila_a_nota(&fila)))
}

/// DELETE /api/notas/:id
pub async fn eliminar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ErrorApi> {
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    sqlx::query("select eliminar_nota($1)")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Nota no encontrada.".into()))?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
