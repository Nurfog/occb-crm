//! Estado del backend: `health`, `ready` y `version` (públicos).

use dioxus::prelude::*;

use crate::{api, components::Layout};

#[component]
pub fn Status() -> Element {
    let mut health = use_resource(|| async move { api::health().await });
    let mut ready = use_resource(|| async move { api::ready().await });
    let mut version = use_resource(|| async move { api::version().await });

    rsx! {
        Layout { title: "Estado del backend".to_string(),
            p { class: "subtitle",
                "Liveness, readiness y versión tal como las expone el proyecto occb."
            }
            div { class: "grid-cards",
                section { class: "card",
                    h2 { class: "card-title", "Health" }
                    code { class: "muted small", "GET /api/v1/health" }
                    match &*health.read() {
                        None => rsx! {
                            p { class: "muted", "Cargando…" }
                        },
                        Some(Ok(h)) => {
                            let env = h.env.clone().unwrap_or_else(|| "—".to_string());
                            let ver = h.version.clone().unwrap_or_else(|| "—".to_string());
                            rsx! {
                                p {
                                    span { class: "dot dot-ok" }
                                    strong { "{h.status}" }
                                    " · {env} · {ver}"
                                }
                            }
                        }
                        Some(Err(e)) => rsx! {
                            p { class: "alert alert-error", "{e}" }
                        },
                    }
                }
                section { class: "card",
                    h2 { class: "card-title", "Ready" }
                    code { class: "muted small", "GET /api/v1/ready" }
                    match &*ready.read() {
                        None => rsx! {
                            p { class: "muted", "Cargando…" }
                        },
                        Some(Ok(r)) => {
                            let db = r
                                .checks
                                .as_ref()
                                .and_then(|c| c.db.clone())
                                .unwrap_or_else(|| "—".to_string());
                            let redis = r
                                .checks
                                .as_ref()
                                .and_then(|c| c.redis.clone())
                                .unwrap_or_else(|| "—".to_string());
                            let ok = r.status == "ready";
                            rsx! {
                                p {
                                    span { class: if ok { "dot dot-ok" } else { "dot dot-bad" } }
                                    strong { "{r.status}" }
                                    " · db: {db} · redis: {redis}"
                                }
                            }
                        }
                        Some(Err(e)) => rsx! {
                            p { class: "alert alert-error", "{e}" }
                        },
                    }
                }
                section { class: "card",
                    h2 { class: "card-title", "Versión" }
                    code { class: "muted small", "GET /api/v1/version" }
                    match &*version.read() {
                        None => rsx! {
                            p { class: "muted", "Cargando…" }
                        },
                        Some(Ok(v)) => {
                            let name = v.name.clone().unwrap_or_else(|| "—".to_string());
                            let ver = v.version.clone().unwrap_or_else(|| "—".to_string());
                            rsx! {
                                p {
                                    strong { "{name}" }
                                    " · {ver}"
                                }
                            }
                        }
                        Some(Err(e)) => rsx! {
                            p { class: "alert alert-error", "{e}" }
                        },
                    }
                }
            }
            button {
                class: "btn btn-small",
                onclick: move |_| {
                    health.restart();
                    ready.restart();
                    version.restart();
                },
                "Actualizar"
            }
        }
    }
}
