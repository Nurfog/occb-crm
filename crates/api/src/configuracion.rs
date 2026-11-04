//! Configuración de la API leída desde el entorno.
//! Buenas prácticas: valores por defecto para desarrollo y error claro si faltan.

/// Configuración efectiva del servicio.
#[derive(Debug, Clone)]
pub struct Configuracion {
    /// URL de PostgreSQL, p. ej. `postgres://occb:occb@localhost:5432/occb`.
    pub url_bd: String,
    /// Puerto HTTP de la API.
    pub puerto: u16,
}

impl Configuracion {
    /// Lee `DATABASE_URL` y `PUERTO_API` del entorno.
    pub fn desde_entorno() -> Self {
        let url_bd = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://occb:occb@localhost:5432/occb".into());
        let puerto = std::env::var("PUERTO_API")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3001);
        Self { url_bd, puerto }
    }
}
