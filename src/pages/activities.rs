//! Actividades: listado, alta, completar y borrado (`/api/v1/activities`).

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError, ACTIVITY_TYPES},
    auth::use_auth,
    components::Layout,
    route::Route,
};

fn none_if_empty(s: &str) -> Option<String> {
    if s.is_empty() { None } else { Some(s.to_string()) }
}

/// Listado con filtro por tipo + alta. Requiere sesión.
#[component]
pub fn Activities() -> Element {
    let auth = use_auth();
    let mut kind = use_signal(String::new);
    let mut show_new = use_signal(|| false);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let mut items = use_resource(move || {
        let token = token_of.clone();
        let k = kind.read().clone();
        async move {
            match token {
                Some(t) => api::list_activities(&t, &k, "", "", "").await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Actividades".to_string(),
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

    let creating = *show_new.read();
    let new_label = if creating { "Cerrar formulario" } else { "+ Nueva actividad" };

    rsx! {
        Layout { title: "Actividades".to_string(),
            div { class: "row",
                button {
                    class: "btn btn-small btn-primary",
                    onclick: move |_| show_new.set(!creating),
                    "{new_label}"
                }
                select {
                    class: "input",
                    value: "{kind}",
                    onchange: move |e| {
                        kind.set(e.value());
                        items.restart();
                    },
                    option { value: "", "Todos los tipos" }
                    for s in ACTIVITY_TYPES {
                        {
                            let label = api::activity_type_label(s).to_string();
                            rsx! { option { value: s, "{label}" } }
                        }
                    }
                }
            }
            if creating {
                NewActivityForm { on_created: move || {
                    show_new.set(false);
                    items.restart();
                } }
            }
            match &*items.read() {
                None => rsx! { p { class: "muted", "Cargando actividades…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p {
                            if e.status == 403 {
                                "No tienes permiso para ver actividades."
                            } else {
                                "No se pudieron cargar."
                            }
                        }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| items.restart(), "Reintentar" }
                    }
                },
                Some(Ok(list)) => rsx! {
                    if list.is_empty() {
                        p { class: "muted", "Sin actividades." }
                    } else {
                        div { class: "table-wrap card",
                            table { class: "table",
                                thead {
                                    tr {
                                        th { "Asunto" }
                                        th { "Tipo" }
                                        th { "Vence" }
                                        th { "Estado" }
                                        th { "Acciones" }
                                    }
                                }
                                tbody {
                                    for a in list {
                                        {
                                            let key = a.id.to_string();
                                            let id_v = key.clone();
                                            let subj = a.subject.clone();
                                            let label = api::activity_type_label(&a.activity_type).to_string();
                                            let due = a
                                                .due_date
                                                .map(|d| d.format("%Y-%m-%d").to_string())
                                                .unwrap_or_else(|| "—".to_string());
                                            let state = if a.completed { "Hecha ✓" } else { "Pendiente" };
                                            let show_done = !a.completed;
                                            let key_done = key.clone();
                                            let key_del = key.clone();
                                            let token_base = auth
                                                .read()
                                                .session
                                                .clone()
                                                .map(|s| s.access_token);
                                            let token_done = token_base.clone();
                                            let token_del = token_base.clone();
                                            rsx! {
                                                tr { key: "{key}",
                                                    td { "{subj}" }
                                                    td { "{label}" }
                                                    td { class: "muted", "{due}" }
                                                    td { "{state}" }
                                                    td {
                                                        if show_done {
                                                            button {
                                                                class: "btn btn-small",
                                                                onclick: move |_| {
                                                                    let token_v = token_done.clone();
                                                                    let id_c = key_done.clone();
                                                                    spawn(async move {
                                                                        if let Some(t) = token_v {
                                                                            let _ = api::complete_activity(&t, &id_c).await;
                                                                        }
                                                                        items.restart();
                                                                    });
                                                                },
                                                                "Marcar hecha"
                                                            }
                                                        }
                                                        button {
                                                            class: "btn btn-small",
                                                            onclick: move |_| {
                                                                let token_v = token_del.clone();
                                                                let id_c = key_del.clone();
                                                                spawn(async move {
                                                                    if let Some(t) = token_v {
                                                                        let _ = api::delete_activity(&t, &id_c).await;
                                                                    }
                                                                    items.restart();
                                                                });
                                                            },
                                                            "Eliminar"
                                                        }
                                                    }
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

/// Alta de actividad.
#[component]
fn NewActivityForm(on_created: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut kind = use_signal(|| "task".to_string());
    let mut subject = use_signal(String::new);
    let mut due = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let busy_v = *busy.read();
    let submit_label = if busy_v { "Guardando…" } else { "Crear actividad" };

    rsx! {
        form {
            class: "card form",
            onsubmit: move |e| {
                e.prevent_default();
                let token = auth.read().session.clone().map(|s| s.access_token);
                let k = kind.read().clone();
                let subj = subject.read().trim().to_string();
                if subj.is_empty() {
                    error.set(Some("El asunto es obligatorio.".to_string()));
                    return;
                }
                let due_v = none_if_empty(due.read().trim());
                busy.set(true);
                error.set(None);
                spawn(async move {
                    let res = match token {
                        Some(t) => {
                            api::create_activity(
                                    &t,
                                    &api::CreateActivityBody {
                                        activity_type: &k,
                                        subject: &subj,
                                        description: None,
                                        contact_id: None,
                                        deal_id: None,
                                        company_id: None,
                                        due_date: due_v.as_deref(),
                                    },
                                )
                                .await
                                .map(|_| ())
                                .map_err(|e| e.to_string())
                        }
                        None => Err("Sin sesión.".to_string()),
                    };
                    busy.set(false);
                    match res {
                        Ok(()) => on_created.call(()),
                        Err(msg) => error.set(Some(msg)),
                    }
                });
            },
            h2 { class: "card-title", "Nueva actividad" }
            div { class: "row",
                div {
                    label { class: "field", "Tipo" }
                    select {
                        class: "input",
                        value: "{kind}",
                        onchange: move |e| kind.set(e.value()),
                        for s in ACTIVITY_TYPES {
                            {
                                let label = api::activity_type_label(s).to_string();
                                rsx! { option { value: s, "{label}" } }
                            }
                        }
                    }
                }
                div {
                    label { class: "field", "Vence (AAAA-MM-DD)" }
                    input {
                        class: "input",
                        placeholder: "2026-12-31",
                        value: "{due}",
                        oninput: move |e| due.set(e.value()),
                    }
                }
            }
            label { class: "field", "Asunto *" }
            input {
                class: "input",
                value: "{subject}",
                oninput: move |e| subject.set(e.value()),
                required: true,
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
            button {
                class: "btn btn-primary",
                r#type: "submit",
                disabled: busy_v,
                "{submit_label}"
            }
        }
    }
}
