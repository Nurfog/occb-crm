//! Administración (`admin.access`): pipelines, perfiles, marca y webhooks.

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError, WEBHOOK_EVENTS},
    auth::use_auth,
    components::Layout,
    route::Route,
};

const TABS: [(&str, &str); 4] = [
    ("pipelines", "Pipelines"),
    ("profiles", "Perfiles"),
    ("branding", "Marca"),
    ("webhooks", "Webhooks"),
];

/// Panel de administración por pestañas. Requiere sesión (y `admin.access`).
#[component]
pub fn Admin() -> Element {
    let auth = use_auth();
    let mut tab = use_signal(|| "pipelines".to_string());

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Administración".to_string(),
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

    let current = tab.read().clone();

    rsx! {
        Layout { title: "Administración".to_string(),
            p { class: "muted small",
                "Solo administradores: el backend responde 403 sin el permiso."
            }
            div { class: "row",
                for (id, label) in TABS {
                    {
                        let active = current == id;
                        let cls = if active { "btn btn-small btn-primary" } else { "btn btn-small" };
                        let id_v = id.to_string();
                        rsx! {
                            button {
                                key: "{id}",
                                class: "{cls}",
                                onclick: move |_| tab.set(id_v.clone()),
                                "{label}"
                            }
                        }
                    }
                }
            }
            if current == "pipelines" {
                PipelinesSection {}
            } else if current == "profiles" {
                ProfilesSection {}
            } else if current == "branding" {
                BrandingSection {}
            } else {
                WebhooksSection {}
            }
        }
    }
}

