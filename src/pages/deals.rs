//! Negocios: tablero por etapas (`PATCH /deals/{id}/stage`), alta, detalle.

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError, DEAL_STAGES},
    auth::use_auth,
    components::{tags::EntityTags, Layout},
    route::Route,
};

fn none_if_empty(s: &str) -> Option<String> {
    if s.is_empty() { None } else { Some(s.to_string()) }
}

/// Tablero kanban por etapa + resumen + alta. Requiere sesión.
#[component]
pub fn Deals() -> Element {
    let auth = use_auth();
    let mut show_new = use_signal(|| false);
    let mut op_msg = use_signal(|| None::<String>);
    let mut view = use_signal(|| "board".to_string());
    let mut stage_filter = use_signal(|| String::new());
    let mut drag_id = use_signal(|| None::<String>);
    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_ex = token_of.clone();
    let token_im = token_of.clone();
    let token_b = token_of.clone();
    let mut board = use_resource(move || {
        let token = token_b.clone();
        async move {
            match token {
                Some(t) => api::list_deals(&t, 1, 200, "", "created_at", "desc").await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_s = token_of.clone();
    let mut summary = use_resource(move || {
        let token = token_s.clone();
        async move {
            match token {
                Some(t) => api::dashboard_pipeline(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Negocios".to_string(),
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
    let new_label = if creating { "Cerrar formulario" } else { "+ Nuevo negocio" };

    rsx! {
        Layout { title: "Negocios".to_string(),
            div { class: "row",
                button {
                    class: "btn btn-small btn-primary",
                    onclick: move |_| show_new.set(!creating),
                    "{new_label}"
                }
                button {
                    class: "btn btn-small",
                    onclick: move |_| {
                        let tok = token_ex.clone();
                        spawn(async move {
                            if let Some(t) = tok {
                                match api::export_deals(&t).await {
                                    Ok(csv) => {
                                        let _ = crate::privacy::download_text_file(
                                            "negocios.csv",
                                            &csv,
                                        );
                                    }
                                    Err(e) => op_msg.set(Some(e.to_string())),
                                }
                            }
                        });
                    },
                    "Exportar CSV"
                }
                button {
                    class: "btn btn-small",
                    onclick: move |_| {
                        board.restart();
                        summary.restart();
                    },
                    "Recargar"
                }
            }
            div { class: "row",
                input {
                    r#type: "file",
                    id: "deals-import",
                    accept: ".csv,text/csv",
                }
                button {
                    class: "btn btn-small",
                    onclick: move |_| {
                        let tok = token_im.clone();
                        op_msg.set(None);
                        spawn(async move {
                            match crate::files::read_first_file("deals-import").await {
                                Ok((_, _, bytes)) => match String::from_utf8(bytes) {
                                    Ok(csv) => {
                                        if let Some(t) = tok {
                                            match api::import_deals(&t, &csv).await {
                                                Ok(r) => {
                                                    op_msg.set(
                                                        Some(
                                                            format!(
                                                                "Importados: {}. Filas con error: {}.",
                                                                r.imported,
                                                                r.errors.len()
                                                            ),
                                                        ),
                                                    );
                                                    board.restart();
                                                    summary.restart();
                                                }
                                                Err(e) => op_msg.set(Some(e.to_string())),
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        op_msg.set(
                                            Some("El archivo no es UTF-8 válido.".to_string()),
                                        );
                                    }
                                },
                                Err(e) => op_msg.set(Some(e)),
                            }
                        });
                    },
                    "Importar CSV"
                }
            }
            if let Some(msg) = op_msg.read().clone() {
                p { class: "alert alert-info", "{msg}" }
            }
            if creating {
                NewDealForm { on_created: move || {
                    show_new.set(false);
                    board.restart();
                    summary.restart();
                } }
            }
            div { class: "row",
                button {
                    class: if *view.read() == "board" { "btn btn-small btn-primary" } else { "btn btn-small" },
                    onclick: move |_| view.set("board".to_string()),
                    "Tablero"
                }
                button {
                    class: if *view.read() == "list" { "btn btn-small btn-primary" } else { "btn btn-small" },
                    onclick: move |_| view.set("list".to_string()),
                    "Lista"
                }
            }
            if let Some(Ok(s)) = &*summary.read() {
                div { class: "row",
                    button {
                        class: if stage_filter.read().is_empty() { "btn btn-small btn-primary" } else { "btn btn-small" },
                        onclick: move |_| stage_filter.set(String::new()),
                        "Todas"
                    }
                    for st in &s.stages {
                        {
                            let code = st.stage.clone();
                            let label = api::deal_stage_label(&st.stage).to_string();
                            let txt = format!("{label} ({})", st.count);
                            let is_on = *stage_filter.read() == code;
                            let cls = if is_on { "btn btn-small btn-primary" } else { "btn btn-small" };
                            rsx! {
                                button { key: "{code}", class: "{cls}",
                                    onclick: move |_| stage_filter.set(code.clone()),
                                    "{txt}"
                                }
                            }
                        }
                    }
                }
            }
            match &*board.read() {
                None => rsx! { p { class: "muted", "Cargando tablero…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p {
                            if e.status == 403 {
                                "No tienes permiso para ver negocios."
                            } else {
                                "No se pudo cargar el tablero."
                            }
                        }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| board.restart(), "Reintentar" }
                    }
                },
                Some(Ok(paged)) => {
                    let filter = stage_filter.read().clone();
                    let is_list = *view.read() == "list";
                    let shown: Vec<api::Deal> = paged
                        .data
                        .iter()
                        .filter(|d| filter.is_empty() || d.stage == filter)
                        .cloned()
                        .collect();
                    let n_shown = shown.len();
                    rsx! {
                        if is_list {
                            DealsTable { deals: shown, on_changed: move || {
                                board.restart();
                                summary.restart();
                            } }
                        } else {
                            div { class: "board",
                                for stage in DEAL_STAGES {
                                    {
                                        if !filter.is_empty() && filter != stage {
                                            rsx! {}
                                        } else {
                                            let label = api::deal_stage_label(stage).to_string();
                                            let cards: Vec<api::Deal> = paged
                                                .data
                                                .iter()
                                                .filter(|d| d.stage == stage)
                                                .cloned()
                                                .collect();
                                            let count = cards.len();
                                            let head = format!("{label} ({count})");
                                            let stage_v = stage.to_string();
                                            let tok = token_of.clone();
                                            rsx! {
                                                section {
                                                    class: "card board-col",
                                                    key: "{stage}",
                                                    ondragover: move |e| e.prevent_default(),
                                                    ondrop: move |_| {
                                                        let tk = tok.clone();
                                                        let st = stage_v.clone();
                                                        let did = drag_id.read().clone();
                                                        drag_id.set(None);
                                                        if let Some(did) = did {
                                                            spawn(async move {
                                                                if let Some(t) = tk {
                                                                    let _ = api::move_deal_stage(&t, &did, &st).await;
                                                                }
                                                                board.restart();
                                                                summary.restart();
                                                            });
                                                        }
                                                    },
                                                    h2 { class: "card-title", "{head}" }
                                                    if cards.is_empty() {
                                                        p { class: "muted small", "Vacía. Arrastra aquí." }
                                                    }
                                                    for d in cards {
                                                        {
                                                            let did = d.id.to_string();
                                                            rsx! {
                                                                DealCard {
                                                                    deal: d.clone(),
                                                                    drag: move |id: String| drag_id.set(Some(id)),
                                                                    on_moved: move || {
                                                                        board.restart();
                                                                        summary.restart();
                                                                    },
                                                                }
                                                                Link {
                                                                    to: Route::DealDetail { id: did },
                                                                    class: "link small",
                                                                    "Ver detalle →"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if n_shown == 0 {
                                p { class: "muted", "Sin negocios con ese filtro." }
                            }
                        }
                    }
                },
            }
        }
    }
}

/// Tarjeta arrastrable: título, valor, empresa y acciones (ver/editar/borrar).
#[component]
fn DealCard(deal: api::Deal, drag: EventHandler<String>, on_moved: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut confirm = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let deal_id = deal.id.to_string();
    let title = deal.title.clone();
    let amount = format!("${:.0} {}", deal.value, deal.currency);
    let company = deal.company_name.clone().unwrap_or_default();
    let busy_v = *busy.read();
    let asking = *confirm.read();

    rsx! {
        div {
            class: "deal-card",
            draggable: "true",
            ondragstart: move |_| drag.call(deal_id.clone()),
            strong { "{title}" }
            p { class: "deal-amount", "{amount}" }
            if !company.is_empty() {
                p { class: "muted small truncate", "{company}" }
            }
            div { class: "deal-actions",
                Link {
                    to: Route::DealDetail { id: deal.id.to_string() },
                    class: "link-btn small",
                    "Ver"
                }
                Link {
                    to: Route::DealDetail { id: deal.id.to_string() },
                    class: "link-btn small",
                    "Editar"
                }
                if !asking {
                    button {
                        class: "link-btn small danger",
                        onclick: move |_| confirm.set(true),
                        "Borrar"
                    }
                } else {
                    button {
                        class: "link-btn small danger",
                        disabled: busy_v,
                        onclick: move |_| {
                            let tok = auth
                                .read()
                                .session
                                .clone()
                                .map(|s| s.access_token);
                            let id_v = deal.id.to_string();
                            busy.set(true);
                            spawn(async move {
                                let res = match tok {
                                    Some(t) => {
                                        api::delete_deal(&t, &id_v)
                                            .await
                                            .map_err(|e| e.to_string())
                                    }
                                    None => Err("Sin sesión.".to_string()),
                                };
                                busy.set(false);
                                match res {
                                    Ok(()) => on_moved.call(()),
                                    Err(msg) => {
                                        error.set(Some(msg));
                                        confirm.set(false);
                                    }
                                }
                            });
                        },
                        "¿Sí?"
                    }
                }
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error small", "{msg}" }
            }
        }
    }
}

/// Vista lista: tabla con monto, etapa, cierre y acciones.
#[component]
fn DealsTable(deals: Vec<api::Deal>, on_changed: EventHandler<()>) -> Element {
    let auth = use_auth();

    rsx! {
        if deals.is_empty() {
            p { class: "muted", "Sin negocios con ese filtro." }
        } else {
            div { class: "table-wrap card",
                table { class: "table",
                    thead {
                        tr {
                            th { "Nombre" }
                            th { "Monto" }
                            th { "Etapa" }
                            th { "Cierre" }
                            th { "Acciones" }
                        }
                    }
                    tbody {
                        for d in deals {
                            {
                                let key = d.id.to_string();
                                let id_v = key.clone();
                                let title = d.title.clone();
                                let amount = format!("${:.0} {}", d.value, d.currency);
                                let stage = api::deal_stage_label(&d.stage).to_string();
                                let close = d
                                    .expected_close_date
                                    .map(|c| c.format("%Y-%m-%d").to_string())
                                    .unwrap_or_else(|| "—".to_string());
                                let tok = auth
                                    .read()
                                    .session
                                    .clone()
                                    .map(|s| s.access_token);
                                rsx! {
                                    tr { key: "{key}",
                                        td {
                                            Link {
                                                to: Route::DealDetail { id: id_v.clone() },
                                                class: "link",
                                                "{title}"
                                            }
                                        }
                                        td { "{amount}" }
                                        td { "{stage}" }
                                        td { class: "muted", "{close}" }
                                        td {
                                            button {
                                                class: "link-btn small danger",
                                                onclick: move |_| {
                                                    let tk = tok.clone();
                                                    let iv = id_v.clone();
                                                    spawn(async move {
                                                        if let Some(t) = tk {
                                                            let _ = api::delete_deal(&t, &iv).await;
                                                        }
                                                        on_changed.call(());
                                                    });
                                                },
                                                "Borrar"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Alta de negocio.
#[component]
fn NewDealForm(on_created: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut title = use_signal(String::new);
    let mut value = use_signal(String::new);
    let mut stage = use_signal(|| "lead".to_string());
    let mut close = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let busy_v = *busy.read();
    let submit_label = if busy_v { "Guardando…" } else { "Crear negocio" };

    rsx! {
        form {
            class: "card form",
            onsubmit: move |e| {
                e.prevent_default();
                let token = auth.read().session.clone().map(|s| s.access_token);
                let t = title.read().trim().to_string();
                if t.is_empty() {
                    error.set(Some("El título es obligatorio.".to_string()));
                    return;
                }
                let v: f64 = value.read().trim().parse().unwrap_or(-1.0);
                if v < 0.0 {
                    error.set(Some("El valor debe ser un número mayor o igual a 0.".to_string()));
                    return;
                }
                let st = stage.read().clone();
                let cl = none_if_empty(close.read().trim());
                let no = none_if_empty(notes.read().trim());
                busy.set(true);
                error.set(None);
                spawn(async move {
                    let res = match token {
                        Some(tok) => {
                            api::create_deal(
                                    &tok,
                                    &api::CreateDealBody {
                                        title: &t,
                                        value: v,
                                        currency: None,
                                        stage: Some(&st),
                                        contact_id: None,
                                        company_id: None,
                                        expected_close_date: cl.as_deref(),
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
                        Ok(()) => on_created.call(()),
                        Err(msg) => error.set(Some(msg)),
                    }
                });
            },
            h2 { class: "card-title", "Nuevo negocio" }
            label { class: "field", "Título *" }
            input {
                class: "input",
                value: "{title}",
                oninput: move |e| title.set(e.value()),
                required: true,
            }
            div { class: "row",
                div {
                    label { class: "field", "Valor (USD) *" }
                    input {
                        class: "input",
                        r#type: "number",
                        min: "0",
                        step: "any",
                        value: "{value}",
                        oninput: move |e| value.set(e.value()),
                        required: true,
                    }
                }
                div {
                    label { class: "field", "Etapa" }
                    select {
                        class: "input",
                        value: "{stage}",
                        onchange: move |e| stage.set(e.value()),
                        for s in DEAL_STAGES {
                            {
                                let label = api::deal_stage_label(s).to_string();
                                rsx! { option { value: s, "{label}" } }
                            }
                        }
                    }
                }
            }
            div { class: "row",
                div {
                    label { class: "field", "Cierre esperado (AAAA-MM-DD)" }
                    input {
                        class: "input",
                        placeholder: "2026-12-31",
                        value: "{close}",
                        oninput: move |e| close.set(e.value()),
                    }
                }
                div {
                    label { class: "field", "Notas" }
                    input {
                        class: "input",
                        value: "{notes}",
                        oninput: move |e| notes.set(e.value()),
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

/// Detalle con edición, cambio de etapa y borrado.
#[component]
pub fn DealDetail(id: String) -> Element {
    let auth = use_auth();
    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_d = token_of.clone();
    let id_d = id.clone();
    let mut deal = use_resource(move || {
        let token = token_d.clone();
        let id_v = id_d.clone();
        async move {
            match token {
                Some(t) => api::get_deal(&t, &id_v).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_a = token_of.clone();
    let id_a = id.clone();
    let mut activities = use_resource(move || {
        let token = token_a.clone();
        let id_v = id_a.clone();
        async move {
            match token {
                Some(t) => api::deal_activities(&t, &id_v).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_h = token_of.clone();
    let id_h = id.clone();
    let mut history = use_resource(move || {
        let token = token_h.clone();
        let id_v = id_h.clone();
        async move {
            match token {
                Some(t) => api::entity_history(&t, "deal", &id_v).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Negocio".to_string(),
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
        Layout { title: "Negocio".to_string(),
            Link { to: Route::Deals {}, class: "link", "← Volver al tablero" }
            match &*deal.read() {
                None => rsx! { p { class: "muted", "Cargando negocio…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p { "No se pudo cargar el negocio." }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| deal.restart(), "Reintentar" }
                    }
                },
                Some(Ok(d)) => {
                    let detail_id = d.id.to_string();
                    let detail_title = d.title.clone();
                    rsx! {
                        DealEditor { deal: d.clone(), on_saved: move || deal.restart() }
                        EntityTags { entity_type: "deal".to_string(), entity_id: detail_id.clone() }
                        DeleteDeal { id: detail_id, title: detail_title }
                        DealHistory { log: history.read().clone() }
                        section { class: "card",
                            h2 { class: "card-title", "Actividad" }
                            match &*activities.read() {
                                None => rsx! { p { class: "muted", "Cargando…" } },
                                Some(Err(e)) => rsx! { p { class: "muted small", "No se pudo cargar: {e}" } },
                                Some(Ok(list)) => rsx! {
                                    if list.is_empty() {
                                        p { class: "muted", "Sin actividad registrada." }
                                    } else {
                                        ul {
                                            for a in list.iter().take(10) {
                                                {
                                                    let key = a.id.to_string();
                                                    let subject = a.subject.clone();
                                                    let kind = a.activity_type.clone();
                                                    let when = a.created_at.format("%Y-%m-%d").to_string();
                                                    rsx! {
                                                        li { key: "{key}",
                                                            strong { "{subject}" }
                                                            span { class: "muted small",
                                                                " · {kind} · {when}"
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
                },
            }
        }
    }
}

/// Historial de cambios del negocio (auditoría por entidad).
#[component]
fn DealHistory(log: Option<Result<Vec<api::AuditLog>, ApiError>>) -> Element {
    rsx! {
        section { class: "card",
            h2 { class: "card-title", "Historial" }
            match log {
                None => rsx! { p { class: "muted", "Cargando…" } },
                Some(Err(e)) => rsx! { p { class: "muted small", "Sin acceso al historial: {e}" } },
                Some(Ok(list)) => rsx! {
                    if list.is_empty() {
                        p { class: "muted", "Sin movimientos registrados." }
                    } else {
                        ul { class: "timeline",
                            for l in list.iter().take(20) {
                                {
                                    let key = l.id.to_string();
                                    let action = l.action.clone();
                                    let when = l.created_at.format("%d %b %H:%M").to_string();
                                    rsx! {
                                        li { key: "{key}", class: "timeline-item",
                                            span { class: "timeline-dot" }
                                            div {
                                                strong { "{action}" }
                                                p { class: "muted small", "{when}" }
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

/// Edición del negocio.
#[component]
fn DealEditor(deal: api::Deal, on_saved: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut title = use_signal(|| deal.title.clone());
    let mut value = use_signal(|| deal.value.to_string());
    let mut stage = use_signal(|| deal.stage.clone());
    let mut notes = use_signal(|| deal.notes.clone().unwrap_or_default());
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut saved = use_signal(|| false);

    let name = deal.title.clone();
    let stage_label = api::deal_stage_label(&deal.stage).to_string();
    let amount = format!("{:.0} {}", deal.value, deal.currency);
    let company = deal.company_name.clone().unwrap_or_default();
    let header_extra = if company.is_empty() {
        format!("{stage_label} · {amount}")
    } else {
        format!("{stage_label} · {amount} · {company}")
    };
    let busy_v = *busy.read();
    let submit_label = if busy_v { "Guardando…" } else { "Guardar cambios" };
    let was_saved = *saved.read();

    rsx! {
        form {
            class: "card form",
            onsubmit: move |e| {
                e.prevent_default();
                let token = auth.read().session.clone().map(|s| s.access_token);
                let id_v = deal.id.to_string();
                let t = title.read().trim().to_string();
                if t.is_empty() {
                    error.set(Some("El título es obligatorio.".to_string()));
                    return;
                }
                let v: f64 = value.read().trim().parse().unwrap_or(-1.0);
                if v < 0.0 {
                    error.set(Some("El valor debe ser mayor o igual a 0.".to_string()));
                    return;
                }
                let st = stage.read().clone();
                let no = none_if_empty(notes.read().trim());
                busy.set(true);
                error.set(None);
                saved.set(false);
                spawn(async move {
                    let res = match token {
                        Some(tok) => {
                            api::update_deal(
                                    &tok,
                                    &id_v,
                                    &api::UpdateDealBody {
                                        title: Some(&t),
                                        value: Some(v),
                                        currency: None,
                                        stage: Some(&st),
                                        contact_id: None,
                                        company_id: None,
                                        expected_close_date: None,
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
            p { class: "muted small", "{header_extra}" }
            label { class: "field", "Título *" }
            input {
                class: "input",
                value: "{title}",
                oninput: move |e| title.set(e.value()),
                required: true,
            }
            div { class: "row",
                div {
                    label { class: "field", "Valor *" }
                    input {
                        class: "input",
                        r#type: "number",
                        min: "0",
                        step: "any",
                        value: "{value}",
                        oninput: move |e| value.set(e.value()),
                        required: true,
                    }
                }
                div {
                    label { class: "field", "Etapa" }
                    select {
                        class: "input",
                        value: "{stage}",
                        onchange: move |e| stage.set(e.value()),
                        for s in DEAL_STAGES {
                            {
                                let label = api::deal_stage_label(s).to_string();
                                rsx! { option { value: s, "{label}" } }
                            }
                        }
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

/// Borrado con confirmación en dos pasos.
#[component]
fn DeleteDeal(id: String, title: String) -> Element {
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
                    "Eliminar negocio"
                }
            } else {
                p { "¿Eliminar «{title}» definitivamente?" }
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
                                        api::delete_deal(&t, &id_v)
                                            .await
                                            .map_err(|e| e.to_string())
                                    }
                                    None => Err("Sin sesión.".to_string()),
                                };
                                busy.set(false);
                                match res {
                                    Ok(()) => {
                                        nav.push(Route::Deals {});
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
