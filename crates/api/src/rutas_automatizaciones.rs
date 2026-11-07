//! CRUD de reglas y consulta de ejecuciones.

use axum::{Json, extract::{Path, State}, http::StatusCode};
use serde::Deserialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use validator::Validate;

use crate::errores::ErrorApi;

#[derive(Debug, Deserialize, Validate)]
pub struct CuerpoAutomatizacion {
    #[validate(length(min=1,max=120,message="El nombre es obligatorio (máximo 120 caracteres)."))]
    pub nombre: String,
    pub disparador: String,
    #[serde(default="array_vacio")]
    pub condiciones: serde_json::Value,
    #[serde(default="array_vacio")]
    pub acciones: serde_json::Value,
}

fn array_vacio() -> serde_json::Value { serde_json::json!([]) }

#[derive(Debug, Deserialize)]
pub struct CuerpoActiva { pub activa: bool }

pub async fn listar(State(pool): State<PgPool>) -> Result<Json<Vec<occb_core::Automatizacion>>,ErrorApi> {
    let filas=sqlx::query("select * from listar_automatizaciones()")
        .fetch_all(&pool).await?;
    Ok(Json(filas.iter().map(fila_a_automatizacion).collect()))
}

pub async fn obtener(State(pool):State<PgPool>,Path(id):Path<Uuid>) -> Result<Json<occb_core::Automatizacion>,ErrorApi> {
    let f=sqlx::query("select * from obtener_automatizacion($1)").bind(id)
        .fetch_optional(&pool).await?
        .ok_or_else(||ErrorApi::NoEncontrado("Automatización no encontrada.".into()))?;
    Ok(Json(fila_a_automatizacion(&f)))
}

pub async fn crear(State(pool):State<PgPool>,Json(c):Json<CuerpoAutomatizacion>)
 -> Result<(StatusCode,Json<occb_core::Automatizacion>),ErrorApi> {
    c.validate().map_err(|e|ErrorApi::Validacion(e.to_string()))?;
    let f=sqlx::query("select * from crear_automatizacion($1,$2,$3,$4)")
        .bind(c.nombre.trim()).bind(c.disparador.trim().to_lowercase())
        .bind(c.condiciones).bind(c.acciones).fetch_one(&pool).await?;
    Ok((StatusCode::CREATED,Json(fila_a_automatizacion(&f))))
}

pub async fn actualizar(State(pool):State<PgPool>,Path(id):Path<Uuid>,Json(c):Json<CuerpoAutomatizacion>)
 -> Result<Json<occb_core::Automatizacion>,ErrorApi> {
    c.validate().map_err(|e|ErrorApi::Validacion(e.to_string()))?;
    let f=sqlx::query("select * from actualizar_automatizacion($1,$2,$3,$4,$5)")
        .bind(id).bind(c.nombre.trim()).bind(c.disparador.trim().to_lowercase())
        .bind(c.condiciones).bind(c.acciones).fetch_optional(&pool).await?
        .ok_or_else(||ErrorApi::NoEncontrado("Automatización no encontrada.".into()))?;
    Ok(Json(fila_a_automatizacion(&f)))
}

pub async fn eliminar(State(pool):State<PgPool>,Path(id):Path<Uuid>)
 -> Result<StatusCode,ErrorApi> {
    sqlx::query("select eliminar_automatizacion($1)").bind(id).fetch_optional(&pool).await?
        .ok_or_else(||ErrorApi::NoEncontrado("Automatización no encontrada.".into()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn alternar(State(pool):State<PgPool>,Path(id):Path<Uuid>,Json(c):Json<CuerpoActiva>)
 -> Result<Json<occb_core::Automatizacion>,ErrorApi> {
    let f=sqlx::query("select * from alternar_automatizacion($1,$2)").bind(id).bind(c.activa)
        .fetch_optional(&pool).await?
        .ok_or_else(||ErrorApi::NoEncontrado("Automatización no encontrada.".into()))?;
    Ok(Json(fila_a_automatizacion(&f)))
}

pub async fn ejecuciones(State(pool):State<PgPool>,Path(id):Path<Uuid>)
 -> Result<Json<Vec<occb_core::Ejecucion>>,ErrorApi> {
    let fs=sqlx::query("select * from listar_ejecuciones($1,50)").bind(id).fetch_all(&pool).await?;
    Ok(Json(fs.iter().map(|f|occb_core::Ejecucion{
        id:f.get("id"), automatizacion_id:f.get("automatizacion_id"), registro_id:f.get("registro_id"),
        disparador:f.get("disparador"), ok:f.get("ok"), detalle:f.get("detalle"),
        usuario_id:f.get("usuario_id"), creado_en:f.get("creado_en"),
    }).collect()))
}

fn fila_a_automatizacion(f:&sqlx::postgres::PgRow)->occb_core::Automatizacion{
    occb_core::Automatizacion{
        id:f.get("id"),nombre:f.get("nombre"),activa:f.get("activa"),disparador:f.get("disparador"),
        objeto:f.get("objeto"),condiciones:f.get("condiciones"),acciones:f.get("acciones"),
        creado_en:f.get("creado_en"),actualizado_en:f.get("actualizado_en"),
    }
}
