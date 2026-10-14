//! Portada: bienvenida + estado en vivo del backend.

use dioxus::prelude::*;

use crate::{api, components::Layout, route::Route};

#[component]
pub fn Home() -> Element {
    let health = use_resource(|| async move { api::health().await });

    rsx! {
        Layout { title: "Centro de relaciones".to_string(),
            p { class: "subtitle",
                "Frontend del CRM construido con Dioxus. Los datos y la autenticación los provee el API del proyecto occb."
            }

            section { class: "card",
                h2 { class: "card-title", "Backend" }
                match &*health.read() {
                    None => rsx! {
                        p { class: "muted", "Consultando /api/v1/health…" }
                    },
                    Some(Ok(h)) => {
                        let env = h.env.clone().unwrap_or_else(|| "—".to_string());
                        let ver = h.version.clone().unwrap_or_else(|| "—".to_string());
                        rsx! {
                            p {
                                span { class: "dot dot-ok" }
                                "En línea · entorno "
                                strong { "{env}" }
                                " · versión "
                                strong { "{ver}" }
                            }
                            Link { to: Route::Status {}, class: "btn btn-small", "Ver estado detallado" }
                        }
                    }
                    Some(Err(e)) => rsx! {
                        div { class: "alert alert-error",
                            p { "El backend no responde." }
                            p { class: "muted small", "{e}" }
                        }
                    },
                }
            }

            section { class: "grid-cards",
                div { class: "card",
                    h2 { class: "card-title", "Panel" }
                    p { class: "muted", "Tu perfil y gestión de la sesión (renovar, salir)." }
                    Link { to: Route::Dashboard {}, class: "btn btn-small", "Abrir panel" }
                }
                div { class: "card",
                    h2 { class: "card-title", "Usuarios" }
                    p { class: "muted",
                        "Listado con búsqueda, orden y paginación (requiere sesión)."
                    }
                    Link { to: Route::Users {}, class: "btn btn-small", "Ver usuarios" }
                }
                div { class: "card",
                    h2 { class: "card-title", "Acceso" }
                    p { class: "muted",
                        "Entra con tu cuenta o crea la primera (el backend la registra)."
                    }
                    div { class: "row",
                        Link { to: Route::Login {}, class: "btn btn-small", "Entrar" }
                        Link {
                            to: Route::Register {},
                            class: "btn btn-small btn-primary",
                            "Crear cuenta"
                        }
                    }
                }
            }
        }
    }
}
