//! Cliente HTTP del API del backend `occb`.
//!
//! Cubre exactamente lo que expone el backend hoy:
//! auth (`register`/`login`/`refresh`/`logout`/`me`), `users` (list/get) y
//! `health`/`ready`/`version`.
//!
//! Los módulos CRM de la referencia `opencrmccb` (contactos, empresas, deals,
//! actividades, …) se añadirán en este archivo cuando el backend los exponga.

use std::sync::OnceLock;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::endpoint;

// ---------------------------------------------------------------------------
// Tipos del API (contrato del backend occb)
// ---------------------------------------------------------------------------

/// Usuario público tal como lo devuelve el backend.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

/// Par de tokens que devuelven `register`, `login` y `refresh`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TokenPair {
    pub user: UserPublic,
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Respuesta paginada de `GET /api/v1/users`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PagedUsers {
    pub data: Vec<UserPublic>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

/// `GET /api/v1/health` — liveness, sin dependencias.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub env: Option<String>,
    pub version: Option<String>,
}

/// `GET /api/v1/ready` — 200 solo si las dependencias responden.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ReadyChecks {
    pub db: Option<String>,
    pub redis: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ReadyStatus {
    pub status: String,
    pub checks: Option<ReadyChecks>,
}

/// `GET /api/v1/version`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VersionInfo {
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize)]
struct RegisterBody<'a> {
    email: &'a str,
    name: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize)]
struct LoginBody<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize)]
struct RefreshBody<'a> {
    refresh_token: &'a str,
}

#[derive(Debug, Serialize)]
struct LogoutBody<'a> {
    refresh_token: &'a str,
}

// ---------------------------------------------------------------------------
// Error del API
// ---------------------------------------------------------------------------

/// Error normalizado. El backend responde `{ "error": { "code", "message" } }`.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiError {
    /// Código HTTP. `0` = fallo de transporte (el backend no respondió).
    pub status: u16,
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn transport(message: String) -> Self {
        Self {
            status: 0,
            code: "transport".to_string(),
            message,
        }
    }

    /// Error local cuando no hay sesión (no se llega a llamar al backend).
    pub fn no_session() -> Self {
        Self {
            status: 401,
            code: "sin_sesion".to_string(),
            message: "Necesitas iniciar sesión para ver esta página.".to_string(),
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.status == 0 {
            write!(f, "No se pudo contactar al backend: {}", self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ApiError {}

fn parse_error_body(body: &serde_json::Value, status: u16) -> (String, String) {
    if let Some(err) = body.get("error") {
        let code = err
            .get("code")
            .and_then(|v| v.as_str())
            .unwrap_or("error")
            .to_string();
        let message = err
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Error del servidor")
            .to_string();
        (code, message)
    } else {
        (
            "http_error".to_string(),
            format!("El servidor respondió HTTP {status}"),
        )
    }
}

async fn into_api_error(res: reqwest::Response) -> ApiError {
    let status = res.status().as_u16();
    let body: serde_json::Value = res.json().await.unwrap_or(serde_json::Value::Null);
    let (code, message) = parse_error_body(&body, status);
    ApiError {
        status,
        code,
        message,
    }
}

// ---------------------------------------------------------------------------
// Cliente
// ---------------------------------------------------------------------------

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn client() -> reqwest::Client {
    CLIENT.get_or_init(reqwest::Client::new).clone()
}

// --- Auth ---------------------------------------------------------------

/// `POST /api/v1/auth/register` → 201 + par de tokens.
pub async fn register(email: &str, name: &str, password: &str) -> Result<TokenPair, ApiError> {
    let res = client()
        .post(endpoint("/api/v1/auth/register"))
        .json(&RegisterBody {
            email,
            name,
            password,
        })
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<TokenPair>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `POST /api/v1/auth/login` → 200 + par de tokens.
/// Con email inexistente o clave errónea el backend devuelve el mismo 401.
pub async fn login(email: &str, password: &str) -> Result<TokenPair, ApiError> {
    let res = client()
        .post(endpoint("/api/v1/auth/login"))
        .json(&LoginBody { email, password })
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<TokenPair>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `POST /api/v1/auth/refresh` — rota el refresh token (el anterior se revoca).
pub async fn refresh(refresh_token: &str) -> Result<TokenPair, ApiError> {
    let res = client()
        .post(endpoint("/api/v1/auth/refresh"))
        .json(&RefreshBody { refresh_token })
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<TokenPair>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `POST /api/v1/auth/logout` — requiere `Authorization: Bearer <access>`
/// y el `refresh_token` en el cuerpo. Responde 204.
pub async fn logout(access_token: &str, refresh_token: &str) -> Result<(), ApiError> {
    let res = client()
        .post(endpoint("/api/v1/auth/logout"))
        .bearer_auth(access_token)
        .json(&LogoutBody { refresh_token })
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

/// `GET /api/v1/auth/me` — perfil del dueño del access token.
pub async fn me(access_token: &str) -> Result<UserPublic, ApiError> {
    let res = client()
        .get(endpoint("/api/v1/auth/me"))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<UserPublic>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

// --- Users (protegido) ---------------------------------------------------

/// `GET /api/v1/users?page&per_page&q&sort&dir`.
/// `sort` solo admite `email|name|created_at`; el backend ignora el resto.
pub async fn list_users(
    access_token: &str,
    page: u32,
    per_page: u32,
    q: &str,
    sort: &str,
    dir: &str,
) -> Result<PagedUsers, ApiError> {
    let mut query: Vec<(&str, String)> = vec![
        ("page", page.max(1).to_string()),
        ("per_page", per_page.clamp(1, 100).to_string()),
        ("sort", sort.to_string()),
        ("dir", dir.to_string()),
    ];
    if !q.trim().is_empty() {
        query.push(("q", q.trim().to_string()));
    }
    let res = client()
        .get(endpoint("/api/v1/users"))
        .bearer_auth(access_token)
        .query(&query)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<PagedUsers>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `GET /api/v1/users/{id}`.
pub async fn get_user(access_token: &str, id: &str) -> Result<UserPublic, ApiError> {
    let res = client()
        .get(endpoint(&format!("/api/v1/users/{id}")))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<UserPublic>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

// --- Observabilidad (público) ---------------------------------------------

/// `GET /api/v1/health`.
pub async fn health() -> Result<HealthStatus, ApiError> {
    let res = client()
        .get(endpoint("/api/v1/health"))
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<HealthStatus>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `GET /api/v1/ready` — 200 si Postgres (y Redis si aplica) responden.
pub async fn ready() -> Result<ReadyStatus, ApiError> {
    let res = client()
        .get(endpoint("/api/v1/ready"))
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<ReadyStatus>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `GET /api/v1/version`.
pub async fn version() -> Result<VersionInfo, ApiError> {
    let res = client()
        .get(endpoint("/api/v1/version"))
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<VersionInfo>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}
