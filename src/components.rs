//! Componentes compartidos: barra de navegación y marco de página.

use dioxus::prelude::*;

use crate::{api, auth::use_auth, route::Route};

/// Barra superior: marca, enlaces y estado de sesión (con salida).
#[component]
pub fn NavBar() -> Element {
    let mut auth = use_auth();
    let nav = use_navigator();
    let mut busy = use_signal(|| false);

    let session = auth.read().session.clone();

    rsx! {
        header { class: "topbar",
            div { class: "topbar-inner",
                Link { to: Route::Home {}, class: "brand", "OCCB · CRM" }
                nav { class: "nav-links",
                    Link { to: Route::Home {}, class: "nav-link", "Inicio" }
                    Link { to: Route::Dashboard {}, class: "nav-link", "Panel" }
                    Link { to: Route::Users {}, class: "nav-link", "Usuarios" }
                    Link { to: Route::Status {}, class: "nav-link", "Estado" }
                }
                div { class: "nav-user",
                    match session {
                        Some(s) => {
                            let email = s.user.email.clone();
                            let access = s.access_token.clone();
                            let refresh = s.refresh_token.clone();
                            rsx! {
                                span { class: "muted nav-email", "{email}" }
                                button {
                                    class: "btn btn-small",
                                    disabled: *busy.read(),
                                    onclick: move |_| {
                                        let access_v = access.clone();
                                        let refresh_v = refresh.clone();
                                        busy.set(true);
                                        spawn(async move {
                                            // Mejor esfuerzo: aunque el backend falle,
                                            // la sesión local se cierra igual.
                                            let _ = api::logout(&access_v, &refresh_v).await;
                                            auth.write().session = None;
                                            busy.set(false);
                                            nav.push(Route::Home {});
                                        });
                                    },
                                    "Salir"
                                }
                            }
                        }
                        None => rsx! {
                            Link { to: Route::Login {}, class: "btn btn-small", "Entrar" }
                            Link {
                                to: Route::Register {},
                                class: "btn btn-small btn-primary",
                                "Crear cuenta"
                            }
                        },
                    }
                }
            }
        }
    }
}

/// Marco de página: navegación + título + contenido + pie.
#[component]
pub fn Layout(title: String, children: Element) -> Element {
    rsx! {
        div { class: "shell",
            NavBar {}
            main { class: "container",
                h1 { class: "page-title", "{title}" }
                {children}
            }
            footer { class: "footer",
                span { class: "muted", "OCCB CRM · frontend Dioxus · el API vive en el proyecto occb" }
            }
        }
    }
}
