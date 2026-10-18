//! Canal de ejercicio de derechos ARCOP+B (Ley 21.719).
//!
//! Registra la solicitud en el backend (`POST /api/v1/privacy/requests`,
//! responde 202) y, solo si no hay red, genera un acuse provisorio local con
//! instrucciones para enviarla al correo de privacidad. Incluye portabilidad
//! básica: descarga de los datos que el frontend maneja.

use dioxus::prelude::*;
use serde::Serialize;

use crate::{
    api::{self, PrivacyRequestPayload},
    auth::use_auth,
    components::Layout,
    privacy::{
        download_text_file, now_iso, privacy_contact_email, provisional_folio, right_from_code,
        RightKind,
    },
    route::Route,
};

#[derive(Debug, Clone, PartialEq, Serialize)]
struct ExportData {
    exported_at: String,
    source: String,
    user: api::MeProfile,
    consent: crate::privacy::ConsentState,
}

/// Resultado visible del envío.
#[derive(Clone, PartialEq)]
enum SubmitOutcome {
    /// El backend registró la solicitud: folio oficial + vencimiento.
    Registered { folio: String, due: String },
    /// Sin red o backend caído: acuse provisorio con instrucciones.
    Provisional {
        folio: String,
        kind: RightKind,
        summary: String,
    },
    /// Error real del backend (p. ej. validación).
    Failed { message: String },
}

