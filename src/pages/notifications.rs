//! Notificaciones: bandeja, conteo y preferencias (`/api/v1/notifications`).

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::use_auth,
    components::Layout,
    route::Route,
};

/// Bandeja de notificaciones. Requiere sesión.
#[component]
pub fn Notifications() -> Element {
    let auth = use_auth();
    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_l = token_of.clone();
    let mut items = use_resource(move || {
        let token = token_l.clone();
        async move {
            match token {
                Some(t) => api::list_notifications(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_p = token_of.clone();
    let mut prefs = use_resource(move || {
        let token = token_p.clone();
        async move {
            match token {
                Some(t) => api::get_prefs(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Notificaciones".to_string(),
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
        Layout { title: "Notificaciones".to_string(),
            div { class: "row",
                button {
                    class: "btn btn-small",
                    onclick: move |_| {
                        let token = token_of.clone();
                        spawn(async move {
                            if let Some(t) = token {
                                let _ = api::mark_all_notifications_read(&t).await;
                            }
                            items.restart();
                        });
                    },
                    "Marcar todas como leídas"
                }
            }
            match &*items.read() {
                None => rsx! { p { class: "muted", "Cargando…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p { "No se pudieron cargar." }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| items.restart(), "Reintentar" }
                    }
                },
                Some(Ok(list)) => rsx! {
                    if list.is_empty() {
                        p { class: "muted", "Sin notificaciones." }
                    } else {
                        ul {
                            for n in list {
                                {
                                    let key = n.id.to_string();
                                    let id_v = key.clone();
                                    let title = n.title.clone();
                                    let msg = n.message.clone();
                                    let when = n.created_at.format("%Y-%m-%d %H:%M").to_string();
                                    let unread = !n.read;
                                    let cls = if unread { "unread" } else { "" };
                                    let token_base = token_of.clone();
                                    let token_read = token_base.clone();
                                    let token_del = token_base.clone();
                                    rsx! {
                                        li { key: "{key}", class: "{cls}",
                                            strong { "{title}" }
                                            p { class: "small", "{msg}" }
                                            p { class: "muted small", "{when}" }
                                            div { class: "row",
                                                if unread {
                                                    button {
                                                        class: "btn btn-small",
                                                        onclick: move |_| {
                                                            let token_v = token_read.clone();
                                                        },
                                                        "Marcar leída"
                                                    }
                                                }
                                                button {
                                                    class: "btn btn-small",
                                                    onclick: move |_| {
                                                        let token_v = token_del.clone();
                                                        let id_c = id_v.clone();
                                                        spawn(async move {
                                                            if let Some(t) = token_v {
                                                                let _ = api::delete_notification(&t, &id_c).await;
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
                },
            }
            section { class: "card",
                h2 { class: "card-title", "Preferencias" }
                match &*prefs.read() {
                    None => rsx! { p { class: "muted", "Cargando…" } },
                    Some(Err(e)) => rsx! { p { class: "muted small", "No se pudieron cargar: {e}" } },
                    Some(Ok(p)) => rsx! {
                        PrefsEditor { prefs: p.clone(), on_saved: move || prefs.restart() }
                    },
                }
            }
        }
    }
}

/// Edición de preferencias de notificación.
#[component]
fn PrefsEditor(prefs: api::NotificationPreferences, on_saved: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut email = use_signal(|| prefs.email_enabled);
    let mut push = use_signal(|| prefs.push_enabled);
    let mut digest = use_signal(|| prefs.weekly_digest);
    let mut marketing = use_signal(|| prefs.marketing_emails);
    let mut busy = use_signal(|| false);
    let mut saved = use_signal(|| false);

    let busy_v = *busy.read();
    let email_v = *email.read();
    let push_v = *push.read();
    let digest_v = *digest.read();
    let marketing_v = *marketing.read();
    let was_saved = *saved.read();

    rsx! {
        div { class: "form",
            label { class: "consent-opt",
                input {
                    r#type: "checkbox",
                    checked: email_v,
                    onclick: move |_| email.set(!email_v),
                }
                " Emails de actividad"
            }
            label { class: "consent-opt",
                input {
                    r#type: "checkbox",
                    checked: push_v,
                    onclick: move |_| push.set(!push_v),
                }
                " Notificaciones push"
            }
            label { class: "consent-opt",
                input {
                    r#type: "checkbox",
                    checked: digest_v,
                    onclick: move |_| digest.set(!digest_v),
                }
                " Resumen semanal"
            }
            label { class: "consent-opt",
                input {
                    r#type: "checkbox",
                    checked: marketing_v,
                    onclick: move |_| marketing.set(!marketing_v),
                }
                " Emails de marketing"
            }
            if was_saved {
                p { class: "alert alert-ok", "Preferencias guardadas." }
            }
            button {
                class: "btn btn-small btn-primary",
                disabled: busy_v,
                onclick: move |_| {
                    let token = auth
                        .read()
                        .session
                        .clone()
                        .map(|s| s.access_token);
                    busy.set(true);
                    saved.set(false);
                    spawn(async move {
                        if let Some(t) = token {
                            let _ = api::update_prefs(
                                    &t,
                                    &api::UpdateNotificationPreferences {
                                        email_enabled: Some(email_v),
                                        push_enabled: Some(push_v),
                                        weekly_digest: Some(digest_v),
                                        marketing_emails: Some(marketing_v),
                                    },
                                )
                                .await;
                        }
                        busy.set(false);
                        saved.set(true);
                        on_saved.call(());
                    });
                },
                "Guardar preferencias"
            }
        }
    }
}
