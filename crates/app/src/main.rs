//! Frontal OCCB en Dioxus, todo en español con i18n ES/EN.
//!
//! Buenas prácticas: textos centralizados en `occb-core::i18n`, componentes
//! pequeños por responsabilidad y estado local con señales.

use dioxus::prelude::*;
use occb_core::{
    Empresa, EtapaOportunidad, Idioma, Oportunidad, Persona, calcular_resumen,
    datos_semilla_empresas, datos_semilla_oportunidades, datos_semilla_personas, textos,
};

/// Hoja de estilos del frontal.
const CSS: Asset = asset!("/assets/main.css");

/// Idioma global de la interfaz (español por defecto, conmutable a inglés).
static IDIOMA: GlobalSignal<Idioma> = Signal::global(|| Idioma::Es);

fn main() {
    dioxus::launch(Aplicacion);
}

/// Componente raíz: estilos + enrutador.
#[component]
fn Aplicacion() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        Router::<Ruta> {}
    }
}

/// Rutas de la aplicación (nombres en español).
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Ruta {
    #[layout(DisposicionApp)]
        #[route("/")]
        Panel {},
        #[route("/personas")]
        VistaPersonas {},
        #[route("/empresas")]
        VistaEmpresas {},
        #[route("/oportunidades")]
        VistaOportunidades {},
        #[route("/ajustes")]
        VistaAjustes {},
    #[route("/acceso")]
    VistaAcceso {},
    #[route("/:..segmentos")]
    NoEncontrada { segmentos: Vec<String> },
}

/// Etiqueta de etapa según el idioma (la BD guarda siempre el texto español).
fn etiqueta_etapa(etapa: EtapaOportunidad, idioma: Idioma) -> &'static str {
    match (etapa, idioma) {
        (EtapaOportunidad::Nueva, Idioma::Es) => "Nueva",
        (EtapaOportunidad::Cribado, Idioma::Es) => "Cribado",
        (EtapaOportunidad::Reunion, Idioma::Es) => "Reunión",
        (EtapaOportunidad::Propuesta, Idioma::Es) => "Propuesta",
        (EtapaOportunidad::Cliente, Idioma::Es) => "Cliente",
        (EtapaOportunidad::Nueva, Idioma::En) => "New",
        (EtapaOportunidad::Cribado, Idioma::En) => "Screening",
        (EtapaOportunidad::Reunion, Idioma::En) => "Meeting",
        (EtapaOportunidad::Propuesta, Idioma::En) => "Proposal",
        (EtapaOportunidad::Cliente, Idioma::En) => "Customer",
    }
}

/// Disposición con barra lateral.
#[component]
fn DisposicionApp() -> Element {
    rsx! {
        div { class: "shell",
            BarraLateral {}
            div { class: "main",
                Outlet::<Ruta> {}
            }
        }
    }
}

