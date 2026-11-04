//! Rutas de territorio: listados de referencia para los selectores del frontal.

use axum::{Json, extract::{Query, State}};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::errores::ErrorApi;

/// Región tal como la guarda `regiones`.
#[derive(Debug, Serialize)]
pub struct RegionSalida {
    pub codigo: i16,
    pub nombre: String,
    pub romano: String,
    pub sigla: String,
}

/// Comuna tal como la guarda `comunas`.
#[derive(Debug, Serialize)]
pub struct ComunaSalida {
    pub codigo: String,
    pub nombre: String,
    pub region: i16,
}

/// Filtro opcional por región.
#[derive(Debug, Deserialize)]
pub struct FiltroComunas {
    pub region: Option<i16>,
}

/// GET /api/regiones — las 16, de norte a sur.
pub async fn listar_regiones(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<RegionSalida>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_regiones()")
        .fetch_all(&pool)
        .await?;
    Ok(Json(
        filas
            .iter()
            .map(|f| RegionSalida {
                codigo: f.get("codigo"),
                nombre: f.get("nombre"),
                romano: f.get::<String, _>("romano"),
                sigla: f.get::<String, _>("sigla"),
            })
            .collect(),
    ))
}

/// GET /api/comunas?region=13 — comunas de la región (o error si falta).
pub async fn listar_comunas(
    State(pool): State<PgPool>,
    Query(filtro): Query<FiltroComunas>,
) -> Result<Json<Vec<ComunaSalida>>, ErrorApi> {
    let region = filtro.region.ok_or_else(|| {
        ErrorApi::Validacion("Indique la región (?region=13) para listar comunas.".into())
    })?;
    let filas = sqlx::query("select * from listar_comunas_region($1)")
        .bind(region)
        .fetch_all(&pool)
        .await?;
    Ok(Json(
        filas
            .iter()
            .map(|f| ComunaSalida {
                codigo: f.get("codigo"),
                nombre: f.get("nombre"),
                region: f.get("region"),
            })
            .collect(),
    ))
}
