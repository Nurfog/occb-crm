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

/// `GET /api/v1/companies?page&per_page&search&sort_by&sort_order`.
pub async fn list_companies(
    access_token: &str,
    page: u32,
    per_page: u32,
    search: &str,
) -> Result<Paged<Company>, ApiError> {
    let mut query: Vec<(&str, String)> = vec![
        ("page", page.max(1).to_string()),
        ("per_page", per_page.clamp(1, 200).to_string()),
    ];
    if !search.trim().is_empty() {
        query.push(("search", search.trim().to_string()));
    }
    get_json(access_token, "/api/v1/companies", &query).await
}

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