/// Barra lateral con navegación y selector de idioma ES/EN.
#[component]
fn BarraLateral() -> Element {
    let t = textos(IDIOMA());
    rsx! {
        aside { class: "sidebar",
            div { class: "brand",
                div { class: "logo", "◧" }
                div {
                    div { class: "brand-name", "{t.nombre_app}" }
                    div { class: "brand-sub", "{t.eslogan}" }
                }
                div { class: "avatar", "OC" }
            }
            div { class: "search",
                span { class: "search-icon", "⌕" }
                input { placeholder: "{t.buscar}" }
            }
            div { class: "side-section",
                div { class: "side-title", "{t.favoritos}" }
                Link { to: Ruta::VistaEmpresas {}, class: "side-item",
                    span { "★" }
                    span { "Acme Corp" }
                }
                Link { to: Ruta::VistaOportunidades {}, class: "side-item",
                    span { "★" }
                    span { "Q3" }
                }
            }
            div { class: "side-section",
                div { class: "side-title", "{t.espacio_trabajo}" }
                Link { to: Ruta::Panel {}, class: "side-item",
                    span { class: "ico", "▦" }
                    span { "{t.panel}" }
                }
                Link { to: Ruta::VistaPersonas {}, class: "side-item",
                    span { class: "ico", "○" }
                    span { "{t.personas}" }
                    span { class: "count", "5" }
                }
                Link { to: Ruta::VistaEmpresas {}, class: "side-item",
                    span { class: "ico", "□" }
                    span { "{t.empresas}" }
                    span { class: "count", "6" }
                }
                Link { to: Ruta::VistaOportunidades {}, class: "side-item active",
                    span { class: "ico", "◇" }
                    span { "{t.oportunidades}" }
                }
                div { class: "side-item dim",
                    span { class: "ico", "☰" }
                    span { "{t.notas}" }
                }
                div { class: "side-item dim",
                    span { class: "ico", "✓" }
                    span { "{t.tareas}" }
                }
                div { class: "side-item dim",
                    span { class: "ico", "⚙" }
                    span { "{t.automatizaciones}" }
                }
            }
            div { class: "side-spacer" }
            div { class: "side-section",
                div { class: "lang-row",
                    button {
                        class: if IDIOMA() == Idioma::Es { "btn-mini active" } else { "btn-mini" },
                        onclick: move |_| *IDIOMA.write() = Idioma::Es,
                        "ES"
                    }
                    button {
                        class: if IDIOMA() == Idioma::En { "btn-mini active" } else { "btn-mini" },
                        onclick: move |_| *IDIOMA.write() = Idioma::En,
                        "EN"
                    }
                }
                Link { to: Ruta::VistaAjustes {}, class: "side-item",
                    span { class: "ico", "⚙" }
                    span { "{t.ajustes}" }
                }
                Link { to: Ruta::VistaAcceso {}, class: "side-item dim",
                    span { class: "ico", "→" }
                    span { "{t.salir}" }
                }
            }
            div { class: "invite",
                div { class: "invite-title", "{t.invitar} · OCCB" }
                div { class: "invite-sub", "{t.invitar_subtitulo}" }
                button { class: "btn-primary", "{t.invitar}" }
            }
        }
    }
}

/// Cabecera de cada vista: título, subtítulo y acciones.
#[component]
fn BarraSuperior(titulo: String, subtitulo: String) -> Element {
    let t = textos(IDIOMA());
    rsx! {
        div { class: "topbar",
            div {
                h1 { class: "page-title", "{titulo}" }
                p { class: "page-sub", "{subtitulo}" }
            }
            div { class: "top-actions",
                button { class: "btn-ghost", "{t.filtrar}" }
                button { class: "btn-ghost", "{t.ordenar}" }
                button { class: "btn-primary", "{t.nueva}" }
            }
        }
    }
}

/// Carga las empresas de demostración (en Fase 1.1 vendrán de `/api/empresas`).
fn cargar_empresas_demo() -> Vec<Empresa> {
    datos_semilla_empresas()
}

/// Carga las personas de demostración con su empresa asignada.
fn cargar_personas_demo(ids_empresas: &[uuid::Uuid]) -> Vec<Persona> {
    let mut lista = datos_semilla_personas();
    if !ids_empresas.is_empty() {
        for (i, persona) in lista.iter_mut().enumerate() {
            persona.empresa_id = ids_empresas.get(i % ids_empresas.len()).copied();
        }
    }
    lista
}

