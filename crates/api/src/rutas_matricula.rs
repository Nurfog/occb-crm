//! Matriculación atómica desde postulación (030, SaaS M3).
//! Aditivo: no toca `mover_etapa` ni `crear_alumno`; expone la función
//! `matricular_postulacion` como `POST /api/oportunidades/:id/matricular`
//! (protegida, idempotente natural por oportunidad + RUT).

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{bd, errores::ErrorApi};

/// Cuerpo para matricular: solo el RUT es obligatorio; nombres y curso se
/// derivan de la postulación cuando se omiten.
#[derive(Debug, Deserialize)]
pub struct CuerpoMatricula {
    pub alumno_rut: String,
    pub nombres: Option<String>,
    pub apellidos: Option<String>,
    pub curso: Option<String>,
    pub colegio_rbd: Option<String>,
    pub apoderados_rut: Option<Vec<String>>,
    pub idempotency_key: Option<String>,
}

/// POST /api/oportunidades/:id/matricular
pub async fn matricular(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoMatricula>,
) -> Result<Json<occb_core::Alumno>, ErrorApi> {
    let colegio_scope = usuario.colegio_rbd.first().map(|s| s.as_str());
    // Enforcement incremental (M1): solo exige membresía cuando el JWT ya
    // trae scope (029); tokens legados sin scope siguen pasando (E2E 122).
    if usuario.holding_id.is_some() || !usuario.colegio_rbd.is_empty() {
        let ok = crate::auth::es_miembro(&pool, &usuario.id, usuario.holding_id, colegio_scope).await;
        if !ok {
            return Err(ErrorApi::Prohibido("Sin autorización para este colegio.".into()));
        }
    }
    let mut tx = bd::transaccion_con_actor_con_scope(
        &pool,
        usuario.id,
        usuario.holding_id,
        colegio_scope,
    )
    .await?;
    let alumno_id: Uuid = sqlx::query_scalar(
        "select matricular_postulacion($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(id)
    .bind(cuerpo.alumno_rut)
    .bind(cuerpo.nombres.unwrap_or_default())
    .bind(cuerpo.apellidos.unwrap_or_default())
    .bind(cuerpo.curso.unwrap_or_else(|| "1 básico".into()))
    .bind(cuerpo.colegio_rbd)
    .bind(cuerpo.apoderados_rut.unwrap_or_default())
    .bind(cuerpo.idempotency_key)
    .fetch_one(&mut *tx)
    .await?;
    let fila = sqlx::query("select * from obtener_alumno($1)")
        .bind(alumno_id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(bd::fila_a_alumno(&fila)))
}
