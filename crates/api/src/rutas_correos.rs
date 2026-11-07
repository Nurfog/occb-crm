//! Rutas de correos: bandeja vinculada a alumnos, apoderados y postulaciones.

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

/// Parámetros de listado: búsqueda, filtro de no leídos y paginación.
#[derive(Debug, Deserialize)]
pub struct ParametrosLista {
    pub busqueda: Option<String>,
    #[serde(default)]
    pub no_leidos: bool,
    pub limite: Option<i32>,
    pub desplazamiento: Option<i32>,
}

/// Cuerpo para registrar un correo, con validación.
#[derive(Debug, Deserialize, Validate)]
pub struct CuerpoCorreo {
    pub asunto: Option<String>,
    #[validate(length(min = 1, message = "El contenido es obligatorio."))]
    pub cuerpo: String,
    pub remitente: String,
    pub alumno_id: Option<Uuid>,
    pub apoderado_id: Option<Uuid>,
    pub oportunidad_id: Option<Uuid>,
}

/// Cuerpo para marcar leído/no leído.
#[derive(Debug, Deserialize)]
pub struct CuerpoLeido {
    pub leido: Option<bool>,
}

/// GET /api/correos
pub async fn listar(
    State(pool): State<PgPool>,
    Query(params): Query<ParametrosLista>,
) -> Result<Json<Vec<occb_core::Correo>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_correos($1, $2, $3, $4)")
        .bind(params.busqueda.unwrap_or_default())
        .bind(params.no_leidos)
        .bind(params.limite.unwrap_or(50))
        .bind(params.desplazamiento.unwrap_or(0))
        .fetch_all(&pool)
        .await?;
    Ok(Json(filas.iter().map(bd::fila_a_correo).collect()))
}

/// GET /api/correos/:id
pub async fn obtener(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<occb_core::Correo>, ErrorApi> {
    let fila = sqlx::query("select * from obtener_correo($1)")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Correo no encontrado.".into()))?;
    Ok(Json(bd::fila_a_correo(&fila)))
}

/// POST /api/correos
pub async fn crear(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Json(cuerpo): Json<CuerpoCorreo>,
) -> Result<(StatusCode, Json<occb_core::Correo>), ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    let entrada = occb_core::validar_correo(occb_core::CrearCorreo {
        asunto: cuerpo.asunto.clone().unwrap_or_default(),
        cuerpo: cuerpo.cuerpo.clone(),
        remitente: cuerpo.remitente.clone(),
        alumno_id: cuerpo.alumno_id,
        apoderado_id: cuerpo.apoderado_id,
        oportunidad_id: cuerpo.oportunidad_id,
    })?;
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query("select * from crear_correo($1, $2, $3, $4, $5, $6)")
        .bind(&entrada.asunto)
        .bind(&entrada.cuerpo)
        .bind(&entrada.remitente)
        .bind(entrada.alumno_id)
        .bind(entrada.apoderado_id)
        .bind(entrada.oportunidad_id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(bd::fila_a_correo(&fila))))
}

/// PATCH /api/correos/:id/leido
pub async fn marcar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoLeido>,
) -> Result<Json<occb_core::Correo>, ErrorApi> {
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query("select * from marcar_correo_leido($1, $2)")
        .bind(id)
        .bind(cuerpo.leido.unwrap_or(true))
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Correo no encontrado.".into()))?;
    tx.commit().await?;
    Ok(Json(bd::fila_a_correo(&fila)))
}

/// DELETE /api/correos/:id
pub async fn eliminar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ErrorApi> {
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    sqlx::query("select eliminar_correo($1)")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Correo no encontrado.".into()))?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