/// Vista del panel principal con tarjetas y resumen.
#[component]
fn Panel() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let empresas: Signal<Vec<Empresa>> = use_signal(cargar_empresas_demo);
    let personas: Signal<Vec<Persona>> = use_signal(|| {
        let ids: Vec<_> = empresas.read().iter().map(|e| e.id).collect();
        cargar_personas_demo(&ids)
    });
    let oportunidades: Signal<Vec<Oportunidad>> = use_signal(|| {
        let ids_empresas: Vec<_> = empresas.read().iter().map(|e| e.id).collect();
        let ids_personas: Vec<_> = personas.read().iter().map(|p| p.id).collect();
        if ids_empresas.is_empty() || ids_personas.is_empty() {
            return vec![];
        }
        datos_semilla_oportunidades(&ids_empresas, &ids_personas)
    });

    let resumen = calcular_resumen(&empresas.read(), &personas.read(), &oportunidades.read());
    let total = (resumen.valor_tuberia + resumen.valor_ganado).max(1) as f64;
    let porcentaje_abierto = (resumen.valor_tuberia as f64 / total * 100.0) as u32;

    rsx! {
        BarraSuperior {
            titulo: t.panel.to_string(),
            subtitulo: t.bienvenida_panel.to_string(),
        }
        div { class: "cards",
            div { class: "card",
                div { class: "card-label", "{t.valor_tuberia}" }
                div { class: "card-value", "${resumen.valor_tuberia}" }
                div { class: "card-sub",
                    "{resumen.total_oportunidades} · {t.oportunidades.to_lowercase()}"
                }
            }
            div { class: "card",
                div { class: "card-label", "{t.ganado_trimestre}" }
                div { class: "card-value green", "${resumen.valor_ganado}" }
                div { class: "card-sub", "{resumen.tasa_exito:.1}% ✓" }
            }
            div { class: "card",
                div { class: "card-label", "{t.cuentas}" }
                div { class: "card-value", "{resumen.total_empresas}" }
                div { class: "card-sub",
                    "{resumen.total_personas} · {t.personas.to_lowercase()}"
                }
            }
            div { class: "card",
                div { class: "card-label", "{t.motor_api}" }
                div { class: "card-value small", "axum :3001" }
                div { class: "card-sub", "GET /api/panel → postgres:18" }
            }
        }
        div { class: "panel",
            div { class: "panel-head",
                span { class: "panel-title", "{t.salud_tuberia}" }
                span { class: "pill", "{porcentaje_abierto}% ✓" }
            }
            div { class: "bar",
                div { class: "bar-fill", width: "{porcentaje_abierto}%" }
            }
            div { class: "panel-grid",
                for op in oportunidades.read().iter().take(4) {
                    div { class: "mini-row", key: "{op.id}",
                        span { class: "mini-name", "{op.nombre}" }
                        span { class: "mini-amt", "${op.importe}" }
                        span { class: "stage stage-{op.etapa:?}", "{etiqueta_etapa(op.etapa, idioma)}" }
                    }
                }
            }
        }
        div { class: "panel",
            div { class: "panel-head",
                span { class: "panel-title", "{t.recientes}" }
                Link { to: Ruta::VistaEmpresas {}, class: "link", "{t.ver_todo}" }
            }
            table { class: "tbl",
                thead {
                    tr { th { "{t.empresas}" } th { "dominio" } th { "sector" } th { "👥" } }
                }
                tbody {
                    for e in empresas.read().iter().take(5) {
                        tr { key: "{e.id}",
                            td { class: "strong", "{e.nombre}" }
                            td { class: "muted", "{e.dominio}" }
                            td { "{e.sector}" }
                            td { "{e.empleados}" }
                        }
                    }
                }
            }
        }
    }
}

/// Vista de personas con búsqueda en español.
#[component]
fn VistaPersonas() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut busqueda = use_signal(String::new);
    let empresas = use_signal(cargar_empresas_demo);
    let personas = use_signal(|| {
        let ids: Vec<_> = empresas.read().iter().map(|e| e.id).collect();
        cargar_personas_demo(&ids)
    });

    let consulta = busqueda.read().to_lowercase();
    let filas: Vec<Persona> = personas
        .read()
        .iter()
        .filter(|p| {
            consulta.is_empty()
                || p.nombre_completo().to_lowercase().contains(&consulta)
                || p.correo.to_lowercase().contains(&consulta)
        })
        .cloned()
        .collect();
    let total = filas.len();

    let nombre_empresa = move |id: Option<uuid::Uuid>| -> String {
        empresas
            .read()
            .iter()
            .find(|e| Some(e.id) == id)
            .map(|e| e.nombre.clone())
            .unwrap_or_else(|| "—".into())
    };

    rsx! {
        BarraSuperior {
            titulo: t.personas.to_string(),
            subtitulo: format!("{total} · {}", t.tabla),
        }
        div { class: "toolbar",
            input {
                class: "input",
                placeholder: "{t.buscar}…",
                value: "{busqueda}",
                oninput: move |e| busqueda.set(e.value()),
            }
            span { class: "pill", "{t.tabla}" }
            span { class: "pill dim", "{t.kanban}" }
        }
        div { class: "panel",
            table { class: "tbl",
                thead {
                    tr {
                        th { if idioma == Idioma::Es { "Nombre" } else { "Name" } }
                        th { "{t.correo}" }
                        th { if idioma == Idioma::Es { "Puesto" } else { "Role" } }
                        th { if idioma == Idioma::Es { "Empresa" } else { "Company" } }
                        th { if idioma == Idioma::Es { "Ciudad" } else { "City" } }
                    }
                }
                tbody {
                    for p in filas.iter() {
                        tr { key: "{p.id}",
                            td { class: "strong",
                                span { class: "mini-avatar", "{p.iniciales()}" }
                                "{p.nombre_completo()}"
                            }
                            td { class: "muted", "{p.correo}" }
                            td { "{p.puesto}" }
                            td { "{nombre_empresa(p.empresa_id)}" }
                            td { "{p.ciudad}" }
                        }
                    }
                }
            }
        }
    }
}

