//! Ajustes: integraciones, IA y reparto de leads.

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::use_auth,
    components::Layout,
    route::Route,
};

/// Abre una URL en pestaña nueva (solo navegador).
fn open_url(url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(w) = web_sys::window() {
            let _ = w.open_with_url_and_target(url, "_blank");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = url;
    }
}

/// Tres pestañas de ajustes. Requiere sesión.
#[component]
pub fn Settings() -> Element {
    let auth = use_auth();
    let mut tab = use_signal(|| "integrations".to_string());

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Ajustes".to_string(),
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

    let current = tab.read().clone();
    let cls = |id: &str| {
        if current == id { "btn btn-small btn-primary" } else { "btn btn-small" }
    };

    rsx! {
        Layout { title: "Ajustes".to_string(),
            div { class: "row",
                button {
                    class: "{cls(\"integrations\")}",
                    onclick: move |_| tab.set("integrations".to_string()),
                    "Integraciones"
                }
                button {
                    class: "{cls(\"ai\")}",
                    onclick: move |_| tab.set("ai".to_string()),
                    "IA"
                }
                button {
                    class: "{cls(\"assignment\")}",
                    onclick: move |_| tab.set("assignment".to_string()),
                    "Reparto"
                }
            }
            if current == "integrations" {
                IntegrationsSection {}
            } else if current == "ai" {
                AISection {}
            } else {
                AssignmentSection {}
            }
        }
    }
}

