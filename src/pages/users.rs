//! Usuarios: listado paginado (`GET /api/v1/users`) y detalle (`GET /api/v1/users/{id}`).

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::use_auth,
    components::Layout,
    route::Route,
};

const SORT_OPTIONS: [(&str, &str); 3] = [
    ("created_at", "Fecha de alta"),
    ("email", "Email"),
    ("name", "Nombre"),
];

/// Listado con búsqueda, orden y paginación. Requiere sesión.
#[component]
pub fn Users() -> Element {
    let auth = use_auth();
    let mut search = use_signal(String::new);
    let mut applied = use_signal(String::new);
    let mut page = use_signal(|| 1u32);
    let mut per_page = use_signal(|| 20u32);
    let mut sort = use_signal(|| "created_at".to_string());
    let mut dir = use_signal(|| "desc".to_string());

    let mut users = use_resource(move || {
        let token = auth.read().session.clone().map(|s| s.access_token);
        let q = applied.read().clone();
        let p = *page.read();
        let pp = *per_page.read();
        let sort_v = sort.read().clone();
        let dir_v = dir.read().clone();
        async move {
            match token {
                Some(t) => api::list_users(&t, p, pp, &q, &sort_v, &dir_v).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Usuarios".to_string(),
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
        Layout { title: "Usuarios".to_string(),
            form {
                class: "toolbar card",
                onsubmit: move |e| {
                    e.prevent_default();
                    applied.set(search.read().clone());
                    page.set(1);
                },
                input {
                    class: "input",
                    r#type: "search",
                    placeholder: "Buscar por email o nombre…",
                    value: "{search}",
                    oninput: move |e| search.set(e.value()),
                }
                select {
                    class: "input",
                    value: "{sort}",
                    onchange: move |e| {
                        sort.set(e.value());
                        page.set(1);
                    },
                    for (value, label) in SORT_OPTIONS {
                        option { value, "{label}" }
                    }
                }
                select {
                    class: "input",
                    value: "{dir}",
                    onchange: move |e| {
                        dir.set(e.value());
                        page.set(1);
                    },
                    option { value: "desc", "Desc" }
                    option { value: "asc", "Asc" }
                }
                select {
                    class: "input",
                    value: "{per_page}",
                    onchange: move |e| {
                        if let Ok(n) = e.value().parse::<u32>() {
                            per_page.set(n.clamp(1, 100));
                            page.set(1);
                        }
                    },
                    option { value: "10", "10" }
                    option { value: "20", "20" }
                    option { value: "50", "50" }
                }
                button { class: "btn btn-small btn-primary", r#type: "submit", "Buscar" }
            }

            match &*users.read() {
                None => rsx! {
                    p { class: "muted", "Cargando usuarios…" }
                },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p { "No se pudo cargar el listado." }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| users.restart(), "Reintentar" }
                    }
                },
                Some(Ok(paged)) => {
                    let total_pages = (paged.total + paged.per_page - 1) / paged.per_page.max(1);
                    let total_pages = total_pages.max(1);
                    let current = *page.read();
                    rsx! {
                        div { class: "table-wrap card",
                            table { class: "table",
                                thead {
                                    tr {
                                        th { "Nombre" }
                                        th { "Email" }
                                        th { "Alta (UTC)" }
                                    }
                                }
                                tbody {
                                    for user in &paged.data {
                                        {
                                            let created = user
                                                .created_at
                                                .format("%Y-%m-%d %H:%M")
                                                .to_string();
                                            let id = user.id.to_string();
                                            rsx! {
                                                tr { key: "{id}",
                                                    td {
                                                        Link {
                                                            to: Route::UserDetail {
                                                                id: id.clone(),
                                                            },
                                                            class: "link",
                                                            "{user.name}"
                                                        }
                                                    }
                                                    td { "{user.email}" }
                                                    td { class: "muted", "{created}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if paged.data.is_empty() {
                                p { class: "muted", "Sin resultados para esta búsqueda." }
                            }
                        }
                        div { class: "pagination",
                            button {
                                class: "btn btn-small",
                                disabled: current <= 1,
                                onclick: move |_| page.set(current.saturating_sub(1).max(1)),
                                "← Anterior"
                            }
                            span { class: "muted small", "Página {current} de {total_pages} · {paged.total} en total" }
                            button {
                                class: "btn btn-small",
                                disabled: current as i64 >= total_pages,
                                onclick: move |_| page.set(current + 1),
                                "Siguiente →"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Detalle de un usuario. Requiere sesión.
#[component]
pub fn UserDetail(id: String) -> Element {
    let auth = use_auth();

    let mut user = use_resource(move || {
        let token = auth.read().session.clone().map(|s| s.access_token);
        let id_v = id.clone();
        async move {
            match token {
                Some(t) => api::get_user(&t, &id_v).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Usuario".to_string(),
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
        Layout { title: "Usuario".to_string(),
            Link { to: Route::Users {}, class: "link", "← Volver al listado" }
            match &*user.read() {
                None => rsx! {
                    p { class: "muted", "Cargando usuario…" }
                },
                Some(Ok(u)) => {
                    let created = u.created_at.format("%Y-%m-%d %H:%M").to_string();
                    rsx! {
                        section { class: "card",
                            h2 { class: "card-title", "{u.name}" }
                            dl { class: "detail-list",
                                div {
                                    dt { "Email" }
                                    dd { "{u.email}" }
                                }
                                div {
                                    dt { "ID" }
                                    dd {
                                        code { "{u.id}" }
                                    }
                                }
                                div {
                                    dt { "Alta" }
                                    dd { "{created} UTC" }
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p { "No se pudo cargar el usuario." }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| user.restart(), "Reintentar" }
                    }
                },
            }
        }
    }
}
