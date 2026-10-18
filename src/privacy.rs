//! Privacidad y Ley 21.719 (Chile): consentimiento y derechos ARCOP+B.
//!
//! Este módulo contiene lo que es responsabilidad del frontend:
//!
//! - Estado del consentimiento (finalidades opcionales, opt-in) con
//!   persistencia local. El registro probatorio vive en el backend
//!   (`POST /api/v1/privacy/consents`, aún pendiente — ver `docs/ley-21719.md`).
//! - Tipos del canal de derechos: Acceso, Rectificación, Cancelación,
//!   Oposición, Portabilidad y Bloqueo temporal.
//!
//! Plazos legales: 30 días corridos para responder (+30 de prórroga
//! justificada), 2 días hábiles para el bloqueo temporal.

use chrono::SecondsFormat;
use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Contacto de privacidad (configurable en tiempo de compilación)
// ---------------------------------------------------------------------------

/// Email del responsable / DPO que recibe las solicitudes de derechos.
/// Fijar con `PRIVACY_CONTACT_EMAIL=privacidad@tu-organizacion.cl`.
pub fn privacy_contact_email() -> String {
    option_env!("PRIVACY_CONTACT_EMAIL")
        .unwrap_or("privacidad@tu-organizacion.cl")
        .to_string()
}

// ---------------------------------------------------------------------------
// Consentimiento (finalidades opcionales, opt-in, revocable)
// ---------------------------------------------------------------------------

/// Versión del texto/finalidades. Subirla invalida decisiones anteriores.
pub const CONSENT_VERSION: u32 = 1;
#[cfg(target_arch = "wasm32")]
const CONSENT_KEY: &str = "occb-consent-v1";

/// Consentimiento de finalidades NO necesarias. Las necesarias (cuenta,
/// autenticación, seguridad) no piden consentimiento: su base es el contrato
/// y se informa en la Política de Privacidad.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsentState {
    pub version: u32,
    pub marketing: bool,
    pub analytics: bool,
    /// Fecha ISO-8601 de la decisión. `None` = la persona aún no decide.
    pub decided_at: Option<String>,
}

impl Default for ConsentState {
    fn default() -> Self {
        Self {
            version: CONSENT_VERSION,
            marketing: false,
            analytics: false,
            decided_at: None,
        }
    }
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(target_arch = "wasm32")]
fn read_stored() -> Option<ConsentState> {
    LocalStorage::get(CONSENT_KEY).ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn read_stored() -> Option<ConsentState> {
    None
}

#[cfg(target_arch = "wasm32")]
fn write_stored(state: &ConsentState) {
    let _ = LocalStorage::set(CONSENT_KEY, state);
}

#[cfg(not(target_arch = "wasm32"))]
fn write_stored(_state: &ConsentState) {}

/// Carga la decisión guardada. Si cambió la versión, se considera no decidida.
pub fn load_consent() -> ConsentState {
    match read_stored() {
        Some(s) if s.version == CONSENT_VERSION => s,
        _ => ConsentState::default(),
    }
}

/// Guarda la decisión localmente (además del registro del backend).
pub fn save_consent(state: &ConsentState) {
    write_stored(state);
}

/// Registra el contexto global. Llamar una vez, en el componente raíz.
pub fn provide_consent() {
    use_context_provider(|| Signal::new(load_consent()));
}

/// Acceso al consentimiento desde páginas y componentes.
pub fn use_consent() -> Signal<ConsentState> {
    use_context::<Signal<ConsentState>>()
}

// ---------------------------------------------------------------------------
// Derechos ARCOP+B
// ---------------------------------------------------------------------------

/// Los seis derechos que la Ley 21.719 otorga a toda persona.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightKind {
    Acceso,
    Rectificacion,
    Cancelacion,
    Oposicion,
    Portabilidad,
    Bloqueo,
}

impl RightKind {
    pub const ALL: [RightKind; 6] = [
        RightKind::Acceso,
        RightKind::Rectificacion,
        RightKind::Cancelacion,
        RightKind::Oposicion,
        RightKind::Portabilidad,
        RightKind::Bloqueo,
    ];

