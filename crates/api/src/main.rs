//! Backend estilo Twenty (NestJS -> Axum).
//! REST + JSON, estado en memoria con seed. Listo para migrar a PostgreSQL + SQLx.
//!
//! Endpoints:
//!   GET  /api/health
//!   GET/POST /api/companies  GET/PATCH/DELETE /api/companies/:id
//!   GET/POST /api/people     GET/PATCH/DELETE /api/people/:id
//!   GET/POST /api/opportunities  PATCH /api/opportunities/:id/stage
//!   GET  /api/dashboard

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use occb_core::{
    Company, DashboardStats, NewCompany, NewOpportunity, NewPerson, Opportunity, OpportunityStage,
    Person, compute_stats, seed_companies, seed_opportunities, seed_people,
};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Debug, Default)]
struct Store {
    companies: Vec<Company>,
    people: Vec<Person>,
    opportunities: Vec<Opportunity>,
}

impl Store {
    fn seeded() -> Self {
        let mut companies = seed_companies();
        let cids: Vec<Uuid> = companies.iter().map(|c| c.id).collect();
        let mut people = seed_people(&cids);
        // Reasigna company_id round-robin para que el clon se vea relacional.
        for (i, p) in people.iter_mut().enumerate() {
            p.company_id = cids.get(i % cids.len()).copied();
        }
        let pids: Vec<Uuid> = people.iter().map(|p| p.id).collect();
        let opportunities = seed_opportunities(&cids, &pids);
        // Orden alfabético como Twenty por defecto.
        companies.sort_by(|a, b| a.name.cmp(&b.name));
        Self {
            companies,
            people,
            opportunities,
        }
    }
}

type AppState = Arc<RwLock<Store>>;

#[tokio::main]
async fn main() {
    tracing_subscriber_simple();
    let state: AppState = Arc::new(RwLock::new(Store::seeded()));

    let app = Router::new()
        .route("/api/health", get(health))
        .route(
            "/api/companies",
            get(list_companies).post(create_company),
        )
        .route(
            "/api/companies/{id}",
            get(get_company).patch(update_company).delete(delete_company),
        )
        .route("/api/people", get(list_people).post(create_person))
        .route("/api/people/{id}", get(get_person).delete(delete_person))
        .route(
            "/api/opportunities",
            get(list_opportunities).post(create_opportunity),
        )
        .route(
            "/api/opportunities/{id}/stage",
            axum::routing::patch(move_stage),
        )
        .route("/api/dashboard", get(dashboard))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    println!("occb-api escuchando en http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn tracing_subscriber_simple() {
    // Sin tracing-subscriber en deps para mantener el MVP ligero; log mínimo.
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok", "service": "occb-api", "twenty_clone": true }))
}

async fn list_companies(State(s): State<AppState>) -> impl IntoResponse {
    let s = s.read().await;
    Json(s.companies.clone())
}

async fn create_company(
    State(s): State<AppState>,
    Json(input): Json<NewCompany>,
) -> impl IntoResponse {
    let mut s = s.write().await;
    let c = Company {
        id: Uuid::new_v4(),
        name: input.name,
        domain: input.domain,
        industry: input.industry,
        employees: input.employees,
        city: input.city,
        created_at: chrono::Utc::now(),
    };
    s.companies.push(c.clone());
    (StatusCode::CREATED, Json(c))
}

async fn get_company(State(s): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let s = s.read().await;
    match s.companies.iter().find(|c| c.id == id).cloned() {
        Some(c) => Ok(Json(c)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn update_company(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<NewCompany>,
) -> impl IntoResponse {
    let mut s = s.write().await;
    match s.companies.iter_mut().find(|c| c.id == id) {
        Some(c) => {
            c.name = input.name;
            c.domain = input.domain;
            c.industry = input.industry;
            c.employees = input.employees;
            c.city = input.city;
            Ok(Json(c.clone()))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn delete_company(State(s): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let mut s = s.write().await;
    let n = s.companies.len();
    s.companies.retain(|c| c.id != id);
    if s.companies.len() == n {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::NO_CONTENT
    }
}

async fn list_people(State(s): State<AppState>) -> impl IntoResponse {
    let s = s.read().await;
    Json(s.people.clone())
}

async fn create_person(
    State(s): State<AppState>,
    Json(input): Json<NewPerson>,
) -> impl IntoResponse {
    let mut s = s.write().await;
    let p = Person {
        id: Uuid::new_v4(),
        first_name: input.first_name,
        last_name: input.last_name,
        email: input.email,
        phone: input.phone,
        job_title: input.job_title,
        company_id: input.company_id,
        city: input.city,
        created_at: chrono::Utc::now(),
    };
    s.people.push(p.clone());
    (StatusCode::CREATED, Json(p))
}

async fn get_person(State(s): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let s = s.read().await;
    match s.people.iter().find(|p| p.id == id).cloned() {
        Some(p) => Ok(Json(p)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn delete_person(State(s): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let mut s = s.write().await;
    let n = s.people.len();
    s.people.retain(|p| p.id != id);
    if s.people.len() == n {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::NO_CONTENT
    }
}

async fn list_opportunities(State(s): State<AppState>) -> impl IntoResponse {
    let s = s.read().await;
    Json(s.opportunities.clone())
}

async fn create_opportunity(
    State(s): State<AppState>,
    Json(input): Json<NewOpportunity>,
) -> impl IntoResponse {
    let mut s = s.write().await;
    let o = Opportunity {
        id: Uuid::new_v4(),
        name: input.name,
        amount: input.amount,
        stage: input.stage,
        company_id: input.company_id,
        point_of_contact_id: None,
        close_date: None,
        created_at: chrono::Utc::now(),
    };
    s.opportunities.push(o.clone());
    (StatusCode::CREATED, Json(o))
}

#[derive(serde::Deserialize)]
struct StageInput {
    stage: OpportunityStage,
}

async fn move_stage(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<StageInput>,
) -> impl IntoResponse {
    let mut s = s.write().await;
    match s.opportunities.iter_mut().find(|o| o.id == id) {
        Some(o) => {
            o.stage = input.stage;
            Ok(Json(o.clone()))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn dashboard(State(s): State<AppState>) -> Json<DashboardStats> {
    let s = s.read().await;
    Json(compute_stats(&s.companies, &s.people, &s.opportunities))
}
