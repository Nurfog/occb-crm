//! Email: redactar, plantillas y bitácora (`/api/v1/email/*`).

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::use_auth,
    components::Layout,
    route::Route,
};

/// Tres pestañas: redactar, plantillas y bitácora. Requiere sesión.
#[component]
pub fn Email() -> Element {
    let auth = use_auth();
    let mut tab = use_signal(|| "compose".to_string());

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Email".to_string(),
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
        Layout { title: "Email".to_string(),
            div { class: "row",
                button {
                    class: "{cls(\"compose\")}",
                    onclick: move |_| tab.set("compose".to_string()),
                    "Redactar"
                }
                button {
                    class: "{cls(\"templates\")}",
                    onclick: move |_| tab.set("templates".to_string()),
                    "Plantillas"
                }
                button {
                    class: "{cls(\"logs\")}",
                    onclick: move |_| tab.set("logs".to_string()),
                    "Bitácora"
                }
            }
            if current == "compose" {
                ComposeSection {}
            } else if current == "templates" {
                TemplatesSection {}
            } else {
                LogsSection {}
            }
        }
    }
}

/// Redacción y envío (con o sin plantilla).
#[component]
fn ComposeSection() -> Element {
    let auth = use_auth();
    let mut to = use_signal(String::new);
    let mut subject = use_signal(String::new);
    let mut body = use_signal(String::new);
    let mut template = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut result = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_t = token_of.clone();
    let mut templates = use_resource(move || {
        let token = token_t.clone();
        async move {
            match token {
                Some(t) => api::list_templates(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    let busy_v = *busy.read();

    rsx! {
        form {
            class: "card form",
            onsubmit: move |e| {
                e.prevent_default();
                let tok = token_of.clone();
                let to_v = to.read().trim().to_string();
                let subj_v = subject.read().trim().to_string();
                let body_v = body.read().clone();
                let tpl_v = template.read().trim().to_string();
                if to_v.is_empty() || body_v.trim().is_empty() {
                    result.set(Some("Destino y cuerpo son obligatorios.".to_string()));
                    return;
                }
                busy.set(true);
                result.set(None);
                spawn(async move {
                    let outcome = match tok {
                        Some(t) => {
                            let payload = api::SendEmailBody {
                                from: None,
                                to: &to_v,
                                cc: None,
                                bcc: None,
                                subject: &subj_v,
                                body: &body_v,
                                body_html: None,
                            };
                            let r = if tpl_v.is_empty() {
                                api::send_email(&t, &payload).await
                            } else {
                                api::send_from_template(&t, &tpl_v, &payload).await
                            };
                            match r {
                                Ok(log) => format!("Enviado: {} ({})", log.id, log.status),
                                Err(e) => e.to_string(),
                            }
                        }
                        None => "Sin sesión.".to_string(),
                    };
                    result.set(Some(outcome));
                    busy.set(false);
                });
            },
            h2 { class: "card-title", "Redactar" }
            label { class: "field", "Para *" }
            input {
                class: "input",
                r#type: "email",
                value: "{to}",
                oninput: move |e| to.set(e.value()),
                required: true,
            }
            label { class: "field", "Asunto" }
            input {
                class: "input",
                value: "{subject}",
                oninput: move |e| subject.set(e.value()),
            }
            label { class: "field", "Cuerpo *" }
            textarea {
                class: "input",
                rows: 6,
                value: "{body}",
                oninput: move |e| body.set(e.value()),
                required: true,
            }
            match &*templates.read() {
                Some(Ok(list)) if !list.is_empty() => rsx! {
                    label { class: "field", "Plantilla (opcional: hereda asunto/cuerpo)" }
                    select {
                        class: "input",
                        value: "{template}",
                        onchange: move |e| template.set(e.value()),
                        option { value: "", "Sin plantilla" }
                        for t in list {
                            {
                                let tid = t.id.to_string();
                                let tname = t.name.clone();
                                rsx! { option { value: "{tid}", "{tname}" } }
                            }
                        }
                    }
                },
                _ => rsx! {},
            }
            if let Some(msg) = result.read().clone() {
                p { class: "alert alert-info", "{msg}" }
            }
            button {
                class: "btn btn-primary",
                r#type: "submit",
                disabled: busy_v,
                if busy_v { "Enviando…" } else { "Enviar" }
            }
            p { class: "muted small",
                "Si el SMTP no está activo el backend responde 503 y queda registrado como fallido."
            }
        }
    }
}

/// Plantillas: crear, renombrar y borrar.
#[component]
fn TemplatesSection() -> Element {
    let auth = use_auth();
    let mut name = use_signal(String::new);
    let mut subject = use_signal(String::new);
    let mut body = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_l = token_of.clone();
    let mut items = use_resource(move || {
        let token = token_l.clone();
        async move {
            match token {
                Some(t) => api::list_templates(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    rsx! {
        section { class: "card form",
            h2 { class: "card-title", "Nueva plantilla" }
            label { class: "field", "Nombre *" }
            input {
                class: "input",
                value: "{name}",
                oninput: move |e| name.set(e.value()),
            }
            label { class: "field", "Asunto *" }
            input {
                class: "input",
                value: "{subject}",
                oninput: move |e| subject.set(e.value()),
            }
            label { class: "field", "Cuerpo *" }
            textarea {
                class: "input",
                rows: 4,
                value: "{body}",
                oninput: move |e| body.set(e.value()),
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
            button {
                class: "btn btn-primary",
                onclick: move |_| {
                    let tok = token_of.clone();
                    let n = name.read().trim().to_string();
                    let s = subject.read().trim().to_string();
                    let b = body.read().clone();
                    if n.is_empty() || s.is_empty() || b.trim().is_empty() {
                        error.set(Some("Nombre, asunto y cuerpo son obligatorios.".to_string()));
                        return;
                    }
                    error.set(None);
                    spawn(async move {
                        if let Some(t) = tok {
                            match api::create_template(
                                    &t,
                                    &api::CreateTemplateBody {
                                        name: &n,
                                        subject: &s,
                                        body: &b,
                                        body_html: None,
                                        category: None,
                                    },
                                )
                                .await
                            {
                                Ok(_) => {
                                    name.set(String::new());
                                    subject.set(String::new());
                                    body.set(String::new());
                                    items.restart();
                                }
                                Err(e) => error.set(Some(e.to_string())),
                            }
                        }
                    });
                },
                "Crear plantilla"
            }
        }
        match &*items.read() {
            None => rsx! { p { class: "muted", "Cargando plantillas…" } },
            Some(Err(e)) => rsx! { p { class: "muted small", "No se pudieron cargar: {e}" } },
            Some(Ok(list)) => rsx! {
                if list.is_empty() {
                    p { class: "muted", "Sin plantillas." }
                } else {
                    div { class: "grid-cards",
                        for t in list {
                            {
                                let tid = t.id.to_string();
                                rsx! {
                                    TemplateCard {
                                        key: "{tid}",
                                        template: t.clone(),
                                        on_changed: move || items.restart(),
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

/// Tarjeta de plantilla: vista previa, enviar, editar y borrar.
#[component]
fn TemplateCard(template: api::EmailTemplate, on_changed: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut editing = use_signal(|| false);
    let mut sending = use_signal(|| false);
    let mut name = use_signal(|| template.name.clone());
    let mut subject = use_signal(|| template.subject.clone());
    let mut body = use_signal(|| template.body.clone());
    let mut to = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut msg = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let tid = template.id.to_string();
    let token_send = token_of.clone();
    let token_edit = token_of.clone();
    let token_del = token_of.clone();
    let tid_send = tid.clone();
    let tid_edit = tid.clone();
    let tid_del = tid.clone();
    let editing_open = *editing.read();
    let sending_open = *sending.read();
    let busy_v = *busy.read();

    rsx! {
        div { class: "card",
            div { class: "row",
                h3 { class: "card-title", "{template.name}" }
                span { class: "tag tag-muted", "{template.category}" }
            }
            p { class: "muted small", "Asunto: {template.subject}" }
            p { class: "muted small template-preview", "{template.body}" }
            div { class: "row",
                button {
                    class: "btn btn-small btn-primary",
                    onclick: move |_| sending.set(!sending_open),
                    if sending_open { "Cerrar envío" } else { "Enviar" }
                }
                button {
                    class: "btn btn-small",
                    onclick: move |_| editing.set(!editing_open),
                    if editing_open { "Cerrar" } else { "Editar" }
                }
                button {
                    class: "btn btn-small danger-btn",
                    disabled: busy_v,
                    onclick: move |_| {
                        let tok = token_del.clone();
                        let iv = tid_del.clone();
                        busy.set(true);
                        spawn(async move {
                            if let Some(t) = tok {
                                let _ = api::delete_template(&t, &iv).await;
                            }
                            busy.set(false);
                            on_changed.call(());
                        });
                    },
                    "Eliminar"
                }
            }
            if sending_open {
                div { class: "form",
                    label { class: "field", "Para *" }
                    input {
                        class: "input",
                        r#type: "email",
                        value: "{to}",
                        oninput: move |e| to.set(e.value()),
                    }
                    button {
                        class: "btn btn-small btn-primary",
                        disabled: busy_v,
                        onclick: move |_| {
                            let tok = token_send.clone();
                            let iv = tid_send.clone();
                            let to_v = to.read().trim().to_string();
                            let subj_v = subject.read().clone();
                            let body_v = body.read().clone();
                            if to_v.is_empty() {
                                msg.set(Some("Indica el destinatario.".to_string()));
                                return;
                            }
                            busy.set(true);
                            msg.set(None);
                            spawn(async move {
                                let res = match tok {
                                    Some(t) => {
                                        api::send_from_template(
                                                &t,
                                                &iv,
                                                &api::SendEmailBody {
                                                    from: None,
                                                    to: &to_v,
                                                    cc: None,
                                                    bcc: None,
                                                    subject: &subj_v,
                                                    body: &body_v,
                                                    body_html: None,
                                                },
                                            )
                                            .await
                                            .map(|l| format!("Enviado: {} ({})", l.id, l.status))
                                            .map_err(|e| e.to_string())
                                    }
                                    None => Err("Sin sesión.".to_string()),
                                };
                                busy.set(false);
                                msg.set(Some(res.unwrap_or_else(|e| e)));
                            });
                        },
                        "Enviar con esta plantilla"
                    }
                }
            }
            if editing_open {
                div { class: "form",
                    label { class: "field", "Nombre" }
                    input {
                        class: "input",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                    label { class: "field", "Asunto" }
                    input {
                        class: "input",
                        value: "{subject}",
                        oninput: move |e| subject.set(e.value()),
                    }
                    label { class: "field", "Cuerpo" }
                    textarea {
                        class: "input",
                        rows: 4,
                        value: "{body}",
                        oninput: move |e| body.set(e.value()),
                    }
                    button {
                        class: "btn btn-small btn-primary",
                        disabled: busy_v,
                        onclick: move |_| {
                            let tok = token_edit.clone();
                            let iv = tid_edit.clone();
                            let n = name.read().trim().to_string();
                            let s = subject.read().trim().to_string();
                            let b = body.read().clone();
                            busy.set(true);
                            msg.set(None);
                            spawn(async move {
                                let res = match tok {
                                    Some(t) => {
                                        api::update_template(
                                                &t,
                                                &iv,
                                                &api::UpdateTemplateBody {
                                                    name: if n.is_empty() { None } else { Some(n.as_str()) },
                                                    subject: if s.is_empty() { None } else { Some(s.as_str()) },
                                                    body: Some(b.as_str()),
                                                    body_html: None,
                                                    category: None,
                                                },
                                            )
                                            .await
                                            .map(|_| "Plantilla actualizada.".to_string())
                                            .map_err(|e| e.to_string())
                                    }
                                    None => Err("Sin sesión.".to_string()),
                                };
                                busy.set(false);
                                match res {
                                    Ok(m) => {
                                        msg.set(Some(m));
                                        on_changed.call(());
                                    }
                                    Err(e) => msg.set(Some(e)),
                                }
                            });
                        },
                        "Guardar"
                    }
                }
            }
            if let Some(m) = msg.read().clone() {
                p { class: "alert alert-info small", "{m}" }
            }
        }
    }
}

/// Bitácora de envíos.
#[component]
fn LogsSection() -> Element {
    let auth = use_auth();
    let mut page = use_signal(|| 1u32);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let mut logs = use_resource(move || {
        let token = token_of.clone();
        let p = *page.read();
        async move {
            match token {
                Some(t) => api::list_email_logs(&t, p).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    rsx! {
        match &*logs.read() {
            None => rsx! { p { class: "muted", "Cargando bitácora…" } },
            Some(Err(e)) => rsx! {
                div { class: "alert alert-error",
                    p {
                        if e.status == 403 {
                            "No tienes permiso para ver los envíos."
                        } else {
                            "No se pudo cargar."
                        }
                    }
                    p { class: "muted small", "{e}" }
                    button { class: "btn btn-small", onclick: move |_| logs.restart(), "Reintentar" }
                }
            },
            Some(Ok(paged)) => {
                let current = *page.read();
                let total_pages = (paged.total + paged.per_page.max(1) - 1) / paged.per_page.max(1);
                let total_pages = total_pages.max(1);
                let total = paged.total;
                rsx! {
                    div { class: "table-wrap card",
                        table { class: "table",
                            thead {
                                tr {
                                    th { "Para" }
                                    th { "Asunto" }
                                    th { "Estado" }
                                    th { "Fecha" }
                                }
                            }
                            tbody {
                                for l in &paged.data {
                                    {
                                        let key = l.id.to_string();
                                        let dest = l.to_email.clone();
                                        let subj = l.subject.clone();
                                        let st = l.status.clone();
                                        let when = l.created_at.format("%Y-%m-%d %H:%M").to_string();
                                        rsx! {
                                            tr { key: "{key}",
                                                td { "{dest}" }
                                                td { "{subj}" }
                                                td { "{st}" }
                                                td { class: "muted", "{when}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if paged.data.is_empty() {
                            p { class: "muted", "Sin envíos." }
                        }
                    }
                    div { class: "pagination",
                        button {
                            class: "btn btn-small",
                            disabled: current <= 1,
                            onclick: move |_| page.set(current.saturating_sub(1).max(1)),
                            "← Anterior"
                        }
                        span { class: "muted small", "Página {current} de {total_pages} · {total} en total" }
                        button {
                            class: "btn btn-small",
                            disabled: current as i64 >= total_pages,
                            onclick: move |_| page.set(current + 1),
                            "Siguiente →"
                        }
                    }
                }
            },
        }
    }
}
