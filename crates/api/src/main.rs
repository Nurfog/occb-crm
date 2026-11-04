//! API OCCB: clon de Twenty en Rust (Axum + PostgreSQL 18 con funciones seguras).
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
mod rutas_empresas;
mod rutas_oportunidades;
mod rutas_personas;

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
        "Iniciando API OCCB (clon de Twenty en Rust)"
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
            "/api/empresas",
            get(rutas_empresas::listar).post(rutas_empresas::crear),
        )
        .route(
            "/api/empresas/{id}",
            get(rutas_empresas::obtener)
                .patch(rutas_empresas::actualizar)
                .delete(rutas_empresas::eliminar),
        )
        .route(
            "/api/personas",
            get(rutas_personas::listar).post(rutas_personas::crear),
        )
        .route(
            "/api/personas/{id}",
            get(rutas_personas::obtener).delete(rutas_personas::eliminar),
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
            axum::routing::delete(rutas_oportunidades::eliminar),
        )
        .route("/api/panel", get(rutas_oportunidades::panel))
        // Alias en inglés (compatibilidad con el MVP anterior).
        .route("/api/health", get(salud))
        .route("/api/companies", get(rutas_empresas::listar))
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
        "clon_de": "twenty",
        "idioma": "es",
        "bd": "postgres:18 con funciones seguras + RLS"
    }))
}
