//! Dominio compartido estilo Twenty: objetos, campos y seed data.
//! Equivalente Rust a `twenty-server` (metadata-driven) simplificado para el MVP.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn now() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Company {
    pub id: Uuid,
    pub name: String,
    pub domain: String,
    pub industry: String,
    pub employees: u32,
    pub city: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Person {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub job_title: String,
    pub company_id: Option<Uuid>,
    pub city: String,
    pub created_at: DateTime<Utc>,
}

impl Person {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityStage {
    New,
    Screening,
    Meeting,
    Proposal,
    Customer,
}

impl OpportunityStage {
    pub fn all() -> &'static [Self] {
        use OpportunityStage::*;
        &[New, Screening, Meeting, Proposal, Customer]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::New => "New",
            Self::Screening => "Screening",
            Self::Meeting => "Meeting",
            Self::Proposal => "Proposal / Proposal Sent",
            Self::Customer => "Customer",
        }
    }

    pub fn probability(&self) -> u8 {
        match self {
            Self::New => 10,
            Self::Screening => 25,
            Self::Meeting => 50,
            Self::Proposal => 75,
            Self::Customer => 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Opportunity {
    pub id: Uuid,
    pub name: String,
    pub amount: i64,
    pub stage: OpportunityStage,
    pub company_id: Option<Uuid>,
    pub point_of_contact_id: Option<Uuid>,
    pub close_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_companies: usize,
    pub total_people: usize,
    pub total_opportunities: usize,
    pub pipeline_value: i64,
    pub won_value: i64,
    pub win_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCompany {
    pub name: String,
    pub domain: String,
    pub industry: String,
    pub employees: u32,
    pub city: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPerson {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub job_title: String,
    pub company_id: Option<Uuid>,
    pub city: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOpportunity {
    pub name: String,
    pub amount: i64,
    pub stage: OpportunityStage,
    pub company_id: Option<Uuid>,
}

/// Seed determinista para demo (clon visual de Twenty).
pub fn seed_companies() -> Vec<Company> {
    let mk = |name: &str, domain: &str, industry: &str, employees: u32, city: &str| Company {
        id: Uuid::new_v4(),
        name: name.into(),
        domain: domain.into(),
        industry: industry.into(),
        employees,
        city: city.into(),
        created_at: now(),
    };
    vec![
        mk("Acme Corp", "acme.com", "SaaS", 240, "San Francisco"),
        mk("Globex", "globex.io", "Fintech", 85, "New York"),
        mk("Initech", "initech.dev", "DevTools", 42, "Austin"),
        mk("Umbrella", "umbrella.health", "Health", 510, "Boston"),
        mk("Hooli", "hooli.com", "Cloud", 1200, "Mountain View"),
        mk("Stark Industries", "stark.ind", "Manufacturing", 3000, "Malibu"),
    ]
}

pub fn seed_people(company_ids: &[Uuid]) -> Vec<Person> {
    let mk = |f: &str,
              l: &str,
              email: &str,
              title: &str,
              c: Option<Uuid>,
              city: &str| Person {
        id: Uuid::new_v4(),
        first_name: f.into(),
        last_name: l.into(),
        email: email.into(),
        phone: "+1 555-0100".into(),
        job_title: title.into(),
        company_id: c,
        city: city.into(),
        created_at: now(),
    };
    // company_ids se reasigna fuera para mantener relaciones reales.
    let _ = company_ids.len();
    vec![
        mk(
            "Ada",
            "Lovelace",
            "ada@acme.com",
            "CTO",
            None,
            "San Francisco",
        ),
        mk(
            "Alan",
            "Turing",
            "alan@globex.io",
            "Head of Data",
            None,
            "New York",
        ),
        mk("Grace", "Hopper", "grace@initech.dev", "VP Eng", None, "Austin"),
        mk(
            "Linus",
            "Torvalds",
            "linus@hooli.com",
            "Staff Engineer",
            None,
            "Mountain View",
        ),
        mk(
            "Marie",
            "Curie",
            "marie@umbrella.health",
            "Research Lead",
            None,
            "Boston",
        ),
    ]
}

pub fn seed_opportunities(
    company_ids: &[Uuid],
    person_ids: &[Uuid],
) -> Vec<Opportunity> {
    let c = |i: usize| company_ids.get(i % company_ids.len()).copied();
    let p = |i: usize| person_ids.get(i % person_ids.len()).copied();
    let mk = |name: &str, amount: i64, stage: OpportunityStage, i: usize| Opportunity {
        id: Uuid::new_v4(),
        name: name.into(),
        amount,
        stage,
        company_id: c(i),
        point_of_contact_id: p(i),
        close_date: None,
        created_at: now(),
    };
    use OpportunityStage::*;
    vec![
        mk("Acme — Enterprise plan", 120_000, New, 0),
        mk("Globex — Payments rollout", 85_000, Screening, 1),
        mk("Initech — DevTools seat expansion", 24_000, Meeting, 2),
        mk("Hooli — Cloud migration", 310_000, Proposal, 4),
        mk("Umbrella — Research suite", 150_000, Meeting, 3),
        mk("Stark — Factory IoT", 480_000, Customer, 5),
    ]
}

pub fn compute_stats(
    companies: &[Company],
    people: &[Person],
    opps: &[Opportunity],
) -> DashboardStats {
    let pipeline_value: i64 = opps
        .iter()
        .filter(|o| o.stage != OpportunityStage::Customer)
        .map(|o| o.amount)
        .sum();
    let won_value: i64 = opps
        .iter()
        .filter(|o| o.stage == OpportunityStage::Customer)
        .map(|o| o.amount)
        .sum();
    let total = opps.len().max(1) as f32;
    let won = opps
        .iter()
        .filter(|o| o.stage == OpportunityStage::Customer)
        .count() as f32;
    DashboardStats {
        total_companies: companies.len(),
        total_people: people.len(),
        total_opportunities: opps.len(),
        pipeline_value,
        won_value,
        win_rate: won / total * 100.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_math() {
        let companies = seed_companies();
        let cids: Vec<Uuid> = companies.iter().map(|c| c.id).collect();
        let people = seed_people(&cids);
        let pids: Vec<Uuid> = people.iter().map(|p| p.id).collect();
        let opps = seed_opportunities(&cids, &pids);
        let stats = compute_stats(&companies, &people, &opps);
        assert_eq!(stats.total_companies, 6);
        assert_eq!(stats.total_opportunities, 6);
        assert!(stats.pipeline_value > 0);
    }
}
