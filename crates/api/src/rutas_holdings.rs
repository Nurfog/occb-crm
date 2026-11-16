//! Tenancy SaaS (M1): holdings y colegios contratados.
//! Lecturas filtradas por membresía en SQL; escrituras exigen admin (Rust).
//! Aditivo: no bloquea rutas existentes ni cambia el flujo legado.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{auth::UsuarioActual, errores::ErrorApi};

/// Fila pública de holding (sin datos internos).
#[derive(Debug, Serialize)]
pub struct HoldingSalida {
    pub id: Uuid,
    pub nombre: String,
    pub plan: String,
    pub colegios: i64,
    pub puede_ver: bool,
}

/// Colegio vinculado a un holding con datos del directorio.
#[derive(Debug, Serialize)]
pub struct ContratadoSalida {
    pub rbd: String,
    pub nombre: String,
    pub licencia: String,
    pub dominio: String,
}

#[derive(Debug, Deserialize)]
pub struct CuerpoHolding {
    pub nombre: String,
    pub plan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CuerpoVinculo {
    pub rbd: String,
    pub licencia: Option<String>,
    pub dominio: Option<String>,
}

fn exigir_admin(usuario: &UsuarioActual) -> Result<(), ErrorApi> {
    if usuario.rol != "admin" {
        return Err(ErrorApi::Prohibido(
            "Solo un admin puede gestionar holdings.".into(),
        ));
    }
    Ok(())
}

/// GET /api/holdings — todos si admin, solo miembros si no.
pub async fn listar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
) -> Result<Json<Vec<HoldingSalida>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_holdings($1)")
        .bind(usuario.id)
        .fetch_all(&pool)
        .await?;
    Ok(Json(
        filas
            .iter()
            .map(|f| HoldingSalida {
                id: f.get("id"),
                nombre: f.get("nombre"),
                plan: f.get("plan"),
                colegios: f.get("n_colegios"),
                puede_ver: f.get("es_miembro"),
            })
            .collect(),
    ))
}

/// POST /api/holdings — admin crea holding y queda como miembro global.
pub async fn crear(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
    Json(cuerpo): Json<CuerpoHolding>,
) -> Result<(StatusCode, Json<HoldingSalida>), ErrorApi> {
    exigir_admin(&usuario)?;
    let nombre = cuerpo.nombre.trim().to_string();
    if nombre.is_empty() {
        return Err(ErrorApi::Validacion("El nombre del holding es obligatorio.".into()));
    }
    let plan = cuerpo.plan.unwrap_or_else(|| "small".to_string()).to_lowercase();
    if !matches!(plan.as_str(), "small" | "group" | "holding") {
        return Err(ErrorApi::Validacion("Plan no válido: small, group o holding.".into()));
    }
    let fila = sqlx::query("select * from crear_holding($1, $2, $3)")
        .bind(usuario.id)
        .bind(&nombre)
        .bind(&plan)
        .fetch_one(&pool)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(HoldingSalida {
            id: fila.get("id"),
            nombre: fila.get("nombre"),
            plan: fila.get("plan"),
            colegios: 0,
            puede_ver: true,
        }),
    ))
}

/// GET /api/holdings/:id/colegios — directorio de los contratados.
pub async fn colegios(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ContratadoSalida>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_colegios_contratados($1, $2)")
        .bind(usuario.id)
        .bind(id)
        .fetch_all(&pool)
        .await?;
    Ok(Json(
        filas
            .iter()
            .map(|f| ContratadoSalida {
                rbd: f.get("rbd"),
                nombre: f.get("nombre"),
                licencia: f.get("licencia"),
                dominio: f.get("dominio"),
            })
            .collect(),
    ))
}

/// POST /api/holdings/:id/colegios — admin vincula (upsert) un RBD.
pub async fn vincular(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoVinculo>,
) -> Result<(StatusCode, Json<ContratadoSalida>), ErrorApi> {
    exigir_admin(&usuario)?;
    let rbd = cuerpo.rbd.trim().to_string();
    if rbd.is_empty() {
        return Err(ErrorApi::Validacion("El RBD es obligatorio.".into()));
    }
    let licencia = cuerpo.licencia.unwrap_or_else(|| "small".to_string()).to_lowercase();
    if !matches!(licencia.as_str(), "small" | "group" | "holding") {
        return Err(ErrorApi::Validacion("Licencia no válida.".into()));
    }
    let dominio = cuerpo.dominio.unwrap_or_default();
    let fila = sqlx::query("select * from vincular_colegio($1, $2, $3, $4)")
        .bind(id)
        .bind(&rbd)
        .bind(&licencia)
        .bind(&dominio)
        .fetch_one(&pool)
        .await?;
    let nombre: String = sqlx::query_scalar("select nombre from colegios where rbd = $1")
        .bind(&rbd)
        .fetch_one(&pool)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(ContratadoSalida {
            rbd: fila.get("rbd"),
            nombre,
            licencia: fila.get("licencia"),
            dominio: fila.get("dominio"),
        }),
    ))
}

/// DELETE /api/holdings/:id — admin elimina (cascada).
pub async fn eliminar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ErrorApi> {
    exigir_admin(&usuario)?;
    sqlx::query_scalar::<_, bool>("select eliminar_holding($1)")
        .bind(id)
        .fetch_one(&pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
