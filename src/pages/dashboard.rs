//! Panel: tablero operativo para usuarios (KPIs, embudo, destacados).
//!
//! El perfil y la seguridad viven en `/cuenta`.

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::use_auth,
    components::Layout,
    route::Route,
};

/// Tablero con métricas del negocio. Requiere sesión.
#[component]
pub fn Dashboard() -> Element {
    let auth = use_auth();

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_s = token_of.clone();
    let mut stats = use_resource(move || {
        let token = token_s.clone();
        async move {
            match token {
                Some(t) => api::dashboard_stats(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_t = token_of.clone();
    let mut top = use_resource(move || {
        let token = token_t.clone();
        async move {
            match token {
                Some(t) => api::dashboard_top_deals(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_p = token_of.clone();
    let mut funnel = use_resource(move || {
        let token = token_p.clone();
        async move {
            match token {
                Some(t) => api::dashboard_pipeline(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_r = token_of.clone();
    let mut recent = use_resource(move || {
        let token = token_r.clone();
        async move {
            match token {
                Some(t) => api::dashboard_recent_activities(&t).await,
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
            match &*stats.read() {
                None => rsx! { p { class: "muted", "Cargando resumen…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p { "No se pudo cargar el resumen." }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| stats.restart(), "Reintentar" }
                    }
                },
                Some(Ok(s)) => {
                    let revenue = format!("${:.0}", s.total_revenue);
                    rsx! {
                        section { class: "grid grid-cols-2 gap-4 lg:grid-cols-4",
                            Kpi { label: "Contactos", value: s.total_contacts.to_string(), sub: "en base".to_string() }
                            Kpi { label: "Negocios activos", value: s.active_deals.to_string(), sub: "en curso".to_string() }
                            Kpi {
                                label: "Ganados / Perdidos",
                                value: format!("{} / {}", s.won_deals, s.lost_deals),
                                sub: "cerrados".to_string(),
                            }
                            Kpi { label: "Ingresos", value: revenue, sub: "negocios cerrados".to_string() }
                        }
                    }
                },
            }
            div { class: "mt-4 grid gap-4 lg:grid-cols-2",
                section { class: "card",
                    div { class: "mb-2 flex items-center justify-between",
                        h2 { class: "card-title", "Embudo por etapa" }
                        Link { to: Route::Deals {}, class: "link small", "Abrir tablero →" }
                    }
                    match &*funnel.read() {
                        None => rsx! { p { class: "muted", "Cargando…" } },
                        Some(Err(e)) => rsx! { p { class: "muted small", "No se pudo cargar: {e}" } },
                        Some(Ok(f)) => {
                            let max = f.stages.iter().map(|s| s.count).max().unwrap_or(1).max(1) as f64;
                            rsx! {
                                div { class: "flex flex-col gap-2",
                                    for st in &f.stages {
                                        {
                                            let label = api::deal_stage_label(&st.stage).to_string();
                                            let detail = format!("{} · ${:.0}", st.count, st.total_value);
                                            let width = (st.count as f64 / max * 100.0).max(4.0);
                                            let bar = format!("width: {:.0}%", width);
                                            rsx! {
                                                div { key: "{st.stage}",
                                                    div { class: "flex justify-between text-sm",
                                                        span { class: "font-semibold", "{label}" }
                                                        span { class: "muted", "{detail}" }
                                                    }
                                                    div { class: "mt-1 h-2.5 overflow-hidden rounded-full bg-muted",
                                                        div { class: "h-full rounded-full bg-primary", style: "{bar}" }
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
                section { class: "card",
                    div { class: "mb-2 flex items-center justify-between",
                        h2 { class: "card-title", "Mejores negocios" }
                        Link { to: Route::Reports {}, class: "link small", "Ver informes →" }
                    }
                    match &*top.read() {
                        None => rsx! { p { class: "muted", "Cargando…" } },
                        Some(Err(e)) => rsx! { p { class: "muted small", "No se pudo cargar: {e}" } },
                        Some(Ok(t)) => rsx! {
                            if t.deals.is_empty() {
                                p { class: "muted", "Aún no hay negocios." }
                            } else {
                                ul { class: "flex flex-col gap-2",
                                    for d in &t.deals {
                                        {
                                            let did = d.id.to_string();
                                            let title = d.title.clone();
                                            let st = api::deal_stage_label(&d.stage).to_string();
                                            let amount = format!("${:.0}", d.value);
                                            rsx! {
                                                li { key: "{did}", class: "flex items-center justify-between gap-2",
                                                    Link {
                                                        to: Route::DealDetail { id: did },
                                                        class: "link truncate",
                                                        "{title}"
                                                    }
                                                    span { class: "shrink-0 text-sm text-muted", "{st} · {amount}" }
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
            section { class: "card",
                div { class: "mb-2 flex items-center justify-between",
                    h2 { class: "card-title", "Actividad reciente" }
                    Link { to: Route::Activities {}, class: "link small", "Ver actividades →" }
                }
                match &*recent.read() {
                    None => rsx! { p { class: "muted", "Cargando…" } },
                    Some(Err(e)) => rsx! { p { class: "muted small", "No se pudo cargar: {e}" } },
                    Some(Ok(r)) => rsx! {
                        if r.activities.is_empty() {
                            p { class: "muted", "Sin actividad todavía." }
                        } else {
                            ul { class: "timeline",
                                for a in &r.activities {
                                    {
                                        let key = a.id.to_string();
                                        let subject = a.subject.clone();
                                        let kind = api::activity_type_label(&a.activity_type).to_string();
                                        let who = a
                                            .contact_name
                                            .clone()
                                            .map(|c| format!(" · {c}"))
                                            .unwrap_or_default();
                                        let when = a.created_at.format("%d %b %H:%M").to_string();
                                        rsx! {
                                            li { key: "{key}", class: "timeline-item",
                                                span { class: "timeline-dot" }
                                                div {
                                                    strong { "{subject}" }
                                                    p { class: "muted small", "{kind}{who} · {when}" }
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
}

/// Tarjeta de indicador.
#[component]
fn Kpi(label: String, value: String, sub: String) -> Element {
    rsx! {
        div { class: "card kpi",
            p { class: "kpi-label", "{label}" }
            p { class: "kpi-value", "{value}" }
            p { class: "muted small", "{sub}" }
        }
    }
}
