//! Cliente HTTP del API del backend `occb`.
//!
//! Cubre auth (`register`/`login`/`refresh`/`logout`/`me`/`profile`), `users`,
//! CRM (`contacts`, `companies`, `deals`, `pipelines`, `dashboard`, `search`),
//! privacidad Ley 21.719 y `health`/`ready`/`version`.

use std::sync::OnceLock;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::endpoint;

// ---------------------------------------------------------------------------
// Tipos del API (contrato del backend occb)
// ---------------------------------------------------------------------------

/// Usuario público tal como lo devuelve el backend (`/users`, TokenPair).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

/// Perfil propio (`GET /api/v1/auth/me`): sin `created_at`, con permisos.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MeProfile {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile_id: Option<Uuid>,
    #[serde(default)]
    pub permissions: Vec<String>,
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

#[derive(Debug, Serialize)]
struct UpdateProfileBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
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
/// Devuelve el perfil extendido (sin `created_at`, con permisos).
pub async fn me(access_token: &str) -> Result<MeProfile, ApiError> {
    let res = client()
        .get(endpoint("/api/v1/auth/me"))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<MeProfile>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `PUT /api/v1/auth/profile` — rectificación directa (nombre y/o email).
/// 409 si el email ya existe.
pub async fn update_profile(
    access_token: &str,
    email: Option<&str>,
    name: Option<&str>,
) -> Result<UserPublic, ApiError> {
    let res = client()
        .put(endpoint("/api/v1/auth/profile"))
        .bearer_auth(access_token)
        .json(&UpdateProfileBody { email, name })
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

#[derive(Debug, Serialize)]
struct ChangePasswordBody<'a> {
    current_password: &'a str,
    new_password: &'a str,
}

/// `PUT /api/v1/auth/password` — cambia la clave del dueño del token.
pub async fn change_password(
    access_token: &str,
    current: &str,
    new: &str,
) -> Result<(), ApiError> {
    let res = client()
        .put(endpoint("/api/v1/auth/password"))
        .bearer_auth(access_token)
        .json(&ChangePasswordBody {
            current_password: current,
            new_password: new,
        })
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

#[derive(Debug, Serialize)]
struct ForgotPasswordBody<'a> {
    email: &'a str,
}

/// `POST /api/v1/auth/forgot-password` — siempre 200 (anti-enumeración).
/// El backend aún no envía el email (TODO SMTP): el token se entrega
/// por fuera y se ingresa en `/restablecer`.
pub async fn forgot_password(email: &str) -> Result<(), ApiError> {
    let res = client()
        .post(endpoint("/api/v1/auth/forgot-password"))
        .json(&ForgotPasswordBody { email })
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

#[derive(Debug, Serialize)]
struct ResetPasswordBody<'a> {
    token: &'a str,
    password: &'a str,
}

/// `POST /api/v1/auth/reset-password` — canjea el token por la nueva clave.
pub async fn reset_password(token: &str, password: &str) -> Result<(), ApiError> {
    let res = client()
        .post(endpoint("/api/v1/auth/reset-password"))
        .json(&ResetPasswordBody { token, password })
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
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

// --- CRM: contactos / empresas / deals (protegido) --------------------------
// El backend pagina con `PaginatedResponse {data, total, page, per_page,
// total_pages}` y filtra con `search`, `sort_by`, `sort_order`.
// Las etapas de deal viajan como texto snake_case
// (lead|qualified|proposal|negotiation|closed_won|closed_lost).

/// Envelope paginado del CRM (`contacts`, `companies`, `deals`).
/// El backend siempre envía todos los campos, sin valores por defecto.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Paged<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

/// Contacto (`GET /contacts`, `GET /contacts/{id}`, …).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Contact {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub company_id: Option<Uuid>,
    #[serde(default)]
    pub position: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Contact {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
            .trim()
            .to_string()
    }
}

#[derive(Debug, Serialize)]
pub struct CreateContactBody<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'a str>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateContactBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'a str>,
}

/// Empresa (`GET /companies`, …).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Company {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub industry: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Negocio (`GET /deals`, …). `stage` en snake_case.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Deal {
    pub id: Uuid,
    pub title: String,
    #[serde(default)]
    pub value: f64,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default = "default_stage")]
    pub stage: String,
    #[serde(default)]
    pub position: Option<i32>,
    #[serde(default)]
    pub contact_id: Option<Uuid>,
    #[serde(default)]
    pub company_id: Option<Uuid>,
    #[serde(default)]
    pub pipeline_id: Option<Uuid>,
    #[serde(default)]
    pub pipeline_stage_id: Option<Uuid>,
    #[serde(default)]
    pub expected_close_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub contact_name: Option<String>,
    #[serde(default)]
    pub company_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_currency() -> String {
    "USD".to_string()
}

fn default_stage() -> String {
    "lead".to_string()
}

/// Etiqueta legible de cada etapa del pipeline.
pub fn deal_stage_label(stage: &str) -> &'static str {
    match stage {
        "lead" => "Lead",
        "qualified" => "Calificado",
        "proposal" => "Propuesta",
        "negotiation" => "Negociación",
        "closed_won" => "Ganado",
        "closed_lost" => "Perdido",
        _ => "Otra",
    }
}

/// Etapas en orden de avance para el tablero.
pub const DEAL_STAGES: [&str; 6] = [
    "lead",
    "qualified",
    "proposal",
    "negotiation",
    "closed_won",
    "closed_lost",
];

