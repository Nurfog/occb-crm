//! Cliente HTTP del frontal hacia la API OCCB.
//! Todas las operaciones devuelven `Result<_, String>` con mensajes en español.
//! La API es la única que habla con PostgreSQL (vía funciones seguras).

use occb_core::{CrearEmpresa, CrearOportunidad, CrearPersona, Empresa, Oportunidad, Persona};
use uuid::Uuid;

/// URL base de la API, inyectada en compilación (`API_BASE_URL`).
/// Por defecto apunta al desarrollo local.
fn base() -> String {
    option_env!("API_BASE_URL")
        .unwrap_or("http://127.0.0.1:3001")
        .to_string()
}

/// GET que espera JSON; traduce errores HTTP al mensaje en español de la API.
async fn obtener_json<T>(ruta: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let respuesta = reqwest::get(format!("{}{ruta}", base()))
        .await
        .map_err(|_| "No se pudo conectar con la API. ¿Está en marcha?".to_string())?;
    leer_cuerpo(respuesta).await
}

/// Lee el cuerpo esperando éxito; si la API responde error, usa su `mensaje`.
async fn leer_cuerpo<T>(respuesta: reqwest::Response) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    if respuesta.status().is_success() {
        respuesta
            .json::<T>()
            .await
            .map_err(|_| "La API devolvió una respuesta inesperada.".to_string())
    } else {
        let cuerpo: serde_json::Value = respuesta.json().await.unwrap_or_default();
        Err(cuerpo
            .get("mensaje")
            .and_then(|m| m.as_str())
            .unwrap_or("La API devolvió un error.")
            .to_string())
    }
}

/// Envía JSON (POST/PATCH) y lee la respuesta con el mismo criterio.
async fn enviar_json<T>(metodo: &str, ruta: &str, cuerpo: &impl serde::Serialize) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let cliente = reqwest::Client::new();
    let url = format!("{}{ruta}", base());
    let peticion = match metodo {
        "POST" => cliente.post(&url),
        "PATCH" => cliente.patch(&url),
        _ => return Err("Método no soportado por el cliente.".to_string()),
    };
    let respuesta = peticion
        .json(cuerpo)
        .send()
        .await
        .map_err(|_| "No se pudo conectar con la API. ¿Está en marcha?".to_string())?;
    leer_cuerpo(respuesta).await
}

/// DELETE que solo exige éxito (muestra el `mensaje` de la API si falla).
async fn borrar(ruta: &str) -> Result<(), String> {
    let respuesta = reqwest::Client::new()
        .delete(format!("{}{ruta}", base()))
        .send()
        .await
        .map_err(|_| "No se pudo conectar con la API. ¿Está en marcha?".to_string())?;
    leer_cuerpo::<serde_json::Value>(respuesta).await.map(|_| ())
}

/// Lista de empresas desde `listar_empresas()`.
pub async fn listar_empresas() -> Result<Vec<Empresa>, String> {
    obtener_json("/api/empresas?limite=100").await
}

/// Crea una empresa vía `crear_empresa()` (valida en API y en PG).
pub async fn crear_empresa(datos: &CrearEmpresa) -> Result<Empresa, String> {
    enviar_json("POST", "/api/empresas", datos).await
}

/// Elimina una empresa vía `eliminar_empresa()`.
pub async fn eliminar_empresa(id: Uuid) -> Result<(), String> {
    borrar(&format!("/api/empresas/{id}")).await
}

/// Lista de personas desde `listar_personas()`.
pub async fn listar_personas() -> Result<Vec<Persona>, String> {
    obtener_json("/api/personas?limite=100").await
}

/// Crea una persona vía `crear_persona()`.
pub async fn crear_persona(datos: &CrearPersona) -> Result<Persona, String> {
    enviar_json("POST", "/api/personas", datos).await
}

/// Elimina una persona vía `eliminar_persona()`.
pub async fn eliminar_persona(id: Uuid) -> Result<(), String> {
    borrar(&format!("/api/personas/{id}")).await
}

/// Lista de oportunidades desde `listar_oportunidades()`.
pub async fn listar_oportunidades() -> Result<Vec<Oportunidad>, String> {
    obtener_json("/api/oportunidades?limite=200").await
}

/// Crea una oportunidad vía `crear_oportunidad()`.
pub async fn crear_oportunidad(datos: &CrearOportunidad) -> Result<Oportunidad, String> {
    enviar_json("POST", "/api/oportunidades", datos).await
}

/// Mueve una oportunidad de etapa vía `mover_etapa_oportunidad()`.
/// `etapa` es el texto que guarda PG: nueva, cribado, reunion, propuesta o cliente.
pub async fn mover_etapa(id: Uuid, etapa: &str) -> Result<Oportunidad, String> {
    enviar_json(
        "PATCH",
        &format!("/api/oportunidades/{id}/etapa"),
        &serde_json::json!({ "etapa": etapa }),
    )
    .await
}
