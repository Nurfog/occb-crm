//! Empresas (042): root de plataforma y asignación de membresías.
//! Lectura: root ve todas; el resto solo sus empresas.
//! Escritura: solo root (403 para el resto).

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{auth::UsuarioActual, errores::ErrorApi};

#[derive(Debug, Serialize)]
pub struct EmpresaSalida {
    pub id: Uuid,
    pub pais_iso: String,
    pub documento_tipo: String,
    pub documento_numero: String,
    pub razon_social: String,
    pub moneda_funcional: String,
    pub activa: bool,
    pub es_mia: bool,
}

#[derive(Debug, Deserialize)]
pub struct CuerpoEmpresa {
    pub razon_social: String,
    #[serde(default)]
    pub pais: Option<String>,
    #[serde(default)]
    pub documento_tipo: Option<String>,
    pub documento: String,
    #[serde(default)]
    pub moneda: Option<String>,
    #[serde(default)]
    pub regimen: Option<String>,
    #[serde(default)]
    pub holding_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CuerpoMembresia {
    pub usuario_id: Uuid,
    #[serde(default)]
    pub colegio_rbd: Option<String>,
}

/// GET /api/empresas
pub async fn listar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
) -> Result<Json<Vec<EmpresaSalida>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_empresas($1)")
        .bind(usuario.id)
        .fetch_all(&pool)
        .await?;
    Ok(Json(
        filas
            .iter()
            .map(|f| EmpresaSalida {
                id: f.get("id"),
                pais_iso: f.get("pais_iso"),
                documento_tipo: f.get("documento_tipo"),
                documento_numero: f.get("documento_numero"),
                razon_social: f.get("razon_social"),
                moneda_funcional: f.get("moneda_funcional"),
                activa: f.get("activa"),
                es_mia: f.get("es_mia"),
            })
            .collect(),
    ))
}

/// POST /api/empresas — solo root.
pub async fn crear(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
    Json(cuerpo): Json<CuerpoEmpresa>,
) -> Result<(StatusCode, Json<EmpresaSalida>), ErrorApi> {
    if !usuario.es_root {
        return Err(ErrorApi::Prohibido(
            "Solo el root de plataforma puede crear empresas.".into(),
        ));
    }
    let fila = sqlx::query("select * from crear_empresa($1,$2,$3,$4,$5,$6,$7,$8)")
        .bind(usuario.id)
        .bind(cuerpo.holding_id)
        .bind(cuerpo.pais.unwrap_or_else(|| "CL".into()))
        .bind(cuerpo.documento_tipo.unwrap_or_else(|| "rut".into()))
        .bind(&cuerpo.documento)
        .bind(&cuerpo.razon_social)
        .bind(cuerpo.moneda.unwrap_or_else(|| "CLP".into()))
        .bind(cuerpo.regimen.unwrap_or_default())
        .fetch_one(&pool)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(EmpresaSalida {
            id: fila.get("id"),
            pais_iso: fila.get("pais_iso"),
            documento_tipo: fila.get("documento_tipo"),
            documento_numero: fila.get("documento_numero"),
            razon_social: fila.get("razon_social"),
            moneda_funcional: fila.get("moneda_funcional"),
            activa: fila.get("activa"),
            es_mia: true,
        }),
    ))
}

/// POST /api/empresas/:id/miembros — root asigna usuario a empresa.
pub async fn asignar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoMembresia>,
) -> Result<StatusCode, ErrorApi> {
    if !usuario.es_root {
        return Err(ErrorApi::Prohibido(
            "Solo el root de plataforma puede asignar membresías.".into(),
        ));
    }
    sqlx::query("select asignar_membresia($1,$2,$3,$4)")
        .bind(usuario.id)
        .bind(cuerpo.usuario_id)
        .bind(id)
        .bind(cuerpo.colegio_rbd)
        .execute(&pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
