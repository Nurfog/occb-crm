//! Rutas de tareas: handlers delgados que llaman solo a funciones SQL.

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

/// Parámetros de listado: búsqueda, estado y paginación.
#[derive(Debug, Deserialize)]
pub struct ParametrosLista {
    pub busqueda: Option<String>,
    pub estado: Option<String>,
    pub limite: Option<i32>,
    pub desplazamiento: Option<i32>,
}

/// Cuerpo para crear/actualizar tarea, con validación.
#[derive(Debug, Deserialize, Validate)]
pub struct CuerpoTarea {
    #[validate(length(min = 1, max = 200, message = "El título es obligatorio."))]
    pub titulo: String,
    pub detalle: Option<String>,
    pub estado: Option<String>,
    pub prioridad: Option<String>,
    pub vencimiento: Option<NaiveDate>,
    pub alumno_id: Option<Uuid>,
    pub apoderado_id: Option<Uuid>,
    pub oportunidad_id: Option<Uuid>,
}

/// GET /api/tareas
pub async fn listar(
    State(pool): State<PgPool>,
    Query(params): Query<ParametrosLista>,
) -> Result<Json<Vec<occb_core::Tarea>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_tareas($1, $2, $3, $4)")
        .bind(params.busqueda.unwrap_or_default())
        .bind(params.estado.unwrap_or_default())
        .bind(params.limite.unwrap_or(50))
        .bind(params.desplazamiento.unwrap_or(0))
        .fetch_all(&pool)
        .await?;
    Ok(Json(filas.iter().map(bd::fila_a_tarea).collect()))
}

/// GET /api/tareas/:id
pub async fn obtener(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<occb_core::Tarea>, ErrorApi> {
    let fila = sqlx::query("select * from obtener_tarea($1)")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Tarea no encontrada.".into()))?;
    Ok(Json(bd::fila_a_tarea(&fila)))
}

fn a_dominio(cuerpo: &CuerpoTarea) -> Result<occb_core::CrearTarea, ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    let estado = cuerpo.estado.as_deref().unwrap_or("pendiente");
    let estado_enum = occb_core::EstadoTarea::desde_texto(&estado.to_lowercase()).ok_or_else(|| {
        ErrorApi::Validacion("Estado no válido. Use: pendiente, en_curso o hecha.".into())
    })?;
    Ok(occb_core::validar_tarea(occb_core::CrearTarea {
        titulo: cuerpo.titulo.clone(),
        detalle: cuerpo.detalle.clone().unwrap_or_default(),
        estado: estado_enum,
        prioridad: cuerpo.prioridad.clone().unwrap_or_else(|| "media".into()),
        vencimiento: cuerpo.vencimiento,
        alumno_id: cuerpo.alumno_id,
        apoderado_id: cuerpo.apoderado_id,
        oportunidad_id: cuerpo.oportunidad_id,
    })?)
}

/// POST /api/tareas
pub async fn crear(
    State(pool): State<PgPool>,
    Json(cuerpo): Json<CuerpoTarea>,
) -> Result<(StatusCode, Json<occb_core::Tarea>), ErrorApi> {
    let entrada = a_dominio(&cuerpo)?;
    let fila = sqlx::query("select * from crear_tarea($1, $2, $3, $4, $5, $6, $7, $8)")
        .bind(&entrada.titulo)
        .bind(&entrada.detalle)
        .bind(entrada.estado.como_texto())
        .bind(&entrada.prioridad)
        .bind(entrada.vencimiento)
        .bind(entrada.alumno_id)
        .bind(entrada.apoderado_id)
        .bind(entrada.oportunidad_id)
        .fetch_one(&pool)
        .await?;
    Ok((StatusCode::CREATED, Json(bd::fila_a_tarea(&fila))))
}

/// PATCH /api/tareas/:id
pub async fn actualizar(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoTarea>,
) -> Result<Json<occb_core::Tarea>, ErrorApi> {
    let entrada = a_dominio(&cuerpo)?;
    let fila = sqlx::query("select * from actualizar_tarea($1, $2, $3, $4, $5, $6)")
        .bind(id)
        .bind(&entrada.titulo)
        .bind(&entrada.detalle)
        .bind(entrada.estado.como_texto())
        .bind(&entrada.prioridad)
        .bind(entrada.vencimiento)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Tarea no encontrada.".into()))?;
    Ok(Json(bd::fila_a_tarea(&fila)))
}

/// Cuerpo para avanzar estado.
#[derive(Debug, Deserialize)]
pub struct CuerpoEstado {
    pub estado: String,
}

/// PATCH /api/tareas/:id/estado
pub async fn marcar(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoEstado>,
) -> Result<Json<occb_core::Tarea>, ErrorApi> {
    let fila = sqlx::query("select * from marcar_tarea($1, $2)")
        .bind(id)
        .bind(cuerpo.estado.to_lowercase())
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Tarea no encontrada.".into()))?;
    Ok(Json(bd::fila_a_tarea(&fila)))
}

/// DELETE /api/tareas/:id
pub async fn eliminar(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ErrorApi> {
    sqlx::query("select eliminar_tarea($1)")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Tarea no encontrada.".into()))?;
    Ok(StatusCode::NO_CONTENT)
}
