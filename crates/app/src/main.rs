//! Frontend Dioxus estilo Twenty.
//! Clon visual: sidebar + tablas + kanban + dashboard + login.

use dioxus::prelude::*;
use occb_core::{Company, Opportunity, OpportunityStage, Person, compute_stats};

const CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        Router::<Route> {}
    }
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(AppLayout)]
        #[route("/")]
        Dashboard {},
        #[route("/people")]
        PeoplePage {},
        #[route("/companies")]
        CompaniesPage {},
        #[route("/opportunities")]
        OpportunitiesPage {},
        #[route("/settings")]
        SettingsPage {},
    #[route("/login")]
    Login {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

#[component]
fn AppLayout() -> Element {
    rsx! {
        div { class: "shell",
            Sidebar {}
            div { class: "main",
                Outlet::<Route> {}
            }
        }
    }
}

#[component]
fn Sidebar() -> Element {
    rsx! {
        aside { class: "sidebar",
            div { class: "brand",
                div { class: "logo", "◧" }
                div {
                    div { class: "brand-name", "OCCB CRM" }
                    div { class: "brand-sub", "twenty clone · rust" }
                }
                div { class: "avatar", "OC" }
            }
            div { class: "search",
                span { class: "search-icon", "⌕" }
                input { placeholder: "Search ⌘K" }
            }
            div { class: "side-section",
                div { class: "side-title", "Favorites" }
                Link { to: Route::CompaniesPage {}, class: "side-item",
                    span { "★" }
                    span { "Acme Corp" }
                }
                Link { to: Route::OpportunitiesPage {}, class: "side-item",
                    span { "★" }
                    span { "Q3 Pipeline" }
                }
            }
            div { class: "side-section",
                div { class: "side-title", "Workspace" }
                Link { to: Route::Dashboard {}, class: "side-item",
                    span { class: "ico", "▦" }
                    span { "Dashboard" }
                }
                Link { to: Route::PeoplePage {}, class: "side-item",
                    span { class: "ico", "○" }
                    span { "People" }
                    span { class: "count", "5" }
                }
                Link { to: Route::CompaniesPage {}, class: "side-item",
                    span { class: "ico", "□" }
                    span { "Companies" }
                    span { class: "count", "6" }
                }
                Link { to: Route::OpportunitiesPage {}, class: "side-item active",
                    span { class: "ico", "◇" }
                    span { "Opportunities" }
                }
                div { class: "side-item dim",
                    span { class: "ico", "☰" }
                    span { "Notes" }
                }
                div { class: "side-item dim",
                    span { class: "ico", "✓" }
                    span { "Tasks" }
                }
                div { class: "side-item dim",
                    span { class: "ico", "⚙" }
                    span { "Workflows" }
                }
            }
            div { class: "side-spacer" }
            div { class: "side-section",
                Link { to: Route::SettingsPage {}, class: "side-item",
                    span { class: "ico", "⚙" }
                    span { "Settings" }
                }
                Link { to: Route::Login {}, class: "side-item dim",
                    span { class: "ico", "→" }
                    span { "Sign out" }
                }
            }
            div { class: "invite",
                div { class: "invite-title", "Invite teammates" }
                div { class: "invite-sub", "occb-api :3001 · dioxus web" }
                button { class: "btn-primary", "Invite" }
            }
        }
    }
}

#[component]
fn Topbar(title: String, subtitle: String) -> Element {
    rsx! {
        div { class: "topbar",
            div {
                h1 { class: "page-title", "{title}" }
                p { class: "page-sub", "{subtitle}" }
            }
            div { class: "top-actions",
                button { class: "btn-ghost", "Filter ▾" }
                button { class: "btn-ghost", "Sort ▾" }
                button { class: "btn-primary", "+ New" }
            }
        }
    }
}

#[component]
fn Dashboard() -> Element {
    let companies: Signal<Vec<Company>> = use_signal(|| {
        let mut c = occb_core::seed_companies();
        c.sort_by(|a, b| a.name.cmp(&b.name));
        c
    });
    let people: Signal<Vec<Person>> = use_signal(|| {
        let cids: Vec<_> = companies.read().iter().map(|c| c.id).collect();
        let mut p = occb_core::seed_people(&cids);
        for (i, person) in p.iter_mut().enumerate() {
            if !cids.is_empty() {
                person.company_id = cids.get(i % cids.len()).copied();
            }
        }
        p
    });
    let opps: Signal<Vec<Opportunity>> = use_signal(|| {
        let cids: Vec<_> = companies.read().iter().map(|c| c.id).collect();
        let pids: Vec<_> = people.read().iter().map(|p| p.id).collect();
        if cids.is_empty() || pids.is_empty() {
            return vec![];
        }
        occb_core::seed_opportunities(&cids, &pids)
    });

    let stats = compute_stats(&companies.read(), &people.read(), &opps.read());
    let pipeline_pct = if stats.pipeline_value + stats.won_value > 0 {
        (stats.pipeline_value as f64 / (stats.pipeline_value + stats.won_value) as f64 * 100.0)
            as u32
    } else {
        0
    };

    rsx! {
        Topbar {
            title: "Dashboard".to_string(),
            subtitle: "Welcome back — here is your pipeline today.".to_string(),
        }
        div { class: "cards",
            div { class: "card",
                div { class: "card-label", "PIPELINE VALUE" }
                div { class: "card-value", "${stats.pipeline_value}" }
                div { class: "card-sub", "{stats.total_opportunities} open opportunities" }
            }
            div { class: "card",
                div { class: "card-label", "WON THIS QUARTER" }
                div { class: "card-value green", "${stats.won_value}" }
                div { class: "card-sub", "{stats.win_rate:.1}% win rate" }
            }
            div { class: "card",
                div { class: "card-label", "COMPANIES" }
                div { class: "card-value", "{stats.total_companies}" }
                div { class: "card-sub", "{stats.total_people} people tracked" }
            }
            div { class: "card",
                div { class: "card-label", "API BACKEND" }
                div { class: "card-value small", "axum :3001" }
                div { class: "card-sub", "GET /api/dashboard → live" }
            }
        }
        div { class: "panel",
            div { class: "panel-head",
                span { class: "panel-title", "Pipeline health" }
                span { class: "pill", "{pipeline_pct}% open" }
            }
            div { class: "bar",
                div { class: "bar-fill", width: "{pipeline_pct}%" }
            }
            div { class: "panel-grid",
                for opp in opps.read().iter().take(4) {
                    div { class: "mini-row", key: "{opp.id}",
                        span { class: "mini-name", "{opp.name}" }
                        span { class: "mini-amt", "${opp.amount}" }
                        span { class: "stage stage-{opp.stage:?}", "{opp.stage.label()}" }
                    }
                }
            }
        }
        div { class: "panel",
            div { class: "panel-head",
                span { class: "panel-title", "Recent companies" }
                Link { to: Route::CompaniesPage {}, class: "link", "View all →" }
            }
            table { class: "tbl",
                thead {
                    tr { th { "Company" } th { "Domain" } th { "Industry" } th { "Employees" } }
                }
                tbody {
                    for c in companies.read().iter().take(5) {
                        tr { key: "{c.id}",
                            td { class: "strong", "{c.name}" }
                            td { class: "muted", "{c.domain}" }
                            td { "{c.industry}" }
                            td { "{c.employees}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PeoplePage() -> Element {
    let mut query = use_signal(String::new);
    let companies = use_signal(|| occb_core::seed_companies());
    let people = use_signal(|| {
        // ids temporales; se reasignan abajo para mostrar empresa real
        let tmp = occb_core::seed_companies();
        let cids: Vec<_> = tmp.iter().map(|c| c.id).collect();
        occb_core::seed_people(&cids)
    });

    let q = query.read().to_lowercase();
    let rows: Vec<Person> = people
        .read()
        .iter()
        .filter(|p| {
            q.is_empty()
                || p.full_name().to_lowercase().contains(&q)
                || p.email.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();

    let rows_len = rows.len();
    let company_name = move |id: Option<uuid::Uuid>| -> String {
        match id {
            Some(cid) => companies
                .read()
                .iter()
                .find(|c| c.id == cid)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "—".into()),
            None => ["Acme Corp", "Globex", "Initech", "Hooli", "Umbrella"]
                .into_iter()
                .cycle()
                .nth(rows_len % 5)
                .unwrap_or("—")
                .to_string(),
        }
    };

    rsx! {
        Topbar {
            title: "People".to_string(),
            subtitle: format!("{} contacts · table view like Twenty", rows_len),
        }
        div { class: "toolbar",
            input {
                class: "input",
                placeholder: "Search people…",
                value: "{query}",
                oninput: move |e| query.set(e.value()),
            }
            span { class: "pill", "Table" }
            span { class: "pill dim", "Kanban" }
        }
        div { class: "panel",
            table { class: "tbl",
                thead {
                    tr {
                        th { "Name" }
                        th { "Email" }
                        th { "Job title" }
                        th { "Company" }
                        th { "City" }
                    }
                }
                tbody {
                    for p in rows.iter() {
                        tr { key: "{p.id}",
                            td { class: "strong",
                                span { class: "mini-avatar", "{p.first_name.chars().next().unwrap_or('?')}{p.last_name.chars().next().unwrap_or('?')}" }
                                "{p.full_name()}"
                            }
                            td { class: "muted", "{p.email}" }
                            td { "{p.job_title}" }
                            td { "{company_name(p.company_id)}" }
                            td { "{p.city}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CompaniesPage() -> Element {
    let mut query = use_signal(String::new);
    let companies = use_signal(|| {
        let mut c = occb_core::seed_companies();
        c.sort_by(|a, b| a.name.cmp(&b.name));
        c
    });

    let q = query.read().to_lowercase();
    let rows: Vec<Company> = companies
        .read()
        .iter()
        .filter(|c| {
            q.is_empty()
                || c.name.to_lowercase().contains(&q)
                || c.domain.to_lowercase().contains(&q)
                || c.industry.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();

    rsx! {
        Topbar {
            title: "Companies".to_string(),
            subtitle: format!("{} accounts · {} total employees", rows.len(), rows.iter().map(|c| c.employees).sum::<u32>()),
        }
        div { class: "toolbar",
            input {
                class: "input",
                placeholder: "Search companies…",
                value: "{query}",
                oninput: move |e| query.set(e.value()),
            }
            button { class: "btn-ghost", "＋ Add view" }
            button { class: "btn-ghost", "⇅ Columns" }
        }
        div { class: "grid2",
            for c in rows.iter() {
                div { class: "company-card", key: "{c.id}",
                    div { class: "cc-head",
                        div { class: "cc-logo", "{c.name.chars().next().unwrap_or('C')}" }
                        div {
                            div { class: "strong", "{c.name}" }
                            div { class: "muted small", "{c.domain}" }
                        }
                        span { class: "pill", "{c.industry}" }
                    }
                    div { class: "cc-meta",
                        span { "👥 {c.employees}" }
                        span { "📍 {c.city}" }
                    }
                }
            }
        }
    }
}

#[component]
fn OpportunitiesPage() -> Element {
    let companies = use_signal(|| occb_core::seed_companies());
    let people = use_signal(|| {
        let cids: Vec<_> = companies.read().iter().map(|c| c.id).collect();
        occb_core::seed_people(&cids)
    });
    let mut opps = use_signal(|| {
        let cids: Vec<_> = companies.read().iter().map(|c| c.id).collect();
        let pids: Vec<_> = people.read().iter().map(|p| p.id).collect();
        if cids.is_empty() || pids.is_empty() {
            return vec![];
        }
        occb_core::seed_opportunities(&cids, &pids)
    });

    let total: i64 = opps.read().iter().map(|o| o.amount).sum();

    let company_of = move |id: Option<uuid::Uuid>| {
        companies
            .read()
            .iter()
            .find(|c| Some(c.id) == id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "—".into())
    };

    rsx! {
        Topbar {
            title: "Opportunities".to_string(),
            subtitle: format!("Pipeline ${total} · drag stages like Twenty kanban (click ▶ to advance)"),
        }
        div { class: "kanban",
            for stage in OpportunityStage::all() {
                div { class: "col", key: "{stage:?}",
                    div { class: "col-head",
                        span { class: "stage stage-{stage:?}", "{stage.label()}" }
                        span { class: "count",
                            "{opps.read().iter().filter(|o| &o.stage == stage).count()}"
                        }
                    }
                    div { class: "col-sum",
                        "${
                            opps
                                .read()
                                .iter()
                                .filter(|o| &o.stage == stage)
                                .map(|o| o.amount)
                                .sum::<i64>()
                        }"
                    }
                    for opp in opps.read().iter().filter(|o| &o.stage == stage).cloned().collect::<Vec<_>>() {
                        div { class: "opp-card", key: "{opp.id}",
                            div { class: "opp-name", "{opp.name}" }
                            div { class: "opp-sub", "{company_of(opp.company_id)}" }
                            div { class: "opp-foot",
                                span { class: "opp-amt", "${opp.amount}" }
                                span { class: "opp-prob", "{stage.probability()}%" }
                                button {
                                    class: "btn-mini",
                                    title: "Advance stage",
                                    onclick: {
                                        let id = opp.id;
                                        move |_| {
                                            let mut list = opps.write();
                                            if let Some(o) = list.iter_mut().find(|x| x.id == id) {
                                                o.stage = match o.stage {
                                                    OpportunityStage::New => OpportunityStage::Screening,
                                                    OpportunityStage::Screening => OpportunityStage::Meeting,
                                                    OpportunityStage::Meeting => OpportunityStage::Proposal,
                                                    OpportunityStage::Proposal => OpportunityStage::Customer,
                                                    OpportunityStage::Customer => OpportunityStage::Customer,
                                                };
                                            }
                                        }
                                    },
                                    "▶"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SettingsPage() -> Element {
    rsx! {
        Topbar {
            title: "Settings".to_string(),
            subtitle: "Workspace · backend Axum + PostgreSQL (docker) · frontend Dioxus".to_string(),
        }
        div { class: "panel form",
            label { "Workspace name" }
            input { class: "input", value: "OCCB CRM — twenty clone" }
            label { "API base URL" }
            input { class: "input", value: "http://localhost:3001/api" }
            label { "Database" }
            input { class: "input", value: "postgres://occb:occb@localhost:5432/occb" }
            p { class: "muted small",
                "MVP usa store en memoria con seed. El esquema SQL en /migrations espeja twenty-server (companies, people, opportunities). Activa la feature postgres del API para persistencia real."
            }
            div { class: "row",
                button { class: "btn-primary", "Save" }
                button { class: "btn-ghost", "Reset demo data" }
            }
        }
    }
}

#[component]
fn Login() -> Element {
    let mut email = use_signal(|| "admin@occb.local".to_string());
    let mut _password = use_signal(|| "password".to_string());
    rsx! {
        div { class: "login-wrap",
            div { class: "login-card",
                div { class: "logo big", "◧" }
                h1 { "Welcome back" }
                p { class: "muted", "Sign in to your OCCB workspace (Twenty clone in Rust + Dioxus)" }
                label { "Email" }
                input {
                    class: "input",
                    value: "{email}",
                    oninput: move |e| email.set(e.value()),
                }
                label { "Password" }
                input { class: "input", r#type: "password", value: "password" }
                Link { to: Route::Dashboard {}, class: "btn-primary block",
                    "Continue →"
                }
                p { class: "muted small center",
                    "Demo sin auth real · el backend expone /api/health en :3001"
                }
            }
        }
    }
}

#[component]
fn NotFound(segments: Vec<String>) -> Element {
    rsx! {
        div { class: "login-wrap",
            div { class: "login-card",
                h1 { "404" }
                p { class: "muted", "Ruta no encontrada: /{segments.join(\"/\")}" }
                Link { to: Route::Dashboard {}, class: "btn-primary block", "Back to dashboard" }
            }
        }
    }
}
