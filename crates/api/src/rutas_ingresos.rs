//! Rutas de ingresos médicos: historial 1:N por alumno.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{bd, errores::ErrorApi};

/// Cuerpo para crear/actualizar un ingreso médico.
#[derive(Debug, Deserialize, Validate)]
pub struct CuerpoIngreso {
    pub alumno_id: Option<Uuid>,
    pub fecha: Option<NaiveDate>,
    #[validate(length(min = 1, max = 200, message = "El motivo es obligatorio."))]
    pub motivo: String,
    pub detalle: Option<String>,
    pub profesional: Option<String>,
}

/// GET /api/ingresos?alumno_id=
pub async fn listar(
    State(pool): State<PgPool>,
    Query(params): Query<std::collections::HashMap<String, Uuid>>,
) -> Result<Json<Vec<occb_core::IngresoMedico>>, ErrorApi> {
    let alumno_id = params.get("alumno_id").copied().ok_or_else(|| {
        ErrorApi::Validacion("El alumno es obligatorio.".into())
    })?;
    let filas = sqlx::query("select * from listar_ingresos($1)")
        .bind(alumno_id)
        .fetch_all(&pool)
        .await?;
    Ok(Json(filas.iter().map(bd::fila_a_ingreso).collect()))
}

/// POST /api/ingresos
pub async fn crear(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Json(cuerpo): Json<CuerpoIngreso>,
) -> Result<(StatusCode, Json<occb_core::IngresoMedico>), ErrorApi> {
    let entrada = a_dominio(&cuerpo)?;
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query("select * from crear_ingreso($1, $2, $3, $4, $5)")
        .bind(entrada.alumno_id)
        .bind(entrada.fecha)
        .bind(&entrada.motivo)
        .bind(&entrada.detalle)
        .bind(&entrada.profesional)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(bd::fila_a_ingreso(&fila))))
}

/// PATCH /api/ingresos/:id
pub async fn actualizar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoIngreso>,
) -> Result<Json<occb_core::IngresoMedico>, ErrorApi> {
    let entrada = a_dominio_sin_dueno(&cuerpo)?;
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query("select * from actualizar_ingreso($1, $2, $3, $4, $5)")
        .bind(id)
        .bind(entrada.fecha)
        .bind(&entrada.motivo)
        .bind(&entrada.detalle)
        .bind(&entrada.profesional)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Ingreso no encontrado.".into()))?;
    tx.commit().await?;
    Ok(Json(bd::fila_a_ingreso(&fila)))
}

/// DELETE /api/ingresos/:id
pub async fn eliminar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ErrorApi> {
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    sqlx::query("select eliminar_ingreso($1)")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Ingreso no encontrado.".into()))?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

fn a_dominio(cuerpo: &CuerpoIngreso) -> Result<occb_core::CrearIngreso, ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    let alumno_id = cuerpo.alumno_id.ok_or_else(|| {
        ErrorApi::Validacion("El alumno es obligatorio.".into())
    })?;
    Ok(occb_core::validar_ingreso(occb_core::CrearIngreso {
        alumno_id,
        fecha: cuerpo.fecha,
        motivo: cuerpo.motivo.clone(),
        detalle: cuerpo.detalle.clone().unwrap_or_default(),
        profesional: cuerpo.profesional.clone().unwrap_or_default(),
    })?)
}

fn a_dominio_sin_dueno(cuerpo: &CuerpoIngreso) -> Result<occb_core::CrearIngreso, ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    Ok(occb_core::validar_ingreso(occb_core::CrearIngreso {
        alumno_id: cuerpo.alumno_id.unwrap_or_default(),
        fecha: cuerpo.fecha,
        motivo: cuerpo.motivo.clone(),
        detalle: cuerpo.detalle.clone().unwrap_or_default(),
        profesional: cuerpo.profesional.clone().unwrap_or_default(),
    })?)
}