/// Vista de empresas con tarjetas y búsqueda.
#[component]
fn VistaEmpresas() -> Element {
    let t = textos(IDIOMA());
    let mut busqueda = use_signal(String::new);
    let empresas = use_signal(cargar_empresas_demo);

    let consulta = busqueda.read().to_lowercase();
    let filas: Vec<Empresa> = empresas
        .read()
        .iter()
        .filter(|e| {
            consulta.is_empty()
                || e.nombre.to_lowercase().contains(&consulta)
                || e.dominio.to_lowercase().contains(&consulta)
                || e.sector.to_lowercase().contains(&consulta)
        })
        .cloned()
        .collect();
    let total_empleados: i32 = filas.iter().map(|e| e.empleados).sum();

    rsx! {
        BarraSuperior {
            titulo: t.empresas.to_string(),
            subtitulo: format!("{} · 👥 {total_empleados}", filas.len()),
        }
        div { class: "toolbar",
            input {
                class: "input",
                placeholder: "{t.buscar}…",
                value: "{busqueda}",
                oninput: move |e| busqueda.set(e.value()),
            }
        }
        div { class: "grid2",
            for e in filas.iter() {
                div { class: "company-card", key: "{e.id}",
                    div { class: "cc-head",
                        div { class: "cc-logo", "{e.nombre.chars().next().unwrap_or('E')}" }
                        div {
                            div { class: "strong", "{e.nombre}" }
                            div { class: "muted small", "{e.dominio}" }
                        }
                        span { class: "pill", "{e.sector}" }
                    }
                    div { class: "cc-meta",
                        span { "👥 {e.empleados}" }
                        span { "📍 {e.ciudad}" }
                    }
                }
            }
        }
    }
}

