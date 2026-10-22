//! Prospectos: listado, detalle, alta, conversión (`/api/v1/leads`).

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError, LEAD_SOURCES, LEAD_STATUSES},
    auth::use_auth,
    components::{tags::EntityTags, Layout},
    route::Route,
};

fn none_if_empty(s: &str) -> Option<String> {
    if s.is_empty() { None } else { Some(s.to_string()) }
}

/// Color del puntaje según tramo, como el original.
fn score_class(score: i32) -> &'static str {
    if score >= 80 {
        "font-bold text-green-700"
    } else if score >= 50 {
        "font-bold text-yellow-700"
    } else if score >= 20 {
        "font-bold text-orange-700"
    } else {
        "font-bold text-red-700"
    }
}

/// Pastilla de estado del prospecto.
fn status_class(status: &str) -> &'static str {
    match status {
        "new" => "tag tag-info",
        "contacted" => "tag tag-warning",
        "qualified" => "tag tag-ok",
        "unqualified" => "tag tag-muted",
        "converted" => "tag tag-violet",
        _ => "tag tag-warning",
    }
}

/// Listado con filtros + estadísticas + alta. Requiere sesión.
#[component]
pub fn Leads() -> Element {
    let auth = use_auth();
    let mut search = use_signal(String::new);
    let mut applied = use_signal(String::new);
    let mut page = use_signal(|| 1u32);
    let per_page = use_signal(|| 20u32);
    let mut status = use_signal(String::new);
    let mut source = use_signal(String::new);
    let mut show_new = use_signal(|| false);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_l = token_of.clone();
    let mut items = use_resource(move || {
        let token = token_l.clone();
        let q = applied.read().clone();
        let p = *page.read();
        let pp = *per_page.read();
        let st = status.read().clone();
        let so = source.read().clone();
        async move {
            match token {
                Some(t) => api::list_leads(&t, p, pp, &q, &st, &so, "created", "desc").await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_s = token_of.clone();
    let mut stats = use_resource(move || {
        let token = token_s.clone();
        async move {
            match token {
                Some(t) => api::lead_stats(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Prospectos".to_string(),
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

    let creating = *show_new.read();
    let new_label = if creating { "Cerrar formulario" } else { "+ Nuevo prospecto" };

    rsx! {
        Layout { title: "Prospectos".to_string(),
            div { class: "row",
                button {
                    class: "btn btn-small btn-primary",
                    onclick: move |_| show_new.set(!creating),
                    "{new_label}"
                }
            }
            if creating {
                NewLeadForm { on_created: move || {
                    show_new.set(false);
                    items.restart();
                    stats.restart();
                } }
            }
            if let Some(Ok(s)) = &*stats.read() {
                {
                    let rate = format!("{:.1}%", s.conversion_rate);
                    rsx! {
                        section { class: "grid-cards",
                            div { class: "card",
                                h2 { class: "card-title", "Total" }
                                p { "{s.total}" }
                            }
                            div { class: "card",
                                h2 { class: "card-title", "Nuevos" }
                                p { "{s.new}" }
                            }
                            div { class: "card",
                                h2 { class: "card-title", "Calificados" }
                                p { "{s.qualified}" }
                            }
                            div { class: "card",
                                h2 { class: "card-title", "Conversión" }
                                p { "{rate}" }
                            }
                        }
                    }
                }
            }
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
                    placeholder: "Buscar por nombre, email o empresa…",
                    value: "{search}",
                    oninput: move |e| search.set(e.value()),
                }
                select {
                    class: "input",
                    value: "{status}",
                    onchange: move |e| {
                        status.set(e.value());
                        page.set(1);
                    },
                    option { value: "", "Todos los estados" }
                    for s in LEAD_STATUSES {
                        {
                            let label = api::lead_status_label(s).to_string();
                            rsx! { option { value: s, "{label}" } }
                        }
                    }
                }
                select {
                    class: "input",
                    value: "{source}",
                    onchange: move |e| {
                        source.set(e.value());
                        page.set(1);
                    },
                    option { value: "", "Todos los orígenes" }
                    for s in LEAD_SOURCES {
                        {
                            let label = api::lead_source_label(s).to_string();
                            rsx! { option { value: s, "{label}" } }
                        }
                    }
                }
                button { class: "btn btn-small btn-primary", r#type: "submit", "Buscar" }
            }

            match &*items.read() {
                None => rsx! { p { class: "muted", "Cargando prospectos…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p {
                            if e.status == 403 {
                                "No tienes permiso para ver prospectos."
                            } else {
                                "No se pudo cargar el listado."
                            }
                        }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| items.restart(), "Reintentar" }
                    }
                },
                Some(Ok(paged)) => {
                    let total_pages = paged.total_pages.max(1);
                    let current = *page.read();
                    let total = paged.total;
                    rsx! {
                        div { class: "table-wrap card",
                            table { class: "table",
                                thead {
                                    tr {
                                        th { "Nombre" }
                                        th { "Empresa" }
                                        th { "Estado" }
                                        th { "Origen" }
                                        th { "Puntaje" }
                                    }
                                }
                                tbody {
                                    for l in &paged.data {
                                        {
                                            let id = l.id.to_string();
                                            let name = l.full_name();
                                            let company = l.company_name.clone().unwrap_or_default();
                                            let st = api::lead_status_label(&l.status).to_string();
                                            let st_cls = status_class(&l.status);
                                            let src = api::lead_source_label(&l.lead_source).to_string();
                                            let score = l.score;
                                            let score_cls = score_class(score);
                                            rsx! {
                                                tr { key: "{id}",
                                                    td {
                                                        Link {
                                                            to: Route::LeadDetail { id: id.clone() },
                                                            class: "link",
                                                            "{name}"
                                                        }
                                                    }
                                                    td { "{company}" }
                                                    td {
                                                        span { class: "{st_cls}", "{st}" }
                                                    }
                                                    td { "{src}" }
                                                    td { class: "{score_cls}", "{score}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if paged.data.is_empty() {
                                p { class: "muted", "Sin prospectos con esos filtros." }
                            }
                        }
                        div { class: "pagination",
                            button {
                                class: "btn btn-small",
                                disabled: current <= 1,
                                onclick: move |_| page.set(current.saturating_sub(1).max(1)),
                                "← Anterior"
                            }
                            span { class: "muted small", "Página {current} de {total_pages} · {total} en total" }
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

/// Alta de prospecto.
#[component]
fn NewLeadForm(on_created: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut first = use_signal(String::new);
    let mut last = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut company = use_signal(String::new);
    let mut source = use_signal(|| "other".to_string());
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let busy_v = *busy.read();
    let submit_label = if busy_v { "Guardando…" } else { "Crear prospecto" };

    rsx! {
        form {
            class: "card form",
            onsubmit: move |e| {
                e.prevent_default();
                let token = auth.read().session.clone().map(|s| s.access_token);
                let f = first.read().trim().to_string();
                let l = last.read().trim().to_string();
                if f.is_empty() || l.is_empty() {
                    error.set(Some("Nombre y apellido son obligatorios.".to_string()));
                    return;
                }
                let em = none_if_empty(email.read().trim());
                let co = none_if_empty(company.read().trim());
                let src = source.read().clone();
                busy.set(true);
                error.set(None);
                spawn(async move {
                    let res = match token {
                        Some(t) => {
                            api::create_lead(
                                    &t,
                                    &api::CreateLeadBody {
                                        first_name: &f,
                                        last_name: &l,
                                        email: em.as_deref(),
                                        phone: None,
                                        company_name: co.as_deref(),
                                        title: None,
                                        industry: None,
                                        website: None,
                                        lead_source: Some(&src),
                                        assigned_to: None,
                                        notes: None,
                                    },
                                )
                                .await
                                .map(|_| ())
                                .map_err(|e| e.to_string())
                        }
                        None => Err("Sin sesión.".to_string()),
                    };
                    busy.set(false);
                    match res {
                        Ok(()) => on_created.call(()),
                        Err(msg) => error.set(Some(msg)),
                    }
                });
            },
            h2 { class: "card-title", "Nuevo prospecto" }
            div { class: "row",
                div {
                    label { class: "field", "Nombre *" }
                    input {
                        class: "input",
                        value: "{first}",
                        oninput: move |e| first.set(e.value()),
                        required: true,
                    }
                }
                div {
                    label { class: "field", "Apellido *" }
                    input {
                        class: "input",
                        value: "{last}",
                        oninput: move |e| last.set(e.value()),
                        required: true,
                    }
                }
            }
            div { class: "row",
                div {
                    label { class: "field", "Email" }
                    input {
                        class: "input",
                        r#type: "email",
                        value: "{email}",
                        oninput: move |e| email.set(e.value()),
                    }
                }
                div {
                    label { class: "field", "Empresa" }
                    input {
                        class: "input",
                        value: "{company}",
                        oninput: move |e| company.set(e.value()),
                    }
                }
            }
            label { class: "field", "Origen" }
            select {
                class: "input",
                value: "{source}",
                onchange: move |e| source.set(e.value()),
                for s in LEAD_SOURCES {
                    {
                        let label = api::lead_source_label(s).to_string();
                        rsx! { option { value: s, "{label}" } }
                    }
                }
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
            button {
                class: "btn btn-primary",
                r#type: "submit",
                disabled: busy_v,
                "{submit_label}"
            }
        }
    }
}

/// Detalle con edición, conversión y bitácora.
#[component]
pub fn LeadDetail(id: String) -> Element {
    let auth = use_auth();
    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_l = token_of.clone();
    let id_l = id.clone();
    let mut lead = use_resource(move || {
        let token = token_l.clone();
        let id_v = id_l.clone();
        async move {
            match token {
                Some(t) => api::get_lead(&t, &id_v).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_a = token_of.clone();
    let id_a = id.clone();
    let mut log = use_resource(move || {
        let token = token_a.clone();
        let id_v = id_a.clone();
        async move {
            match token {
                Some(t) => api::lead_activities(&t, &id_v).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_p = token_of.clone();
    let mut pipes = use_resource(move || {
        let token = token_p.clone();
        async move {
            match token {
                Some(t) => api::list_pipelines(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Prospecto".to_string(),
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
        Layout { title: "Prospecto".to_string(),
            Link { to: Route::Leads {}, class: "link", "← Volver a prospectos" }
            match &*lead.read() {
                None => rsx! { p { class: "muted", "Cargando prospecto…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p { "No se pudo cargar el prospecto." }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| lead.restart(), "Reintentar" }
                    }
                },
                Some(Ok(l)) => {
                    let detail_id = l.id.to_string();
                    let detail_name = l.full_name();
                    let is_converted = l.status == "converted";
                    rsx! {
                        div { class: "detail-grid",
                            div {
                                LeadEditor { lead: l.clone(), on_saved: move || lead.restart() }
                                EntityTags { entity_type: "lead".to_string(), entity_id: detail_id.clone() }
                                if !is_converted {
                                    ConvertLead {
                                        id: detail_id.clone(),
                                        name: detail_name.clone(),
                                        pipes: pipes.read().clone(),
                                        on_done: move || {
                                            lead.restart();
                                            log.restart();
                                        },
                                    }
                                } else {
                                    ConvertedLinks { lead: l.clone() }
                                }
                                DeleteLead { id: detail_id, name: detail_name }
                                LeadLog {
                                    lead_id: l.id.to_string(),
                                    log: log.read().clone(),
                                    on_changed: move || log.restart(),
                                }
                            }
                            div {
                                LeadScore { lead: l.clone() }
                                LeadQuickStatus {
                                    lead: l.clone(),
                                    on_changed: move || {
                                        lead.restart();
                                        log.restart();
                                    },
                                }
                                LeadMeta { lead: l.clone() }
                            }
                        }
                    }
                },
            }
        }
    }
}

/// Puntaje del prospecto con barra de progreso.
#[component]
fn LeadScore(lead: api::Lead) -> Element {
    let width = format!("width: {}%", lead.score.clamp(0, 100));
    rsx! {
        section { class: "card",
            h2 { class: "card-title", "Puntaje" }
            p { class: "{score_class(lead.score)} text-3xl", "{lead.score}" }
            div { class: "mt-2 h-2.5 overflow-hidden rounded-full bg-muted",
                div { class: "h-full rounded-full bg-primary", style: "{width}" }
            }
        }
    }
}

/// Cambio rápido de estado.
#[component]
fn LeadQuickStatus(lead: api::Lead, on_changed: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut busy = use_signal(|| false);

    rsx! {
        section { class: "card",
            h2 { class: "card-title", "Estado" }
            div { class: "flex flex-col gap-1",
                for s in LEAD_STATUSES {
                    {
                        if s == "converted" {
                            rsx! {}
                        } else {
                            let label = api::lead_status_label(s).to_string();
                            let active = lead.status == s;
                            let cls = if active {
                                "rounded-lg bg-primary/10 px-3 py-1.5 text-left text-sm font-semibold text-primary"
                            } else {
                                "rounded-lg px-3 py-1.5 text-left text-sm text-muted-foreground hover:bg-muted"
                            };
                            let tok = auth
                                .read()
                                .session
                                .clone()
                                .map(|s| s.access_token);
                            let id_v = lead.id.to_string();
                            let st_v = s.to_string();
                            rsx! {
                                button { key: "{s}", class: "{cls}", disabled: *busy.read(),
                                    onclick: move |_| {
                                        let tk = tok.clone();
                                        let iv = id_v.clone();
                                        let sv = st_v.clone();
                                        busy.set(true);
                                        spawn(async move {
                                            if let Some(t) = tk {
                                                let _ = api::update_lead(
                                                        &t,
                                                        &iv,
                                                        &api::UpdateLeadBody {
                                                            status: Some(&sv),
                                                            ..Default::default()
                                                        },
                                                    )
                                                    .await;
                                            }
                                            busy.set(false);
                                            on_changed.call(());
                                        });
                                    },
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Metadatos del prospecto.
#[component]
fn LeadMeta(lead: api::Lead) -> Element {
    let created = lead.created_at.format("%d %b %Y %H:%M").to_string();
    let updated = lead.updated_at.format("%d %b %Y %H:%M").to_string();
    let source = api::lead_source_label(&lead.lead_source).to_string();
    rsx! {
        section { class: "card",
            h2 { class: "card-title", "Ficha" }
            dl { class: "detail-list",
                div {
                    dt { "Origen" }
                    dd { "{source}" }
                }
                div {
                    dt { "Creado" }
                    dd { "{created}" }
                }
                div {
                    dt { "Actualizado" }
                    dd { "{updated}" }
                }
            }
        }
    }
}

/// Enlaces creados al convertir.
#[component]
fn ConvertedLinks(lead: api::Lead) -> Element {
    rsx! {
        section { class: "card converted",
            h2 { class: "card-title", "Convertido ✓" }
            div { class: "row",
                if let Some(cid) = lead.converted_contact_id {
                    {
                        let id = cid.to_string();
                        rsx! {
                            Link { to: Route::ContactDetail { id }, class: "btn btn-small", "Ver contacto" }
                        }
                    }
                }
                if let Some(cid) = lead.converted_company_id {
                    {
                        let id = cid.to_string();
                        rsx! {
                            Link { to: Route::CompanyDetail { id }, class: "btn btn-small", "Ver empresa" }
                        }
                    }
                }
                if let Some(did) = lead.converted_deal_id {
                    {
                        let id = did.to_string();
                        rsx! {
                            Link {
                                to: Route::DealDetail { id },
                                class: "btn btn-small btn-primary",
                                "Ver negocio"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Edición del prospecto (incluye estado y puntaje).
#[component]
fn LeadEditor(lead: api::Lead, on_saved: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut first = use_signal(|| lead.first_name.clone());
    let mut last = use_signal(|| lead.last_name.clone());
    let mut email = use_signal(|| lead.email.clone().unwrap_or_default());
    let mut company = use_signal(|| lead.company_name.clone().unwrap_or_default());
    let mut status = use_signal(|| lead.status.clone());
    let mut score = use_signal(|| lead.score.to_string());
    let mut notes = use_signal(|| lead.notes.clone().unwrap_or_default());
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut saved = use_signal(|| false);

    let name = lead.full_name();
    let busy_v = *busy.read();
    let submit_label = if busy_v { "Guardando…" } else { "Guardar cambios" };
    let was_saved = *saved.read();

    rsx! {
        form {
            class: "card form",
            onsubmit: move |e| {
                e.prevent_default();
                let token = auth.read().session.clone().map(|s| s.access_token);
                let id_v = lead.id.to_string();
                let f = first.read().trim().to_string();
                let l = last.read().trim().to_string();
                if f.is_empty() || l.is_empty() {
                    error.set(Some("Nombre y apellido son obligatorios.".to_string()));
                    return;
                }
                let sc: i32 = score.read().trim().parse().unwrap_or(-1);
                if sc < 0 {
                    error.set(Some("El puntaje debe ser un número mayor o igual a 0.".to_string()));
                    return;
                }
                let em = none_if_empty(email.read().trim());
                let co = none_if_empty(company.read().trim());
                let st = status.read().clone();
                let no = none_if_empty(notes.read().trim());
                busy.set(true);
                error.set(None);
                saved.set(false);
                spawn(async move {
                    let res = match token {
                        Some(t) => {
                            api::update_lead(
                                    &t,
                                    &id_v,
                                    &api::UpdateLeadBody {
                                        first_name: Some(&f),
                                        last_name: Some(&l),
                                        email: em.as_deref(),
                                        phone: None,
                                        company_name: co.as_deref(),
                                        title: None,
                                        industry: None,
                                        website: None,
                                        lead_source: None,
                                        status: Some(&st),
                                        score: Some(sc),
                                        notes: no.as_deref(),
                                    },
                                )
                                .await
                                .map(|_| ())
                                .map_err(|e| e.to_string())
                        }
                        None => Err("Sin sesión.".to_string()),
                    };
                    busy.set(false);
                    match res {
                        Ok(()) => {
                            saved.set(true);
                            on_saved.call(());
                        }
                        Err(msg) => error.set(Some(msg)),
                    }
                });
            },
            h2 { class: "card-title", "{name}" }
            div { class: "row",
                div {
                    label { class: "field", "Nombre *" }
                    input {
                        class: "input",
                        value: "{first}",
                        oninput: move |e| first.set(e.value()),
                        required: true,
                    }
                }
                div {
                    label { class: "field", "Apellido *" }
                    input {
                        class: "input",
                        value: "{last}",
                        oninput: move |e| last.set(e.value()),
                        required: true,
                    }
                }
            }
            div { class: "row",
                div {
                    label { class: "field", "Email" }
                    input {
                        class: "input",
                        r#type: "email",
                        value: "{email}",
                        oninput: move |e| email.set(e.value()),
                    }
                }
                div {
                    label { class: "field", "Empresa" }
                    input {
                        class: "input",
                        value: "{company}",
                        oninput: move |e| company.set(e.value()),
                    }
                }
            }
            div { class: "row",
                div {
                    label { class: "field", "Estado" }
                    select {
                        class: "input",
                        value: "{status}",
                        onchange: move |e| status.set(e.value()),
                        for s in LEAD_STATUSES {
                            {
                                let label = api::lead_status_label(s).to_string();
                                rsx! { option { value: s, "{label}" } }
                            }
                        }
                    }
                }
                div {
                    label { class: "field", "Puntaje" }
                    input {
                        class: "input",
                        r#type: "number",
                        min: "0",
                        value: "{score}",
                        oninput: move |e| score.set(e.value()),
                    }
                }
            }
            label { class: "field", "Notas" }
            textarea {
                class: "input",
                rows: 2,
                value: "{notes}",
                oninput: move |e| notes.set(e.value()),
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
            if was_saved {
                p { class: "alert alert-ok", "Cambios guardados." }
            }
            button {
                class: "btn btn-primary",
                r#type: "submit",
                disabled: busy_v,
                "{submit_label}"
            }
        }
    }
}

/// Conversión a contacto/empresa/negocio.
#[component]
fn ConvertLead(
    id: String,
    name: String,
    pipes: Option<Result<Vec<api::PipelineWithStages>, ApiError>>,
    on_done: EventHandler<()>,
) -> Element {
    let auth = use_auth();
    let mut pipeline = use_signal(String::new);
    let mut value = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut done = use_signal(|| None::<String>);

    let busy_v = *busy.read();
    let submit_label = if busy_v { "Convirtiendo…" } else { "Convertir" };

    rsx! {
        form {
            class: "card form",
            onsubmit: move |e| {
                e.prevent_default();
                let token = auth.read().session.clone().map(|s| s.access_token);
                let id_v = id.clone();
                let pipe = pipeline.read().trim().to_string();
                if pipe.is_empty() {
                    error.set(Some("Elige el pipeline del negocio.".to_string()));
                    return;
                }
                let val: Option<f64> = value.read().trim().parse().ok();
                busy.set(true);
                error.set(None);
                done.set(None);
                spawn(async move {
                    let res = match token {
                        Some(t) => {
                            api::convert_lead(
                                    &t,
                                    &id_v,
                                    &api::ConvertLeadBody {
                                        pipeline_id: &pipe,
                                        deal_title: None,
                                        deal_value: val,
                                    },
                                )
                                .await
                                .map_err(|e| e.to_string())
                        }
                        None => Err("Sin sesión.".to_string()),
                    };
                    busy.set(false);
                    match res {
                        Ok(r) => {
                            let msg = format!(
                                "Convertido: contacto {}, negocio {}.",
                                r.contact_id
                                    .map(|c| c.to_string())
                                    .unwrap_or_else(|| "—".to_string()),
                                r.deal_id
                                    .map(|d| d.to_string())
                                    .unwrap_or_else(|| "—".to_string()),
                            );
                            done.set(Some(msg));
                            on_done.call(());
                        }
                        Err(msg) => error.set(Some(msg)),
                    }
                });
            },
            h2 { class: "card-title", "Convertir «{name}»" }
            match pipes {
                None => rsx! { p { class: "muted", "Cargando pipelines…" } },
                Some(Err(e)) => rsx! { p { class: "muted small", "No se pudieron cargar pipelines: {e}" } },
                Some(Ok(list)) => rsx! {
                    if list.is_empty() {
                        p { class: "muted", "No hay pipelines activos." }
                    } else {
                        label { class: "field", "Pipeline del negocio" }
                        select {
                            class: "input",
                            value: "{pipeline}",
                            onchange: move |e| pipeline.set(e.value()),
                            option { value: "", "Elegir…" }
                            for p in list {
                                {
                                    let pid = p.pipeline.id.to_string();
                                    let pname = p.pipeline.name.clone();
                                    rsx! { option { value: "{pid}", "{pname}" } }
                                }
                            }
                        }
                    }
                },
            }
            label { class: "field", "Valor inicial del negocio (opcional)" }
            input {
                class: "input",
                r#type: "number",
                min: "0",
                step: "any",
                value: "{value}",
                oninput: move |e| value.set(e.value()),
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
            if let Some(msg) = done.read().clone() {
                p { class: "alert alert-ok", "{msg}" }
            }
            button {
                class: "btn btn-primary",
                r#type: "submit",
                disabled: busy_v,
                "{submit_label}"
            }
        }
    }
}

/// Bitácora del prospecto.
#[component]
fn LeadLog(
    lead_id: String,
    log: Option<Result<Vec<api::LeadActivity>, ApiError>>,
    on_changed: EventHandler<()>,
) -> Element {
    let auth = use_auth();
    let mut kind = use_signal(|| "note".to_string());
    let mut subject = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let lid_form = lead_id.clone();

    let busy_v = *busy.read();

    rsx! {
        section { class: "card",
            h2 { class: "card-title", "Bitácora" }
            form {
                class: "form",
                onsubmit: move |e| {
                    e.prevent_default();
                    let token = auth.read().session.clone().map(|s| s.access_token);
                    let id_v = lid_form.clone();
                    let k = kind.read().clone();
                    let subj = subject.read().trim().to_string();
                    if subj.is_empty() {
                        error.set(Some("El asunto es obligatorio.".to_string()));
                        return;
                    }
                    busy.set(true);
                    error.set(None);
                    spawn(async move {
                        let res = match token {
                            Some(t) => {
                                api::create_lead_activity(
                                        &t,
                                        &id_v,
                                        &api::CreateLeadActivityBody {
                                            activity_type: &k,
                                            subject: &subj,
                                            description: None,
                                            due_date: None,
                                        },
                                    )
                                    .await
                                    .map(|_| ())
                                    .map_err(|e| e.to_string())
                            }
                            None => Err("Sin sesión.".to_string()),
                        };
                        busy.set(false);
                        match res {
                            Ok(()) => {
                                subject.set(String::new());
                                on_changed.call(());
                            }
                            Err(msg) => error.set(Some(msg)),
                        }
                    });
                },
                div { class: "row",
                    select {
                        class: "input",
                        value: "{kind}",
                        onchange: move |e| kind.set(e.value()),
                        for s in crate::api::ACTIVITY_TYPES {
                            {
                                let label = api::activity_type_label(s).to_string();
                                rsx! { option { value: s, "{label}" } }
                            }
                        }
                    }
                    input {
                        class: "input",
                        placeholder: "Asunto…",
                        value: "{subject}",
                        oninput: move |e| subject.set(e.value()),
                    }
                    button {
                        class: "btn btn-small btn-primary",
                        r#type: "submit",
                        disabled: busy_v,
                        "Anotar"
                    }
                }
                if let Some(msg) = error.read().clone() {
                    p { class: "alert alert-error", "{msg}" }
                }
            }
            match log {
                None => rsx! { p { class: "muted", "Cargando…" } },
                Some(Err(e)) => rsx! { p { class: "muted small", "No se pudo cargar: {e}" } },
                Some(Ok(list)) => rsx! {
                    if list.is_empty() {
                        p { class: "muted", "Sin anotaciones." }
                    } else {
                        ul {
                            for a in list {
                                {
                                    let key = a.id.to_string();
                                    let aid_c = key.clone();
                                    let aid_d = key.clone();
                                    let subj = a.subject.clone();
                                    let done = a.completed;
                                    let mark = if done { "✓" } else { "○" };
                                    let subj_cls = if done { "line-through muted" } else { "" };
                                    let tok = auth
                                        .read()
                                        .session
                                        .clone()
                                        .map(|s| s.access_token);
                                    let tok_c = tok.clone();
                                    let lid = lead_id.clone();
                                    let lid_c = lid.clone();
                                    let lid_d = lid;
                                    rsx! {
                                        li { key: "{key}",
                                            "{mark} "
                                            span { class: "{subj_cls}", "{subj}" }
                                            " "
                                            if !done {
                                                button {
                                                    class: "link-btn small",
                                                    onclick: move |_| {
                                                        let tk = tok_c.clone();
                                                        let lv = lid_c.clone();
                                                        let av = aid_c.clone();
                                                        spawn(async move {
                                                            if let Some(t) = tk {
                                                                let _ = api::complete_lead_activity(&t, &lv, &av).await;
                                                            }
                                                            on_changed.call(());
                                                        });
                                                    },
                                                    "hecha"
                                                }
                                            }
                                            button {
                                                class: "link-btn small danger",
                                                onclick: move |_| {
                                                    let tk = tok.clone();
                                                    let lv = lid_d.clone();
                                                    let av = aid_d.clone();
                                                    spawn(async move {
                                                        if let Some(t) = tk {
                                                            let _ = api::delete_lead_activity(&t, &lv, &av).await;
                                                        }
                                                        on_changed.call(());
                                                    });
                                                },
                                                "×"
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

/// Borrado con confirmación en dos pasos.
#[component]
fn DeleteLead(id: String, name: String) -> Element {
    let auth = use_auth();
    let nav = use_navigator();
    let mut confirm = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let asking = *confirm.read();
    let busy_v = *busy.read();
    let yes_label = if busy_v { "Eliminando…" } else { "Sí, eliminar" };

    rsx! {
        section { class: "card",
            h2 { class: "card-title", "Zona de peligro" }
            if !asking {
                button {
                    class: "btn btn-small",
                    onclick: move |_| confirm.set(true),
                    "Eliminar prospecto"
                }
            } else {
                p { "¿Eliminar «{name}» definitivamente?" }
                if let Some(msg) = error.read().clone() {
                    p { class: "alert alert-error", "{msg}" }
                }
                div { class: "row",
                    button {
                        class: "btn btn-small",
                        disabled: busy_v,
                        onclick: move |_| {
                            let token = auth
                                .read()
                                .session
                                .clone()
                                .map(|s| s.access_token);
                            let id_v = id.clone();
                            busy.set(true);
                            spawn(async move {
                                let res = match token {
                                    Some(t) => {
                                        api::delete_lead(&t, &id_v)
                                            .await
                                            .map_err(|e| e.to_string())
                                    }
                                    None => Err("Sin sesión.".to_string()),
                                };
                                busy.set(false);
                                match res {
                                    Ok(()) => {
                                        nav.push(Route::Leads {});
                                    }
                                    Err(msg) => error.set(Some(msg)),
                                }
                            });
                        },
                        "{yes_label}"
                    }
                    button {
                        class: "btn btn-small",
                        onclick: move |_: MouseEvent | {
                            confirm.set(false);
                            error.set(None);
                        },
                        "Cancelar"
                    }
                }
            }
        }
    }
}
