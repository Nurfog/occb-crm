//! Cliente HTTP del frontal hacia la API OCCB.
//! Todas las operaciones devuelven `Result<_, String>` con mensajes en español.
//! La API es la única que habla con PostgreSQL (vía funciones seguras).

use occb_core::{Colegio, CrearColegio, CrearOportunidad, CrearPersona, Oportunidad, Persona, ResumenPanel};
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

/// DELETE que solo exige éxito (el 204 no trae cuerpo; el mensaje solo se lee si falla).
async fn borrar(ruta: &str) -> Result<(), String> {
    let respuesta = reqwest::Client::new()
        .delete(format!("{}{ruta}", base()))
        .send()
        .await
        .map_err(|_| "No se pudo conectar con la API. ¿Está en marcha?".to_string())?;
    if respuesta.status().is_success() {
        Ok(())
    } else {
        let cuerpo: serde_json::Value = respuesta.json().await.unwrap_or_default();
        Err(cuerpo
            .get("mensaje")
            .and_then(|m| m.as_str())
            .unwrap_or("La API devolvió un error.")
            .to_string())
    }
}

/// Resumen del panel calculado en PostgreSQL.
pub async fn obtener_panel() -> Result<ResumenPanel, String> {
    obtener_json("/api/panel").await
}

/// Lista colegios del directorio con filtros (región, comuna, texto).
pub async fn listar_colegios(
    region: Option<i16>,
    comuna: &str,
    busqueda: &str,
) -> Result<Vec<Colegio>, String> {
    let mut ruta = String::from("/api/colegios?limite=100");
    if let Some(r) = region {
        ruta.push_str(&format!("&region={r}"));
    }
    if !comuna.trim().is_empty() {
        ruta.push_str(&format!("&comuna={}", comuna.trim()));
    }
    if !busqueda.trim().is_empty() {
        ruta.push_str(&format!("&busqueda={}", busqueda.trim()));
    }
    obtener_json(&ruta).await
}

/// Un colegio por RBD.
pub async fn obtener_colegio(rbd: &str) -> Result<Colegio, String> {
    obtener_json(&format!("/api/colegios/{rbd}")).await
}

/// Registra un colegio manualmente.
pub async fn crear_colegio(datos: &CrearColegio) -> Result<Colegio, String> {
    enviar_json("POST", "/api/colegios", datos).await
}

/// Elimina un colegio manual del directorio.
pub async fn eliminar_colegio(rbd: &str) -> Result<(), String> {
    borrar(&format!("/api/colegios/{rbd}")).await
}

/// Lista de personas desde `listar_personas()`.
pub async fn listar_personas() -> Result<Vec<Persona>, String> {
    obtener_json("/api/personas?limite=100").await
}

/// Una persona por id (`obtener_persona()`).
pub async fn obtener_persona(id: Uuid) -> Result<Persona, String> {
    obtener_json(&format!("/api/personas/{id}")).await
}

/// Crea una persona vía `crear_persona()`.
pub async fn crear_persona(datos: &CrearPersona) -> Result<Persona, String> {
    enviar_json("POST", "/api/personas", datos).await
}

/// Elimina una persona vía `eliminar_persona()`.
pub async fn eliminar_persona(id: Uuid) -> Result<(), String> {
    borrar(&format!("/api/personas/{id}")).await
}

/// Actualiza una persona vía `actualizar_persona()`.
pub async fn actualizar_persona(id: Uuid, datos: &CrearPersona) -> Result<Persona, String> {
    enviar_json("PATCH", &format!("/api/personas/{id}"), datos).await
}

/// Lista de oportunidades desde `listar_oportunidades()`.
pub async fn listar_oportunidades() -> Result<Vec<Oportunidad>, String> {
    obtener_json("/api/oportunidades?limite=200").await
}

/// Crea una oportunidad vía `crear_oportunidad()`.
pub async fn crear_oportunidad(datos: &CrearOportunidad) -> Result<Oportunidad, String> {
    enviar_json("POST", "/api/oportunidades", datos).await
}

/// Actualiza una oportunidad vía `actualizar_oportunidad()`.
pub async fn actualizar_oportunidad(id: Uuid, datos: &CrearOportunidad) -> Result<Oportunidad, String> {
    enviar_json("PATCH", &format!("/api/oportunidades/{id}"), datos).await
}

/// Mueve una oportunidad de etapa vía `mover_etapa_oportunidad()`.
/// `etapa` es el texto que guarda PG: nueva, contacto, reunion, propuesta o cliente.
pub async fn mover_etapa(id: Uuid, etapa: &str) -> Result<Oportunidad, String> {
    enviar_json(
        "PATCH",
        &format!("/api/oportunidades/{id}/etapa"),
        &serde_json::json!({ "etapa": etapa }),
    )
    .await
}
