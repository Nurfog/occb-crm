//! Auditoría: quién hizo qué y cuándo (`/api/v1/audit`).

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::use_auth,
    components::Layout,
    route::Route,
};

const ENTITY_TYPES: [&str; 7] = [
    "contact",
    "company",
    "deal",
    "lead",
    "activity",
    "document",
    "user",
];

/// Registro de auditoría con filtros. Requiere sesión (y permiso).
#[component]
pub fn Audit() -> Element {
    let auth = use_auth();
    let mut entity_id = use_signal(String::new);
    let mut applied_type = use_signal(String::new);
    let mut applied_id = use_signal(String::new);
    let mut page = use_signal(|| 1u32);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let mut logs = use_resource(move || {
        let token = token_of.clone();
        let et = applied_type.read().clone();
        let eid = applied_id.read().clone();
        let p = *page.read();
        async move {
            match token {
                Some(t) => api::list_audit_logs(&t, p, &et, &eid).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Auditoría".to_string(),
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

    rsx! {
        Layout { title: "Auditoría".to_string(),
            p { class: "muted small",
                "Quién hizo qué y cuándo. Requiere permiso de auditoría."
            }
            div { class: "row",
                button {
                    class: if applied_type.read().is_empty() { "btn btn-small btn-primary" } else { "btn btn-small" },
                    onclick: move |_| {
                        applied_type.set(String::new());
                        applied_id.set(String::new());
                        entity_id.set(String::new());
                        page.set(1);
                    },
                    "Todos"
                }
                for t in ENTITY_TYPES {
                    {
                        let active = *applied_type.read() == *t;
                        let cls = if active { "btn btn-small btn-primary" } else { "btn btn-small" };
                        let tv = t.to_string();
                        let label = t.to_string();
                        rsx! {
                            button { key: "{t}", class: "{cls}",
                                onclick: move |_| {
                                    applied_type.set(tv.clone());
                                    applied_id.set(String::new());
                                    entity_id.set(String::new());
                                    page.set(1);
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }
            form {
                class: "toolbar card",
                onsubmit: move |e| {
                    e.prevent_default();
                    applied_id.set(entity_id.read().trim().to_string());
                    page.set(1);
                },
                input {
                    class: "input",
                    placeholder: "ID de entidad (UUID, opcional)",
                    value: "{entity_id}",
                    oninput: move |e| entity_id.set(e.value()),
                }
                button { class: "btn btn-small btn-primary", r#type: "submit", "Filtrar" }
            }
            match &*logs.read() {
                None => rsx! { p { class: "muted", "Cargando registro…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p {
                            if e.status == 403 {
                                "No tienes permiso de auditoría."
                            } else {
                                "No se pudo cargar el registro."
                            }
                        }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| logs.restart(), "Reintentar" }
                    }
                },
                Some(Ok(list)) => rsx! {
                    if list.is_empty() {
                        p { class: "muted", "Sin movimientos." }
                    } else {
                        section { class: "card audit-list",
                            ul {
                                for l in list {
                                    {
                                        let key = l.id.to_string();
                                        rsx! {
                                            AuditRow { key: "{key}", entry: l.clone() }
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

/// Una fila de auditoría con icono por acción y detalle expandible.
#[component]
fn AuditRow(entry: api::AuditLog) -> Element {
    let mut open = use_signal(|| false);

    let when = entry.created_at.format("%d %b %H:%M").to_string();
    let action = entry.action.clone();
    let entity = entry.entity_type.clone();
    let short_id = entry.id.to_string().chars().take(8).collect::<String>();
    let (tile, glyph) = audit_badge(&entry.action);
    let detail = audit_detail(&entry);
    let is_open = *open.read();

    rsx! {
        li { class: "audit-row",
            span { class: "tile-sm {tile}", "{glyph}" }
            div { class: "audit-main",
                p {
                    strong { "{action}" }
                    span { class: "tag tag-muted", "{entity}" }
                    span { class: "muted small", " #{short_id}" }
                }
                p { class: "muted small", "{when}" }
                button {
                    class: "link-btn small",
                    onclick: move |_| open.set(!is_open),
                    if is_open { "ocultar detalle" } else { "ver detalle" }
                }
                if is_open {
                    pre { class: "audit-json", "{detail}" }
                }
            }
        }
    }
}

/// Icono por tipo de acción.
fn audit_badge(action: &str) -> (&'static str, &'static str) {
    let a = action.to_lowercase();
    if a.contains("delete") || a.contains("remove") {
        ("tile-note", "−")
    } else if a.contains("create") || a.contains("convert") {
        ("tile-ok", "+")
    } else if a.contains("update") || a.contains("edit") || a.contains("stage") {
        ("tile-email", "✎")
    } else if a.contains("login") {
        ("tile-meeting", "→")
    } else {
        ("tile-call", "•")
    }
}

/// Resumen legible de un movimiento.
fn audit_detail(log: &api::AuditLog) -> String {
    if let Some(new) = &log.new_values {
        let s = new.to_string();
        if s.len() > 160 { format!("{}…", &s[..160]) } else { s }
    } else if let Some(old) = &log.old_values {
        let s = old.to_string();
        if s.len() > 160 { format!("{}…", &s[..160]) } else { s }
    } else {
        log.ip_address.clone().unwrap_or_default()
    }
}
