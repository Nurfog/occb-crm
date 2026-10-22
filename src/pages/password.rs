//! Recupero y cambio de clave (`/auth/forgot-password|reset-password|password`).

use dioxus::prelude::*;

use crate::{api, auth::use_auth, components::Layout, route::Route};

/// Lee `?token=` de la URL en el navegador (para pre-rellenar el reset).
fn query_token() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.location().search().ok())
            .and_then(|search| {
                search
                    .trim_start_matches('?')
                    .split('&')
                    .find_map(|pair| {
                        let mut parts = pair.splitn(2, '=');
                        match (parts.next(), parts.next()) {
                            (Some("token"), Some(v)) => Some(v.to_string()),
                            _ => None,
                        }
                    })
            })
            .unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        String::new()
    }
}

/// Solicita el reseteo. Requiere tener el email registrado.
#[component]
pub fn ForgotPassword() -> Element {
    let mut email = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut sent = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let busy_v = *busy.read();
    let label = if busy_v { "Enviando…" } else { "Solicitar reseteo" };
    let was_sent = *sent.read();

    rsx! {
        Layout { title: "Recuperar clave".to_string(),
            if was_sent {
                div { class: "alert alert-ok",
                    p { "Si el email está registrado, preparamos un token de reseteo (válido 1 hora). El envío automático aún no está activo: pide tu token al equipo e ingrésalo en «Restablecer clave»." }
                    Link {
                        to: Route::ResetPassword {},
                        class: "btn btn-small btn-primary",
                        "Ir a restablecer clave"
                    }
                }
            } else {
                form {
                    class: "form card",
                    onsubmit: move |e| {
                        e.prevent_default();
                        let em = email.read().trim().to_lowercase();
                        if em.is_empty() {
                            error.set(Some("Indica tu email.".to_string()));
                            return;
                        }
                        busy.set(true);
                        error.set(None);
                        spawn(async move {
                            match api::forgot_password(&em).await {
                                Ok(()) => sent.set(true),
                                Err(err) => error.set(Some(err.to_string())),
                            }
                            busy.set(false);
                        });
                    },
                    if let Some(msg) = error.read().clone() {
                        div { class: "alert alert-error", "{msg}" }
                    }
                    p { class: "muted small",
                        "Te preparamos un token de un solo uso (expira en 1 hora)."
                    }
                    label { class: "field", r#for: "fp-email", "Email" }
                    input {
                        id: "fp-email",
                        class: "input",
                        r#type: "email",
                        value: "{email}",
                        oninput: move |e| email.set(e.value()),
                        required: true,
                        autocomplete: "email",
                    }
                    button {
                        class: "btn btn-primary",
                        r#type: "submit",
                        disabled: busy_v,
                        "{label}"
                    }
                }
            }
        }
    }
}

/// Canjea el token por una clave nueva.
#[component]
pub fn ResetPassword() -> Element {
    let mut token = use_signal(query_token);
    let mut password = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut done = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let busy_v = *busy.read();
    let label = if busy_v { "Guardando…" } else { "Fijar nueva clave" };
    let was_done = *done.read();

    rsx! {
        Layout { title: "Restablecer clave".to_string(),
            if was_done {
                div { class: "alert alert-ok",
                    p { "Clave actualizada. Ya puedes entrar." }
                    Link {
                        to: Route::Login {},
                        class: "btn btn-small btn-primary",
                        "Iniciar sesión"
                    }
                }
            } else {
                form {
                    class: "form card",
                    onsubmit: move |e| {
                        e.prevent_default();
                        let tok = token.read().trim().to_string();
                        let pw = password.read().clone();
                        if tok.len() < 32 {
                            error.set(Some("El token parece incompleto.".to_string()));
                            return;
                        }
                        if pw.len() < 12 {
                            error.set(Some("La clave debe tener al menos 12 caracteres.".to_string()));
                            return;
                        }
                        if pw != *confirm.read() {
                            error.set(Some("Las claves no coinciden.".to_string()));
                            return;
                        }
                        busy.set(true);
                        error.set(None);
                        spawn(async move {
                            match api::reset_password(&tok, &pw).await {
                                Ok(()) => done.set(true),
                                Err(err) => error.set(Some(err.to_string())),
                            }
                            busy.set(false);
                        });
                    },
                    if let Some(msg) = error.read().clone() {
                        div { class: "alert alert-error", "{msg}" }
                    }
                    label { class: "field", r#for: "rp-token", "Token" }
                    input {
                        id: "rp-token",
                        class: "input",
                        value: "{token}",
                        oninput: move |e| token.set(e.value()),
                        required: true,
                        autocomplete: "off",
                    }
                    label { class: "field", r#for: "rp-pass", "Nueva clave (mínimo 12)" }
                    input {
                        id: "rp-pass",
                        class: "input",
                        r#type: "password",
                        value: "{password}",
                        oninput: move |e| password.set(e.value()),
                        required: true,
                        autocomplete: "new-password",
                    }
                    label { class: "field", r#for: "rp-confirm", "Repite la clave" }
                    input {
                        id: "rp-confirm",
                        class: "input",
                        r#type: "password",
                        value: "{confirm}",
                        oninput: move |e| confirm.set(e.value()),
                        required: true,
                        autocomplete: "new-password",
                    }
                    button {
                        class: "btn btn-primary",
                        r#type: "submit",
                        disabled: busy_v,
                        "{label}"
                    }
                }
            }
        }
    }
}

/// Rectificación directa: edita nombre/email vía `PUT /api/v1/auth/profile`.
/// Al guardar, avisa al padre para recargar y actualiza la sesión local.
#[component]
pub fn EditProfile(on_saved: EventHandler<()>) -> Element {
    let mut auth = use_auth();
    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut ok = use_signal(|| false);

    let busy_v = *busy.read();
    let label = if busy_v { "Guardando…" } else { "Guardar cambios" };
    let was_ok = *ok.read();

    rsx! {
        form {
            class: "form",
            onsubmit: move |e| {
                e.prevent_default();
                let name_v = name.read().trim().to_string();
                let email_v = email.read().trim().to_lowercase();
                if name_v.is_empty() && email_v.is_empty() {
                    error.set(Some("Cambia tu nombre o tu email para guardar.".to_string()));
                    return;
                }
                let token = auth
                    .read()
                    .session
                    .clone()
                    .map(|s| s.access_token)
                    .unwrap_or_default();
                busy.set(true);
                error.set(None);
                ok.set(false);
                spawn(async move {
                    let email_opt = if email_v.is_empty() {
                        None
                    } else {
                        Some(email_v.as_str())
                    };
                    let name_opt = if name_v.is_empty() { None } else { Some(name_v.as_str()) };
                    match api::update_profile(&token, email_opt, name_opt).await {
                        Ok(updated) => {
                            let current = auth.read().session.clone();
                            if let Some(mut session) = current {
                                session.user.email = updated.email.clone();
                                session.user.name = updated.name.clone();
                                auth.write().session = Some(session);
                            }
                            ok.set(true);
                            on_saved.call(());
                        }
                        Err(err) => error.set(Some(err.to_string())),
                    }
                    busy.set(false);
                });
            },
            h3 { class: "card-title", "Rectificar mis datos" }
            if let Some(msg) = error.read().clone() {
                div { class: "alert alert-error", "{msg}" }
            }
            if was_ok {
                div { class: "alert alert-ok", "Datos actualizados." }
            }
            label { class: "field", r#for: "edit-name", "Nombre" }
            input {
                id: "edit-name",
                class: "input",
                r#type: "text",
                value: "{name}",
                oninput: move |e| name.set(e.value()),
                maxlength: 100,
                autocomplete: "name",
            }
            label { class: "field", r#for: "edit-email", "Email" }
            input {
                id: "edit-email",
                class: "input",
                r#type: "email",
                value: "{email}",
                oninput: move |e| email.set(e.value()),
                autocomplete: "email",
            }
            button {
                class: "btn btn-small",
                r#type: "submit",
                disabled: busy_v,
                "{label}"
            }
        }
    }
}

/// Cambio de clave con sesión (tarjeta para Mi cuenta).
#[component]
pub fn ChangePassword() -> Element {
    let auth = use_auth();
    let mut current = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut done = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let busy_v = *busy.read();
    let label = if busy_v { "Guardando…" } else { "Cambiar clave" };
    let was_done = *done.read();

    rsx! {
        form {
            class: "form",
            onsubmit: move |e| {
                e.prevent_default();
                let token = auth.read().session.clone().map(|s| s.access_token);
                let cur = current.read().clone();
                let pw = password.read().clone();
                if cur.is_empty() || pw.len() < 12 {
                    error.set(Some("Revisa la clave actual y que la nueva tenga 12+ caracteres.".to_string()));
                    return;
                }
                if pw != *confirm.read() {
                    error.set(Some("Las claves no coinciden.".to_string()));
                    return;
                }
                busy.set(true);
                error.set(None);
                done.set(false);
                spawn(async move {
                    let res = match token {
                        Some(t) => {
                            api::change_password(&t, &cur, &pw)
                                .await
                                .map_err(|e| e.to_string())
                        }
                        None => Err("Sin sesión.".to_string()),
                    };
                    busy.set(false);
                    match res {
                        Ok(()) => {
                            done.set(true);
                            current.set(String::new());
                            password.set(String::new());
                            confirm.set(String::new());
                        }
                        Err(msg) => error.set(Some(msg)),
                    }
                });
            },
            h3 { class: "card-title", "Cambiar clave" }
            if let Some(msg) = error.read().clone() {
                div { class: "alert alert-error", "{msg}" }
            }
            if was_done {
                div { class: "alert alert-ok", "Clave actualizada." }
            }
            label { class: "field", "Clave actual" }
            input {
                class: "input",
                r#type: "password",
                value: "{current}",
                oninput: move |e| current.set(e.value()),
                required: true,
                autocomplete: "current-password",
            }
            label { class: "field", "Nueva clave (mínimo 12)" }
            input {
                class: "input",
                r#type: "password",
                value: "{password}",
                oninput: move |e| password.set(e.value()),
                required: true,
                autocomplete: "new-password",
            }
            label { class: "field", "Repite la nueva" }
            input {
                class: "input",
                r#type: "password",
                value: "{confirm}",
                oninput: move |e| confirm.set(e.value()),
                required: true,
                autocomplete: "new-password",
            }
            button {
                class: "btn btn-small",
                r#type: "submit",
                disabled: busy_v,
                "{label}"
            }
        }
    }
}
