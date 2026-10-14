//! Panel: perfil (`GET /api/v1/auth/me`) y gestión de la sesión.

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::{use_auth, Session},
    components::Layout,
    route::Route,
};

#[component]
pub fn Dashboard() -> Element {
    let mut auth = use_auth();
    let nav = use_navigator();
    let mut action_error = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);

    // Se suscribe a la sesión: al renovar el token se recarga el perfil.
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
            Layout { title: "Panel".to_string(),
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
        Layout { title: "Panel".to_string(),
            if let Some(msg) = action_error.read().clone() {
                div { class: "alert alert-error", "{msg}" }
            }
            match &*me.read() {
                None => rsx! {
                    p { class: "muted", "Cargando tu perfil…" }
                },
                Some(Ok(user)) => {
                    let created = user.created_at.format("%Y-%m-%d %H:%M").to_string();
                    let refresh_token = auth
                        .read()
                        .session
                        .clone()
                        .map(|s| s.refresh_token)
                        .unwrap_or_default();
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
                                div {
                                    dt { "Alta" }
                                    dd { "{created} UTC" }
                                }
                            }
                            div { class: "row",
                                button {
                                    class: "btn btn-small",
                                    disabled: *busy.read(),
                                    onclick: move |_| {
                                        let refresh_v = refresh_token.clone();
                                        action_error.set(None);
                                        busy.set(true);
                                        spawn(async move {
                                            match api::refresh(&refresh_v).await {
                                                Ok(pair) => {
                                                    auth.write().session = Some(Session::from_pair(pair));
                                                    me.restart();
                                                }
                                                Err(err) => action_error.set(Some(err.to_string())),
                                            }
                                            busy.set(false);
                                        });
                                    },
                                    if *busy.read() {
                                        "Renovando…"
                                    } else {
                                        "Renovar tokens"
                                    }
                                }
                                button { class: "btn btn-small", onclick: move |_| me.restart(), "Recargar perfil" }
                                Link { to: Route::Users {}, class: "btn btn-small", "Ver usuarios" }
                            }
                            p { class: "muted small",
                                "Renovar rota el refresh token en el backend: el anterior queda revocado."
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p { "No se pudo cargar tu perfil." }
                        p { class: "muted small", "{e}" }
                        div { class: "row",
                            button { class: "btn btn-small", onclick: move |_| me.restart(), "Reintentar" }
                            button {
                                class: "btn btn-small",
                                onclick: move |_| {
                                    auth.write().session = None;
                                    nav.push(Route::Login {});
                                },
                                "Cerrar sesión local"
                            }
                        }
                    }
                },
            }
        }
    }
}