#[derive(Debug, Serialize)]
pub struct CreateDealBody<'a> {
    pub title: &'a str,
    pub value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_close_date: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'a str>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateDealBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_close_date: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct UpdateDealStageBody<'a> {
    pub stage: &'a str,
}

/// Actividad ligada a contacto/deal/empresa.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Activity {
    pub id: Uuid,
    #[serde(default)]
    pub activity_type: String,
    pub subject: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub contact_id: Option<Uuid>,
    #[serde(default)]
    pub deal_id: Option<Uuid>,
    #[serde(default)]
    pub company_id: Option<Uuid>,
    #[serde(default)]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pipeline con sus etapas (`GET /api/v1/pipelines`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Pipeline {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PipelineStage {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub position: i32,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub probability: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PipelineWithStages {
    pub pipeline: Pipeline,
    pub stages: Vec<PipelineStage>,
}

/// `GET /api/v1/dashboard/stats`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DashboardStats {
    #[serde(default)]
    pub total_contacts: i64,
    #[serde(default)]
    pub total_companies: i64,
    #[serde(default)]
    pub total_deals: i64,
    #[serde(default)]
    pub total_revenue: f64,
    #[serde(default)]
    pub active_deals: i64,
    #[serde(default)]
    pub won_deals: i64,
    #[serde(default)]
    pub lost_deals: i64,
}

/// `GET /api/v1/dashboard/pipeline`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PipelineStageSummary {
    pub stage: String,
    #[serde(default)]
    pub count: i64,
    #[serde(default)]
    pub total_value: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PipelineSummary {
    #[serde(default)]
    pub stages: Vec<PipelineStageSummary>,
}

/// `GET /api/v1/dashboard/top-deals`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TopDeal {
    pub id: Uuid,
    pub title: String,
    #[serde(default)]
    pub value: f64,
    #[serde(default)]
    pub stage: String,
    #[serde(default)]
    pub company_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TopDeals {
    #[serde(default)]
    pub deals: Vec<TopDeal>,
}

/// `GET /api/v1/search?q=…`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SearchHit {
    pub id: String,
    #[serde(default)]
    pub entity_type: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub subtitle: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SearchResults {
    #[serde(default)]
    pub contacts: Vec<SearchHit>,
    #[serde(default)]
    pub companies: Vec<SearchHit>,
    #[serde(default)]
    pub deals: Vec<SearchHit>,
}