    /// Valor que viaja al backend (`privacy_request_type`).
    /// Ojo: la cancelación viaja como `supresion` (nombre del enum).
    pub fn code(self) -> &'static str {
        match self {
            RightKind::Acceso => "acceso",
            RightKind::Rectificacion => "rectificacion",
            RightKind::Cancelacion => "supresion",
            RightKind::Oposicion => "oposicion",
            RightKind::Portabilidad => "portabilidad",
            RightKind::Bloqueo => "bloqueo",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            RightKind::Acceso => "Acceso",
            RightKind::Rectificacion => "Rectificación",
            RightKind::Cancelacion => "Cancelación / supresión",
            RightKind::Oposicion => "Oposición",
            RightKind::Portabilidad => "Portabilidad",
            RightKind::Bloqueo => "Bloqueo temporal",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            RightKind::Acceso => {
                "Saber qué datos personales tenemos sobre ti, con qué finalidad \
                 los tratamos y a quién se los hemos comunicado."
            }
            RightKind::Rectificacion => {
                "Corregir datos inexactos, desactualizados o incompletos."
            }
            RightKind::Cancelacion => {
                "Eliminar tus datos cuando ya no sean necesarios para la finalidad \
                 con que se recogieron (derecho al olvido en la práctica)."
            }
            RightKind::Oposicion => {
                "Oponerte al tratamiento para un fin específico, como marketing \
                 directo o elaboración de perfiles."
            }
            RightKind::Portabilidad => {
                "Recibir tus datos en un formato estructurado y de uso común, \
                 para llevarlos a otro proveedor si quieres."
            }
            RightKind::Bloqueo => {
                "Suspender temporalmente el tratamiento mientras se resuelve otra \
                 solicitud (p. ej. de rectificación u oposición)."
            }
        }
    }

    /// Plazo legal de respuesta.
    pub fn deadline(self) -> &'static str {
        match self {
            RightKind::Bloqueo => "2 días hábiles",
            _ => "30 días corridos",
        }
    }
}

impl std::fmt::Display for RightKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title())
    }
}

pub fn right_from_code(code: &str) -> RightKind {
    match code {
        "rectificacion" => RightKind::Rectificacion,
        "supresion" | "cancelacion" => RightKind::Cancelacion,
        "oposicion" => RightKind::Oposicion,
        "portabilidad" => RightKind::Portabilidad,
        "bloqueo" => RightKind::Bloqueo,
        _ => RightKind::Acceso,
    }
}

/// Envía la decisión al registro probatorio del backend (mejor esfuerzo).
/// Se llama con el token cuando hay sesión; sin sesión solo queda local.
pub async fn push_consent(access_token: &str, state: &ConsentState) {
    let _ = crate::api::record_consent(
        access_token,
        state.marketing,
        state.analytics,
        state.version as i32,
        "app",
    )
    .await;
}

/// Trae la última decisión registrada en el backend, si existe.
pub async fn pull_consent(access_token: &str) -> Option<ConsentState> {
    match crate::api::latest_consent(access_token).await {
        Ok(Some(rec)) => Some(ConsentState {
            version: rec.version.max(1) as u32,
            marketing: rec.marketing,
            analytics: rec.analytics,
            decided_at: Some(
                rec.created_at
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            ),
        }),
        _ => None,
    }
}

/// Folio provisorio local (el folio oficial lo entrega el backend al acusar
/// recibo). Se etiqueta como provisorio en la UI.
pub fn provisional_folio() -> String {
    let now = chrono::Utc::now();
    let ts = now.format("%Y%m%d%H%M%S").to_string();
    let rand = (now.timestamp_subsec_millis() % 9000 + 1000).to_string();
    format!("ARC-{ts}-{rand}")
}

// ---------------------------------------------------------------------------
// Descarga de archivos (portabilidad) — solo wasm en el navegador
// ---------------------------------------------------------------------------

/// Descarga `content` como archivo. Fuera de wasm devuelve `Err` y la UI
/// muestra el contenido para copiar manualmente.
pub fn download_text_file(filename: &str, content: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use web_sys::{Blob, BlobPropertyBag, Url};

        let window = web_sys::window().ok_or("Sin ventana de navegador")?;
        let document = window.document().ok_or("Sin documento")?;
        let parts = js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(content));
        let options = BlobPropertyBag::new();
        options.set_type("application/json");
        let blob = Blob::new_with_str_sequence_and_options(&parts, &options)
            .map_err(|_| "No se pudo crear el archivo")?;
        let url = Url::create_object_url_with_blob(&blob)
            .map_err(|_| "No se pudo crear la URL de descarga")?;
        let anchor: web_sys::HtmlAnchorElement = document
            .create_element("a")
            .map_err(|_| "No se pudo crear el enlace")?
            .dyn_into()
            .map_err(|_| "Enlace inválido")?;
        anchor.set_href(&url);
        anchor.set_download(filename);
        anchor.click();
        let _ = Url::revoke_object_url(&url);
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (filename, content);
        Err("Descarga disponible solo en el navegador".to_string())
    }
}
