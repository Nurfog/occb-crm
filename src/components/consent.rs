//! Banner de consentimiento (Ley 21.719): opt-in, granular y revocable.
//!
//! Solo cubre finalidades NO necesarias (marketing, analítica), que por
//! defecto están desactivadas. Las finalidades necesarias (cuenta,
//! autenticación, seguridad) no piden consentimiento: su base es el contrato
//! y se informan en la Política de Privacidad.

use dioxus::prelude::*;

use crate::{
    auth::use_auth,
    privacy::{now_iso, push_consent, save_consent, use_consent, ConsentState, CONSENT_VERSION},
    route::Route,
};

fn store_decision(
    mut consent: Signal<ConsentState>,
    token: Option<String>,
    marketing: bool,
    analytics: bool,
) {
    let state = ConsentState {
        version: CONSENT_VERSION,
        marketing,
        analytics,
        decided_at: Some(now_iso()),
    };
    save_consent(&state);
    consent.set(state.clone());
    // Registro probatorio en el backend (mejor esfuerzo, requiere sesión).
    if let Some(t) = token {
        spawn(async move {
            push_consent(&t, &state).await;
        });
    }
}

/// Banner inferior. Se muestra mientras la persona no haya decidido.
#[component]
pub fn ConsentBanner() -> Element {
    let consent = use_consent();
    let auth = use_auth();
    let mut marketing = use_signal(|| consent.read().marketing);
    let mut analytics = use_signal(|| consent.read().analytics);

    rsx! {
        div { class: "consent-banner",
            div { class: "consent-inner",
                p {
                    strong { "Tus datos, tu decisión. " }
                    "Usamos lo mínimo para tu cuenta. El marketing y la analítica son opcionales: "
                    Link { to: Route::Privacy {}, class: "link", "Política de Privacidad" }
                    "."
                }
                label { class: "consent-opt",
                    input {
                        r#type: "checkbox",
                        checked: *marketing.read(),
                        onclick: move |_| {
                            let next = !*marketing.read();
                            marketing.set(next);
                        },
                    }
                    " Marketing directo (ofertas por email)"
                }
                label { class: "consent-opt",
                    input {
                        r#type: "checkbox",
                        checked: *analytics.read(),
                        onclick: move |_| {
                            let next = !*analytics.read();
                            analytics.set(next);
                        },
                    }
                    " Analítica de uso (mejorar la app)"
                }
                div { class: "row",
                    button {
                        class: "btn btn-small btn-primary",
                        onclick: move |_| {
                            let token = auth
                                .read()
                                .session
                                .clone()
                                .map(|s| s.access_token);
                            store_decision(consent, token, *marketing.read(), *analytics.read());
                        },
                        "Aceptar selección"
                    }
                    button {
                        class: "btn btn-small",
                        onclick: move |_| {
                            let token = auth
                                .read()
                                .session
                                .clone()
                                .map(|s| s.access_token);
                            store_decision(consent, token, false, false);
                        },
                        "Rechazar opcionales"
                    }
                }
            }
        }
    }
}
