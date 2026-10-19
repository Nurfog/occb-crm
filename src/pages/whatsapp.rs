//! WhatsApp: configuración, conversaciones y envío.

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::use_auth,
    components::Layout,
    route::Route,
};

/// Bandeja y configuración. Requiere sesión.
#[component]
pub fn Whatsapp() -> Element {
    let auth = use_auth();
    let mut active = use_signal(String::new);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_c = token_of.clone();
    let mut convs = use_resource(move || {
        let token = token_c.clone();
        async move {
            match token {
                Some(t) => api::whatsapp_conversations(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_g = token_of.clone();
    let mut config = use_resource(move || {
        let token = token_g.clone();
        async move {
            match token {
                Some(t) => api::get_whatsapp_config(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "WhatsApp".to_string(),
                div { class: "alert alert-info",
                    p { "Esta página requiere sesión." }
                    Link {
                        to: Route::Login {},
                        class: "btn btn-small btn-primary",
                        "Iniciar sesión"
                    }
                }
            }
        };
    }

    let current = active.read().clone();

    rsx! {
        Layout { title: "WhatsApp".to_string(),
            section { class: "card",
                h2 { class: "card-title", "Estado" }
                match &*config.read() {
                    None => rsx! { p { class: "muted", "Cargando…" } },
                    Some(Err(_)) => rsx! { p { class: "muted", "Sin configuración (se envía nada hasta configurarlo)." } },
                    Some(Ok(c)) => {
                        let state = if c.is_active { "activo" } else { "pausado" };
                        let num = c.phone_number.clone().unwrap_or_else(|| "—".to_string());
                        rsx! { p { "Número: {num} · {state}" } }
                    },
                }
                Link {
                    to: Route::WhatsappConfig {},
                    class: "btn btn-small",
                    "Configurar"
                }
            }
            section { class: "card",
                h2 { class: "card-title", "Conversaciones" }
                match &*convs.read() {
                    None => rsx! { p { class: "muted", "Cargando…" } },
                    Some(Err(e)) => rsx! {
                        div { class: "alert alert-error",
                            p { "No se pudieron cargar." }
                            p { class: "muted small", "{e}" }
                            button { class: "btn btn-small", onclick: move |_| convs.restart(), "Reintentar" }
                        }
                    },
                    Some(Ok(list)) => rsx! {
                        if list.is_empty() {
                            p { class: "muted", "Sin conversaciones." }
                        } else {
                            ul {
                                for c in list {
                                    {
                                        let cphone = c.phone.clone();
                                        let who = c
                                            .contact_name
                                            .clone()
                                            .unwrap_or_else(|| cphone.clone());
                                        let last = c.last_message.clone();
                                        let dir = if c.last_direction == "outbound" { "→" } else { "←" };
                                        let key = cphone.clone();
                                        rsx! {
                                            li { key: "{key}",
                                                button {
                                                    class: "link-btn",
                                                    onclick: move |_| {
                                                        active.set(cphone.clone());
                                                    },
                                                    "{who}"
                                                }
                                                p { class: "muted small", "{dir} {last}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
            }
            if !current.is_empty() {
                Thread { phone: current, on_sent: move || convs.restart() }
            }
        }
    }
}

/// Hilo de un número + envío.
#[component]
fn Thread(phone: String, on_sent: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut text = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_m = token_of.clone();
    let phone_m = phone.clone();
    let mut msgs = use_resource(move || {
        let token = token_m.clone();
        let ph = phone_m.clone();
        async move {
            match token {
                Some(t) => api::whatsapp_messages(&t, "").await.map(|all| {
                    all.into_iter().filter(|m| m.from_number == ph || m.to_number == ph).collect::<Vec<_>>()
                }),
                None => Err(ApiError::no_session()),
            }
        }
    });

    let busy_v = *busy.read();

    rsx! {
        section { class: "card",
            h2 { class: "card-title", "{phone}" }
            match &*msgs.read() {
                None => rsx! { p { class: "muted", "Cargando…" } },
                Some(Err(e)) => rsx! { p { class: "muted small", "No se pudo cargar: {e}" } },
                Some(Ok(list)) => rsx! {
                    if list.is_empty() {
                        p { class: "muted", "Sin mensajes con este número." }
                    } else {
                        ul {
                            for m in list {
                                {
                                    let key = m.id.to_string();
                                    let dir = if m.direction == "outbound" { "→" } else { "←" };
                                    let content = m.content.clone();
                                    let when = m.created_at.format("%m-%d %H:%M").to_string();
                                    rsx! {
                                        li { key: "{key}", class: "small",
                                            "{dir} {content} "
                                            span { class: "muted", "({when} · {m.status})" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            }
            div { class: "row",
                input {
                    class: "input",
                    placeholder: "Escribe un mensaje…",
                    value: "{text}",
                    oninput: move |e| text.set(e.value()),
                }
                button {
                    class: "btn btn-small btn-primary",
                    disabled: busy_v,
                    onclick: move |_| {
                        let tok = token_of.clone();
                        let ph = phone.clone();
                        let content = text.read().trim().to_string();
                        if content.is_empty() {
                            return;
                        }
                        busy.set(true);
                        error.set(None);
                        spawn(async move {
                            let res = match tok {
                                Some(t) => {
                                    api::send_whatsapp(
                                        &t,
                                        &api::SendWhatsappBody { to: &ph, content: &content, contact_id: None },
                                    )
                                    .await
                                    .map(|_| ())
                                    .map_err(|e| e.to_string())
                                }
                                None => Err("Sin sesión.".to_string()),
                            };
                            busy.set(false);
                            match res {
                                Ok(()) => {
                                    text.set(String::new());
                                    msgs.restart();
                                    on_sent.call(());
                                }
                                Err(e) => error.set(Some(e)),
                            }
                        });
                    },
                    "Enviar"
                }
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
        }
    }
}

/// Configuración de la conexión (números, cuenta y token).
#[component]
pub fn WhatsappConfig() -> Element {
    let auth = use_auth();
    let mut phone_id = use_signal(String::new);
    let mut account_id = use_signal(String::new);
    let mut token_field = use_signal(String::new);
    let mut number = use_signal(String::new);
    let mut saved = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "WhatsApp: configuración".to_string(),
                div { class: "alert alert-info",
                    p { "Esta página requiere sesión." }
                    Link {
                        to: Route::Login {},
                        class: "btn btn-small btn-primary",
                        "Iniciar sesión"
                    }
                }
            }
        };
    }

    let was_saved = *saved.read();

    rsx! {
        Layout { title: "WhatsApp: configuración".to_string(),
            Link { to: Route::Whatsapp {}, class: "link", "← Volver" }
            form {
                class: "card form",
                onsubmit: move |e| {
                    e.prevent_default();
                    let tok = token_of.clone();
                    let pid = phone_id.read().trim().to_string();
                    let acc = account_id.read().trim().to_string();
                    let tk = token_field.read().clone();
                    let num = number.read().trim().to_string();
                    if pid.is_empty() || acc.is_empty() || tk.is_empty() {
                        error.set(
                            Some("ID de número, ID de cuenta y token son obligatorios.".to_string()),
                        );
                        return;
                    }
                    error.set(None);
                    saved.set(false);
                    spawn(async move {
                        let res = match tok {
                            Some(t) => {
                                api::update_whatsapp_config(
                                    &t,
                                    &api::WhatsappConfigInput {
                                        phone_number_id: &pid,
                                        business_account_id: &acc,
                                        api_token: &tk,
                                        phone_number: if num.is_empty() { None } else { Some(num.as_str()) },
                                    },
                                )
                                .await
                                .map(|_| ())
                                .map_err(|e| e.to_string())
                            }
                            None => Err("Sin sesión.".to_string()),
                        };
                        match res {
                            Ok(()) => {
                                saved.set(true);
                                token_field.set(String::new());
                            }
                            Err(msg) => error.set(Some(msg)),
                        }
                    });
                },
                p { class: "muted small",
                    "El token se cifra en el backend y nunca se muestra de vuelta."
                }
                label { class: "field", "ID de número *" }
                input {
                    class: "input",
                    value: "{phone_id}",
                    oninput: move |e| phone_id.set(e.value()),
                    required: true,
                }
                label { class: "field", "ID de cuenta comercial *" }
                input {
                    class: "input",
                    value: "{account_id}",
                    oninput: move |e| account_id.set(e.value()),
                    required: true,
                }
                label { class: "field", "Token de la API *" }
                input {
                    class: "input",
                    r#type: "password",
                    value: "{token_field}",
                    oninput: move |e| token_field.set(e.value()),
                    required: true,
                    autocomplete: "off",
                }
                label { class: "field", "Número (opcional)" }
                input {
                    class: "input",
                    value: "{number}",
                    oninput: move |e| number.set(e.value()),
                }
                if let Some(msg) = error.read().clone() {
                    p { class: "alert alert-error", "{msg}" }
                }
                if was_saved {
                    p { class: "alert alert-ok", "Configuración guardada." }
                }
                button { class: "btn btn-primary", r#type: "submit", "Guardar" }
            }
        }
    }
}
