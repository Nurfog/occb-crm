//! Rutas de alumnos: handlers delgados que llaman solo a funciones SQL.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{bd, errores::ErrorApi};

/// Parámetros de listado: búsqueda, estado, colegio y paginación.
#[derive(Debug, Deserialize)]
pub struct ParametrosLista {
    pub busqueda: Option<String>,
    pub estado: Option<String>,
    pub colegio_rbd: Option<String>,
    pub limite: Option<i32>,
    pub desplazamiento: Option<i32>,
}

/// Cuerpo para matricular/actualizar alumno, con validación.
#[derive(Debug, Deserialize, Validate)]
pub struct CuerpoAlumno {
    #[validate(length(min = 1, message = "El RUT es obligatorio."))]
    pub rut: String,
    #[validate(length(min = 1, max = 120, message = "Los nombres son obligatorios."))]
    pub nombres: String,
    #[validate(length(min = 1, max = 120, message = "Los apellidos son obligatorios."))]
    pub apellidos: String,
    pub fecha_nacimiento: Option<NaiveDate>,
    pub curso: Option<String>,
    pub colegio_rbd: Option<String>,
    pub estado: Option<String>,
}

/// GET /api/alumnos
pub async fn listar(
    State(pool): State<PgPool>,
    Query(params): Query<ParametrosLista>,
) -> Result<Json<Vec<occb_core::Alumno>>, ErrorApi> {
    let filas = sqlx::query("select * from listar_alumnos($1, $2, $3, $4, $5)")
        .bind(params.busqueda.unwrap_or_default())
        .bind(params.estado.unwrap_or_default())
        .bind(params.colegio_rbd.unwrap_or_default())
        .bind(params.limite.unwrap_or(50))
        .bind(params.desplazamiento.unwrap_or(0))
        .fetch_all(&pool)
        .await?;
    Ok(Json(filas.iter().map(bd::fila_a_alumno).collect()))
}

/// GET /api/alumnos/:id
pub async fn obtener(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<occb_core::Alumno>, ErrorApi> {
    let fila = sqlx::query("select * from obtener_alumno($1)")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Alumno no encontrado.".into()))?;
    Ok(Json(bd::fila_a_alumno(&fila)))
}

fn a_dominio(cuerpo: &CuerpoAlumno) -> Result<occb_core::CrearAlumno, ErrorApi> {
    cuerpo
        .validate()
        .map_err(|e| ErrorApi::Validacion(e.to_string()))?;
    let etapa = cuerpo.estado.as_deref().unwrap_or("postulante");
    let estado = occb_core::EstadoAlumno::desde_texto(&etapa.to_lowercase()).ok_or_else(|| {
        ErrorApi::Validacion(
            "Estado no válido. Use: postulante, matriculado, trasladado o retirado.".into(),
        )
    })?;
    Ok(occb_core::validar_alumno(occb_core::CrearAlumno {
        rut: cuerpo.rut.clone(),
        nombres: cuerpo.nombres.clone(),
        apellidos: cuerpo.apellidos.clone(),
        fecha_nacimiento: cuerpo.fecha_nacimiento,
        curso: cuerpo.curso.clone().unwrap_or_default(),
        colegio_rbd: cuerpo.colegio_rbd.clone(),
        estado,
    })?)
}

/// POST /api/alumnos — valida en Rust y delega a `crear_alumno()`.
pub async fn crear(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Json(cuerpo): Json<CuerpoAlumno>,
) -> Result<(StatusCode, Json<occb_core::Alumno>), ErrorApi> {
    let entrada = a_dominio(&cuerpo)?;
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query("select * from crear_alumno($1, $2, $3, $4, $5, $6, $7)")
        .bind(&entrada.rut)
        .bind(&entrada.nombres)
        .bind(&entrada.apellidos)
        .bind(entrada.fecha_nacimiento)
        .bind(&entrada.curso)
        .bind(entrada.colegio_rbd.clone())
        .bind(entrada.estado.como_texto())
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(bd::fila_a_alumno(&fila))))
}

/// PATCH /api/alumnos/:id — valida y llama a `actualizar_alumno()`.
pub async fn actualizar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoAlumno>,
) -> Result<Json<occb_core::Alumno>, ErrorApi> {
    let entrada = a_dominio(&cuerpo)?;
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query("select * from actualizar_alumno($1, $2, $3, $4, $5, $6, $7, $8)")
        .bind(id)
        .bind(&entrada.rut)
        .bind(&entrada.nombres)
        .bind(&entrada.apellidos)
        .bind(entrada.fecha_nacimiento)
        .bind(&entrada.curso)
        .bind(entrada.colegio_rbd.clone())
        .bind(entrada.estado.como_texto())
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Alumno no encontrado.".into()))?;
    tx.commit().await?;
    Ok(Json(bd::fila_a_alumno(&fila)))
}

