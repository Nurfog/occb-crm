//! Informes: pipeline y ganados/perdidos (`/api/v1/reports/*`).

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::use_auth,
    components::Layout,
    route::Route,
};

/// Reportes del negocio. Requiere sesión.
#[component]
pub fn Reports() -> Element {
    let auth = use_auth();
    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_p = token_of.clone();
    let mut pipeline = use_resource(move || {
        let token = token_p.clone();
        async move {
            match token {
                Some(t) => api::pipeline_report(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_w = token_of.clone();
    let mut winloss = use_resource(move || {
        let token = token_w.clone();
        async move {
            match token {
                Some(t) => api::win_loss_report(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Informes".to_string(),
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
        Layout { title: "Informes".to_string(),
            section { class: "card",
                h2 { class: "card-title", "Pipeline" }
                match &*pipeline.read() {
                    None => rsx! { p { class: "muted", "Cargando…" } },
                    Some(Err(e)) => rsx! { p { class: "muted small", "No se pudo cargar: {e}" } },
                    Some(Ok(r)) => {
                        let total = format!("{} negocios · {:.0} en total", r.total_deals, r.total_value);
                        rsx! {
                            p { class: "muted", "{total}" }
                            div { class: "grid-cards",
                                for st in &r.stages {
                                    {
                                        let label = api::deal_stage_label(&st.stage).to_string();
                                        let pct = format!("{:.1}%", st.percentage);
                                        let detail = format!(
                                            "{} negocios · {:.0} · promedio {:.0} · {pct}",
                                            st.count, st.total_value, st.avg_value
                                        );
                                        rsx! {
                                            div { class: "card", key: "{st.stage}",
                                                h2 { class: "card-title", "{label}" }
                                                p { "{detail}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
            }
            section { class: "card",
                h2 { class: "card-title", "Ganados vs perdidos" }
                match &*winloss.read() {
                    None => rsx! { p { class: "muted", "Cargando…" } },
                    Some(Err(e)) => rsx! { p { class: "muted small", "No se pudo cargar: {e}" } },
                    Some(Ok(w)) => {
                        let win = format!("{:.1}%", w.win_rate);
                        let loss = format!("{:.1}%", w.loss_rate);
                        let won = format!("{} ({win}) · {:.0}", w.won_count, w.won_value);
                        let lost = format!("{} ({loss}) · {:.0}", w.lost_count, w.lost_value);
                        rsx! {
                            div { class: "grid-cards",
                                div { class: "card",
                                    h2 { class: "card-title", "Ganados" }
                                    p { "{won}" }
                                }
                                div { class: "card",
                                    h2 { class: "card-title", "Perdidos" }
                                    p { "{lost}" }
                                }
                                div { class: "card",
                                    h2 { class: "card-title", "Cerrados" }
                                    p { "{w.total_closed}" }
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}
