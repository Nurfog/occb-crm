//! Búsqueda global (`/api/v1/search?q=…`).

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::use_auth,
    components::Layout,
    route::Route,
};

/// Buscador global de contactos, empresas y negocios. Requiere sesión.
#[component]
pub fn Search(q: String) -> Element {
    let auth = use_auth();
    let initial = q.clone();
    let mut q = use_signal(|| initial.clone());
    let mut applied = use_signal(|| initial);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let mut results = use_resource(move || {
        let token = token_of.clone();
        let query = applied.read().clone();
        async move {
            match token {
                Some(t) if !query.trim().is_empty() => {
                    api::global_search(&t, &query).await.map(Some)
                }
                Some(_) => Ok(None),
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Buscar".to_string(),
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
        Layout { title: "Buscar".to_string(),
            form {
                class: "toolbar card",
                onsubmit: move |e| {
                    e.prevent_default();
                    applied.set(q.read().clone());
                },
                input {
                    class: "input",
                    r#type: "search",
                    placeholder: "Buscar en contactos, empresas y negocios…",
                    value: "{q}",
                    oninput: move |e| q.set(e.value()),
                }
                button { class: "btn btn-small btn-primary", r#type: "submit", "Buscar" }
            }
            match &*results.read() {
                None => rsx! { p { class: "muted", "Escribe y pulsa Buscar." } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p { "No se pudo buscar." }
                        p { class: "muted small", "{e}" }
                    }
                },
                Some(Ok(None)) => rsx! { p { class: "muted", "Escribe y pulsa Buscar." } },
                Some(Ok(Some(r))) => rsx! {
                    section { class: "card",
                        h2 { class: "card-title", "Contactos" }
                        if r.contacts.is_empty() {
                            p { class: "muted", "Sin resultados." }
                        } else {
                            ul {
                                for h in &r.contacts {
                                    {
                                        let key = h.id.clone();
                                        let label = h.label.clone();
                                        let sub = h.subtitle.clone();
                                        rsx! {
                                            li { key: "{key}",
                                                Link {
                                                    to: Route::ContactDetail { id: key.clone() },
                                                    class: "link",
                                                    "{label}"
                                                }
                                                span { class: "muted small", " {sub}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    section { class: "card",
                        h2 { class: "card-title", "Empresas" }
                        if r.companies.is_empty() {
                            p { class: "muted", "Sin resultados." }
                        } else {
                            ul {
                                for h in &r.companies {
                                    {
                                        let key = h.id.clone();
                                        let label = h.label.clone();
                                        let sub = h.subtitle.clone();
                                        rsx! {
                                            li { key: "{key}",
                                                Link {
                                                    to: Route::CompanyDetail { id: key.clone() },
                                                    class: "link",
                                                    "{label}"
                                                }
                                                span { class: "muted small", " {sub}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    section { class: "card",
                        h2 { class: "card-title", "Negocios" }
                        if r.deals.is_empty() {
                            p { class: "muted", "Sin resultados." }
                        } else {
                            ul {
                                for h in &r.deals {
                                    {
                                        let key = h.id.clone();
                                        let label = h.label.clone();
                                        let sub = h.subtitle.clone();
                                        rsx! {
                                            li { key: "{key}",
                                                Link {
                                                    to: Route::DealDetail { id: key.clone() },
                                                    class: "link",
                                                    "{label}"
                                                }
                                                span { class: "muted small", " {sub}" }
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
