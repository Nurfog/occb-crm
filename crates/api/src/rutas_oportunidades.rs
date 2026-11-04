//! Rutas de oportunidades y panel: solo funciones SQL (`listar_oportunidades`,
//! `crear_oportunidad`, `mover_etapa_oportunidad`, `panel_resumen`).

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

/// Filtros del kanban.
#[derive(Debug, Deserialize)]
pub struct FiltrosOportunidad {
    pub etapa: Option<String>,
    pub limite: Option<i32>,
    pub desplazamiento: Option<i32>,
}

/// Cuerpo para crear oportunidad.
#[derive(Debug, Deserialize, Validate)]
pub struct CuerpoOportunidad {
    #[validate(length(min = 1, message = "El nombre de la oportunidad es obligatorio."))]
    pub nombre: String,
    pub importe: Option<i64>,
    pub etapa: Option<String>,
    pub empresa_id: Option<Uuid>,
}

/// Cuerpo para mover de etapa.
#[derive(Debug, Deserialize)]
pub struct CuerpoEtapa {
    pub etapa: String,
}

/// GET /api/oportunidades
pub async fn listar(
    State(pool): State<PgPool>,
    Query(filtros): Query<FiltrosOportunidad>,
) -> Result<Json<Vec<occb_core::Oportunidad>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_oportunidades($1, $2, $3)")
        .bind(filtros.etapa.unwrap_or_default())
        .bind(filtros.limite.unwrap_or(100))
        .bind(filtros.desplazamiento.unwrap_or(0))
        .fetch_all(&pool)
        .await?;
    Ok(Json(filas.iter().map(bd::fila_a_oportunidad).collect()))
}

/// POST /api/oportunidades
pub async fn crear(
    State(pool): State<PgPool>,
    Json(cuerpo): Json<CuerpoOportunidad>,
) -> Result<(StatusCode, Json<occb_core::Oportunidad>), ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    let etapa = cuerpo.etapa.as_deref().unwrap_or("nueva");
    let etapa_enum = occb_core::EtapaOportunidad::desde_texto(&etapa.to_lowercase())
        .ok_or_else(|| {
            ErrorApi::Validacion(
                "Etapa no válida. Use: nueva, cribado, reunion, propuesta o cliente.".into(),
            )
        })?;
    let entrada = occb_core::validar_oportunidad(occb_core::CrearOportunidad {
        nombre: cuerpo.nombre,
        importe: cuerpo.importe.unwrap_or(0),
        etapa: etapa_enum,
        empresa_id: cuerpo.empresa_id,
    })?;
    let fila = sqlx::query(
        "select * from crear_oportunidad($1, $2, $3, $4)",
    )
    .bind(&entrada.nombre)
    .bind(entrada.importe)
    .bind(entrada.etapa.como_texto())
    .bind(entrada.empresa_id)
    .fetch_one(&pool)
    .await?;
    Ok((StatusCode::CREATED, Json(bd::fila_a_oportunidad(&fila))))
}

/// PATCH /api/oportunidades/:id/etapa
pub async fn mover_etapa(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoEtapa>,
) -> Result<Json<occb_core::Oportunidad>, ErrorApi> {
    let fila = sqlx::query("select * from mover_etapa_oportunidad($1, $2)")
        .bind(id)
        .bind(cuerpo.etapa.to_lowercase())
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Oportunidad no encontrada.".into()))?;
    Ok(Json(bd::fila_a_oportunidad(&fila)))
}

/// DELETE /api/oportunidades/:id
pub async fn eliminar(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ErrorApi> {
    sqlx::query("select eliminar_oportunidad($1)")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Oportunidad no encontrada.".into()))?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/panel — resumen calculado en la base de datos.
pub async fn panel(
    State(pool): State<PgPool>,
) -> Result<Json<occb_core::ResumenPanel>, ErrorApi> {
    let fila = sqlx::query(
        "select total_empresas, total_personas, total_oportunidades, valor_tuberia, valor_ganado, tasa_exito::float8 as tasa_exito from panel_resumen()",
    )
    .fetch_one(&pool)
    .await?;
    use sqlx::Row;
    Ok(Json(occb_core::ResumenPanel {
        total_empresas: fila.get::<i64, _>("total_empresas"),
        total_personas: fila.get::<i64, _>("total_personas"),
        total_oportunidades: fila.get::<i64, _>("total_oportunidades"),
        valor_tuberia: fila.get::<i64, _>("valor_tuberia"),
        valor_ganado: fila.get::<i64, _>("valor_ganado"),
        tasa_exito: fila.get::<f64, _>("tasa_exito") as f32,
    }))
}
