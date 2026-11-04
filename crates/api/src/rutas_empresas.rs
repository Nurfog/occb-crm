//! Rutas de empresas: handlers delgados que validan y llaman a funciones SQL.

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

/// Parámetros de listado: búsqueda, límite y desplazamiento.
#[derive(Debug, Deserialize)]
pub struct ParametrosLista {
    pub busqueda: Option<String>,
    pub limite: Option<i32>,
    pub desplazamiento: Option<i32>,
}

/// Cuerpo para crear/actualizar empresa, con validación.
#[derive(Debug, Deserialize, Validate)]
pub struct CuerpoEmpresa {
    #[validate(length(min = 1, max = 200, message = "El nombre de la empresa es obligatorio."))]
    pub nombre: String,
    pub dominio: Option<String>,
    pub sector: Option<String>,
    pub empleados: Option<i32>,
    pub ciudad: Option<String>,
}

/// GET /api/empresas — solo llama a `listar_empresas()`.
pub async fn listar(
    State(pool): State<PgPool>,
    Query(params): Query<ParametrosLista>,
) -> Result<Json<Vec<occb_core::Empresa>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_empresas($1, $2, $3)")
        .bind(params.busqueda.unwrap_or_default())
        .bind(params.limite.unwrap_or(50))
        .bind(params.desplazamiento.unwrap_or(0))
        .fetch_all(&pool)
        .await?;
    Ok(Json(filas.iter().map(bd::fila_a_empresa).collect()))
}

/// GET /api/empresas/:id — solo llama a `obtener_empresa()`.
pub async fn obtener(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<occb_core::Empresa>, ErrorApi> {
    let fila = sqlx::query("select * from obtener_empresa($1)")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Empresa no encontrada.".into()))?;
    Ok(Json(bd::fila_a_empresa(&fila)))
}

/// POST /api/empresas — valida en Rust y delega a `crear_empresa()`.
pub async fn crear(
    State(pool): State<PgPool>,
    Json(cuerpo): Json<CuerpoEmpresa>,
) -> Result<(StatusCode, Json<occb_core::Empresa>), ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    let entrada = occb_core::validar_empresa(occb_core::CrearEmpresa {
        nombre: cuerpo.nombre,
        dominio: cuerpo.dominio.unwrap_or_default(),
        sector: cuerpo.sector.unwrap_or_default(),
        empleados: cuerpo.empleados.unwrap_or(0),
        ciudad: cuerpo.ciudad.unwrap_or_default(),
    })?;
    let fila = sqlx::query(
        "select * from crear_empresa($1, $2, $3, $4, $5)",
    )
    .bind(&entrada.nombre)
    .bind(&entrada.dominio)
    .bind(&entrada.sector)
    .bind(entrada.empleados)
    .bind(&entrada.ciudad)
    .fetch_one(&pool)
    .await?;
    Ok((StatusCode::CREATED, Json(bd::fila_a_empresa(&fila))))
}

/// PATCH /api/empresas/:id — valida y llama a `actualizar_empresa()`.
pub async fn actualizar(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoEmpresa>,
) -> Result<Json<occb_core::Empresa>, ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    let entrada = occb_core::validar_empresa(occb_core::CrearEmpresa {
        nombre: cuerpo.nombre,
        dominio: cuerpo.dominio.unwrap_or_default(),
        sector: cuerpo.sector.unwrap_or_default(),
        empleados: cuerpo.empleados.unwrap_or(0),
        ciudad: cuerpo.ciudad.unwrap_or_default(),
    })?;
    let fila = sqlx::query(
        "select * from actualizar_empresa($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(&entrada.nombre)
    .bind(&entrada.dominio)
    .bind(&entrada.sector)
    .bind(entrada.empleados)
    .bind(&entrada.ciudad)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| ErrorApi::NoEncontrado("Empresa no encontrada.".into()))?;
    Ok(Json(bd::fila_a_empresa(&fila)))
}

/// DELETE /api/empresas/:id — llama a `eliminar_empresa()`.
pub async fn eliminar(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ErrorApi> {
    sqlx::query("select eliminar_empresa($1)")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Empresa no encontrada.".into()))?;
    Ok(StatusCode::NO_CONTENT)
}
