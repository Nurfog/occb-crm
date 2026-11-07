//! Rutas de eventos: agenda escolar vinculada a alumnos, apoderados y postulaciones.

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

/// Parámetros del mes: año y mes (1 a 12).
#[derive(Debug, Deserialize)]
pub struct ParametrosMes {
    pub ano: Option<i32>,
    pub mes: Option<i32>,
}

/// Cuerpo para crear/actualizar evento, con validación.
#[derive(Debug, Deserialize, Validate)]
pub struct CuerpoEvento {
    #[validate(length(min = 1, max = 200, message = "El título es obligatorio."))]
    pub titulo: String,
    pub inicio: chrono::DateTime<chrono::Utc>,
    pub fin: chrono::DateTime<chrono::Utc>,
    pub lugar: Option<String>,
    pub descripcion: Option<String>,
    pub alumno_id: Option<Uuid>,
    pub apoderado_id: Option<Uuid>,
    pub oportunidad_id: Option<Uuid>,
}

/// GET /api/eventos
pub async fn listar(
    State(pool): State<PgPool>,
    Query(params): Query<ParametrosLista>,
) -> Result<Json<Vec<occb_core::Evento>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_eventos($1, $2, $3)")
        .bind(params.busqueda.unwrap_or_default())
        .bind(params.limite.unwrap_or(50))
        .bind(params.desplazamiento.unwrap_or(0))
        .fetch_all(&pool)
        .await?;
    Ok(Json(filas.iter().map(bd::fila_a_evento).collect()))
}

/// GET /api/eventos/mes?ano=&mes=
pub async fn listar_mes(
    State(pool): State<PgPool>,
    Query(params): Query<ParametrosMes>,
) -> Result<Json<Vec<occb_core::Evento>>, ErrorApi> {
    let ano = params
        .ano
        .ok_or_else(|| ErrorApi::Validacion("El año es obligatorio.".into()))?;
    let mes = params
        .mes
        .ok_or_else(|| ErrorApi::Validacion("El mes es obligatorio (1 a 12).".into()))?;
    let filas = sqlx::query("select * from listar_eventos_mes($1, $2)")
        .bind(ano)
        .bind(mes)
        .fetch_all(&pool)
        .await?;
    Ok(Json(filas.iter().map(bd::fila_a_evento).collect()))
}

/// GET /api/eventos/:id
pub async fn obtener(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<occb_core::Evento>, ErrorApi> {
    let fila = sqlx::query("select * from obtener_evento($1)")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Evento no encontrado.".into()))?;
    Ok(Json(bd::fila_a_evento(&fila)))
}

fn a_dominio(cuerpo: &CuerpoEvento) -> Result<occb_core::CrearEvento, ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    Ok(occb_core::validar_evento(occb_core::CrearEvento {
        titulo: cuerpo.titulo.clone(),
        inicio: cuerpo.inicio,
        fin: cuerpo.fin,
        lugar: cuerpo.lugar.clone().unwrap_or_default(),
        descripcion: cuerpo.descripcion.clone().unwrap_or_default(),
        alumno_id: cuerpo.alumno_id,
        apoderado_id: cuerpo.apoderado_id,
        oportunidad_id: cuerpo.oportunidad_id,
    })?)
}

/// POST /api/eventos
pub async fn crear(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Json(cuerpo): Json<CuerpoEvento>,
) -> Result<(StatusCode, Json<occb_core::Evento>), ErrorApi> {
    let entrada = a_dominio(&cuerpo)?;
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query("select * from crear_evento($1, $2, $3, $4, $5, $6, $7, $8)")
        .bind(&entrada.titulo)
        .bind(entrada.inicio)
        .bind(entrada.fin)
        .bind(&entrada.lugar)
        .bind(&entrada.descripcion)
        .bind(entrada.alumno_id)
        .bind(entrada.apoderado_id)
        .bind(entrada.oportunidad_id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(bd::fila_a_evento(&fila))))
}

/// PATCH /api/eventos/:id
pub async fn actualizar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoEvento>,
) -> Result<Json<occb_core::Evento>, ErrorApi> {
    let entrada = a_dominio(&cuerpo)?;
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query("select * from actualizar_evento($1, $2, $3, $4, $5, $6)")
        .bind(id)
        .bind(&entrada.titulo)
        .bind(entrada.inicio)
        .bind(entrada.fin)
        .bind(&entrada.lugar)
        .bind(&entrada.descripcion)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Evento no encontrado.".into()))?;
    tx.commit().await?;
    Ok(Json(bd::fila_a_evento(&fila)))
}

/// DELETE /api/eventos/:id
pub async fn eliminar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ErrorApi> {
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    sqlx::query("select eliminar_evento($1)")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Evento no encontrado.".into()))?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
