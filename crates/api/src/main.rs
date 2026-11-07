//! API OCCB (Axum + PostgreSQL 18 con funciones seguras).
//!
//! Buenas prácticas aplicadas:
//! - Módulos por responsabilidad (`configuracion`, `errores`, `bd`, `rutas_*`).
//! - La API nunca escribe con SQL directo: solo llama a funciones `SECURITY DEFINER`.
//! - Validación en dos capas: `validator` + dominio puro en `occb-core`.
//! - Respuestas de error unificadas en español.
//! - `tracing` estructurado y tiempos de espera.

mod auth;
mod bd;
mod configuracion;
mod errores;
mod rutas_alumnos;
mod rutas_automatizaciones;
mod rutas_apoderados;
mod rutas_colegios;
mod rutas_correos;
mod rutas_eventos;
mod rutas_espacio;
mod rutas_notas;
mod rutas_oportunidades;
mod rutas_personas;
mod rutas_tareas;
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

    // Públicas: salud, acceso y directorio de referencia.
    let publicas = Router::new()
        .route("/api/salud", get(salud))
        .route("/api/health", get(salud))
        .route("/api/acceso/registro", axum::routing::post(auth::registro))
        .route("/api/acceso/login", axum::routing::post(auth::login))
        .route("/api/acceso/renovar", axum::routing::post(auth::renovar))
        .route("/api/acceso/salir", axum::routing::post(auth::salir))
        .route("/api/regiones", get(rutas_territorio::listar_regiones))
        .route("/api/comunas", get(rutas_territorio::listar_comunas))
        .route(
            "/api/colegios",
            get(rutas_colegios::listar).post(rutas_colegios::crear),
        )
        .route(
            "/api/colegios/{rbd}",
            get(rutas_colegios::obtener).delete(rutas_colegios::eliminar),
        )
        .route("/api/schools", get(rutas_colegios::listar))
        .route("/api/panel", get(rutas_oportunidades::panel))
        .route("/api/dashboard", get(rutas_oportunidades::panel));

    // Protegidas: exigen JWT (ver Fase 3 del roadmap).
    let protegidas = Router::new()
        .route("/api/acceso/yo", get(auth::yo))
        .route("/api/buscar", get(rutas_espacio::buscar))
        .route("/api/favoritos", get(rutas_espacio::favoritos))
        .route(
            "/api/favoritos/alternar",
            axum::routing::post(rutas_espacio::alternar_favorito),
        )
        .route(
            "/api/vistas",
            get(rutas_espacio::listar_vistas).post(rutas_espacio::crear_vista),
        )
        .route(
            "/api/vistas/{id}",
            axum::routing::patch(rutas_espacio::actualizar_vista)
                .delete(rutas_espacio::eliminar_vista),
        )
        .route("/api/actividad", get(rutas_espacio::actividad))
        .route(
            "/api/automatizaciones",
            get(rutas_automatizaciones::listar).post(rutas_automatizaciones::crear),
        )
        .route(
            "/api/automatizaciones/{id}/activa",
            axum::routing::patch(rutas_automatizaciones::alternar),
        )
        .route(
            "/api/automatizaciones/{id}/ejecuciones",
            get(rutas_automatizaciones::ejecuciones),
        )
        .route(
            "/api/automatizaciones/{id}",
            get(rutas_automatizaciones::obtener)
                .patch(rutas_automatizaciones::actualizar)
                .delete(rutas_automatizaciones::eliminar),
        )
        .route("/api/miembros", get(auth::miembros))
        .route(
            "/api/miembros/{id}/rol",
            axum::routing::patch(auth::cambiar_rol),
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
        .route(
            "/api/tareas",
            get(rutas_tareas::listar).post(rutas_tareas::crear),
        )
        .route(
            "/api/tareas/{id}",
            get(rutas_tareas::obtener)
                .patch(rutas_tareas::actualizar)
                .delete(rutas_tareas::eliminar),
        )
        .route(
            "/api/tareas/{id}/estado",
            axum::routing::patch(rutas_tareas::marcar),
        )
        .route(
            "/api/notas",
            get(rutas_notas::listar).post(rutas_notas::crear),
        )
        .route(
            "/api/notas/{id}",
            get(rutas_notas::obtener)
                .patch(rutas_notas::actualizar)
                .delete(rutas_notas::eliminar),
        )
        .route(
            "/api/eventos",
            get(rutas_eventos::listar).post(rutas_eventos::crear),
        )
        .route(
            "/api/eventos/mes",
            get(rutas_eventos::listar_mes),
        )
        .route(
            "/api/eventos/{id}",
            get(rutas_eventos::obtener)
                .patch(rutas_eventos::actualizar)
                .delete(rutas_eventos::eliminar),
        )
        .route(
            "/api/correos",
            get(rutas_correos::listar).post(rutas_correos::crear),
        )
        .route(
            "/api/correos/{id}",
            get(rutas_correos::obtener).delete(rutas_correos::eliminar),
        )
        .route(
            "/api/correos/{id}/leido",
            axum::routing::patch(rutas_correos::marcar),
        )
        // Alias en inglés (protegidos igual que sus pares).
        .route("/api/people", get(rutas_personas::listar))
        .route("/api/opportunities", get(rutas_oportunidades::listar))
        .route_layer(axum::middleware::from_fn(auth::exigir_sesion));

    let app = publicas
        .merge(protegidas)
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
