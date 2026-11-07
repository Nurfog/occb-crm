//! Búsqueda global, favoritos, vistas y actividad del usuario autenticado.

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{auth::UsuarioActual, errores::ErrorApi};

#[derive(Debug, Deserialize)]
pub struct ParametrosBusqueda {
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CuerpoFavorito {
    pub tipo: String,
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ParametrosVistas {
    pub objeto: String,
}

#[derive(Debug, Deserialize)]
pub struct CuerpoVista {
    pub objeto: Option<String>,
    pub nombre: String,
    pub tipo: String,
    #[serde(default = "objeto_json_vacio")]
    pub filtros: serde_json::Value,
    #[serde(default = "objeto_json_vacio")]
    pub orden: serde_json::Value,
}

fn objeto_json_vacio() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Deserialize)]
pub struct ParametrosActividad {
    pub tipo: String,
    pub id: Uuid,
    pub limite: Option<i32>,
}

/// GET /api/buscar?q= — resultados globales, máximo 20.
pub async fn buscar(
    State(pool): State<PgPool>,
    Query(params): Query<ParametrosBusqueda>,
) -> Result<Json<Vec<occb_core::ResultadoBusqueda>>, ErrorApi> {
    let filas = sqlx::query("select * from buscar_espacio($1)")
        .bind(params.q.unwrap_or_default())
        .fetch_all(&pool)
        .await?;
    Ok(Json(
        filas
            .iter()
            .map(|f| occb_core::ResultadoBusqueda {
                tipo: f.get("tipo"),
                id: f.get("id"),
                titulo: f.get("titulo"),
                detalle: f.get("detalle"),
                puntaje: f.get("puntaje"),
            })
            .collect(),
    ))
}

/// GET /api/favoritos — favoritos del usuario del JWT, no del request.
pub async fn favoritos(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
) -> Result<Json<Vec<occb_core::Favorito>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_favoritos($1)")
        .bind(usuario.id)
        .fetch_all(&pool)
        .await?;
    Ok(Json(
        filas
            .iter()
            .map(|f| occb_core::Favorito {
                tipo: f.get("tipo"),
                id: f.get("id"),
                titulo: f.get("titulo"),
                detalle: f.get("detalle"),
                creado_en: f.get("creado_en"),
            })
            .collect(),
    ))
}

/// POST /api/favoritos/alternar.
pub async fn alternar_favorito(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
    Json(cuerpo): Json<CuerpoFavorito>,
) -> Result<Json<serde_json::Value>, ErrorApi> {
    let agregado: bool = sqlx::query_scalar("select alternar_favorito($1,$2,$3)")
        .bind(usuario.id)
        .bind(&cuerpo.tipo)
        .bind(cuerpo.id)
        .fetch_one(&pool)
        .await?;
    Ok(Json(serde_json::json!({"agregado": agregado})))
}

/// GET /api/vistas?objeto=alumnos — vistas propias.
pub async fn listar_vistas(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
    Query(params): Query<ParametrosVistas>,
) -> Result<Json<Vec<occb_core::VistaGuardada>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_vistas($1,$2)")
        .bind(usuario.id)
        .bind(&params.objeto)
        .fetch_all(&pool)
        .await?;
    Ok(Json(filas.iter().map(fila_a_vista).collect()))
}

/// POST /api/vistas — crea una vista personal.
pub async fn crear_vista(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
    Query(params): Query<ParametrosVistas>,
    Json(cuerpo): Json<CuerpoVista>,
) -> Result<Json<occb_core::VistaGuardada>, ErrorApi> {
    let objeto = cuerpo.objeto.as_deref().unwrap_or(&params.objeto);
    let fila = sqlx::query("select * from crear_vista($1,$2,$3,$4,$5,$6)")
        .bind(usuario.id)
        .bind(objeto)
        .bind(&cuerpo.nombre)
        .bind(&cuerpo.tipo)
        .bind(&cuerpo.filtros)
        .bind(&cuerpo.orden)
        .fetch_one(&pool)
        .await?;
    Ok(Json(fila_a_vista(&fila)))
}

/// PATCH /api/vistas/:id — solo puede modificar la vista del usuario actual.
pub async fn actualizar_vista(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoVista>,
) -> Result<Json<occb_core::VistaGuardada>, ErrorApi> {
    let fila = sqlx::query("select * from actualizar_vista($1,$2,$3,$4,$5,$6)")
        .bind(usuario.id)
        .bind(id)
        .bind(&cuerpo.nombre)
        .bind(&cuerpo.tipo)
        .bind(&cuerpo.filtros)
        .bind(&cuerpo.orden)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Vista no encontrada.".into()))?;
    Ok(Json(fila_a_vista(&fila)))
}

/// DELETE /api/vistas/:id.
pub async fn eliminar_vista(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ErrorApi> {
    sqlx::query("select eliminar_vista($1,$2)")
        .bind(usuario.id)
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Vista no encontrada.".into()))?;
    Ok(Json(serde_json::json!({"eliminada": true})))
}

/// GET /api/actividad?tipo=&id= — timeline del registro.
pub async fn actividad(
    State(pool): State<PgPool>,
    Query(params): Query<ParametrosActividad>,
) -> Result<Json<Vec<occb_core::Actividad>>, ErrorApi> {
    let filas = sqlx::query("select * from timeline($1,$2,$3)")
        .bind(&params.tipo)
        .bind(params.id)
        .bind(params.limite.unwrap_or(50))
        .fetch_all(&pool)
        .await?;
    Ok(Json(
        filas
            .iter()
            .map(|f| occb_core::Actividad {
                id: f.get("id"),
                tipo: f.get("tipo"),
                referencia_id: f.get("referencia_id"),
                accion: f.get("accion"),
                detalle: f.get("detalle"),
                usuario_id: f.get("usuario_id"),
                autor: f.get("autor"),
                creado_en: f.get("creado_en"),
            })
            .collect(),
    ))
}

fn fila_a_vista(f: &sqlx::postgres::PgRow) -> occb_core::VistaGuardada {
    occb_core::VistaGuardada {
        id: f.get("id"),
        objeto: f.get("objeto"),
        nombre: f.get("nombre"),
        tipo: f.get("tipo"),
        filtros: f.get("filtros"),
        orden: f.get("orden"),
        creado_en: f.get("creado_en"),
        actualizado_en: f.get("actualizado_en"),
    }
}
