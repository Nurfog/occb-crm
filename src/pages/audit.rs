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
    let mut entity_type = use_signal(String::new);
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
            form {
                class: "toolbar card",
                onsubmit: move |e| {
                    e.prevent_default();
                    applied_type.set(entity_type.read().clone());
                    applied_id.set(entity_id.read().trim().to_string());
                    page.set(1);
                },
                select {
                    class: "input",
                    value: "{entity_type}",
                    onchange: move |e| entity_type.set(e.value()),
                    option { value: "", "Todas las entidades" }
                    for t in ENTITY_TYPES {
                        option { value: "{t}", "{t}" }
                    }
                }
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
                        div { class: "table-wrap card",
                            table { class: "table",
                                thead {
                                    tr {
                                        th { "Fecha (UTC)" }
                                        th { "Acción" }
                                        th { "Entidad" }
                                        th { "Detalle" }
                                    }
                                }
                                tbody {
                                    for l in list {
                                        {
                                            let key = l.id.to_string();
                                            let when = l.created_at.format("%Y-%m-%d %H:%M").to_string();
                                            let action = l.action.clone();
                                            let entity = format!("{} {}", l.entity_type, l.entity_id);
                                            let detail = audit_detail(l);
                                            rsx! {
                                                tr { key: "{key}",
                                                    td { class: "muted", "{when}" }
                                                    td { "{action}" }
                                                    td { class: "muted small", "{entity}" }
                                                    td { class: "muted small", "{detail}" }
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
