//! Rutas de colegios: directorio oficial con búsqueda por región/comuna.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::PgPool;
use validator::Validate;

use crate::{bd, errores::ErrorApi};

/// Filtros del directorio: región, comuna, texto, paginación.
#[derive(Debug, Deserialize)]
pub struct FiltrosColegios {
    pub region: Option<i16>,
    pub comuna: Option<String>,
    pub busqueda: Option<String>,
    pub limite: Option<i32>,
    pub desplazamiento: Option<i32>,
}

/// Cuerpo para registrar un colegio manualmente.
#[derive(Debug, Deserialize, Validate)]
pub struct CuerpoColegio {
    #[validate(length(min = 1, message = "El RBD es obligatorio."))]
    pub rbd: String,
    #[validate(length(min = 1, max = 200, message = "El nombre del colegio es obligatorio."))]
    pub nombre: String,
    pub region: i16,
    pub comuna: String,
    pub dependencia: Option<String>,
    pub dv: Option<String>,
}

/// GET /api/colegios — busca en el directorio (ideal para traslados).
pub async fn listar(
    State(pool): State<PgPool>,
    Query(filtros): Query<FiltrosColegios>,
) -> Result<Json<Vec<occb_core::Colegio>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_colegios($1, $2, $3, $4, $5)")
        .bind(filtros.region)
        .bind(filtros.comuna.unwrap_or_default())
        .bind(filtros.busqueda.unwrap_or_default())
        .bind(filtros.limite.unwrap_or(50))
        .bind(filtros.desplazamiento.unwrap_or(0))
        .fetch_all(&pool)
        .await?;
    Ok(Json(filas.iter().map(bd::fila_a_colegio).collect()))
}

/// GET /api/colegios/:rbd
pub async fn obtener(
    State(pool): State<PgPool>,
    Path(rbd): Path<String>,
) -> Result<Json<occb_core::Colegio>, ErrorApi> {
    let fila = sqlx::query("select * from obtener_colegio($1)")
        .bind(&rbd)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Colegio no encontrado.".into()))?;
    Ok(Json(bd::fila_a_colegio(&fila)))
}

/// POST /api/colegios — registro manual con validación.
pub async fn crear(
    State(pool): State<PgPool>,
    Json(cuerpo): Json<CuerpoColegio>,
) -> Result<(StatusCode, Json<occb_core::Colegio>), ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    let entrada = occb_core::validar_colegio(occb_core::CrearColegio {
        rbd: cuerpo.rbd,
        nombre: cuerpo.nombre,
        region: cuerpo.region,
        comuna: cuerpo.comuna,
        dependencia: cuerpo.dependencia.unwrap_or_default(),
        dv: cuerpo.dv.unwrap_or_default(),
    })?;
    let fila = sqlx::query("select * from crear_colegio($1, $2, $3, $4, $5, $6)")
        .bind(&entrada.rbd)
        .bind(&entrada.nombre)
        .bind(entrada.region)
        .bind(&entrada.comuna)
        .bind(&entrada.dependencia)
        .bind(&entrada.dv)
        .fetch_one(&pool)
        .await?;
    Ok((StatusCode::CREATED, Json(bd::fila_a_colegio(&fila))))
}

/// DELETE /api/colegios/:rbd
pub async fn eliminar(
    State(pool): State<PgPool>,
    Path(rbd): Path<String>,
) -> Result<StatusCode, ErrorApi> {
    sqlx::query("select eliminar_colegio($1)")
        .bind(&rbd)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Colegio no encontrado.".into()))?;
    Ok(StatusCode::NO_CONTENT)
}