#[component]
pub fn Rights() -> Element {
    let auth = use_auth();
    let mut kind = use_signal(|| "acceso".to_string());
    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut detail = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut outcome = use_signal(|| None::<SubmitOutcome>);
    let mut export_json = use_signal(|| None::<String>);
    let mut export_error = use_signal(|| None::<String>);

    let session = auth.read().session.clone();
    let contact = privacy_contact_email();

    rsx! {
        Layout { title: "Tus derechos (ARCOP+B)".to_string(),
            p { class: "subtitle",
                "La Ley 21.719 te da seis derechos sobre tus datos. Este es el canal formal y gratuito "
                "para ejercerlos. Respondemos en 30 días corridos (el bloqueo temporal en 2 días hábiles)."
            }

            section { class: "grid-cards",
                for right in RightKind::ALL {
                    div { class: "card", key: "{right.code()}",
                        h2 { class: "card-title", "{right.title()}" }
                        p { class: "muted small", "{right.description()}" }
                        p { class: "small",
                            "Plazo: "
                            strong { "{right.deadline()}" }
                        }
                    }
                }
            }

            section { class: "card",
                h2 { class: "card-title", "Enviar una solicitud" }
                if session.is_some() {
                    p { class: "alert alert-info",
                        "Tienes sesión iniciada: tu identidad ya está verificada y prellenamos tus datos."
                    }
                }
                form {
                    class: "form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        let kind_v = right_from_code(&kind.read());
                        let (name_v, email_v) = match auth.read().session.clone() {
                            Some(s) => (s.user.name.clone(), s.user.email.clone()),
                            None => (name.read().trim().to_string(), email.read().trim().to_lowercase()),
                        };
                        let detail_v = detail.read().trim().to_string();
                        if name_v.is_empty() || email_v.is_empty() || detail_v.is_empty() {
                            outcome
                                .set(
                                    Some(SubmitOutcome::Failed {
                                        message: "Completa nombre, email de contacto y el detalle de tu solicitud."
                                            .to_string(),
                                    }),
                                );
                            return;
                        }
                        busy.set(true);
                        outcome.set(None);
                        spawn(async move {
                            let payload = PrivacyRequestPayload {
                                email: &email_v,
                                request_type: kind_v.code(),
                                motivo: &detail_v,
                                solicitante_nombre: &name_v,
                            };
                            match api::submit_privacy_request(&payload).await {
                                Ok(receipt) => {
                                    let due = receipt
                                        .due_at
                                        .format("%Y-%m-%d")
                                        .to_string();
                                    outcome
                                        .set(
                                            Some(SubmitOutcome::Registered {
                                                folio: receipt.id.to_string(),
                                                due,
                                            }),
                                        )
                                }
                                Err(err) if err.status == 0 => {
                                    let folio = provisional_folio();
                                    let summary = format!(
                                        "Folio provisorio: {folio}\nFecha: {}\nDerecho: {}\nNombre: {name_v}\nContacto: {email_v}\nDetalle: {detail_v}",
                                        now_iso(),
                                        kind_v.title(),
                                    );
                                    outcome
                                        .set(
                                            Some(SubmitOutcome::Provisional {
                                                folio,
                                                kind: kind_v,
                                                summary,
                                            }),
                                        );
                                }
                                Err(err) => {
                                    outcome
                                        .set(
                                            Some(SubmitOutcome::Failed {
                                                message: err.to_string(),
                                            }),
                                        )
                                }
                            }
                            busy.set(false);
                        });
                    },

                    label { class: "field", r#for: "right-kind", "Derecho que quieres ejercer" }
                    select {
                        id: "right-kind",
                        class: "input",
                        value: "{kind}",
                        onchange: move |e| kind.set(e.value()),
                        for right in RightKind::ALL {
                            option { value: "{right.code()}", "{right.title()} — {right.deadline()}" }
                        }
                    }
                    p { class: "muted small", "{right_from_code(&kind.read()).description()}" }

                    if session.is_none() {
                        label { class: "field", r#for: "right-name", "Tu nombre" }
                        input {
                            id: "right-name",
                            class: "input",
                            r#type: "text",
                            value: "{name}",
                            oninput: move |e| name.set(e.value()),
                            required: true,
                            autocomplete: "name",
                        }
                        label { class: "field", r#for: "right-email", "Email de contacto" }
                        input {
                            id: "right-email",
                            class: "input",
                            r#type: "email",
                            value: "{email}",
                            oninput: move |e| email.set(e.value()),
                            required: true,
                            autocomplete: "email",
                        }
                    }

                    label { class: "field", r#for: "right-detail", "Detalle de tu solicitud" }
                    textarea {
                        id: "right-detail",
                        class: "input",
                        rows: 4,
                        value: "{detail}",
                        oninput: move |e| detail.set(e.value()),
                        placeholder: "Cuéntanos qué necesitas: qué datos, qué corrección, qué fin al que te opones…",
                        required: true,
                    }

                    button {
                        class: "btn btn-primary",
                        r#type: "submit",
                        disabled: *busy.read(),
                        if *busy.read() {
                            "Enviando…"
                        } else {
                            "Enviar solicitud"
                        }
                    }
                }

                if let Some(result) = outcome.read().clone() {
                    match result {
                        SubmitOutcome::Registered { folio, due } => rsx! {
                            div { class: "alert alert-ok",
                                p {
                                    strong { "Solicitud registrada. Folio: {folio}" }
                                }
                                p { class: "small",
                                    "Vence el {due}. Te enviamos un correo para verificar tu identidad: "
                                    "la solicitud avanza cuando la verifiques. Si no lo reconoces, ignóralo."
                                }
                            }
                        },
                        SubmitOutcome::Provisional { folio, kind: k, summary } => rsx! {
                            div { class: "alert alert-info",
                                p {
                                    strong { "Acuse provisorio. Folio: {folio}" }
                                }
                                p { class: "small",
                                    "No pudimos contactar al registro (sin red o backend no disponible). Para que tu "
                                    "solicitud de «{k}» quede formalmente ingresada, envíala a "
                                    strong { "{contact}" }
                                    " con asunto «{folio}»."
                                }
                                p { class: "small", "Plazo legal: {k.deadline()} desde que la recibamos." }
                                textarea {
                                    class: "input",
                                    rows: 7,
                                    readonly: true,
                                    value: "{summary}",
                                }
                            }
                        },
                        SubmitOutcome::Failed { message } => rsx! {
                            div { class: "alert alert-error", "{message}" }
                        },
                    }
                }
            }

            section { class: "card",
                h2 { class: "card-title", "Portabilidad: descarga tus datos" }
                p { class: "muted small",
                    "Descarga en JSON los datos que esta app maneja sobre ti (tu perfil y tu decisión "
                    "de consentimiento). La exportación completa de tus datos comerciales la entrega el "
                    "equipo al resolver una solicitud de portabilidad."
                }
                if session.is_none() {
                    div { class: "alert alert-info",
                        p { "Inicia sesión para descargar tus datos." }
                        Link {
                            to: Route::Login {},
                            class: "btn btn-small btn-primary",
                            "Iniciar sesión"
                        }
                    }
                } else {
                    div { class: "row",
                        button {
                            class: "btn btn-small",
                            disabled: *busy.read(),
                            onclick: move |_| {
                                export_error.set(None);
                                let token = auth
                                    .read()
                                    .session
                                    .clone()
                                    .map(|s| s.access_token)
                                    .unwrap_or_default();
                                let consent_state = crate::privacy::load_consent();
                                spawn(async move {
                                    match api::me(&token).await {
                                        Ok(user) => {
                                            let export = ExportData {
                                                exported_at: now_iso(),
                                                source: "occb-crm-frontend".to_string(),
                                                user,
                                                consent: consent_state,
                                            };
                                            match serde_json::to_string_pretty(&export) {
                                                Ok(json) => {
                                                    if let Err(msg) = download_text_file(
                                                        "mis-datos.json",
                                                        &json,
                                                    ) {
                                                        export_json.set(Some(json));
                                                        export_error.set(Some(msg));
                                                    }
                                                }
                                                Err(e) => export_error.set(Some(e.to_string())),
                                            }
                                        }
                                        Err(e) => export_error.set(Some(e.to_string())),
                                    }
                                });
                            },
                            "Descargar mis datos (JSON)"
                        }
                    }
                    if let Some(msg) = export_error.read().clone() {
                        p { class: "alert alert-error small", "{msg}" }
                    }
                    if let Some(json) = export_json.read().clone() {
                        p { class: "small", "Copia tus datos desde aquí:" }
                        textarea {
                            class: "input",
                            rows: 10,
                            readonly: true,
                            value: "{json}",
                        }
                    }
                }
            }
        }
    }
}
