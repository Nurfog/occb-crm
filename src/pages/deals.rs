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
            if let Some(Ok(s)) = &*summary.read() {
                section { class: "grid-cards",
                    for st in &s.stages {
                        {
                            let label = api::deal_stage_label(&st.stage).to_string();
                            let detail = format!("{} negocios · {:.0} en total", st.count, st.total_value);
                            rsx! {
                                div { class: "card", key: "{st.stage}",
                                    h2 { class: "card-title", "{label}" }
                                    p { class: "muted", "{detail}" }
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
                Some(Ok(paged)) => rsx! {
                    div { class: "board",
                        for stage in DEAL_STAGES {
                            {
                                let label = api::deal_stage_label(stage).to_string();
                                let cards: Vec<api::Deal> = paged
                                    .data
                                    .iter()
                                    .filter(|d| d.stage == stage)
                                    .cloned()
                                    .collect();
                                let count = cards.len();
                                let head = format!("{label} ({count})");
                                rsx! {
                                    section { class: "card board-col", key: "{stage}",
                                        h2 { class: "card-title", "{head}" }
                                        if cards.is_empty() {
                                            p { class: "muted small", "Vacía." }
                                        }
                                        for d in cards {
                                            {
                                                let did = d.id.to_string();
                                                rsx! {
                                                    DealCard {
                                                        deal: d.clone(),
                                                        on_moved: move || board.restart(),
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
                },
            }
        }
    }
}

/// Tarjeta de negocio con selector para mover de etapa.
#[component]
fn DealCard(deal: api::Deal, on_moved: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut stage = use_signal(|| deal.stage.clone());
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let stage_v = stage.read().clone();
    let busy_v = *busy.read();
    let title = deal.title.clone();
    let amount = format!("{:.0} {}", deal.value, deal.currency);
    let company = deal.company_name.clone().unwrap_or_default();
    let contact = deal.contact_name.clone().unwrap_or_default();
    let extra = match (company.is_empty(), contact.is_empty()) {
        (true, true) => String::new(),
        (false, true) => format!(" · {company}"),
        (true, false) => format!(" · {contact}"),
        (false, false) => format!(" · {company} · {contact}"),
    };

    rsx! {
        div { class: "deal-card",
            strong { "{title}" }
            p { class: "muted small", "{amount}{extra}" }
            select {
                class: "input",
                value: "{stage_v}",
                disabled: busy_v,
                onchange: move |e| {
                    let next = e.value();
                    stage.set(next.clone());
                    let token = auth
                        .read()
                        .session
                        .clone()
                        .map(|s| s.access_token);
                    let id_v = deal.id.to_string();
                    busy.set(true);
                    error.set(None);
                    spawn(async move {
                        let res = match token {
                            Some(t) => {
                                api::move_deal_stage(&t, &id_v, &next)
                                    .await
                                    .map(|_| ())
                                    .map_err(|e| e.to_string())
                            }
                            None => Err("Sin sesión.".to_string()),
                        };
                        busy.set(false);
                        match res {
                            Ok(()) => on_moved.call(()),
                            Err(msg) => error.set(Some(msg)),
                        }
                    });
                },
                for s in DEAL_STAGES {
                    {
                        let label = api::deal_stage_label(s).to_string();
                        rsx! { option { value: s, "{label}" } }
                    }
                }
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error small", "{msg}" }
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
