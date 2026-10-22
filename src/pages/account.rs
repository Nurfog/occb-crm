//! Mi cuenta: perfil, rectificación y cambio de clave.

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::use_auth,
    components::Layout,
    pages::password::{ChangePassword, EditProfile},
    route::Route,
};

/// Datos de la cuenta del dueño de la sesión. Requiere sesión.
#[component]
pub fn Account() -> Element {
    let auth = use_auth();

    let mut me = use_resource(move || {
        let token = auth.read().session.clone().map(|s| s.access_token);
        async move {
            match token {
                Some(t) => api::me(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Mi cuenta".to_string(),
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
        Layout { title: "Mi cuenta".to_string(),
            match &*me.read() {
                None => rsx! { p { class: "muted", "Cargando tu perfil…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p { "No se pudo cargar tu perfil." }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| me.restart(), "Reintentar" }
                    }
                },
                Some(Ok(user)) => {
                    let perms = if user.permissions.is_empty() {
                        "—".to_string()
                    } else {
                        user.permissions.join(", ")
                    };
                    rsx! {
                        section { class: "card",
                            h2 { class: "card-title", "Tu perfil" }
                            dl { class: "detail-list",
                                div {
                                    dt { "Nombre" }
                                    dd { "{user.name}" }
                                }
                                div {
                                    dt { "Email" }
                                    dd { "{user.email}" }
                                }
                                if let Some(first) = user.first_name.clone() {
                                    div {
                                        dt { "Nombres" }
                                        dd { "{first}" }
                                    }
                                }
                                if let Some(last) = user.last_name.clone() {
                                    div {
                                        dt { "Apellidos" }
                                        dd { "{last}" }
                                    }
                                }
                                div {
                                    dt { "Permisos" }
                                    dd { class: "muted small", "{perms}" }
                                }
                            }
                            EditProfile { on_saved: move |_| me.restart() }
                        }
                        section { class: "card",
                            ChangePassword {}
                        }
                    }
                },
            }
        }
    }
}
