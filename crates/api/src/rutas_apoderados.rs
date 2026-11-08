//! Rutas de apoderados: handlers delgados que llaman solo a funciones SQL.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::{PgPool, Row};
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

/// Cuerpo para registrar/actualizar apoderado, con validación.
#[derive(Debug, Deserialize, Validate)]
pub struct CuerpoApoderado {
    #[validate(length(min = 1, message = "El RUT es obligatorio."))]
    pub rut: String,
    #[validate(length(min = 1, max = 120, message = "Los nombres son obligatorios."))]
    pub nombres: String,
    #[validate(length(min = 1, max = 120, message = "Los apellidos son obligatorios."))]
    pub apellidos: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub direccion: Option<String>,
    pub region_codigo: Option<i16>,
    pub comuna_codigo: Option<String>,
    pub ocupacion: Option<String>,
}

/// GET /api/apoderados
pub async fn listar(
    State(pool): State<PgPool>,
    Query(params): Query<ParametrosLista>,
) -> Result<Json<Vec<occb_core::Apoderado>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_apoderados($1, $2, $3)")
        .bind(params.busqueda.unwrap_or_default())
        .bind(params.limite.unwrap_or(50))
        .bind(params.desplazamiento.unwrap_or(0))
        .fetch_all(&pool)
        .await?;
    Ok(Json(filas.iter().map(bd::fila_a_apoderado).collect()))
}

/// GET /api/apoderados/:id
pub async fn obtener(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<occb_core::Apoderado>, ErrorApi> {
    let fila = sqlx::query("select * from obtener_apoderado($1)")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Apoderado no encontrado.".into()))?;
    Ok(Json(bd::fila_a_apoderado(&fila)))
}

fn a_dominio(cuerpo: &CuerpoApoderado) -> Result<occb_core::CrearApoderado, ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    Ok(occb_core::validar_apoderado(occb_core::CrearApoderado {
        rut: cuerpo.rut.clone(),
        nombres: cuerpo.nombres.clone(),
        apellidos: cuerpo.apellidos.clone(),
        telefono: cuerpo.telefono.clone().unwrap_or_default(),
        correo: cuerpo.correo.clone().unwrap_or_default(),
        direccion: cuerpo.direccion.clone().unwrap_or_default(),
        region_codigo: cuerpo.region_codigo,
        comuna_codigo: cuerpo.comuna_codigo.clone(),
        ocupacion: cuerpo.ocupacion.clone().unwrap_or_default(),
    })?)
}

/// POST /api/apoderados
pub async fn crear(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Json(cuerpo): Json<CuerpoApoderado>,
) -> Result<(StatusCode, Json<occb_core::Apoderado>), ErrorApi> {
    let entrada = a_dominio(&cuerpo)?;
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query("select * from crear_apoderado($1, $2, $3, $4, $5, $6, $7, $8, $9)")
        .bind(&entrada.rut)
        .bind(&entrada.nombres)
        .bind(&entrada.apellidos)
        .bind(&entrada.telefono)
        .bind(&entrada.correo)
        .bind(&entrada.direccion)
        .bind(entrada.region_codigo)
        .bind(&entrada.comuna_codigo)
        .bind(&entrada.ocupacion)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(bd::fila_a_apoderado(&fila))))
}

/// PATCH /api/apoderados/:id
pub async fn actualizar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoApoderado>,
) -> Result<Json<occb_core::Apoderado>, ErrorApi> {
    let entrada = a_dominio(&cuerpo)?;
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query("select * from actualizar_apoderado($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
        .bind(id)
        .bind(&entrada.rut)
        .bind(&entrada.nombres)
        .bind(&entrada.apellidos)
        .bind(&entrada.telefono)
        .bind(&entrada.correo)
        .bind(&entrada.direccion)
        .bind(entrada.region_codigo)
        .bind(&entrada.comuna_codigo)
        .bind(&entrada.ocupacion)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Apoderado no encontrado.".into()))?;
    tx.commit().await?;
    Ok(Json(bd::fila_a_apoderado(&fila)))
}

/// DELETE /api/apoderados/:id
pub async fn eliminar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ErrorApi> {
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    sqlx::query("select eliminar_apoderado($1)")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Apoderado no encontrado.".into()))?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Alumnos vinculados a un apoderado.
#[derive(Debug, serde::Serialize)]
pub struct AlumnoVinculado {
    pub id: Uuid,
    pub rut: String,
    pub nombres: String,
    pub apellidos: String,
    pub curso: String,
    pub estado: String,
    pub vinculo: String,
}

/// GET /api/apoderados/:id/alumnos
pub async fn alumnos_de(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AlumnoVinculado>>, ErrorApi> {
    let filas = sqlx::query("select * from alumnos_de_apoderado($1)")
        .bind(id)
        .fetch_all(&pool)
        .await?;
    Ok(Json(
        filas
            .iter()
            .map(|f| AlumnoVinculado {
                id: f.get("id"),
                rut: f.get("rut"),
                nombres: f.get("nombres"),
                apellidos: f.get("apellidos"),
                curso: f.get::<String, _>("curso"),
                estado: f.get::<String, _>("estado"),
                vinculo: f.get::<String, _>("vinculo"),
            })
            .collect(),
    ))
}