/// Conexiones OAuth (Google, Microsoft, …).
#[component]
fn IntegrationsSection() -> Element {
    let auth = use_auth();
    let mut error = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_l = token_of.clone();
    let mut items = use_resource(move || {
        let token = token_l.clone();
        async move {
            match token {
                Some(t) => api::list_integrations(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    rsx! {
        section { class: "card",
            h2 { class: "card-title", "Integraciones" }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
            match &*items.read() {
                None => rsx! { p { class: "muted", "Cargando…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p { "No se pudieron cargar." }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| items.restart(), "Reintentar" }
                    }
                },
                Some(Ok(list)) => rsx! {
                    if list.is_empty() {
                        p { class: "muted", "Sin integraciones registradas." }
                    } else {
                        ul {
                            for i in list {
                                {
                                    let provider = i.provider.clone();
                                    let state = if i.connected { "conectada" } else { "desconectada" };
                                    let who = i
                                        .provider_email
                                        .clone()
                                        .or(i.provider_name.clone())
                                        .unwrap_or_default();
                                    let tok = token_of.clone();
                                    let pv_c = provider.clone();
                                    let pv_d = provider.clone();
                                    rsx! {
                                        li { key: "{provider}",
                                            strong { "{provider}" }
                                            span { class: "muted small", " · {state} {who} " }
                                            if i.connected {
                                                button {
                                                    class: "link-btn small",
                                                    onclick: move |_| {
                                                        let tk = tok.clone();
                                                        let pv = pv_d.clone();
                                                        error.set(None);
                                                        spawn(async move {
                                                            if let Some(t) = tk {
                                                                if let Err(e) = api::integration_disconnect(&t, &pv)
                                                                    .await
                                                                {
                                                                    error.set(Some(e.to_string()));
                                                                }
                                                                items.restart();
                                                            }
                                                        });
                                                    },
                                                    "desconectar"
                                                }
                                            } else {
                                                button {
                                                    class: "link-btn small",
                                                    onclick: move |_| {
                                                        let tk = tok.clone();
                                                        let pv = pv_c.clone();
                                                        error.set(None);
                                                        spawn(async move {
                                                            if let Some(t) = tk {
                                                                match api::integration_connect_url(&t, &pv).await {
                                                                    Ok(u) => open_url(&u.auth_url),
                                                                    Err(e) => error.set(Some(e.to_string())),
                                                                }
                                                            }
                                                        });
                                                    },
                                                    "conectar"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}

/// Configuración del proveedor de IA.
#[component]
fn AISection() -> Element {
    let auth = use_auth();
    let mut provider = use_signal(String::new);
    let mut api_url = use_signal(String::new);
    let mut api_key = use_signal(String::new);
    let mut model = use_signal(String::new);
    let mut loaded = use_signal(|| false);
    let mut saved = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_g = token_of.clone();
    let mut cfg = use_resource(move || {
        let token = token_g.clone();
        async move {
            match token {
                Some(t) => api::get_ai_config(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if let Some(Ok(c)) = &*cfg.read() {
        if !*loaded.read() {
            loaded.set(true);
            provider.set(c.provider.clone());
            api_url.set(c.api_url.clone());
            model.set(c.model.clone());
        }
    }
    let was_saved = *saved.read();

    rsx! {
        match &*cfg.read() {
            None => rsx! { p { class: "muted", "Cargando…" } },
            Some(Err(e)) => rsx! {
                div { class: "alert alert-error",
                    p { "No se pudo cargar (¿falta permiso?)." }
                    p { class: "muted small", "{e}" }
                }
            },
            Some(Ok(_)) => rsx! {
                form {
                    class: "card form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        let tok = token_of.clone();
                        let p = provider.read().trim().to_string();
                        let u = api_url.read().trim().to_string();
                        let k = api_key.read().clone();
                        let m = model.read().trim().to_string();
                        error.set(None);
                        saved.set(false);
                        spawn(async move {
                            if let Some(t) = tok {
                                let res = api::update_ai_config(
                                        &t,
                                        &api::AIConfigInput {
                                            provider: if p.is_empty() { None } else { Some(p.as_str()) },
                                            api_url: if u.is_empty() { None } else { Some(u.as_str()) },
                                            api_key: if k.is_empty() { None } else { Some(k.as_str()) },
                                            model: if m.is_empty() { None } else { Some(m.as_str()) },
                                        },
                                    )
                                    .await;
                                match res {
                                    Ok(_) => {
                                        saved.set(true);
                                        api_key.set(String::new());
                                        cfg.restart();
                                    }
                                    Err(err) => error.set(Some(err.to_string())),
                                }
                            }
                        });
                    },
                    h2 { class: "card-title", "Proveedor de IA" }
                    p { class: "muted small",
                        "La clave nunca se muestra de vuelta; se guarda cifrada."
                    }
                    label { class: "field", "Proveedor (openai, anthropic, ollama…)" }
                    input {
                        class: "input",
                        value: "{provider}",
                        oninput: move |e| provider.set(e.value()),
                    }
                    label { class: "field", "URL de la API" }
                    input {
                        class: "input",
                        value: "{api_url}",
                        oninput: move |e| api_url.set(e.value()),
                    }
                    label { class: "field", "Clave (secreto, solo para cambiar)" }
                    input {
                        class: "input",
                        r#type: "password",
                        value: "{api_key}",
                        oninput: move |e| api_key.set(e.value()),
                        autocomplete: "off",
                    }
                    label { class: "field", "Modelo" }
                    input {
                        class: "input",
                        value: "{model}",
                        oninput: move |e| model.set(e.value()),
                    }
                    if let Some(msg) = error.read().clone() {
                        p { class: "alert alert-error", "{msg}" }
                    }
                    if was_saved {
                        p { class: "alert alert-ok", "Configuración guardada." }
                    }
                    button { class: "btn btn-primary", r#type: "submit", "Guardar" }
                }
            },
        }
    }
}

/// Reparto automático de leads + asignación manual.
#[component]
fn AssignmentSection() -> Element {
    let auth = use_auth();
    let mut strategy = use_signal(|| "round_robin".to_string());
    let mut max_active = use_signal(|| "10".to_string());
    let mut loaded = use_signal(|| false);
    let mut saved = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut lead_id = use_signal(String::new);
    let mut assign_msg = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_g = token_of.clone();
    let token_save = token_of.clone();
    let token_asg = token_of.clone();
    let mut cfg = use_resource(move || {
        let token = token_g.clone();
        async move {
            match token {
                Some(t) => api::get_lead_assignment(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if let Some(Ok(c)) = &*cfg.read() {
        if !*loaded.read() {
            loaded.set(true);
            strategy.set(c.strategy.clone());
            max_active.set(c.max_active_leads.to_string());
        }
    }
    let was_saved = *saved.read();

    rsx! {
        match &*cfg.read() {
            None => rsx! { p { class: "muted", "Cargando…" } },
            Some(Err(e)) => rsx! {
                div { class: "alert alert-error",
                    p { "No se pudo cargar." }
                    p { class: "muted small", "{e}" }
                }
            },
            Some(Ok(_)) => rsx! {
                form {
                    class: "card form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        let tok = token_save.clone();
                        let st = strategy.read().clone();
                        let max: i32 = max_active.read().trim().parse().unwrap_or(-1);
                        if max < 1 {
                            error.set(Some("El máximo debe ser mayor o igual a 1.".to_string()));
                            return;
                        }
                        error.set(None);
                        saved.set(false);
                        spawn(async move {
                            if let Some(t) = tok {
                                match api::update_lead_assignment(
                                        &t,
                                        &api::LeadAssignmentInput {
                                            strategy: Some(&st),
                                            max_active_leads: Some(max),
                                            territory_enabled: None,
                                            notify_on_assign: None,
                                        },
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        saved.set(true);
                                        cfg.restart();
                                    }
                                    Err(err) => error.set(Some(err.to_string())),
                                }
                            }
                        });
                    },
                    h2 { class: "card-title", "Reparto automático" }
                    label { class: "field", "Estrategia" }
                    select {
                        class: "input",
                        value: "{strategy}",
                        onchange: move |e| strategy.set(e.value()),
                        option { value: "round_robin", "Por turnos" }
                        option { value: "least_busy", "Al menos ocupado" }
                        option { value: "manual", "Manual" }
                    }
                    label { class: "field", "Máximo de leads activos por agente" }
                    input {
                        class: "input",
                        r#type: "number",
                        min: "1",
                        value: "{max_active}",
                        oninput: move |e| max_active.set(e.value()),
                    }
                    if let Some(msg) = error.read().clone() {
                        p { class: "alert alert-error", "{msg}" }
                    }
                    if was_saved {
                        p { class: "alert alert-ok", "Configuración guardada." }
                    }
                    button { class: "btn btn-primary", r#type: "submit", "Guardar" }
                }
                form {
                    class: "card form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        let tok = token_asg.clone();
                        let lid = lead_id.read().trim().to_string();
                        if lid.is_empty() {
                            assign_msg.set(Some("Indica el ID del prospecto.".to_string()));
                            return;
                        }
                        assign_msg.set(None);
                        spawn(async move {
                            let outcome = match tok {
                                Some(t) => {
                                    match api::assign_lead(
                                            &t,
                                            &api::AssignLeadBody {
                                                lead_id: &lid,
                                                lead_type: "lead",
                                                source: Some("manual"),
                                            },
                                        )
                                        .await
                                    {
                                        Ok(a) => format!("Asignado a {} ({})", a.agent_name, a.agent_id),
                                        Err(err) => err.to_string(),
                                    }
                                }
                                None => "Sin sesión.".to_string(),
                            };
                            assign_msg.set(Some(outcome));
                        });
                    },
                    h2 { class: "card-title", "Asignación manual" }
                    label { class: "field", "ID del prospecto (UUID)" }
                    input {
                        class: "input",
                        value: "{lead_id}",
                        oninput: move |e| lead_id.set(e.value()),
                    }
                    if let Some(msg) = assign_msg.read().clone() {
                        p { class: "alert alert-info", "{msg}" }
                    }
                    button { class: "btn btn-small", r#type: "submit", "Asignar" }
                }
            },
        }
    }
}
