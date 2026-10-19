//! Entrada y registro contra `POST /api/v1/auth/login|register`.

use dioxus::prelude::*;

use crate::{
    api,
    auth::{use_auth, Session},
    components::Layout,
    privacy::{
        CONSENT_VERSION, ConsentState, now_iso, pull_consent, push_consent, save_consent,
        use_consent,
    },
    route::Route,
};

/// Formulario de entrada. Guarda el par de tokens en memoria y va al panel.
#[component]
pub fn Login() -> Element {
    let mut auth = use_auth();
    let mut consent = use_consent();
    let nav = use_navigator();
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    if auth.read().session.is_some() {
        return rsx! {
            Layout { title: "Iniciar sesión".to_string(),
                p { "Ya tienes una sesión activa." }
                Link { to: Route::Dashboard {}, class: "btn btn-primary", "Ir al panel" }
            }
        };
    }

    rsx! {
        Layout { title: "Iniciar sesión".to_string(),
            form {
                class: "form card",
                onsubmit: move |e| {
                    e.prevent_default();
                    let email_v = email.read().trim().to_lowercase();
                    let password_v = password.read().clone();
                    if email_v.is_empty() || password_v.is_empty() {
                        error.set(Some("Indica tu email y tu contraseña.".to_string()));
                        return;
                    }
                    busy.set(true);
                    error.set(None);
                    spawn(async move {
                        match api::login(&email_v, &password_v).await {
                            Ok(pair) => {
                                let token = pair.access_token.clone();
                                auth.write().session = Some(Session::from_pair(pair));
                                // Reconcilia el consentimiento con el registro del backend.
                                spawn(async move {
                                    if let Some(server) = pull_consent(&token).await {
                                        save_consent(&server);
                                        consent.set(server);
                                    }
                                });
                                nav.push(Route::Dashboard {});
                            }
                            Err(err) => error.set(Some(err.to_string())),
                        }
                        busy.set(false);
                    });
                },
                if let Some(msg) = error.read().clone() {
                    div { class: "alert alert-error", "{msg}" }
                }
                label { class: "field", r#for: "login-email", "Email" }
                input {
                    id: "login-email",
                    class: "input",
                    r#type: "email",
                    value: "{email}",
                    oninput: move |e| email.set(e.value()),
                    required: true,
                    autocomplete: "email",
                }
                label { class: "field", r#for: "login-password", "Contraseña" }
                input {
                    id: "login-password",
                    class: "input",
                    r#type: "password",
                    value: "{password}",
                    oninput: move |e| password.set(e.value()),
                    required: true,
                    autocomplete: "current-password",
                }
                button {
                    class: "btn btn-primary",
                    r#type: "submit",
                    disabled: *busy.read(),
                    if *busy.read() {
                        "Entrando…"
                    } else {
                        "Entrar"
                    }
                }
                p { class: "muted small",
                    "¿Sin cuenta? "
                    Link { to: Route::Register {}, class: "link", "Crea una cuenta" }
                    " · ¿Olvidaste tu clave? "
                    Link { to: Route::ForgotPassword {}, class: "link", "Recupérala" }
                    "."
                }
            }
        }
    }
}

/// Formulario de registro. El backend responde 409 si el email ya existe y
/// 422 si los datos no validan (la clave debe tener 12–128 caracteres).
#[component]
pub fn Register() -> Element {
    let mut auth = use_auth();
    let mut consent = use_consent();
    let nav = use_navigator();
    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut privacy_ok = use_signal(|| false);
    let mut marketing = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    if auth.read().session.is_some() {
        return rsx! {
            Layout { title: "Crear cuenta".to_string(),
                p { "Ya tienes una sesión activa." }
                Link { to: Route::Dashboard {}, class: "btn btn-primary", "Ir al panel" }
            }
        };
    }

    rsx! {
        Layout { title: "Crear cuenta".to_string(),
            form {
                class: "form card",
                onsubmit: move |e| {
                    e.prevent_default();
                    let name_v = name.read().trim().to_string();
                    let email_v = email.read().trim().to_lowercase();
                    let password_v = password.read().clone();
                    if name_v.is_empty() || email_v.is_empty() || password_v.is_empty() {
                        error.set(Some("Completa nombre, email y contraseña.".to_string()));
                        return;
                    }
                    if !*privacy_ok.read() {
                        error
                            .set(
                                Some(
                                    "Debes aceptar la Política de Privacidad para crear tu cuenta."
                                        .to_string(),
                                ),
                            );
                        return;
                    }
                    let marketing_v = *marketing.read();
                    busy.set(true);
                    error.set(None);
                    spawn(async move {
                        match api::register(&email_v, &name_v, &password_v).await {
                            Ok(pair) => {
                                let state = ConsentState {
                                    version: CONSENT_VERSION,
                                    marketing: marketing_v,
                                    analytics: false,
                                    decided_at: Some(now_iso()),
                                };
                                save_consent(&state);
                                consent.set(state.clone());
                                let token = pair.access_token.clone();
                                auth.write().session = Some(Session::from_pair(pair));
                                // Registro probatorio en el backend (mejor esfuerzo).
                                spawn(async move {
                                    push_consent(&token, &state).await;
                                });
                                nav.push(Route::Dashboard {});
                            }
                            Err(err) => error.set(Some(err.to_string())),
                        }
                        busy.set(false);
                    });
                },
                if let Some(msg) = error.read().clone() {
                    div { class: "alert alert-error", "{msg}" }
                }
                label { class: "field", r#for: "reg-name", "Nombre" }
                input {
                    id: "reg-name",
                    class: "input",
                    r#type: "text",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                    required: true,
                    maxlength: 100,
                    autocomplete: "name",
                }
                label { class: "field", r#for: "reg-email", "Email" }
                input {
                    id: "reg-email",
                    class: "input",
                    r#type: "email",
                    value: "{email}",
                    oninput: move |e| email.set(e.value()),
                    required: true,
                    autocomplete: "email",
                }
                label { class: "field", r#for: "reg-password", "Contraseña (12–128 caracteres)" }
                input {
                    id: "reg-password",
                    class: "input",
                    r#type: "password",
                    value: "{password}",
                    oninput: move |e| password.set(e.value()),
                    required: true,
                    minlength: 12,
                    maxlength: 128,
                    autocomplete: "new-password",
                }
                button {
                    class: "btn btn-primary",
                    r#type: "submit",
                    disabled: *busy.read(),
                    if *busy.read() {
                        "Creando…"
                    } else {
                        "Crear cuenta"
                    }
                }
                label { class: "check",
                    input {
                        r#type: "checkbox",
                        checked: *privacy_ok.read(),
                        onclick: move |_| {
                            let next = !*privacy_ok.read();
                            privacy_ok.set(next);
                        },
                    }
                    span {
                        "He leído y acepto la "
                        Link { to: Route::Privacy {}, class: "link", "Política de Privacidad" }
                        " (obligatorio)."
                    }
                }
                label { class: "check",
                    input {
                        r#type: "checkbox",
                        checked: *marketing.read(),
                        onclick: move |_| {
                            let next = !*marketing.read();
                            marketing.set(next);
                        },
                    }
                    "Quiero recibir novedades por email (opcional, revocable cuando quiera)."
                }
                p { class: "muted small",
                    "¿Ya tienes cuenta? "
                    Link { to: Route::Login {}, class: "link", "Entra aquí" }
                    ". El primer registro crea la cuenta administradora; después, los usuarios los crea un admin."
                }
            }
        }
    }
}