async fn get_json<T: serde::de::DeserializeOwned>(
    token: &str,
    path: &str,
    query: &[(&str, String)],
) -> Result<T, ApiError> {
    let res = client()
        .get(endpoint(path))
        .bearer_auth(token)
        .query(query)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<T>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `GET /api/v1/contacts?page&per_page&search&sort_by&sort_order`.
/// Orden válido: `first_name|last_name|email|phone|position|created_at`.
pub async fn list_contacts(
    access_token: &str,
    page: u32,
    per_page: u32,
    search: &str,
    sort_by: &str,
    sort_order: &str,
) -> Result<Paged<Contact>, ApiError> {
    let mut query: Vec<(&str, String)> = vec![
        ("page", page.max(1).to_string()),
        ("per_page", per_page.clamp(1, 200).to_string()),
        ("sort_by", sort_by.to_string()),
        ("sort_order", sort_order.to_string()),
    ];
    if !search.trim().is_empty() {
        query.push(("search", search.trim().to_string()));
    }
    get_json(access_token, "/api/v1/contacts", &query).await
}

/// `GET /api/v1/contacts/{id}`.
pub async fn get_contact(access_token: &str, id: &str) -> Result<Contact, ApiError> {
    get_json(access_token, &format!("/api/v1/contacts/{id}"), &[]).await
}

/// `POST /api/v1/contacts` → 201.
pub async fn create_contact(
    access_token: &str,
    body: &CreateContactBody<'_>,
) -> Result<Contact, ApiError> {
    let res = client()
        .post(endpoint("/api/v1/contacts"))
        .bearer_auth(access_token)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<Contact>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `PUT /api/v1/contacts/{id}`.
pub async fn update_contact(
    access_token: &str,
    id: &str,
    body: &UpdateContactBody<'_>,
) -> Result<Contact, ApiError> {
    let res = client()
        .put(endpoint(&format!("/api/v1/contacts/{id}")))
        .bearer_auth(access_token)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<Contact>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `DELETE /api/v1/contacts/{id}` → 204.
pub async fn delete_contact(access_token: &str, id: &str) -> Result<(), ApiError> {
    let res = client()
        .delete(endpoint(&format!("/api/v1/contacts/{id}")))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

/// `GET /api/v1/contacts/{id}/deals`.
pub async fn contact_deals(access_token: &str, id: &str) -> Result<Vec<Deal>, ApiError> {
    get_json(access_token, &format!("/api/v1/contacts/{id}/deals"), &[]).await
}

/// `GET /api/v1/contacts/{id}/activities`.
pub async fn contact_activities(
    access_token: &str,
    id: &str,
) -> Result<Vec<Activity>, ApiError> {
    get_json(
        access_token,
        &format!("/api/v1/contacts/{id}/activities"),
        &[],
    )
    .await
}

/// `GET /api/v1/companies/{id}`.

/// `GET /api/v1/deals?page&per_page&search&sort_by&sort_order`.
/// Orden válido: `title|value|stage|created_at`.
pub async fn list_deals(
    access_token: &str,
    page: u32,
    per_page: u32,
    search: &str,
    sort_by: &str,
    sort_order: &str,
) -> Result<Paged<Deal>, ApiError> {
    let mut query: Vec<(&str, String)> = vec![
        ("page", page.max(1).to_string()),
        ("per_page", per_page.clamp(1, 200).to_string()),
        ("sort_by", sort_by.to_string()),
        ("sort_order", sort_order.to_string()),
    ];
    if !search.trim().is_empty() {
        query.push(("search", search.trim().to_string()));
    }
    get_json(access_token, "/api/v1/deals", &query).await
}

/// `GET /api/v1/deals/{id}`.
pub async fn get_deal(access_token: &str, id: &str) -> Result<Deal, ApiError> {
    get_json(access_token, &format!("/api/v1/deals/{id}"), &[]).await
}

/// `POST /api/v1/deals` → 201.
pub async fn create_deal(
    access_token: &str,
    body: &CreateDealBody<'_>,
) -> Result<Deal, ApiError> {
    let res = client()
        .post(endpoint("/api/v1/deals"))
        .bearer_auth(access_token)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<Deal>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `PUT /api/v1/deals/{id}`.
pub async fn update_deal(
    access_token: &str,
    id: &str,
    body: &UpdateDealBody<'_>,
) -> Result<Deal, ApiError> {
    let res = client()
        .put(endpoint(&format!("/api/v1/deals/{id}")))
        .bearer_auth(access_token)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<Deal>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `PATCH /api/v1/deals/{id}/stage` — mueve el negocio de etapa.
pub async fn move_deal_stage(
    access_token: &str,
    id: &str,
    stage: &str,
) -> Result<Deal, ApiError> {
    let res = client()
        .patch(endpoint(&format!("/api/v1/deals/{id}/stage")))
        .bearer_auth(access_token)
        .json(&UpdateDealStageBody { stage })
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<Deal>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `DELETE /api/v1/deals/{id}` → 204.
pub async fn delete_deal(access_token: &str, id: &str) -> Result<(), ApiError> {
    let res = client()
        .delete(endpoint(&format!("/api/v1/deals/{id}")))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

/// `GET /api/v1/deals/{id}/activities`.
pub async fn deal_activities(access_token: &str, id: &str) -> Result<Vec<Activity>, ApiError> {
    get_json(access_token, &format!("/api/v1/deals/{id}/activities"), &[]).await
}

/// `GET /api/v1/pipelines` — pipelines activos con sus etapas.
pub async fn list_pipelines(access_token: &str) -> Result<Vec<PipelineWithStages>, ApiError> {
    get_json(access_token, "/api/v1/pipelines", &[]).await
}

/// `GET /api/v1/dashboard/stats`.
pub async fn dashboard_stats(access_token: &str) -> Result<DashboardStats, ApiError> {
    get_json(access_token, "/api/v1/dashboard/stats", &[]).await
}

/// `GET /api/v1/dashboard/pipeline` — conteo y valor por etapa.
pub async fn dashboard_pipeline(access_token: &str) -> Result<PipelineSummary, ApiError> {
    get_json(access_token, "/api/v1/dashboard/pipeline", &[]).await
}

/// `GET /api/v1/dashboard/top-deals`.
pub async fn dashboard_top_deals(access_token: &str) -> Result<TopDeals, ApiError> {
    get_json(access_token, "/api/v1/dashboard/top-deals", &[]).await
}

/// `GET /api/v1/search?q=…` — búsqueda global (5 por entidad).
pub async fn global_search(access_token: &str, q: &str) -> Result<SearchResults, ApiError> {
    let query = vec![("q", q.trim().to_string())];
    get_json(access_token, "/api/v1/search", &query).await
}

/// `GET /api/v1/companies?page&per_page&search&sort_by&sort_order`.
/// Orden válido: `name|industry|city|country|created_at`.
pub async fn list_companies(
    access_token: &str,
    page: u32,
    per_page: u32,
    search: &str,
    sort_by: &str,
    sort_order: &str,
) -> Result<Paged<Company>, ApiError> {
    let mut query: Vec<(&str, String)> = vec![
        ("page", page.max(1).to_string()),
        ("per_page", per_page.clamp(1, 200).to_string()),
        ("sort_by", sort_by.to_string()),
        ("sort_order", sort_order.to_string()),
    ];
    if !search.trim().is_empty() {
        query.push(("search", search.trim().to_string()));
    }
    get_json(access_token, "/api/v1/companies", &query).await
}

/// `GET /api/v1/companies/{id}`.
pub async fn get_company(access_token: &str, id: &str) -> Result<Company, ApiError> {
    get_json(access_token, &format!("/api/v1/companies/{id}"), &[]).await
}

#[derive(Debug, Serialize)]
pub struct CreateCompanyBody<'a> {
    pub name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'a str>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateCompanyBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'a str>,
}

/// `POST /api/v1/companies` → 201.
pub async fn create_company(
    access_token: &str,
    body: &CreateCompanyBody<'_>,
) -> Result<Company, ApiError> {
    let res = client()
        .post(endpoint("/api/v1/companies"))
        .bearer_auth(access_token)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<Company>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `PUT /api/v1/companies/{id}`.
pub async fn update_company(
    access_token: &str,
    id: &str,
    body: &UpdateCompanyBody<'_>,
) -> Result<Company, ApiError> {
    let res = client()
        .put(endpoint(&format!("/api/v1/companies/{id}")))
        .bearer_auth(access_token)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<Company>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `DELETE /api/v1/companies/{id}` → 204.
pub async fn delete_company(access_token: &str, id: &str) -> Result<(), ApiError> {
    let res = client()
        .delete(endpoint(&format!("/api/v1/companies/{id}")))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

/// `GET /api/v1/companies/{id}/contacts`.
pub async fn company_contacts(
    access_token: &str,
    id: &str,
) -> Result<Vec<Contact>, ApiError> {
    get_json(
        access_token,
        &format!("/api/v1/companies/{id}/contacts"),
        &[],
    )
    .await
}

/// `GET /api/v1/companies/{id}/deals`.
pub async fn company_deals(access_token: &str, id: &str) -> Result<Vec<Deal>, ApiError> {
    get_json(access_token, &format!("/api/v1/companies/{id}/deals"), &[]).await
}

/// `GET /api/v1/companies/{id}/revenue` → `{total_value, deal_count}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct CompanyRevenue {
    #[serde(default)]
    pub total_value: f64,
    #[serde(default)]
    pub deal_count: i64,
}

pub async fn company_revenue(
    access_token: &str,
    id: &str,
) -> Result<CompanyRevenue, ApiError> {
    get_json(
        access_token,
        &format!("/api/v1/companies/{id}/revenue"),
        &[],
    )
    .await
}

// --- CRM: leads -----------------------------------------------------------
// `status`: new|contacted|qualified|unqualified|converted|recycled.
// `lead_source`: web|referral|cold_call|advertisement|email|social|partner|event|other.

/// Prospecto (`GET /api/v1/leads`, …).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Lead {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub company_name: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub industry: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default = "default_lead_source")]
    pub lead_source: String,
    #[serde(default = "default_lead_status")]
    pub status: String,
    #[serde(default)]
    pub score: i32,
    #[serde(default)]
    pub assigned_to: Option<Uuid>,
    #[serde(default)]
    pub converted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub converted_contact_id: Option<Uuid>,
    #[serde(default)]
    pub converted_company_id: Option<Uuid>,
    #[serde(default)]
    pub converted_deal_id: Option<Uuid>,
    #[serde(default)]
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_lead_source() -> String {
    "other".to_string()
}

fn default_lead_status() -> String {
    "new".to_string()
}

impl Lead {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
            .trim()
            .to_string()
    }
}

/// Etiquetas legibles de estado y origen.
pub fn lead_status_label(status: &str) -> &'static str {
    match status {
        "new" => "Nuevo",
        "contacted" => "Contactado",
        "qualified" => "Calificado",
        "unqualified" => "Descartado",
        "converted" => "Convertido",
        "recycled" => "Reciclado",
        _ => "Otro",
    }
}

pub fn lead_source_label(source: &str) -> &'static str {
    match source {
        "web" => "Web",
        "referral" => "Referido",
        "cold_call" => "Llamada en frío",
        "advertisement" => "Publicidad",
        "email" => "Email",
        "social" => "Redes",
        "partner" => "Socio",
        "event" => "Evento",
        "other" => "Otro",
        _ => "Otro",
    }
}

/// Estados en orden de avance para filtros.
pub const LEAD_STATUSES: [&str; 6] = [
    "new",
    "contacted",
    "qualified",
    "unqualified",
    "converted",
    "recycled",
];

pub const LEAD_SOURCES: [&str; 9] = [
    "web",
    "referral",
    "cold_call",
    "advertisement",
    "email",
    "social",
    "partner",
    "event",
    "other",
];

#[derive(Debug, Serialize)]
pub struct CreateLeadBody<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_source: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'a str>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateLeadBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_source: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct ConvertLeadBody<'a> {
    pub pipeline_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deal_title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deal_value: Option<f64>,
}

/// Resultado de `POST /api/v1/leads/{id}/convert`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ConvertLeadResult {
    #[serde(default)]
    pub contact_id: Option<Uuid>,
    #[serde(default)]
    pub company_id: Option<Uuid>,
    #[serde(default)]
    pub deal_id: Option<Uuid>,
}

/// `GET /api/v1/leads/stats`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LeadSourceCount {
    pub source: String,
    #[serde(default)]
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LeadStats {
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub new: i64,
    #[serde(default)]
    pub contacted: i64,
    #[serde(default)]
    pub qualified: i64,
    #[serde(default)]
    pub converted: i64,
    #[serde(default)]
    pub conversion_rate: f64,
    #[serde(default)]
    pub by_source: Vec<LeadSourceCount>,
}

/// Actividad propia de un lead (`GET /api/v1/leads/{id}/activities`).
/// Ojo: la clave JSON del tipo es `type`, no `activity_type`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LeadActivity {
    pub id: Uuid,
    pub lead_id: Uuid,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub subject: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CreateLeadActivityBody<'a> {
    #[serde(rename = "type")]
    pub activity_type: &'a str,
    pub subject: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<&'a str>,
}

/// `GET /api/v1/leads?page&per_page&search&status&lead_source&sort&sort_dir`.
pub async fn list_leads(
    access_token: &str,
    page: u32,
    per_page: u32,
    search: &str,
    status: &str,
    source: &str,
    sort: &str,
    sort_dir: &str,
) -> Result<Paged<Lead>, ApiError> {
    let mut query: Vec<(&str, String)> = vec![
        ("page", page.max(1).to_string()),
        ("per_page", per_page.clamp(1, 100).to_string()),
        ("sort", sort.to_string()),
        ("sort_dir", sort_dir.to_string()),
    ];
    if !search.trim().is_empty() {
        query.push(("search", search.trim().to_string()));
    }
    if !status.trim().is_empty() {
        query.push(("status", status.trim().to_string()));
    }
    if !source.trim().is_empty() {
        query.push(("lead_source", source.trim().to_string()));
    }
    get_json(access_token, "/api/v1/leads", &query).await
}

/// `GET /api/v1/leads/{id}`.
pub async fn get_lead(access_token: &str, id: &str) -> Result<Lead, ApiError> {
    get_json(access_token, &format!("/api/v1/leads/{id}"), &[]).await
}

/// `POST /api/v1/leads` → 201.
pub async fn create_lead(access_token: &str, body: &CreateLeadBody<'_>) -> Result<Lead, ApiError> {
    let res = client()
        .post(endpoint("/api/v1/leads"))
        .bearer_auth(access_token)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<Lead>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `PUT /api/v1/leads/{id}`.
pub async fn update_lead(
    access_token: &str,
    id: &str,
    body: &UpdateLeadBody<'_>,
) -> Result<Lead, ApiError> {
    let res = client()
        .put(endpoint(&format!("/api/v1/leads/{id}")))
        .bearer_auth(access_token)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<Lead>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `DELETE /api/v1/leads/{id}` → 204.
pub async fn delete_lead(access_token: &str, id: &str) -> Result<(), ApiError> {
    let res = client()
        .delete(endpoint(&format!("/api/v1/leads/{id}")))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

/// `POST /api/v1/leads/{id}/convert` → contacto + empresa/deal.
pub async fn convert_lead(
    access_token: &str,
    id: &str,
    body: &ConvertLeadBody<'_>,
) -> Result<ConvertLeadResult, ApiError> {
    let res = client()
        .post(endpoint(&format!("/api/v1/leads/{id}/convert")))
        .bearer_auth(access_token)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<ConvertLeadResult>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `GET /api/v1/leads/stats`.
pub async fn lead_stats(access_token: &str) -> Result<LeadStats, ApiError> {
    get_json(access_token, "/api/v1/leads/stats", &[]).await
}

/// `GET /api/v1/leads/{id}/activities`.
pub async fn lead_activities(
    access_token: &str,
    id: &str,
) -> Result<Vec<LeadActivity>, ApiError> {
    get_json(
        access_token,
        &format!("/api/v1/leads/{id}/activities"),
        &[],
    )
    .await
}

/// `POST /api/v1/leads/{id}/activities` → 201.
pub async fn create_lead_activity(
    access_token: &str,
    id: &str,
    body: &CreateLeadActivityBody<'_>,
) -> Result<LeadActivity, ApiError> {
    let res = client()
        .post(endpoint(&format!("/api/v1/leads/{id}/activities")))
        .bearer_auth(access_token)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<LeadActivity>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `POST /api/v1/leads/{lead_id}/activities/{activity_id}` (completar) → 200 sin body.
pub async fn complete_lead_activity(
    access_token: &str,
    lead_id: &str,
    activity_id: &str,
) -> Result<(), ApiError> {
    let res = client()
        .post(endpoint(&format!(
            "/api/v1/leads/{lead_id}/activities/{activity_id}"
        )))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

/// `DELETE /api/v1/leads/{lead_id}/activities/{activity_id}` → 204.
pub async fn delete_lead_activity(
    access_token: &str,
    lead_id: &str,
    activity_id: &str,
) -> Result<(), ApiError> {
    let res = client()
        .delete(endpoint(&format!(
            "/api/v1/leads/{lead_id}/activities/{activity_id}"
        )))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

// --- CRM: actividades ------------------------------------------------------
// `activity_type`: call|email|meeting|task|note (minúsculas).

/// Etiqueta legible del tipo de actividad.
pub fn activity_type_label(kind: &str) -> &'static str {
    match kind {
        "call" => "Llamada",
        "email" => "Email",
        "meeting" => "Reunión",
        "task" => "Tarea",
        "note" => "Nota",
        _ => "Otra",
    }
}

pub const ACTIVITY_TYPES: [&str; 5] = ["call", "email", "meeting", "task", "note"];

#[derive(Debug, Serialize)]
pub struct CreateActivityBody<'a> {
    pub activity_type: &'a str,
    pub subject: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deal_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<&'a str>,
}

/// `GET /api/v1/activities` — array plano (orden fijo por creación).
pub async fn list_activities(
    access_token: &str,
    activity_type: &str,
    contact_id: &str,
    deal_id: &str,
    company_id: &str,
) -> Result<Vec<Activity>, ApiError> {
    let mut query: Vec<(&str, String)> = Vec::new();
    if !activity_type.trim().is_empty() {
        query.push(("activity_type", activity_type.trim().to_string()));
    }
    if !contact_id.trim().is_empty() {
        query.push(("contact_id", contact_id.trim().to_string()));
    }
    if !deal_id.trim().is_empty() {
        query.push(("deal_id", deal_id.trim().to_string()));
    }
    if !company_id.trim().is_empty() {
        query.push(("company_id", company_id.trim().to_string()));
    }
    get_json(access_token, "/api/v1/activities", &query).await
}

/// `POST /api/v1/activities` → 201.
pub async fn create_activity(
    access_token: &str,
    body: &CreateActivityBody<'_>,
) -> Result<Activity, ApiError> {
    let res = client()
        .post(endpoint("/api/v1/activities"))
        .bearer_auth(access_token)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<Activity>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `POST /api/v1/activities/{id}/complete` → 200 con la actividad.
pub async fn complete_activity(access_token: &str, id: &str) -> Result<Activity, ApiError> {
    let res = client()
        .post(endpoint(&format!("/api/v1/activities/{id}/complete")))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<Activity>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `DELETE /api/v1/activities/{id}` → 204.
pub async fn delete_activity(access_token: &str, id: &str) -> Result<(), ApiError> {
    let res = client()
        .delete(endpoint(&format!("/api/v1/activities/{id}")))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

// --- CRM: documentos -------------------------------------------------------

/// Documento (`GET /api/v1/documents`, …).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Document {
    pub id: Uuid,
    pub filename: String,
    pub original_name: String,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub file_size: i64,
    #[serde(default)]
    pub folder: Option<String>,
    #[serde(default)]
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `GET /api/v1/documents?search=&folder=&mime_type=` — array plano.
pub async fn list_documents(
    access_token: &str,
    search: &str,
) -> Result<Vec<Document>, ApiError> {
    let mut query: Vec<(&str, String)> = Vec::new();
    if !search.trim().is_empty() {
        query.push(("search", search.trim().to_string()));
    }
    get_json(access_token, "/api/v1/documents", &query).await
}

/// `GET /api/v1/documents/{id}`.
pub async fn get_document(access_token: &str, id: &str) -> Result<Document, ApiError> {
    get_json(access_token, &format!("/api/v1/documents/{id}"), &[]).await
}

/// `GET /api/v1/documents/{id}/download` — bytes del archivo.
pub async fn download_document(
    access_token: &str,
    id: &str,
) -> Result<Vec<u8>, ApiError> {
    let res = client()
        .get(endpoint(&format!("/api/v1/documents/{id}/download")))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `DELETE /api/v1/documents/{id}` → 204.
pub async fn delete_document(access_token: &str, id: &str) -> Result<(), ApiError> {
    let res = client()
        .delete(endpoint(&format!("/api/v1/documents/{id}")))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

// --- CRM: reportes ---------------------------------------------------------

/// `GET /api/v1/reports/pipeline`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PipelineReportItem {
    pub stage: String,
    #[serde(default)]
    pub count: i64,
    #[serde(default)]
    pub total_value: f64,
    #[serde(default)]
    pub avg_value: f64,
    #[serde(default)]
    pub percentage: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PipelineReport {
    #[serde(default)]
    pub stages: Vec<PipelineReportItem>,
    #[serde(default)]
    pub total_deals: i64,
    #[serde(default)]
    pub total_value: f64,
}

/// `GET /api/v1/reports/win-loss`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WinLossReport {
    #[serde(default)]
    pub won_count: i64,
    #[serde(default)]
    pub lost_count: i64,
    #[serde(default)]
    pub win_rate: f64,
    #[serde(default)]
    pub loss_rate: f64,
    #[serde(default)]
    pub won_value: f64,
    #[serde(default)]
    pub lost_value: f64,
    #[serde(default)]
    pub total_closed: i64,
}

pub async fn pipeline_report(access_token: &str) -> Result<PipelineReport, ApiError> {
    get_json(access_token, "/api/v1/reports/pipeline", &[]).await
}

pub async fn win_loss_report(access_token: &str) -> Result<WinLossReport, ApiError> {
    get_json(access_token, "/api/v1/reports/win-loss", &[]).await
}

// --- CRM: notificaciones ---------------------------------------------------

/// Notificación (`GET /api/v1/notifications`, …).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub entity_type: Option<String>,
    #[serde(default)]
    pub entity_id: Option<Uuid>,
    #[serde(default)]
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

/// `GET /api/v1/notifications` — últimas 50.
pub async fn list_notifications(
    access_token: &str,
) -> Result<Vec<Notification>, ApiError> {
    get_json(access_token, "/api/v1/notifications", &[]).await
}

/// `GET /api/v1/notifications/unread-count` → `{count}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UnreadCount {
    #[serde(default)]
    pub count: i64,
}

pub async fn unread_count(access_token: &str) -> Result<UnreadCount, ApiError> {
    get_json(access_token, "/api/v1/notifications/unread-count", &[]).await
}

/// `PUT /api/v1/notifications/{id}/read` → 200 vacío.
pub async fn mark_notification_read(access_token: &str, id: &str) -> Result<(), ApiError> {
    let res = client()
        .put(endpoint(&format!("/api/v1/notifications/{id}/read")))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

/// `PUT /api/v1/notifications/read-all` → 200 vacío.
pub async fn mark_all_notifications_read(access_token: &str) -> Result<(), ApiError> {
    let res = client()
        .put(endpoint("/api/v1/notifications/read-all"))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

/// `DELETE /api/v1/notifications/{id}` → 204.
pub async fn delete_notification(access_token: &str, id: &str) -> Result<(), ApiError> {
    let res = client()
        .delete(endpoint(&format!("/api/v1/notifications/{id}")))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

/// Preferencias (`GET /api/v1/notifications/preferences`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: Uuid,
    #[serde(default = "default_true")]
    pub email_enabled: bool,
    #[serde(default = "default_true")]
    pub push_enabled: bool,
    #[serde(default)]
    pub weekly_digest: bool,
    #[serde(default)]
    pub marketing_emails: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Default, Serialize)]
pub struct UpdateNotificationPreferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly_digest: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marketing_emails: Option<bool>,
}

pub async fn get_prefs(access_token: &str) -> Result<NotificationPreferences, ApiError> {
    get_json(access_token, "/api/v1/notifications/preferences", &[]).await
}

/// `PUT /api/v1/notifications/preferences` → 200 vacío.
pub async fn update_prefs(
    access_token: &str,
    body: &UpdateNotificationPreferences,
) -> Result<(), ApiError> {
    let res = client()
        .put(endpoint("/api/v1/notifications/preferences"))
        .bearer_auth(access_token)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

// --- Admin: gestión de usuarios --------------------------------------------

#[derive(Debug, Serialize)]
pub struct AdminCreateUserBody<'a> {
    pub email: &'a str,
    pub name: &'a str,
    pub password: &'a str,
}

#[derive(Debug, Serialize)]
pub struct AdminUpdateUserBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<&'a str>,
}

/// `POST /api/v1/users` (admin) → 201.
pub async fn admin_create_user(
    access_token: &str,
    body: &AdminCreateUserBody<'_>,
) -> Result<UserPublic, ApiError> {
    let res = client()
        .post(endpoint("/api/v1/users"))
        .bearer_auth(access_token)
        .json(body)
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

/// `PUT /api/v1/users/{id}/profile` (admin).
pub async fn admin_update_user(
    access_token: &str,
    id: &str,
    body: &AdminUpdateUserBody<'_>,
) -> Result<UserPublic, ApiError> {
    let res = client()
        .put(endpoint(&format!("/api/v1/users/{id}/profile")))
        .bearer_auth(access_token)
        .json(body)
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

/// `DELETE /api/v1/users/{id}` (admin) → 204. 422 si es uno mismo.
pub async fn admin_delete_user(access_token: &str, id: &str) -> Result<(), ApiError> {
    let res = client()
        .delete(endpoint(&format!("/api/v1/users/{id}")))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

// --- CRM: etiquetas ----------------------------------------------------------
// `entity_type`: contact|company|deal|lead.

/// Etiqueta (`GET /api/v1/tags`, …).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub color: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CreateTagBody<'a> {
    pub name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct AssignTagBody<'a> {
    pub tag_id: &'a str,
    pub entity_type: &'a str,
    pub entity_id: &'a str,
}

/// `GET /api/v1/tags` — todas, ordenadas por nombre.
pub async fn list_tags(access_token: &str) -> Result<Vec<Tag>, ApiError> {
    get_json(access_token, "/api/v1/tags", &[]).await
}

/// `POST /api/v1/tags` → 201.
pub async fn create_tag(access_token: &str, body: &CreateTagBody<'_>) -> Result<Tag, ApiError> {
    let res = client()
        .post(endpoint("/api/v1/tags"))
        .bearer_auth(access_token)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<Tag>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `DELETE /api/v1/tags/{id}` → 204.
pub async fn delete_tag(access_token: &str, id: &str) -> Result<(), ApiError> {
    let res = client()
        .delete(endpoint(&format!("/api/v1/tags/{id}")))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

/// `GET /api/v1/tags/{entity_type}/{entity_id}`.
pub async fn entity_tags(
    access_token: &str,
    entity_type: &str,
    entity_id: &str,
) -> Result<Vec<Tag>, ApiError> {
    get_json(
        access_token,
        &format!("/api/v1/tags/{entity_type}/{entity_id}"),
        &[],
    )
    .await
}

/// `POST /api/v1/tags/assign` → 201.
pub async fn assign_tag(access_token: &str, body: &AssignTagBody<'_>) -> Result<(), ApiError> {
    let res = client()
        .post(endpoint("/api/v1/tags/assign"))
        .bearer_auth(access_token)
        .json(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

/// `DELETE /api/v1/tags/{tag_id}/{entity_type}/{entity_id}` → 204.
pub async fn remove_tag(
    access_token: &str,
    tag_id: &str,
    entity_type: &str,
    entity_id: &str,
) -> Result<(), ApiError> {
    let res = client()
        .delete(endpoint(&format!(
            "/api/v1/tags/{tag_id}/{entity_type}/{entity_id}"
        )))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(into_api_error(res).await)
    }
}

// --- CRM: importar / exportar / borrado masivo --------------------------------

async fn get_text(
    token: &str,
    path: &str,
    query: &[(&str, String)],
) -> Result<String, ApiError> {
    let res = client()
        .get(endpoint(path))
        .bearer_auth(token)
        .query(query)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.text()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `GET /api/v1/contacts/export?search=` → CSV.
pub async fn export_contacts(access_token: &str, search: &str) -> Result<String, ApiError> {
    let mut query: Vec<(&str, String)> = Vec::new();
    if !search.trim().is_empty() {
        query.push(("search", search.trim().to_string()));
    }
    get_text(access_token, "/api/v1/contacts/export", &query).await
}

/// `GET /api/v1/companies/export?search=` → CSV.
pub async fn export_companies(access_token: &str, search: &str) -> Result<String, ApiError> {
    let mut query: Vec<(&str, String)> = Vec::new();
    if !search.trim().is_empty() {
        query.push(("search", search.trim().to_string()));
    }
    get_text(access_token, "/api/v1/companies/export", &query).await
}

/// `GET /api/v1/deals/export` → CSV.
pub async fn export_deals(access_token: &str) -> Result<String, ApiError> {
    get_text(access_token, "/api/v1/deals/export", &[]).await
}

/// Resultado de una importación CSV.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ImportResult {
    #[serde(default)]
    pub imported: u32,
    #[serde(default)]
    pub errors: Vec<String>,
}

async fn post_csv(token: &str, path: &str, csv: &str) -> Result<ImportResult, ApiError> {
    let res = client()
        .post(endpoint(path))
        .bearer_auth(token)
        .header("Content-Type", "text/csv")
        .body(csv.to_string())
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<ImportResult>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `POST /api/v1/contacts/import` (CSV UTF-8, máx 2 MB).
pub async fn import_contacts(access_token: &str, csv: &str) -> Result<ImportResult, ApiError> {
    post_csv(access_token, "/api/v1/contacts/import", csv).await
}

/// `POST /api/v1/companies/import` (CSV UTF-8, máx 2 MB, máx 1000 filas).
pub async fn import_companies(access_token: &str, csv: &str) -> Result<ImportResult, ApiError> {
    post_csv(access_token, "/api/v1/companies/import", csv).await
}

/// `POST /api/v1/deals/import`.
pub async fn import_deals(access_token: &str, csv: &str) -> Result<ImportResult, ApiError> {
    post_csv(access_token, "/api/v1/deals/import", csv).await
}

/// `POST /api/v1/contacts/bulk-delete` → `{deleted}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BulkDeleteResult {
    #[serde(default)]
    pub deleted: usize,
}

#[derive(Debug, Serialize)]
struct BulkDeleteBody<'a> {
    ids: &'a [String],
}

pub async fn bulk_delete_contacts(
    access_token: &str,
    ids: &[String],
) -> Result<BulkDeleteResult, ApiError> {
    let res = client()
        .post(endpoint("/api/v1/contacts/bulk-delete"))
        .bearer_auth(access_token)
        .json(&BulkDeleteBody { ids })
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<BulkDeleteResult>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

// --- CRM: subida de documentos ------------------------------------------------

/// Acuse de `POST /api/v1/documents/upload` (multipart `file` + `folder`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UploadResponse {
    pub id: Uuid,
    pub filename: String,
    pub original_name: String,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub file_size: i64,
    #[serde(default)]
    pub folder: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Sube un archivo construyendo el multipart a mano (compatible con wasm).
pub async fn upload_document(
    access_token: &str,
    filename: &str,
    mime: &str,
    folder: Option<&str>,
    bytes: &[u8],
) -> Result<UploadResponse, ApiError> {
    let clean = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("archivo")
        .to_string();
    let boundary = format!("occb{}", Uuid::new_v4().simple());
    let mut body: Vec<u8> = Vec::new();
    if let Some(f) = folder {
        if !f.trim().is_empty() {
            body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
            body.extend_from_slice(
                b"Content-Disposition: form-data; name=\"folder\"\r\n\r\n",
            );
            body.extend_from_slice(f.trim().as_bytes());
            body.extend_from_slice(b"\r\n");
        }
    }
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"file\"; filename=\"{clean}\"\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(format!("Content-Type: {mime}\r\n\r\n").as_bytes());
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let res = client()
        .post(endpoint("/api/v1/documents/upload"))
        .bearer_auth(access_token)
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(body)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<UploadResponse>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

// --- Privacidad Ley 21.719 (contrato real del backend `occb`) ----------------
// `POST /api/v1/privacy/requests` (público, rate-limit estricto) responde 202
// con `{id, status, due_at}` y envía un email para verificar identidad.
// Tipos válidos: acceso|rectificacion|supresion|oposicion|portabilidad|bloqueo.

/// Solicitud de ejercicio de un derecho ARCOP+B.
#[derive(Debug, Serialize)]
pub struct PrivacyRequestPayload<'a> {
    pub email: &'a str,
    pub request_type: &'a str,
    pub motivo: &'a str,
    pub solicitante_nombre: &'a str,
}

/// Acuse del backend: `id` = folio, `due_at` = vencimiento del plazo legal.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PrivacyReceipt {
    pub id: Uuid,
    pub status: String,
    pub due_at: DateTime<Utc>,
}

/// Envía la solicitud al registro del backend.
pub async fn submit_privacy_request(
    payload: &PrivacyRequestPayload<'_>,
) -> Result<PrivacyReceipt, ApiError> {
    let res = client()
        .post(endpoint("/api/v1/privacy/requests"))
        .json(payload)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<PrivacyReceipt>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

// --- Registro probatorio de consentimientos ---------------------------------

/// Fila vigente de `privacy_consents` (`GET .../latest`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ServerConsent {
    pub marketing: bool,
    pub analytics: bool,
    pub version: i32,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

/// Acuse de `POST /api/v1/privacy/consents`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ConsentReceipt {
    pub id: Uuid,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct RecordConsentBody<'a> {
    marketing: bool,
    analytics: bool,
    version: i32,
    source: &'a str,
}

/// `POST /api/v1/privacy/consents` (protegido): registra la decisión del dueño
/// del token. Cada llamada crea una fila (historial); revocar = todo-falso.
pub async fn record_consent(
    access_token: &str,
    marketing: bool,
    analytics: bool,
    version: i32,
    source: &str,
) -> Result<ConsentReceipt, ApiError> {
    let res = client()
        .post(endpoint("/api/v1/privacy/consents"))
        .bearer_auth(access_token)
        .json(&RecordConsentBody {
            marketing,
            analytics,
            version,
            source,
        })
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<ConsentReceipt>()
            .await
            .map_err(|e| ApiError::transport(e.to_string()))
    } else {
        Err(into_api_error(res).await)
    }
}

/// `GET /api/v1/privacy/consents/latest` (protegido): última decisión.
/// Devuelve `None` si la persona nunca ha decidido (404 del backend).
pub async fn latest_consent(access_token: &str) -> Result<Option<ServerConsent>, ApiError> {
    let res = client()
        .get(endpoint("/api/v1/privacy/consents/latest"))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::transport(e.to_string()))?;
    if res.status().is_success() {
        res.json::<ServerConsent>()
            .await
            .map(Some)
            .map_err(|e| ApiError::transport(e.to_string()))
    } else if res.status().as_u16() == 404 {
        Ok(None)
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