/// Vista kanban de oportunidades: clic en ▶ para avanzar de etapa.
#[component]
fn VistaOportunidades() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let empresas = use_signal(cargar_empresas_demo);
    let personas = use_signal(|| {
        let ids: Vec<_> = empresas.read().iter().map(|e| e.id).collect();
        cargar_personas_demo(&ids)
    });
    let mut oportunidades = use_signal(|| {
        let ids_empresas: Vec<_> = empresas.read().iter().map(|e| e.id).collect();
        let ids_personas: Vec<_> = personas.read().iter().map(|p| p.id).collect();
        if ids_empresas.is_empty() || ids_personas.is_empty() {
            return vec![];
        }
        datos_semilla_oportunidades(&ids_empresas, &ids_personas)
    });

    let total: i64 = oportunidades.read().iter().map(|o| o.importe).sum();
    let nombre_empresa = move |id: Option<uuid::Uuid>| {
        empresas
            .read()
            .iter()
            .find(|e| Some(e.id) == id)
            .map(|e| e.nombre.clone())
            .unwrap_or_else(|| "—".into())
    };

    rsx! {
        BarraSuperior {
            titulo: t.oportunidades.to_string(),
            subtitulo: format!(
                "${total} · {}",
                if idioma == Idioma::Es { "pulsa ▶ para avanzar de etapa" } else { "click ▶ to advance stage" }
            ),
        }
        div { class: "kanban",
            for etapa in EtapaOportunidad::todas() {
                div { class: "col", key: "{etapa:?}",
                    div { class: "col-head",
                        span { class: "stage stage-{etapa:?}", "{etiqueta_etapa(*etapa, idioma)}" }
                        span { class: "count",
                            "{oportunidades.read().iter().filter(|o| &o.etapa == etapa).count()}"
                        }
                    }
                    div { class: "col-sum",
                        "${
                            oportunidades
                                .read()
                                .iter()
                                .filter(|o| &o.etapa == etapa)
                                .map(|o| o.importe)
                                .sum::<i64>()
                        }"
                    }
                    for op in oportunidades.read().iter().filter(|o| &o.etapa == etapa).cloned().collect::<Vec<_>>() {
                        div { class: "opp-card", key: "{op.id}",
                            div { class: "opp-name", "{op.nombre}" }
                            div { class: "opp-sub", "{nombre_empresa(op.empresa_id)}" }
                            div { class: "opp-foot",
                                span { class: "opp-amt", "${op.importe}" }
                                span { class: "opp-prob", "{etapa.probabilidad()}%" }
                                button {
                                    class: "btn-mini",
                                    title: if idioma == Idioma::Es { "Avanzar etapa" } else { "Advance stage" },
                                    onclick: {
                                        let id = op.id;
                                        move |_| {
                                            let mut lista = oportunidades.write();
                                            if let Some(o) = lista.iter_mut().find(|x| x.id == id) {
                                                o.etapa = o.etapa.siguiente();
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

/// Vista de ajustes del espacio de trabajo.
#[component]
fn VistaAjustes() -> Element {
    let t = textos(IDIOMA());
    rsx! {
        BarraSuperior {
            titulo: t.ajustes.to_string(),
            subtitulo: "Axum + PostgreSQL 18 · Dioxus web".to_string(),
        }
        div { class: "panel form",
            label { if IDIOMA() == Idioma::Es { "Nombre del espacio" } else { "Workspace name" } }
            input { class: "input", value: "OCCB CRM" }
            label { "API" }
            input { class: "input", value: "http://localhost:3001/api" }
            label { if IDIOMA() == Idioma::Es { "Base de datos" } else { "Database" } }
            input { class: "input", value: "postgres://occb:occb@localhost:5432/occb" }
            p { class: "muted small",
                if IDIOMA() == Idioma::Es {
                    "El CRUD pasa solo por funciones SECURITY DEFINER con RLS. Ver carpeta /migrations."
                } else {
                    "CRUD goes only through SECURITY DEFINER functions with RLS. See /migrations."
                }
            }
            div { class: "lang-row",
                button {
                    class: if IDIOMA() == Idioma::Es { "btn-mini active" } else { "btn-mini" },
                    onclick: move |_| *IDIOMA.write() = Idioma::Es,
                    "Español"
                }
                button {
                    class: if IDIOMA() == Idioma::En { "btn-mini active" } else { "btn-mini" },
                    onclick: move |_| *IDIOMA.write() = Idioma::En,
                    "English"
                }
            }
            div { class: "row",
                button { class: "btn-primary", "{t.guardar}" }
            }
        }
    }
}

/// Pantalla de acceso (demo sin autenticación real en Fase 1).
#[component]
fn VistaAcceso() -> Element {
    let t = textos(IDIOMA());
    let mut correo = use_signal(|| "admin@occb.local".to_string());
    rsx! {
        div { class: "login-wrap",
            div { class: "login-card",
                div { class: "logo big", "◧" }
                h1 { "{t.acceso_titulo}" }
                p { class: "muted", "{t.acceso_subtitulo}" }
                div { class: "lang-row center",
                    button {
                        class: if IDIOMA() == Idioma::Es { "btn-mini active" } else { "btn-mini" },
                        onclick: move |_| *IDIOMA.write() = Idioma::Es,
                        "ES"
                    }
                    button {
                        class: if IDIOMA() == Idioma::En { "btn-mini active" } else { "btn-mini" },
                        onclick: move |_| *IDIOMA.write() = Idioma::En,
                        "EN"
                    }
                }
                label { "{t.correo}" }
                input {
                    class: "input",
                    value: "{correo}",
                    oninput: move |e| correo.set(e.value()),
                }
                label { "{t.contrasena}" }
                input { class: "input", r#type: "password", value: "password" }
                Link { to: Ruta::Panel {}, class: "btn-primary block", "{t.continuar}" }
            }
        }
    }
}

/// Página 404 en el idioma activo.
#[component]
fn NoEncontrada(segmentos: Vec<String>) -> Element {
    let idioma = IDIOMA();
    rsx! {
        div { class: "login-wrap",
            div { class: "login-card",
                h1 { "404" }
                p { class: "muted",
                    if idioma == Idioma::Es {
                        "Ruta no encontrada: /{segmentos.join(\"/\")}"
                    } else {
                        "Route not found: /{segmentos.join(\"/\")}"
                    }
                }
                Link { to: Ruta::Panel {}, class: "btn-primary block", "←" }
            }
        }
    }
}
