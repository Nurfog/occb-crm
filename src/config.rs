//! URL base del API del backend `occb`.
//!
//! Se fija en tiempo de compilación con la variable de entorno `API_BASE_URL`
//! (ver `.env.example`). Siempre debe ser absoluta: `reqwest` no acepta
//! URLs relativas, ni siquiera en wasm.

/// Valor por defecto: backend occb en desarrollo local.
pub const DEFAULT_API_BASE_URL: &str = "http://127.0.0.1:8000";

/// URL base del API, sin `/` final.
pub fn api_base_url() -> String {
    let raw = option_env!("API_BASE_URL").unwrap_or(DEFAULT_API_BASE_URL);
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        DEFAULT_API_BASE_URL.to_string()
    } else {
        trimmed.to_string()
    }
}

/// Une la base con una ruta `/api/v1/...`.
pub fn endpoint(path: &str) -> String {
    format!("{}{}", api_base_url(), path)
}