/// DELETE /api/alumnos/:id
pub async fn eliminar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ErrorApi> {
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    sqlx::query("select eliminar_alumno($1)")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Alumno no encontrado.".into()))?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Apoderados vinculados a un alumno (con vínculo y permisos).
#[derive(Debug, serde::Serialize)]
pub struct ApoderadoVinculado {
    pub id: Uuid,
    pub rut: String,
    pub nombres: String,
    pub apellidos: String,
    pub telefono: String,
    pub correo: String,
    pub vinculo: String,
    pub retiro_autorizado: bool,
    pub notificaciones: bool,
}

/// GET /api/alumnos/:id/apoderados
pub async fn apoderados_de(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ApoderadoVinculado>>, ErrorApi> {
    use sqlx::Row;
    let filas = sqlx::query("select * from apoderados_de_alumno($1)")
        .bind(id)
        .fetch_all(&pool)
        .await?;
    Ok(Json(
        filas
            .iter()
            .map(|f| ApoderadoVinculado {
                id: f.get("id"),
                rut: f.get("rut"),
                nombres: f.get("nombres"),
                apellidos: f.get("apellidos"),
                telefono: f.get::<String, _>("telefono"),
                correo: f.get::<String, _>("correo"),
                vinculo: f.get::<String, _>("vinculo"),
                retiro_autorizado: f.get("retiro_autorizado"),
                notificaciones: f.get("notificaciones"),
            })
            .collect(),
    ))
}

/// Cuerpo para trasladar un alumno a otro colegio.
#[derive(Debug, Deserialize)]
pub struct CuerpoTraslado {
    pub destino_rbd: String,
    pub motivo: Option<String>,
}

/// POST /api/alumnos/:id/traslados — traslada y registra historial.
pub async fn trasladar(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<crate::auth::UsuarioActual>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoTraslado>,
) -> Result<Json<occb_core::Alumno>, ErrorApi> {
    let mut tx = bd::transaccion_con_actor(&pool, usuario.id).await?;
    let fila = sqlx::query("select * from trasladar_alumno($1, $2, $3)")
        .bind(id)
        .bind(cuerpo.destino_rbd.trim())
        .bind(cuerpo.motivo.clone().unwrap_or_default())
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Alumno no encontrado.".into()))?;
    tx.commit().await?;
    Ok(Json(bd::fila_a_alumno(&fila)))
}

/// GET /api/alumnos/:id/traslados — historial del alumno.
pub async fn historial(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<occb_core::Traslado>>, ErrorApi> {
    let filas = sqlx::query("select * from traslados_de_alumno($1)")
        .bind(id)
        .fetch_all(&pool)
        .await?;
    Ok(Json(filas.iter().map(bd::fila_a_traslado).collect()))
}
/// Cuerpo para vincular un apoderado existente a un alumno.
#[derive(Debug, Deserialize)]
pub struct CuerpoVinculo {
    pub apoderado_id: Uuid,
    pub vinculo: Option<String>,
    pub retiro_autorizado: Option<bool>,
    pub notificaciones: Option<bool>,
}

/// POST /api/alumnos/:id/apoderados — vincula uno existente.
pub async fn vincular(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(cuerpo): Json<CuerpoVinculo>,
) -> Result<(StatusCode, Json<serde_json::Value>), ErrorApi> {
    sqlx::query("select vincular_apoderado($1, $2, $3, $4, $5)")
        .bind(id)
        .bind(cuerpo.apoderado_id)
        .bind(cuerpo.vinculo.clone().unwrap_or_default())
        .bind(cuerpo.retiro_autorizado.unwrap_or(false))
        .bind(cuerpo.notificaciones.unwrap_or(true))
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Alumno o apoderado inexistente.".into()))?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"vinculado": true}))))
}

/// DELETE /api/alumnos/:alumno_id/apoderados/:apoderado_id
pub async fn desvincular(
    State(pool): State<PgPool>,
    Path((alumno_id, apoderado_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ErrorApi> {
    sqlx::query("select desvincular_apoderado($1, $2)")
        .bind(alumno_id)
        .bind(apoderado_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Vínculo no encontrado.".into()))?;
    Ok(StatusCode::NO_CONTENT)
}
