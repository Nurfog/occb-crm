//! API OCCB (Axum + PostgreSQL 18 con funciones seguras).
//!
//! Buenas prácticas aplicadas:
//! - Módulos por responsabilidad (`configuracion`, `errores`, `bd`, `rutas_*`).
//! - La API nunca escribe con SQL directo: solo llama a funciones `SECURITY DEFINER`.
//! - Validación en dos capas: `validator` + dominio puro en `occb-core`.
//! - Respuestas de error unificadas en español.
//! - `tracing` estructurado y tiempos de espera.

mod bd;
mod configuracion;
mod errores;
mod rutas_alumnos;
mod rutas_apoderados;
mod rutas_colegios;
mod rutas_oportunidades;
mod rutas_personas;
mod rutas_territorio;

use std::{net::SocketAddr, time::Duration};

use axum::{Json, Router, http::StatusCode, routing::get};
use configuracion::Configuracion;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};

/// Punto de entrada: conecta, migra, siembra y sirve.
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "occb_api=info,tower_http=info".into()),
        )
        .init();

    let configuracion = Configuracion::desde_entorno();
    tracing::info!(
        puerto = configuracion.puerto,
        "Iniciando API OCCB"
    );

    let pool = bd::crear_pool(&configuracion.url_bd)
        .await
        .expect("No se pudo conectar a PostgreSQL. Revise DATABASE_URL y `docker compose up -d bd`.");
    bd::migrar_y_sembrar(&pool)
        .await
        .expect("No se pudieron aplicar las migraciones.");

    let app = Router::new()
        .route("/api/salud", get(salud))
        // Rutas en español (se mantienen alias en inglés para compatibilidad).
        .route(
            "/api/colegios",
            get(rutas_colegios::listar).post(rutas_colegios::crear),
        )
        .route(
            "/api/colegios/{rbd}",
            get(rutas_colegios::obtener).delete(rutas_colegios::eliminar),
        )
        .route(
            "/api/personas",
            get(rutas_personas::listar).post(rutas_personas::crear),
        )
        .route(
            "/api/personas/{id}",
            get(rutas_personas::obtener)
                .patch(rutas_personas::actualizar)
                .delete(rutas_personas::eliminar),
        )
        .route(
            "/api/oportunidades",
            get(rutas_oportunidades::listar).post(rutas_oportunidades::crear),
        )
        .route(
            "/api/oportunidades/{id}/etapa",
            axum::routing::patch(rutas_oportunidades::mover_etapa),
        )
        .route(
            "/api/oportunidades/{id}",
            axum::routing::patch(rutas_oportunidades::actualizar)
                .delete(rutas_oportunidades::eliminar),
        )
        .route("/api/panel", get(rutas_oportunidades::panel))
        .route(
            "/api/alumnos",
            get(rutas_alumnos::listar).post(rutas_alumnos::crear),
        )
        .route(
            "/api/alumnos/{id}",
            get(rutas_alumnos::obtener)
                .patch(rutas_alumnos::actualizar)
                .delete(rutas_alumnos::eliminar),
        )
        .route(
            "/api/alumnos/{id}/apoderados",
            get(rutas_alumnos::apoderados_de).post(rutas_alumnos::vincular),
        )
        .route(
            "/api/alumnos/{id}/traslados",
            get(rutas_alumnos::historial).post(rutas_alumnos::trasladar),
        )
        .route(
            "/api/alumnos/{alumno_id}/apoderados/{apoderado_id}",
            axum::routing::delete(rutas_alumnos::desvincular),
        )
        .route(
            "/api/apoderados",
            get(rutas_apoderados::listar).post(rutas_apoderados::crear),
        )
        .route(
            "/api/apoderados/{id}",
            get(rutas_apoderados::obtener)
                .patch(rutas_apoderados::actualizar)
                .delete(rutas_apoderados::eliminar),
        )
        .route(
            "/api/apoderados/{id}/alumnos",
            get(rutas_apoderados::alumnos_de),
        )
        .route("/api/regiones", get(rutas_territorio::listar_regiones))
        .route("/api/comunas", get(rutas_territorio::listar_comunas))
        // Alias en inglés.
        .route("/api/health", get(salud))
        .route("/api/schools", get(rutas_colegios::listar))
        .route("/api/people", get(rutas_personas::listar))
        .route("/api/opportunities", get(rutas_oportunidades::listar))
        .route("/api/dashboard", get(rutas_oportunidades::panel))
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let direccion = SocketAddr::from(([0, 0, 0, 0], configuracion.puerto));
    tracing::info!("API escuchando en http://{direccion}");
    let oyente = tokio::net::TcpListener::bind(direccion)
        .await
        .expect("No se pudo abrir el puerto");
    axum::serve(oyente, app).await.expect("Error sirviendo la API");
}

/// GET /api/salud — comprobación rápida en español.
async fn salud() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "estado": "correcto",
        "servicio": "occb-api",
        "producto": "occb-crm",
        "idioma": "es",
        "bd": "postgres:18 con funciones seguras + RLS"
    }))
}