/// Pipelines y sus etapas.
#[component]
fn PipelinesSection() -> Element {
    let auth = use_auth();
    let mut name = use_signal(String::new);
    let mut slug = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_l = token_of.clone();
    let mut items = use_resource(move || {
        let token = token_l.clone();
        async move {
            match token {
                Some(t) => api::list_pipelines(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    rsx! {
        section { class: "card",
            h2 { class: "card-title", "Nuevo pipeline" }
            div { class: "row",
                input {
                    class: "input",
                    placeholder: "Nombre *",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                }
                input {
                    class: "input",
                    placeholder: "slug (minúsculas, guiones) *",
                    value: "{slug}",
                    oninput: move |e| slug.set(e.value()),
                }
                button {
                    class: "btn btn-small btn-primary",
                    onclick: move |_| {
                        let tok = token_of.clone();
                        let n = name.read().trim().to_string();
                        let s = slug.read().trim().to_lowercase();
                        if n.is_empty() || s.is_empty() {
                            error.set(Some("Nombre y slug son obligatorios.".to_string()));
                            return;
                        }
                        error.set(None);
                        spawn(async move {
                            if let Some(t) = tok {
                                match api::create_pipeline(
                                        &t,
                                        &api::PipelineInput {
                                            name: &n,
                                            slug: &s,
                                            description: None,
                                            entity_type: None,
                                        },
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        name.set(String::new());
                                        slug.set(String::new());
                                        items.restart();
                                    }
                                    Err(e) => error.set(Some(e.to_string())),
                                }
                            }
                        });
                    },
                    "Crear"
                }
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
        }
        match &*items.read() {
            None => rsx! { p { class: "muted", "Cargando pipelines…" } },
            Some(Err(e)) => rsx! {
                div { class: "alert alert-error",
                    p { "No se pudieron cargar (¿falta admin.access?)." }
                    p { class: "muted small", "{e}" }
                    button { class: "btn btn-small", onclick: move |_| items.restart(), "Reintentar" }
                }
            },
            Some(Ok(list)) => rsx! {
                for p in list {
                    {
                        let pid = p.pipeline.id.to_string();
                        rsx! {
                            PipelineCard {
                                key: "{pid}",
                                pipe: p.clone(),
                                on_changed: move || items.restart(),
                            }
                        }
                    }
                }
            },
        }
    }
}

/// Un pipeline con alta y borrado de etapas.
#[component]
fn PipelineCard(pipe: api::PipelineWithStages, on_changed: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut stage_name = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let title = pipe.pipeline.name.clone();
    let pid = pipe.pipeline.id.to_string();

    rsx! {
        section { class: "card",
            h2 { class: "card-title", "{title}" }
            ul {
                for s in &pipe.stages {
                    {
                        let sid = s.id.to_string();
                        let sname = s.name.clone();
                        let prob = s.probability.map(|p| format!(" · {p}%")).unwrap_or_default();
                        let tok = token_of.clone();
                        let pid_v = pid.clone();
                        rsx! {
                            li { key: "{sid}",
                                "{sname} (orden {s.position}{prob}) "
                                button {
                                    class: "link-btn small",
                                    onclick: move |_| {
                                        let tk = tok.clone();
                                        let pv = pid_v.clone();
                                        let sv = sid.clone();
                                        spawn(async move {
                                            if let Some(t) = tk {
                                                let _ = api::delete_stage(&t, &pv, &sv).await;
                                            }
                                            on_changed.call(());
                                        });
                                    },
                                    "quitar"
                                }
                            }
                        }
                    }
                }
            }
            div { class: "row",
                input {
                    class: "input",
                    placeholder: "Nueva etapa…",
                    value: "{stage_name}",
                    oninput: move |e| stage_name.set(e.value()),
                }
                button {
                    class: "btn btn-small",
                    onclick: move |_| {
                        let tok = token_of.clone();
                        let pid_v = pid.clone();
                        let n = stage_name.read().trim().to_string();
                        if n.is_empty() {
                            return;
                        }
                        error.set(None);
                        spawn(async move {
                            if let Some(t) = tok {
                                match api::create_stage(
                                        &t,
                                        &pid_v,
                                        &api::StageInput {
                                            name: &n,
                                            position: 0,
                                            color: None,
                                            probability: None,
                                        },
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        stage_name.set(String::new());
                                        on_changed.call(());
                                    }
                                    Err(e) => error.set(Some(e.to_string())),
                                }
                            }
                        });
                    },
                    "Agregar etapa"
                }
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
        }
    }
}

/// Perfiles y permisos.
#[component]
fn ProfilesSection() -> Element {
    let auth = use_auth();
    let mut name = use_signal(String::new);
    let mut perms = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_l = token_of.clone();
    let mut items = use_resource(move || {
        let token = token_l.clone();
        async move {
            match token {
                Some(t) => api::list_profiles(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    rsx! {
        section { class: "card",
            h2 { class: "card-title", "Nuevo perfil" }
            div { class: "row",
                input {
                    class: "input",
                    placeholder: "Nombre *",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                }
                input {
                    class: "input",
                    placeholder: "Permisos separados por coma",
                    value: "{perms}",
                    oninput: move |e| perms.set(e.value()),
                }
                button {
                    class: "btn btn-small btn-primary",
                    onclick: move |_| {
                        let tok = token_of.clone();
                        let n = name.read().trim().to_string();
                        let list: Vec<String> = perms
                            .read()
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        if n.is_empty() {
                            error.set(Some("El nombre es obligatorio.".to_string()));
                            return;
                        }
                        error.set(None);
                        spawn(async move {
                            if let Some(t) = tok {
                                let perms_opt = if list.is_empty() { None } else { Some(list) };
                                match api::create_profile(
                                        &t,
                                        &api::ProfileInput {
                                            name: &n,
                                            description: None,
                                            permissions: perms_opt.as_deref(),
                                        },
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        name.set(String::new());
                                        perms.set(String::new());
                                        items.restart();
                                    }
                                    Err(e) => error.set(Some(e.to_string())),
                                }
                            }
                        });
                    },
                    "Crear"
                }
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
        }
        match &*items.read() {
            None => rsx! { p { class: "muted", "Cargando perfiles…" } },
            Some(Err(e)) => rsx! {
                div { class: "alert alert-error",
                    p { "No se pudieron cargar." }
                    p { class: "muted small", "{e}" }
                    button { class: "btn btn-small", onclick: move |_| items.restart(), "Reintentar" }
                }
            },
            Some(Ok(list)) => rsx! {
                div { class: "table-wrap card",
                    table { class: "table",
                        thead {
                            tr {
                                th { "Nombre" }
                                th { "Sistema" }
                                th { "Permisos" }
                            }
                        }
                        tbody {
                            for p in list {
                                {
                                    let key = p.id.to_string();
                                    let pname = p.name.clone();
                                    let sys = if p.is_system { "sí" } else { "no" };
                                    let plist = if p.permissions.is_empty() {
                                        "—".to_string()
                                    } else {
                                        p.permissions.join(", ")
                                    };
                                    rsx! {
                                        tr { key: "{key}",
                                            td { "{pname}" }
                                            td { "{sys}" }
                                            td { class: "muted small", "{plist}" }
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

/// Marca pública.
#[component]
fn BrandingSection() -> Element {
    let auth = use_auth();
    let mut company = use_signal(String::new);
    let mut primary = use_signal(String::new);
    let mut loaded = use_signal(|| false);
    let mut saved = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let mut brand = use_resource(move || async move {
        match api::get_branding().await {
            Ok(b) => Some(b),
            Err(_) => None,
        }
    });

    if let Some(Some(b)) = &*brand.read() {
        if !*loaded.read() {
            loaded.set(true);
            if let Some(c) = b.company_name.clone() {
                company.set(c);
            }
            if let Some(c) = b.primary_color.clone() {
                primary.set(c);
            }
        }
    }
    let was_saved = *saved.read();

    rsx! {
        section { class: "card form",
            h2 { class: "card-title", "Marca" }
            label { class: "field", "Nombre de la empresa" }
            input {
                class: "input",
                value: "{company}",
                oninput: move |e| company.set(e.value()),
            }
            label { class: "field", "Color primario (hex)" }
            input {
                class: "input",
                placeholder: "#2563eb",
                value: "{primary}",
                oninput: move |e| primary.set(e.value()),
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
            if was_saved {
                p { class: "alert alert-ok", "Marca actualizada." }
            }
            button {
                class: "btn btn-primary",
                onclick: move |_| {
                    let tok = token_of.clone();
                    let co = company.read().trim().to_string();
                    let pr = primary.read().trim().to_string();
                    error.set(None);
                    saved.set(false);
                    spawn(async move {
                        if let Some(t) = tok {
                            let co_opt = if co.is_empty() { None } else { Some(co.as_str()) };
                            let pr_opt = if pr.is_empty() { None } else { Some(pr.as_str()) };
                            match api::update_branding(
                                    &t,
                                    &api::BrandingInput {
                                        company_name: co_opt,
                                        primary_color: pr_opt,
                                        ..Default::default()
                                    },
                                )
                                .await
                            {
                                Ok(_) => {
                                    saved.set(true);
                                    brand.restart();
                                }
                                Err(e) => error.set(Some(e.to_string())),
                            }
                        }
                    });
                },
                "Guardar"
            }
        }
    }
}

/// Webhooks de salida.
#[component]
fn WebhooksSection() -> Element {
    let auth = use_auth();
    let mut url = use_signal(String::new);
    let mut event = use_signal(|| "deal_created".to_string());
    let mut error = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_l = token_of.clone();
    let mut items = use_resource(move || {
        let token = token_l.clone();
        async move {
            match token {
                Some(t) => api::list_webhooks(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    rsx! {
        section { class: "card",
            h2 { class: "card-title", "Nuevo webhook" }
            div { class: "row",
                input {
                    class: "input",
                    placeholder: "https://… *",
                    value: "{url}",
                    oninput: move |e| url.set(e.value()),
                }
                select {
                    class: "input",
                    value: "{event}",
                    onchange: move |e| event.set(e.value()),
                    for e in WEBHOOK_EVENTS {
                        option { value: "{e}", "{e}" }
                    }
                }
                button {
                    class: "btn btn-small btn-primary",
                    onclick: move |_| {
                        let tok = token_of.clone();
                        let u = url.read().trim().to_string();
                        let ev = event.read().clone();
                        if u.is_empty() {
                            error.set(Some("La URL es obligatoria.".to_string()));
                            return;
                        }
                        error.set(None);
                        spawn(async move {
                            if let Some(t) = tok {
                                match api::create_webhook(
                                        &t,
                                        &api::CreateWebhookBody { url: &u, event: &ev, secret: None },
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        url.set(String::new());
                                        items.restart();
                                    }
                                    Err(e) => error.set(Some(e.to_string())),
                                }
                            }
                        });
                    },
                    "Crear"
                }
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
        }
        match &*items.read() {
            None => rsx! { p { class: "muted", "Cargando webhooks…" } },
            Some(Err(e)) => rsx! {
                div { class: "alert alert-error",
                    p { "No se pudieron cargar." }
                    p { class: "muted small", "{e}" }
                    button { class: "btn btn-small", onclick: move |_| items.restart(), "Reintentar" }
                }
            },
            Some(Ok(list)) => rsx! {
                if list.is_empty() {
                    p { class: "muted", "Sin webhooks." }
                } else {
                    for w in list {
                        {
                            let key = w.id.to_string();
                            rsx! {
                                WebhookRow {
                                    key: "{key}",
                                    hook: w.clone(),
                                    on_changed: move || items.restart(),
                                }
                            }
                        }
                    }
                }
            },
        }
    }
}

/// Una fila de webhook con activar/desactivar, borrado y entregas.
#[component]
fn WebhookRow(hook: api::Webhook, on_changed: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut show_dl = use_signal(|| false);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_tg = token_of.clone();
    let token_dl = token_of.clone();
    let id = hook.id.to_string();
    let id_toggle = id.clone();
    let id_del = id.clone();
    let id_dl = id.clone();
    let state = if hook.active { "activo" } else { "pausado" };
    let toggle_label = if hook.active { "Pausar" } else { "Activar" };
    let showing = *show_dl.read();

    let mut deliveries = use_resource(move || {
        let token = token_of.clone();
        let wid = id_dl.clone();
        let open = showing;
        async move {
            match (token, open) {
                (Some(t), true) => api::webhook_deliveries(&t, &wid).await.map(Some),
                (Some(_), false) => Ok(None),
                (None, _) => Err(ApiError::no_session()),
            }
        }
    });

    rsx! {
        section { class: "card",
            strong { "{hook.event}" }
            span { class: "muted small", " · {hook.url} · {state}" }
            div { class: "row",
                button {
                    class: "btn btn-small",
                    onclick: move |_| {
                        let tok = token_tg.clone();
                        let wid = id_toggle.clone();
                        let next = !hook.active;
                        spawn(async move {
                            if let Some(t) = tok {
                                let _ = api::update_webhook(
                                        &t,
                                        &wid,
                                        &api::UpdateWebhookBody {
                                            active: Some(next),
                                            ..Default::default()
                                        },
                                    )
                                    .await;
                            }
                            on_changed.call(());
                        });
                    },
                    "{toggle_label}"
                }
                button {
                    class: "btn btn-small",
                    onclick: move |_| {
                        let tok = token_dl.clone();
                        let wid = id_del.clone();
                        spawn(async move {
                            if let Some(t) = tok {
                                let _ = api::delete_webhook(&t, &wid).await;
                            }
                            on_changed.call(());
                        });
                    },
                    "Eliminar"
                }
                button {
                    class: "btn btn-small",
                    onclick: move |_| {
                        let next = !showing;
                        show_dl.set(next);
                        if next {
                            deliveries.restart();
                        }
                    },
                    if showing { "Ocultar entregas" } else { "Ver entregas" }
                }
            }
            if showing {
                match &*deliveries.read() {
                    None => rsx! { p { class: "muted", "Cargando…" } },
                    Some(Err(e)) => rsx! { p { class: "muted small", "No se pudieron cargar: {e}" } },
                    Some(Ok(None)) => rsx! {},
                    Some(Ok(Some(list))) => rsx! {
                        if list.is_empty() {
                            p { class: "muted", "Sin entregas." }
                        } else {
                            ul {
                                for d in list.iter().take(20) {
                                    {
                                        let key = d.id.to_string();
                                        let status = d.status.clone();
                                        let code = d
                                            .response_status
                                            .map(|c| c.to_string())
                                            .unwrap_or_else(|| "—".to_string());
                                        let when = d.created_at.format("%Y-%m-%d %H:%M").to_string();
                                        rsx! {
                                            li { key: "{key}", class: "small",
                                                "{d.event_type} · {status} · HTTP {code} · {when}"
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
