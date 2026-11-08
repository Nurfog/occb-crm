//! Frontal OCCB en Dioxus, todo en español con i18n ES/EN.
//!
//! Buenas prácticas: textos centralizados en `occb-core::i18n`, un módulo
//! `cliente` para hablar con la API y componentes pequeños por vista.
//! Las vistas leen y escriben en la API (`:3001`); si no responde,
//! muestran datos de demostración locales.

mod cliente;

use dioxus::prelude::*;
use uuid::Uuid;
use chrono::Datelike;
use occb_core::{
    Alumno, Apoderado, Colegio, Correo, CrearAlumno, CrearApoderado, CrearCorreo, CrearEvento,
    CrearNota, CrearOportunidad, CrearPersona, CrearTarea, EstadoAlumno, EstadoTarea,
    EtapaOportunidad, Evento, Idioma, Nota, Oportunidad, Persona, Tarea, Traslado,
    calcular_resumen, datos_semilla_alumnos, datos_semilla_apoderados,
    datos_semilla_colegios, datos_semilla_notas, datos_semilla_oportunidades,
    datos_semilla_personas, datos_semilla_tareas, textos,
};

/// Hoja de estilos del frontal.
const CSS: Asset = asset!("/assets/main.css");

/// Idioma global de la interfaz (español por defecto, conmutable a inglés).
static IDIOMA: GlobalSignal<Idioma> = Signal::global(|| Idioma::Es);

/// Destino pendiente tras entrar (el guardia lo fija antes de mandar a acceso).
static VOLVER: GlobalSignal<Option<Ruta>> = Signal::global(|| None);
/// Diálogo de búsqueda global (⌘K / Ctrl+K).
static BUSQUEDA_ABIERTA: GlobalSignal<bool> = Signal::global(|| false);

fn main() {
    dioxus::launch(Aplicacion);
}

/// Componente raíz: estilos + enrutador.
#[component]
fn Aplicacion() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: "anonymous" }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap",
        }
        Router::<Ruta> {}
    }
}

/// Rutas de la aplicación (nombres en español).
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Ruta {
    #[route("/acceso")]
    VistaAcceso {},
    #[layout(DisposicionApp)]
        #[route("/")]
        Panel {},
        #[route("/alumnos")]
        VistaAlumnos {},
        #[route("/alumnos/:id")]
        DetalleAlumno { id: Uuid },
        #[route("/apoderados")]
        VistaApoderados {},
        #[route("/apoderados/:id")]
        DetalleApoderado { id: Uuid },
        #[route("/tareas")]
        VistaTareas {},
        #[route("/notas")]
        VistaNotas {},
        #[route("/personas")]
        VistaPersonas {},
        #[route("/personas/:id")]
        DetallePersona { id: Uuid },
        #[route("/colegios/:rbd")]
        DetalleColegio { rbd: String },
        #[route("/oportunidades")]
        VistaOportunidades {},
        #[route("/oportunidades/:id")]
        DetalleOportunidad { id: Uuid },
        #[route("/miembros")]
        VistaMiembros {},
        #[route("/calendario")]
        VistaCalendario {},
        #[route("/correos")]
        VistaCorreos {},
        #[route("/automatizaciones")]
        VistaAutomatizaciones {},
        #[route("/agentes")]
        VistaAgentes {},
        #[route("/ajustes")]
        VistaAjustes {},
    #[route("/:..segmentos")]
    NoEncontrada { segmentos: Vec<String> },
}

/// Fecha relativa estilo Twenty ("hace 39 minutos" / "39 minutes ago").
fn hace_tiempo(fecha: chrono::DateTime<chrono::Utc>, idioma: Idioma) -> String {
    let mins = (chrono::Utc::now() - fecha).num_minutes().max(0);
    if idioma == Idioma::Es {
        if mins < 1 {
            "ahora mismo".to_string()
        } else if mins < 60 {
            format!("hace {mins} minutos")
        } else if mins < 60 * 24 {
            format!("hace {} horas", mins / 60)
        } else {
            format!("hace {} días", mins / 60 / 24)
        }
    } else if mins < 1 {
        "just now".to_string()
    } else if mins < 60 {
        format!("{mins} minutes ago")
    } else if mins < 60 * 24 {
        format!("{} hours ago", mins / 60)
    } else {
        format!("{} days ago", mins / 60 / 24)
    }
}

/// Etiqueta de etapa según el idioma (la BD guarda siempre el texto español).
fn etiqueta_etapa(etapa: EtapaOportunidad, idioma: Idioma) -> &'static str {
    match (etapa, idioma) {
        (EtapaOportunidad::Contacto, Idioma::Es) => "Contacto",
        (EtapaOportunidad::Postulante, Idioma::Es) => "Postulante",
        (EtapaOportunidad::Evaluado, Idioma::Es) => "Evaluado",
        (EtapaOportunidad::Aceptado, Idioma::Es) => "Aceptado",
        (EtapaOportunidad::Matriculado, Idioma::Es) => "Matriculado",
        (EtapaOportunidad::Contacto, Idioma::En) => "Contact",
        (EtapaOportunidad::Postulante, Idioma::En) => "Applicant",
        (EtapaOportunidad::Evaluado, Idioma::En) => "Evaluated",
        (EtapaOportunidad::Aceptado, Idioma::En) => "Accepted",
        (EtapaOportunidad::Matriculado, Idioma::En) => "Enrolled",
    }
}

/// Disposición con barra lateral.
/// El panel es público; el resto exige sesión (sin ella, vuelve a acceso
/// recordando el destino para volver tras entrar).
#[component]
fn DisposicionApp() -> Element {
    let nav = use_navigator();
    let ruta_actual: Ruta = use_route();
    if cliente::SESION().is_none() && !matches!(ruta_actual, Ruta::Panel {}) {
        *VOLVER.write() = Some(ruta_actual);
        nav.replace(Ruta::VistaAcceso {});
        return rsx! {
            div { class: "login-wrap",
                p { class: "muted", "…" }
            }
        };
    }
    rsx! {
        div {
            class: "shell",
            tabindex: "-1",
            onkeydown: move |e| {
                let k = e.key().to_string().to_lowercase() == "k";
                if k && (e.modifiers().meta() || e.modifiers().ctrl()) {
                    e.prevent_default();
                    *BUSQUEDA_ABIERTA.write() = true;
                } else if e.key().to_string() == "Escape" {
                    *BUSQUEDA_ABIERTA.write() = false;
                }
            },
            BarraLateral {}
            div { class: "main",
                Outlet::<Ruta> {}
            }
        }
    }
}

/// Convierte un resultado global a su pantalla más útil.
fn ruta_busqueda(tipo: &str, id: &str) -> Ruta {
    match tipo {
        "alumno" => Uuid::parse_str(id).map(|id| Ruta::DetalleAlumno { id }).unwrap_or(Ruta::VistaAlumnos {}),
        "apoderado" => Uuid::parse_str(id).map(|id| Ruta::DetalleApoderado { id }).unwrap_or(Ruta::VistaApoderados {}),
        "persona" => Uuid::parse_str(id).map(|id| Ruta::DetallePersona { id }).unwrap_or(Ruta::VistaPersonas {}),
        "postulacion" => Uuid::parse_str(id).map(|id| Ruta::DetalleOportunidad { id }).unwrap_or(Ruta::VistaOportunidades {}),
        "tarea" => Ruta::VistaTareas {},
        "nota" => Ruta::VistaNotas {},
        "evento" => Ruta::VistaCalendario {},
        "correo" => Ruta::VistaCorreos {},
        "colegio" => Ruta::DetalleColegio { rbd: id.to_string() },
        _ => Ruta::Panel {},
    }
}

fn etiqueta_tipo_busqueda(tipo: &str, idioma: Idioma) -> &'static str {
    match (tipo, idioma) {
        ("alumno", Idioma::Es) => "Alumnos",
        ("alumno", Idioma::En) => "Students",
        ("apoderado", Idioma::Es) => "Apoderados",
        ("apoderado", Idioma::En) => "Guardians",
        ("persona", Idioma::Es) => "Personas",
        ("persona", Idioma::En) => "People",
        ("postulacion", Idioma::Es) => "Admisión",
        ("postulacion", Idioma::En) => "Admissions",
        ("tarea", Idioma::Es) => "Tareas",
        ("tarea", Idioma::En) => "Tasks",
        ("nota", Idioma::Es) => "Notas",
        ("nota", Idioma::En) => "Notes",
        ("evento", Idioma::Es) => "Eventos",
        ("evento", Idioma::En) => "Events",
        ("correo", Idioma::Es) => "Correos",
        ("correo", Idioma::En) => "Mail",
        ("colegio", Idioma::Es) => "Colegios",
        ("colegio", Idioma::En) => "Schools",
        _ => "",
    }
}

fn ruta_favorito(tipo: &str, id: Uuid) -> Ruta {
    match tipo {
        "alumno" => Ruta::DetalleAlumno { id },
        "apoderado" => Ruta::DetalleApoderado { id },
        "persona" => Ruta::DetallePersona { id },
        _ => Ruta::VistaOportunidades {},
    }
}

/// Estrella personal persistida para el registro dado.
#[component]
fn EstrellaFavorito(tipo: String, id: Uuid) -> Element {
    let marcado = cliente::FAVORITOS()
        .iter()
        .any(|f| f.tipo == tipo && f.id == id);
    rsx! {
        button {
            class: if marcado { "star-favorite marked" } else { "star-favorite" },
            title: if marcado { "Quitar de favoritos" } else { "Agregar a favoritos" },
            onclick: move |e| {
                e.stop_propagation();
                let accion = tipo.clone();
                spawn(async move {
                    let _ = cliente::alternar_favorito(&accion, id).await;
                });
            },
            if marcado { "★" } else { "☆" }
        }
    }
}

/// Barra lateral con navegación y selector de idioma ES/EN.
#[component]
fn BarraLateral() -> Element {
    let t = textos(IDIOMA());
    let nav = use_navigator();
    let sesion = cliente::SESION();
    let mut busqueda = use_signal(String::new);
    let mut resultados = use_signal(Vec::<occb_core::ResultadoBusqueda>::new);
    let mut cargando_busqueda = use_signal(|| false);
    use_effect(move || {
        if cliente::SESION().is_some() {
            spawn(async move {
                cliente::refrescar_no_leidos().await;
                cliente::refrescar_favoritos().await;
            });
        }
    });
    use_effect(move || {
        let q = busqueda().trim().to_string();
        if q.is_empty() {
            resultados.set(Vec::new());
            return;
        }
        cargando_busqueda.set(true);
        spawn(async move {
            match cliente::buscar_espacio(&q).await {
                Ok(lista) if busqueda().trim() == q => resultados.set(lista),
                Err(_) if busqueda().trim() == q => resultados.set(Vec::new()),
                _ => {}
            }
            if busqueda().trim() == q {
                cargando_busqueda.set(false);
            }
        });
    });
    let salir = move |_| {
        spawn(async move {
            let _ = cliente::cerrar_sesion().await;
            nav.push(Ruta::VistaAcceso {});
        });
    };
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
            button { class: "search", onclick: move |_| *BUSQUEDA_ABIERTA.write() = true,
                span { class: "search-icon", "⌕" }
                span { "{t.buscar_global}" }
                span { style: "margin-left: auto; display: flex; gap: 2px;",
                    kbd { "⌘" }
                    kbd { "K" }
                }
            }
            div { class: "side-section",
                div { class: "side-title", "{t.favoritos}" }
                if cliente::FAVORITOS().is_empty() {
                    div { class: "side-item dim", "{t.favoritos_vacios}" }
                }
                for f in cliente::FAVORITOS().iter().take(5) {
                    Link { to: ruta_favorito(&f.tipo, f.id), class: "side-item", key: "{f.tipo}-{f.id}",
                        span { "★" }
                        span { "{f.titulo}" }
                    }
                }
            }
            div { class: "side-section",
                div { class: "side-title", "{t.espacio_trabajo}" }
                Link { to: Ruta::Panel {}, class: "side-item",
                    span { class: "ico ico-panel", "▦" }
                    span { "{t.panel}" }
                    span { class: "count", "⌘" }
                }
                Link { to: Ruta::VistaAlumnos {}, class: "side-item",
                    span { class: "ico ico-personas", "🎒" }
                    span { "{t.alumnos}" }
                }
                Link { to: Ruta::VistaApoderados {}, class: "side-item",
                    span { class: "ico ico-personas", "🧑‍🤝‍🧑" }
                    span { "{t.apoderados}" }
                }
                Link { to: Ruta::VistaOportunidades {}, class: "side-item active",
                    span { class: "ico ico-admision", "◇" }
                    span { "{t.oportunidades}" }
                }
                Link { to: Ruta::VistaTareas {}, class: "side-item",
                    span { class: "ico ico-tareas", "✓" }
                    span { "{t.tareas}" }
                }
                Link { to: Ruta::VistaNotas {}, class: "side-item",
                    span { class: "ico ico-notas", "☰" }
                    span { "{t.notas}" }
                }
                Link { to: Ruta::VistaMiembros {}, class: "side-item",
                    span { class: "ico ico-personas", "👥" }
                    span { "{t.miembros}" }
                }
                Link { to: Ruta::VistaCalendario {}, class: "side-item",
                    span { class: "ico ico-notas", "📅" }
                    span { "{t.calendario}" }
                }
                Link { to: Ruta::VistaCorreos {}, class: "side-item",
                    span { class: "ico ico-notas", "✉" }
                    span { "{t.correos}" }
                    if cliente::NO_LEIDOS() > 0 {
                        span { class: "count", "{cliente::NO_LEIDOS()}" }
                    }
                }
                Link { to: Ruta::VistaAutomatizaciones {}, class: "side-item",
                    span { class: "ico", "⚙" }
                    span { "{t.automatizaciones}" }
                }
                Link { to: Ruta::VistaAgentes {}, class: "side-item",
                    span { class: "ico", "✦" }
                    span { "{t.agentes}" }
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
                if let Some(s) = sesion {
                    div { class: "side-item",
                        span { class: "avatar", "{s.nombre.chars().next().unwrap_or('·')}" }
                        span { "{s.nombre} · {s.rol}" }
                    }
                    button { class: "side-item dim", onclick: salir,
                        span { class: "ico", "→" }
                        span { "{t.salir}" }
                    }
                } else {
                    Link { to: Ruta::VistaAcceso {}, class: "side-item dim",
                        span { class: "ico", "→" }
                        span { "{t.salir}" }
                    }
                }
            }
            div { class: "invite",
                div { class: "invite-title", "{t.invitar} · OCCB" }
                div { class: "invite-sub", "{t.invitar_subtitulo}" }
                button { class: "btn-primary", "{t.invitar}" }
            }
            if BUSQUEDA_ABIERTA() {
                div { class: "search-overlay",
                    div { class: "search-dialog",
                        div { class: "cal-bar",
                            input {
                                class: "input",
                                autofocus: true,
                                placeholder: "{t.buscar_global}",
                                value: "{busqueda}",
                                oninput: move |e| busqueda.set(e.value()),
                                onkeydown: move |e| {
                                    if e.key().to_string() == "Escape" {
                                        *BUSQUEDA_ABIERTA.write() = false;
                                    } else if e.key().to_string() == "Enter"
                                        && let Some(r) = resultados.read().first()
                                    {
                                        nav.push(ruta_busqueda(&r.tipo, &r.id));
                                        *BUSQUEDA_ABIERTA.write() = false;
                                    }
                                },
                            }
                            button { class: "btn-mini", onclick: move |_| *BUSQUEDA_ABIERTA.write() = false, "Esc" }
                        }
                        if cargando_busqueda() {
                            p { class: "muted", "…" }
                        }
                        if !busqueda.read().is_empty() && resultados.read().is_empty() && !cargando_busqueda() {
                            p { class: "muted", if IDIOMA() == Idioma::Es { "Sin resultados" } else { "No results" } }
                        }
                        for tipo in ["alumno", "apoderado", "persona", "postulacion", "tarea", "nota", "evento", "correo", "colegio"] {
                            {
                                let grupo: Vec<_> = resultados.read().iter().filter(|r| r.tipo == tipo).cloned().collect();
                                rsx! {
                                    if !grupo.is_empty() {
                                        div { class: "search-group", key: "{tipo}",
                                            div { class: "search-group-title", "{etiqueta_tipo_busqueda(tipo, IDIOMA())}" }
                                            for r in grupo.iter() {
                                                Link {
                                                    to: ruta_busqueda(&r.tipo, &r.id),
                                                    class: "search-result",
                                                    key: "{r.id}",
                                                    onclick: move |_| *BUSQUEDA_ABIERTA.write() = false,
                                                    span { class: "mini-name", "{r.titulo}" }
                                                    span { class: "mini-amt", "{r.detalle}" }
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
}

/// Cabecera de cada vista: título y subtítulo (sin acciones decorativas).
#[component]
fn BarraSuperior(titulo: String, subtitulo: String) -> Element {
    rsx! {
        div { class: "topbar",
            div {
                h1 { class: "page-title", "{titulo}" }
                p { class: "page-sub", "{subtitulo}" }
            }
        }
    }
}

/// Selector Región → Comuna con datos oficiales chilenos (16 + 346).
/// Al cambiar de región se limpia la comuna; las comunas se filtran solas.
#[component]
fn SelectorTerritorio(region_sel: Signal<String>, comuna_sel: Signal<String>) -> Element {
    let idioma = IDIOMA();
    let codigo: u8 = region_sel.read().parse().unwrap_or(0);
    let comunas = occb_core::territorio::comunas_de_region(codigo);
    rsx! {
        div { class: "campo",
            label { if idioma == Idioma::Es { "Región" } else { "Region" } }
            select {
                class: "input",
                value: "{region_sel}",
                onchange: move |e| {
                    region_sel.set(e.value());
                    comuna_sel.set(String::new());
                },
                option { value: "", "—" }
                for r in occb_core::territorio::REGIONES.iter() {
                    option { value: "{r.codigo}", "{r.nombre}" }
                }
            }
        }
        div { class: "campo",
            label { if idioma == Idioma::Es { "Comuna" } else { "Commune" } }
            select {
                class: "input",
                value: "{comuna_sel}",
                onchange: move |e| comuna_sel.set(e.value()),
                option { value: "", "—" }
                for c in comunas.iter() {
                    option { value: "{c.cut}", "{c.nombre}" }
                }
            }
        }
    }
}

/// Código de región desde el selector (1–16) o `None`.
fn region_codigo_de(sel: &Signal<String>) -> Option<i16> {
    sel.read().parse::<i16>().ok().filter(|r| (1..=16).contains(r))
}

/// CUT de comuna desde el selector o `None` si está vacío.
fn comuna_codigo_de(sel: &Signal<String>) -> Option<String> {
    let texto = sel.read().clone();
    if texto.trim().is_empty() {
        None
    } else {
        Some(texto)
    }
}

/// Ciudad para guardar: el nombre oficial de la comuna elegida (o vacío).
fn ciudad_de_comuna(sel: &Signal<String>) -> String {
    let cut = sel.read().clone();
    if cut.trim().is_empty() {
        String::new()
    } else {
        occb_core::territorio::comuna_por_cut(cut.trim())
            .map(|c| c.nombre.to_string())
            .unwrap_or_default()
    }
}

/// Colegios de demostración (el directorio oficial vive en PG con 12.038).
fn cargar_colegios_demo() -> Vec<Colegio> {
    datos_semilla_colegios()
}

/// Personas de demostración.
fn cargar_personas_demo() -> Vec<Persona> {
    datos_semilla_personas()
}

/// Recarga personas desde la API; usa demo solo si aún no hay nada.
fn recargar_personas(mut personas: Signal<Vec<Persona>>) {
    spawn(async move {
        match cliente::listar_personas().await {
            Ok(lista) => personas.set(lista),
            Err(_) if personas.read().is_empty() => personas.set(cargar_personas_demo()),
            Err(_) => {}
        }
    });
}

/// Recarga oportunidades desde la API; usa demo solo si aún no hay nada.
fn recargar_oportunidades(mut oportunidades: Signal<Vec<Oportunidad>>) {
    spawn(async move {
        match cliente::listar_oportunidades().await {
            Ok(lista) => oportunidades.set(lista),
            Err(_) if oportunidades.read().is_empty() => {
                let demo_p = cargar_personas_demo();
                let ids: Vec<_> = demo_p.iter().map(|p| p.id).collect();
                oportunidades.set(datos_semilla_oportunidades(&ids));
            }
            Err(_) => {}
        }
    });
}

/// Vista del panel principal con tarjetas y resumen (datos vivos de la API).
#[component]
fn Panel() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut colegios = use_signal(Vec::<Colegio>::new);
    let mut personas = use_signal(Vec::<Persona>::new);
    let mut oportunidades = use_signal(Vec::<Oportunidad>::new);
    let mut en_linea = use_signal(|| false);
    let mut total_colegios = use_signal(|| 0i64);

    use_effect(move || {
        spawn(async move {
            match cliente::listar_colegios(None, "", "").await {
                Ok(lista_c) => {
                    en_linea.set(true);
                    let per =
                        cliente::listar_personas().await.unwrap_or_default();
                    let opp =
                        cliente::listar_oportunidades().await.unwrap_or_default();
                    colegios.set(lista_c);
                    personas.set(per);
                    oportunidades.set(opp);
                    if let Ok(panel) = cliente::obtener_panel().await {
                        total_colegios.set(panel.total_colegios);
                    }
                }
                Err(_) => {
                    let demo_c = cargar_colegios_demo();
                    total_colegios.set(demo_c.len() as i64);
                    let demo_p = cargar_personas_demo();
                    let ids_p: Vec<_> = demo_p.iter().map(|p| p.id).collect();
                    let demo_o = datos_semilla_oportunidades(&ids_p);
                    colegios.set(demo_c);
                    personas.set(demo_p);
                    oportunidades.set(demo_o);
                }
            }
        });
    });

    let resumen = calcular_resumen(&colegios.read(), &personas.read(), &oportunidades.read());
    let total = (resumen.valor_tuberia + resumen.valor_ganado).max(1) as f64;
    let porcentaje_abierto = (resumen.valor_tuberia as f64 / total * 100.0) as u32;
    let estado_api = if en_linea() { "● API :3001" } else { "○ demo local" };

    rsx! {
        BarraSuperior {
            titulo: t.panel.to_string(),
            subtitulo: format!("{} · {estado_api}", t.bienvenida_panel),
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
                div { class: "card-value", "{total_colegios}" }
                div { class: "card-sub",
                    "{resumen.total_personas} · {t.personas.to_lowercase()}"
                }
            }
            div { class: "card",
                div { class: "card-label", "{t.motor_api}" }
                div { class: "card-value small", "{estado_api}" }
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
            }
            table { class: "tbl",
                thead {
                    tr { th { "{t.colegios}" } th { "RBD" } th { "Comuna" } th { "Dep." } }
                }
                tbody {
                    for c in colegios.read().iter().take(5) {
                        tr { key: "{c.rbd}",
                            td { class: "strong",
                                Link { to: Ruta::DetalleColegio { rbd: c.rbd.clone() }, class: "link",
                                    "{c.nombre}"
                                }
                            }
                            td { class: "muted", "{c.rbd}" }
                            td { "{c.comuna}" }
                            td { "{c.dependencia}" }
                        }
                    }
                }
            }
        }
    }
}

/// Vista de personas: lista viva con búsqueda, alta y baja.
#[component]
fn VistaPersonas() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut busqueda = use_signal(String::new);
    let personas = use_signal(Vec::<Persona>::new);
    let mut listo = use_signal(|| false);
    let mut aviso = use_signal(String::new);
    let mut mostrar_form = use_signal(|| false);
    let mut nombre = use_signal(String::new);
    let mut apellido = use_signal(String::new);
    let mut correo = use_signal(String::new);
    let mut telefono = use_signal(String::new);
    let mut puesto = use_signal(String::new);
    let mut region_sel = use_signal(String::new);
    let mut comuna_sel = use_signal(String::new);

    use_effect(move || {
        recargar_personas(personas);
        listo.set(true);
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

    let crear = move |_| {
        let datos = CrearPersona {
            nombre: nombre.read().clone(),
            apellido: apellido.read().clone(),
            correo: correo.read().clone(),
            telefono: telefono.read().clone(),
            puesto: puesto.read().clone(),
            ciudad: ciudad_de_comuna(&comuna_sel),
            region_codigo: region_codigo_de(&region_sel),
            comuna_codigo: comuna_codigo_de(&comuna_sel),
        };
        spawn(async move {
            match cliente::crear_persona(&datos).await {
                Ok(_) => {
                    aviso.set(String::new());
                    nombre.set(String::new());
                    apellido.set(String::new());
                    correo.set(String::new());
                    telefono.set(String::new());
                    puesto.set(String::new());
                    region_sel.set(String::new());
                    comuna_sel.set(String::new());
                    recargar_personas(personas);
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
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
        }
        SelectorVistas {
            objeto: "personas".to_string(),
            modo: "tabla".to_string(),
            filtros: serde_json::json!({ "busqueda": busqueda() }),
            on_cargar: move |v: occb_core::VistaGuardada| {
                if let Some(q) = v.filtros.get("busqueda").and_then(|x| x.as_str()) {
                    busqueda.set(q.to_string());
                }
            },
        }
        if mostrar_form() {
            div { class: "panel form",
                label { if idioma == Idioma::Es { "Nueva persona" } else { "New person" } }
            div { class: "row",
                input {
                    class: "input",
                    placeholder: if idioma == Idioma::Es { "Nombre" } else { "First name" },
                    value: "{nombre}",
                    oninput: move |e| nombre.set(e.value()),
                }
                input {
                    class: "input",
                    placeholder: if idioma == Idioma::Es { "Apellidos" } else { "Last name" },
                    value: "{apellido}",
                    oninput: move |e| apellido.set(e.value()),
                }
                input {
                    class: "input",
                    placeholder: "{t.correo}",
                    value: "{correo}",
                    oninput: move |e| correo.set(e.value()),
                }
                input {
                    class: "input",
                    placeholder: if idioma == Idioma::Es { "Teléfono" } else { "Phone" },
                    value: "{telefono}",
                    oninput: move |e| telefono.set(e.value()),
                }
                input {
                    class: "input",
                    placeholder: if idioma == Idioma::Es { "Puesto" } else { "Role" },
                    value: "{puesto}",
                    oninput: move |e| puesto.set(e.value()),
                }
                SelectorTerritorio {
                    region_sel: region_sel,
                    comuna_sel: comuna_sel,
                }
                button { class: "btn-primary", onclick: crear, "{t.nueva}" }
            }
            if !aviso.read().is_empty() {
                p { class: "muted small", "{aviso}" }
            }
        }
        }
        div { class: "tbl-wrap",
            if !listo() {
                p { class: "muted", style: "padding: 12px;", "…" }
            } else {
                table { class: "tbl",
                    thead {
                        tr {
                            th { class: "check", "☐" }
                            th { "👤 ", if idioma == Idioma::Es { "Nombre" } else { "Name" } }
                            th { "✉️ ", "{t.correo}" }
                            th { "💼 ", if idioma == Idioma::Es { "Puesto" } else { "Role" } }
                            th { "📍 ", if idioma == Idioma::Es { "Ciudad" } else { "City" } }
                            th { "" }
                        }
                    }
                    tbody {
                        for p in filas.iter() {
                            tr { key: "{p.id}",
                                td { class: "check", "☐" }
                                td {
                                    span { class: "chip",
                                        span { class: "mini-avatar", "{p.iniciales()}" }
                                        Link { to: Ruta::DetallePersona { id: p.id }, class: "link",
                                            "{p.nombre_completo()}"
                                        }
        EstrellaFavorito { tipo: "persona".to_string(), id: p.id }
                                    }
                                }
                                td { span { class: "chip plain", "{p.correo}" } }
                                td { "{p.puesto}" }
                                td { "{p.ciudad}" }
                                td {
                                    button {
                                        class: "btn-mini",
                                        title: "✕",
                                        onclick: {
                                            let id = p.id;
                                            move |_| {
                                                spawn(async move {
                                                    if cliente::eliminar_persona(id).await.is_ok() {
                                                        recargar_personas(personas);
                                                    }
                                                });
                                            }
                                        },
                                        "✕"
                                    }
                                }
                            }
                        }
                    }
                    tfoot {
                        tr {
                            td { colspan: "6",
                                button {
                                    class: "ghost-new",
                                    onclick: move |_| mostrar_form.set(true),
                                    "+ Agregar nuevo"
                                }
                            }
                        }
                        tr { class: "tbl-foot",
                            td { colspan: "2", "Calcular ▾" }
                            td { colspan: "2", "Contar todo ", b { "{total}" } }
                            td { colspan: "2", class: "stat",
                                if idioma == Idioma::Es {
                                    "Creados ", b { "{hace_tiempo(filas.first().map(|x| x.creado_en).unwrap_or(chrono::Utc::now()), idioma)}" }
                                } else {
                                    "Created ", b { "{hace_tiempo(filas.first().map(|x| x.creado_en).unwrap_or(chrono::Utc::now()), idioma)}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Vista kanban de oportunidades: datos vivos y ▶ que persiste en la API.
#[component]
fn VistaOportunidades() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let oportunidades = use_signal(Vec::<Oportunidad>::new);
    let mut colegios = use_signal(Vec::<Colegio>::new);
    let mut personas = use_signal(Vec::<Persona>::new);
    let mut aviso = use_signal(String::new);
    let mut nombre = use_signal(String::new);
    let mut importe_txt = use_signal(String::new);
    let mut colegio_sel = use_signal(String::new);
    let mut editando = use_signal(|| None::<Uuid>);
    let mut nombre_ed = use_signal(String::new);
    let mut importe_ed = use_signal(String::new);
    let mut mostrar_form = use_signal(|| false);
    let mut vista_tabla = use_signal(|| false);
    let mut etapa_nueva = use_signal(|| EtapaOportunidad::Contacto);

    use_effect(move || {
        recargar_oportunidades(oportunidades);
        spawn(async move {
            if let Ok(lista) = cliente::listar_colegios(None, "", "").await {
                colegios.set(lista);
            }
            if let Ok(lista) = cliente::listar_personas().await {
                personas.set(lista);
            }
        });
    });

    let total: i64 = oportunidades.read().iter().map(|o| o.importe).sum();

    let crear = move |_| {
        let colegio_rbd = if colegio_sel.read().trim().is_empty() {
            None
        } else {
            Some(colegio_sel.read().trim().to_string())
        };
        let datos = CrearOportunidad {
            nombre: nombre.read().clone(),
            importe: importe_txt.read().parse().unwrap_or(0),
            etapa: etapa_nueva(),
            colegio_rbd,
        };
        spawn(async move {
            match cliente::crear_oportunidad(&datos).await {
                Ok(_) => {
                    aviso.set(String::new());
                    nombre.set(String::new());
                    importe_txt.set(String::new());
                    colegio_sel.set(String::new());
                    etapa_nueva.set(EtapaOportunidad::Contacto);
                    recargar_oportunidades(oportunidades);
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    rsx! {
        BarraSuperior {
            titulo: t.oportunidades.to_string(),
            subtitulo: format!(
                "${total} · {}",
                if idioma == Idioma::Es { "pulsa ▶ para avanzar de etapa" } else { "click ▶ to advance stage" }
            ),
        }
        div { class: "tabs",
            button {
                class: if vista_tabla() { "tab active" } else { "tab" },
                onclick: move |_| vista_tabla.set(true),
                "{t.tabla}"
            }
            button {
                class: if !vista_tabla() { "tab active" } else { "tab" },
                onclick: move |_| vista_tabla.set(false),
                "{t.kanban}"
            }
        }
        SelectorVistas {
            objeto: "oportunidades".to_string(),
            modo: if vista_tabla() { "tabla".to_string() } else { "kanban".to_string() },
            filtros: serde_json::json!({}),
            on_cargar: move |v: occb_core::VistaGuardada| vista_tabla.set(v.tipo == "tabla"),
        }
        if mostrar_form() {
        div { class: "panel form",
            label {
                if idioma == Idioma::Es { "Nueva oportunidad · " } else { "New opportunity · " }
                "{etiqueta_etapa(etapa_nueva(), idioma)}"
            }
            div { class: "row",
                input {
                    class: "input",
                    placeholder: if idioma == Idioma::Es { "Nombre *" } else { "Name *" },
                    value: "{nombre}",
                    oninput: move |e| nombre.set(e.value()),
                }
                input {
                    class: "input",
                    placeholder: "$",
                    value: "{importe_txt}",
                    oninput: move |e| importe_txt.set(e.value()),
                }
                select {
                    class: "input",
                    value: "{colegio_sel}",
                    onchange: move |e| colegio_sel.set(e.value()),
                    option { value: "", "—" }
                    for c in colegios.read().iter().take(100) {
                        option { value: "{c.rbd}", "{c.nombre}" }
                    }
                }
                button { class: "btn-primary", onclick: crear, "{t.nueva}" }
            }
            if !aviso.read().is_empty() {
                p { class: "muted small", "{aviso}" }
            }
        }
        }
        if vista_tabla() {
            div { class: "tbl-wrap",
                table { class: "tbl",
                    thead {
                        tr {
                            th { class: "check", "☐" }
                            th { if idioma == Idioma::Es { "Nombre" } else { "Name" } }
                            th { if idioma == Idioma::Es { "Importe" } else { "Amount" } }
                            th { if idioma == Idioma::Es { "Etapa" } else { "Stage" } }
                            th { "RBD" }
                        }
                    }
                    tbody {
                        for o in oportunidades.read().iter() {
                            tr { key: "{o.id}",
                                td { class: "check", "☐" }
                                td { class: "strong",
                                    Link { to: Ruta::DetalleOportunidad { id: o.id }, class: "link", "{o.nombre}" }
                                    EstrellaFavorito { tipo: "postulacion".to_string(), id: o.id }
                                }
                                td { "${o.importe}" }
                                td { span { class: "stage stage-{o.etapa:?}", "{etiqueta_etapa(o.etapa, idioma)}" } }
                                td { class: "muted", "{o.colegio_rbd.clone().unwrap_or_default()}" }
                            }
                        }
                    }
                    tfoot {
                        tr {
                            td { colspan: "5",
                                button {
                                    class: "ghost-new",
                                    onclick: move |_| mostrar_form.set(true),
                                    "+ Nuevo"
                                }
                            }
                        }
                    }
                }
            }
        } else {
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
                            if editando() == Some(op.id) {
                                input {
                                    class: "input",
                                    placeholder: if idioma == Idioma::Es { "Nombre *" } else { "Name *" },
                                    value: "{nombre_ed}",
                                    oninput: move |e| nombre_ed.set(e.value()),
                                }
                                input {
                                    class: "input",
                                    placeholder: "$",
                                    value: "{importe_ed}",
                                    oninput: move |e| importe_ed.set(e.value()),
                                }
                                div { class: "opp-foot",
                                    button {
                                        class: "btn-mini",
                                        onclick: {
                                            let id = op.id;
                                            let etapa_actual = op.etapa;
                                            let colegio_actual = op.colegio_rbd.clone();
                                            move |_| {
                                                let colegio_clon = colegio_actual.clone();
                                                let datos = CrearOportunidad {
                                                    nombre: nombre_ed.read().clone(),
                                                    importe: importe_ed.read().parse().unwrap_or(0),
                                                    etapa: etapa_actual,
                                                    colegio_rbd: colegio_clon,
                                                };
                                                spawn(async move {
                                                    match cliente::actualizar_oportunidad(id, &datos).await {
                                                        Ok(_) => {
                                                            editando.set(None);
                                                            recargar_oportunidades(oportunidades);
                                                        }
                                                        Err(mensaje) => aviso.set(mensaje),
                                                    }
                                                });
                                            }
                                        },
                                        if idioma == Idioma::Es { "Guardar" } else { "Save" }
                                    }
                                    button {
                                        class: "btn-mini",
                                        onclick: move |_| editando.set(None),
                                        "✕"
                                    }
                                }
                            } else {
                                div { class: "opp-name",
                                    span { class: "mini-avatar", "{op.nombre.chars().next().unwrap_or('?')}" }
                                    Link { to: Ruta::DetalleOportunidad { id: op.id }, class: "link", "{op.nombre}" }
                                    EstrellaFavorito { tipo: "postulacion".to_string(), id: op.id }
                                }
                                div { class: "opp-row", "💲 ", span { class: "opp-amt", "${op.importe}" } }
                                div { class: "opp-row muted",
                                    "👤 ", if idioma == Idioma::Es { "Sistema" } else { "System" }
                                }
                                div { class: "opp-row muted", "📅 {hace_tiempo(op.creado_en, idioma)}" }
                                if op.colegio_rbd.is_some() {
                                    div { class: "opp-row",
                                        "🏫 ",
                                        span { class: "chip plain",
                                            "{colegios.read().iter().find(|c| Some(c.rbd.as_str()) == op.colegio_rbd.as_deref()).map(|c| c.nombre.clone()).unwrap_or_default()}"
                                        }
                                    }
                                }
                                if let Some(pid) = op.contacto_id {
                                    match personas
                                        .read()
                                        .iter()
                                        .find(|p| p.id == pid)
                                        .map(|p| p.nombre_completo())
                                    {
                                        Some(nomb) => rsx! {
                                            div { class: "opp-row",
                                                "👤 ",
                                                span { class: "chip plain", "{nomb}" }
                                            }
                                        },
                                        None => rsx! {},
                                    }
                                }
                                div { class: "opp-foot",
                                    span { class: "opp-prob", "{etapa.probabilidad()}%" }
                                    button {
                                        class: "btn-mini",
                                        title: "✎",
                                        onclick: {
                                            let (id, nomb, imp) = (op.id, op.nombre.clone(), op.importe.to_string());
                                            move |_| {
                                                nombre_ed.set(nomb.clone());
                                                importe_ed.set(imp.clone());
                                                editando.set(Some(id));
                                            }
                                        },
                                        "✎"
                                    }
                                    button {
                                        class: "btn-mini",
                                        title: if idioma == Idioma::Es { "Avanzar etapa" } else { "Advance stage" },
                                        onclick: {
                                            let id = op.id;
                                            let siguiente = op.etapa.siguiente().como_texto();
                                            move |_| {
                                                spawn(async move {
                                                    match cliente::mover_etapa(id, siguiente).await {
                                                        Ok(_) => recargar_oportunidades(oportunidades),
                                                        Err(mensaje) => aviso.set(mensaje),
                                                    }
                                                });
                                            }
                                        },
                                        "▶"
                                    }
                                }
                            }
                        }
                    }
                    button {
                        class: "ghost-new",
                        onclick: {
                            let e = *etapa;
                            move |_| {
                                etapa_nueva.set(e);
                                mostrar_form.set(true);
                            }
                        },
                        "+ Nuevo"
                    }
                }
            }
        }
        }
    }
}

/// Ficha de postulación: etapa, colegio asociado y actividad del registro.
#[component]
fn DetalleOportunidad(id: Uuid) -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut oportunidad = use_signal(|| None::<Oportunidad>);
    let mut aviso = use_signal(String::new);
    let mut id_actual = use_signal(|| id);
    if id_actual() != id {
        id_actual.set(id);
    }
    use_effect(move || {
        let id = id_actual();
        spawn(async move {
            match cliente::listar_oportunidades().await {
                Ok(lista) => oportunidad.set(lista.into_iter().find(|o| o.id == id)),
                Err(m) => aviso.set(m),
            }
        });
    });
    let titulo = oportunidad.read().as_ref().map(|o| o.nombre.clone()).unwrap_or_default();
    rsx! {
        VistaOportunidades {}
        div { class: "record-panel",
            Link { to: Ruta::VistaOportunidades {}, class: "record-close", "✕ {t.oportunidades}" }
            h2 { class: "record-title", if titulo.is_empty() { "…" } else { "{titulo}" } }
            p { class: "page-sub", if idioma == Idioma::Es { "Ficha de postulación" } else { "Application record" } }
            if !aviso.read().is_empty() { p { class: "form-error", "{aviso}" } }
            if let Some(o) = oportunidad.read().as_ref() {
                div { class: "panel",
                    div { class: "panel-head",
                        span { class: "panel-title", if idioma == Idioma::Es { "General" } else { "General" } }
                        EstrellaFavorito { tipo: "postulacion".to_string(), id: o.id }
                    }
                    div { class: "mini-row",
                        span { class: "muted", "{t.oportunidades}" }
                        span { class: "stage stage-{o.etapa:?}", "{etiqueta_etapa(o.etapa, idioma)}" }
                    }
                    div { class: "mini-row",
                        span { class: "muted", if idioma == Idioma::Es { "Aporte" } else { "Amount" } }
                        span { class: "mini-name", "${o.importe}" }
                    }
                    div { class: "mini-row",
                        span { class: "muted", "RBD" }
                        span { class: "mini-name", "{o.colegio_rbd.clone().unwrap_or_default()}" }
                    }
                    div { class: "mini-row",
                        span { class: "muted", if idioma == Idioma::Es { "Creada" } else { "Created" } }
                        span { class: "mini-name", "{fmt_dia_hora(o.creado_en)}" }
                    }
                }
                Timeline { tipo: "postulacion".to_string(), id: o.id }
            }
        }
    }
}

/// Ficha de colegio: datos del directorio + postulaciones vinculadas.
#[component]
fn DetalleColegio(rbd: String) -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut colegio = use_signal(|| None::<Colegio>);
    let mut opps_rel = use_signal(Vec::<Oportunidad>::new);
    let mut aviso = use_signal(String::new);
    let mut listo = use_signal(|| false);

    // Sigue al rbd de la ruta: si cambia (navegación entre fichas), recarga.
    let mut rbd_actual = use_signal(|| rbd.clone());
    if rbd_actual() != rbd {
        rbd_actual.set(rbd.clone());
    }
    use_effect(move || {
        let rbd_clon = rbd_actual.read().clone();
        spawn(async move {
            match cliente::obtener_colegio(&rbd_clon).await {
                Ok(c) => {
                    colegio.set(Some(c));
                    if let Ok(o) = cliente::listar_oportunidades().await {
                        opps_rel.set(
                            o.into_iter()
                                .filter(|x| x.colegio_rbd.as_deref() == Some(rbd_clon.as_str()))
                                .collect(),
                        );
                    }
                    listo.set(true);
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    });

    let titulo = colegio.read().as_ref().map(|c| c.nombre.clone()).unwrap_or_default();    let nombre_region = colegio
        .read()
        .as_ref()
        .and_then(|c| occb_core::territorio::region(c.region as u8))
        .map(|r| r.nombre.to_string())
        .unwrap_or_default();
    let nombre_comuna = colegio
        .read()
        .as_ref()
        .and_then(|c| occb_core::territorio::comuna_por_cut(&c.comuna))
        .map(|c| c.nombre.to_string())
        .unwrap_or_default();
    rsx! {
        Panel {}
        div { class: "record-panel",
        Link { to: Ruta::Panel {}, class: "record-close", "✕ {t.panel}" }
        h2 { class: "record-title", {if titulo.is_empty() { "…".to_string() } else { titulo }} }
        p { class: "page-sub", {(if idioma == Idioma::Es { "Ficha de colegio" } else { "School record" }).to_string()} }
        if !aviso.read().is_empty() && !listo() {
            div { class: "panel",
                p { class: "muted", "{aviso}" }
            }
        } else {
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title", "RBD {rbd}" }
                    if let Some(c) = colegio.read().as_ref() {
                        span { class: "pill", "{c.dependencia}" }
                    }
                }
                p { class: "muted", "📍 {nombre_comuna} · {nombre_region}" }
            }
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title",
                        "{opps_rel.read().len()} · {t.oportunidades}"
                    }
                }
                for o in opps_rel.read().iter() {
                    div { class: "mini-row", key: "{o.id}",
                        span { class: "mini-name", "{o.nombre}" }
                        span { class: "mini-amt", "${o.importe}" }
                        span { class: "stage stage-{o.etapa:?}", "{etiqueta_etapa(o.etapa, idioma)}" }
                    }
                }
            }
        }
        }
    }
}

/// Ficha de persona: datos editables con territorio.
#[component]
fn DetallePersona(id: Uuid) -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut persona = use_signal(|| None::<Persona>);
    let mut opps_rel = use_signal(Vec::<Oportunidad>::new);
    let mut editando = use_signal(|| false);
    let mut aviso = use_signal(String::new);
    let mut listo = use_signal(|| false);
    let mut nombre = use_signal(String::new);
    let mut apellido = use_signal(String::new);
    let mut correo = use_signal(String::new);
    let mut telefono = use_signal(String::new);
    let mut puesto = use_signal(String::new);
    let mut ciudad = use_signal(String::new);
    let mut region_sel = use_signal(String::new);
    let mut comuna_sel = use_signal(String::new);
    // Sigue al id de la ruta: si cambia (navegación entre fichas), recarga.
    let mut id_actual = use_signal(|| id);
    if id_actual() != id {
        id_actual.set(id);
    }

    use_effect(move || {
        let pid = id_actual();
        spawn(async move {
            match cliente::obtener_persona(pid).await {
                Ok(p) => {
                    nombre.set(p.nombre.clone());
                    apellido.set(p.apellido.clone());
                    correo.set(p.correo.clone());
                    telefono.set(p.telefono.clone());
                    puesto.set(p.puesto.clone());
                    ciudad.set(p.ciudad.clone());
                    region_sel.set(p.region_codigo.map(|r| r.to_string()).unwrap_or_default());
                    comuna_sel.set(p.comuna_codigo.clone().unwrap_or_default());
                    persona.set(Some(p.clone()));
                    if let Ok(o) = cliente::listar_oportunidades().await {
                        opps_rel.set(o.into_iter().filter(|x| x.contacto_id == Some(pid)).collect());
                    }
                    listo.set(true);
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    });

    let guardar = move |_| {
        let pid = id_actual();
        let datos = CrearPersona {
            nombre: nombre.read().clone(),
            apellido: apellido.read().clone(),
            correo: correo.read().clone(),
            telefono: telefono.read().clone(),
            puesto: puesto.read().clone(),
            ciudad: ciudad_de_comuna(&comuna_sel),
            region_codigo: region_codigo_de(&region_sel),
            comuna_codigo: comuna_codigo_de(&comuna_sel),
        };
        spawn(async move {
            match cliente::actualizar_persona(pid, &datos).await {
                Ok(p) => {
                    aviso.set(String::new());
                    persona.set(Some(p));
                    editando.set(false);
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    let titulo = persona.read().as_ref().map(|p| p.nombre_completo()).unwrap_or_default();
    rsx! {
        VistaPersonas {}
        div { class: "record-panel",
        Link { to: Ruta::VistaPersonas {}, class: "record-close", "✕ {t.personas}" }
        h2 { class: "record-title", {if titulo.is_empty() { "…".to_string() } else { titulo.clone() }} EstrellaFavorito { tipo: "persona".to_string(), id } }
        p { class: "page-sub", {(if idioma == Idioma::Es { "Ficha de persona" } else { "Person record" }).to_string()} }
        if !aviso.read().is_empty() && !listo() {
            div { class: "panel",
                p { class: "muted", "{aviso}" }
            }
        } else {
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title", if idioma == Idioma::Es { "General" } else { "General" } }
                    button { class: "btn-mini", onclick: move |_| editando.set(!editando()), "✎" }
                }
                if let Some(p) = persona.read().as_ref() {
                    div { class: "mini-row", span { class: "muted", "✉️ {t.correo}" } span { class: "mini-name", "{p.correo}" } }
                    div { class: "mini-row", span { class: "muted", "📞" } span { class: "mini-name", "{p.telefono}" } }
                    div { class: "mini-row", span { class: "muted", if idioma == Idioma::Es { "💼 Puesto" } else { "💼 Role" } } span { class: "mini-name", "{p.puesto}" } }
                    div { class: "mini-row", span { class: "muted", "📍" } span { class: "mini-name", "{p.ciudad}" } }
                }
            }
            if editando() {
            div { class: "panel form",
                label { if idioma == Idioma::Es { "Editar (se guarda en la API)" } else { "Edit (saved to the API)" } }
                div { class: "row",
                    div { class: "campo",
                        label { if idioma == Idioma::Es { "Nombre *" } else { "First name *" } }
                        input { class: "input", placeholder: "Ada", value: "{nombre}", oninput: move |e| nombre.set(e.value()) }
                    }
                    div { class: "campo",
                        label { if idioma == Idioma::Es { "Apellidos *" } else { "Last name *" } }
                        input { class: "input", placeholder: "Lovelace", value: "{apellido}", oninput: move |e| apellido.set(e.value()) }
                    }
                    div { class: "campo",
                        label { "{t.correo} *" }
                        input { class: "input", placeholder: "ada@acme.com", value: "{correo}", oninput: move |e| correo.set(e.value()) }
                    }
                }
                div { class: "row",
                    div { class: "campo",
                        label { if idioma == Idioma::Es { "Teléfono" } else { "Phone" } }
                        input { class: "input", placeholder: "+34 555 0101", value: "{telefono}", oninput: move |e| telefono.set(e.value()) }
                    }
                    div { class: "campo",
                        label { if idioma == Idioma::Es { "Puesto" } else { "Role" } }
                        input { class: "input", placeholder: "Directora técnica", value: "{puesto}", oninput: move |e| puesto.set(e.value()) }
                    }
                    SelectorTerritorio {
                        region_sel: region_sel,
                        comuna_sel: comuna_sel,
                    }
                }
                div { class: "row",
                    div { class: "campo campo-btn",
                        button { class: "btn-primary", onclick: guardar, "{t.guardar}" }
                    }
                }
                if !aviso.read().is_empty() {
                    p { class: "muted small", "{aviso}" }
                }
            }
            }
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title",
                        "{opps_rel.read().len()} · {t.oportunidades}"
                    }
                    span { class: "link", "↗" }
                }
                for o in opps_rel.read().iter() {
                    div { class: "mini-row", key: "{o.id}",
                        span { class: "mini-name", "{o.nombre}" }
                        span { class: "mini-amt", "${o.importe}" }
                        span { class: "stage stage-{o.etapa:?}", "{etiqueta_etapa(o.etapa, idioma)}" }
                    }
                }
            }
            Timeline { tipo: "persona".to_string(), id }
        }
        }
    }
}

/// Reglas guiadas para el flujo de admisión; sin código arbitrario.
#[component]
fn VistaAutomatizaciones() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut reglas = use_signal(Vec::<occb_core::Automatizacion>::new);
    let mut aviso = use_signal(String::new);
    let mut nombre = use_signal(String::new);
    let mut disparador = use_signal(|| "al_crear".to_string());
    let mut con_condicion = use_signal(|| true);
    let mut campo = use_signal(|| "importe".to_string());
    let mut operador = use_signal(|| "mayor_que".to_string());
    let mut valor = use_signal(|| "100000".to_string());
    let mut accion = use_signal(|| "crear_tarea".to_string());
    let mut titulo_accion = use_signal(String::new);
    let mut cuerpo_accion = use_signal(String::new);
    let mut prioridad_accion = use_signal(|| "media".to_string());
    let mut etapa_accion = use_signal(|| "postulante".to_string());
    let mut regla_elegida = use_signal(|| None::<Uuid>);
    let mut ejecuciones = use_signal(Vec::<occb_core::Ejecucion>::new);

    let recargar = move || {
        spawn(async move {
            match cliente::listar_automatizaciones().await {
                Ok(lista) => reglas.set(lista),
                Err(m) => aviso.set(m),
            }
        });
    };
    use_effect(move || {
        spawn(async move {
            match cliente::listar_automatizaciones().await {
                Ok(lista) => reglas.set(lista),
                Err(m) => aviso.set(m),
            }
        });
    });

    let crear = move |_| {
        let nombre_clon = nombre.read().trim().to_string();
        let campo_clon = campo.read().clone();
        let operador_clon = operador.read().clone();
        let valor_clon = valor.read().trim().to_string();
        let condiciones = if con_condicion() && !valor_clon.is_empty() {
            serde_json::json!([{ "campo": campo_clon, "operador": operador_clon, "valor": valor_clon }])
        } else {
            serde_json::json!([])
        };
        let tipo_accion = accion.read().clone();
        let params = match tipo_accion.as_str() {
            "crear_nota" => serde_json::json!({ "titulo": titulo_accion.read().clone(), "cuerpo": cuerpo_accion.read().clone() }),
            "mover_etapa" => serde_json::json!({ "etapa": etapa_accion.read().clone() }),
            _ => serde_json::json!({ "titulo": titulo_accion.read().clone(), "detalle": cuerpo_accion.read().clone(), "prioridad": prioridad_accion.read().clone() }),
        };
        let acciones = serde_json::json!([{ "tipo": tipo_accion, "params": params }]);
        let disp = disparador.read().clone();
        spawn(async move {
            match cliente::crear_automatizacion(&nombre_clon, &disp, condiciones, acciones).await {
                Ok(_) => {
                    nombre.set(String::new());
                    titulo_accion.set(String::new());
                    cuerpo_accion.set(String::new());
                    aviso.set(String::new());
                    recargar();
                }
                Err(m) => aviso.set(m),
            }
        });
    };

    rsx! {
        BarraSuperior {
            titulo: t.automatizaciones.to_string(),
            subtitulo: if idioma == Idioma::Es { "Reglas para el proceso de admisión".to_string() } else { "Rules for the admissions process".to_string() },
        }
        if !aviso.read().is_empty() { p { class: "form-error", "{aviso}" } }
        div { class: "panel form automation-form",
            div { class: "panel-head", span { class: "panel-title", if idioma == Idioma::Es { "Nueva regla" } else { "New rule" } } }
            div { class: "row",
                div { class: "campo",
                    label { "{t.nombre}" }
                    input { class: "input", value: "{nombre}", oninput: move |e| nombre.set(e.value()) }
                }
                div { class: "campo",
                    label { "{t.disparador}" }
                    select { class: "input", value: "{disparador}", onchange: move |e| disparador.set(e.value()),
                        option { value: "al_crear", if idioma == Idioma::Es { "Al crear postulación" } else { "When application is created" } }
                        option { value: "al_actualizar", if idioma == Idioma::Es { "Al actualizar postulación" } else { "When application is updated" } }
                    }
                }
            }
            div { class: "automation-subform",
                label { class: "check-label",
                    input { r#type: "checkbox", checked: con_condicion(), onchange: move |e| con_condicion.set(e.checked()) }
                    "{t.condicion}"
                }
                if con_condicion() {
                    div { class: "row",
                        select { class: "input", value: "{campo}", onchange: move |e| campo.set(e.value()),
                            option { value: "importe", if idioma == Idioma::Es { "Importe" } else { "Amount" } }
                            option { value: "nombre", if idioma == Idioma::Es { "Nombre" } else { "Name" } }
                            option { value: "etapa", if idioma == Idioma::Es { "Etapa" } else { "Stage" } }
                            option { value: "colegio_rbd", "RBD" }
                        }
                        select { class: "input", value: "{operador}", onchange: move |e| operador.set(e.value()),
                            option { value: "igual", "{t.igual}" }
                            option { value: "contiene", "{t.contiene}" }
                            option { value: "mayor_que", "{t.mayor_que}" }
                        }
                        input { class: "input", placeholder: "{t.valor}", value: "{valor}", oninput: move |e| valor.set(e.value()) }
                    }
                }
            }
            div { class: "automation-subform",
                label { "{t.accion}" }
                select { class: "input", value: "{accion}", onchange: move |e| accion.set(e.value()),
                    option { value: "crear_tarea", "{t.crear_tarea}" }
                    option { value: "crear_nota", "{t.crear_nota}" }
                    option { value: "mover_etapa", "{t.mover_etapa}" }
                }
                if accion() == "crear_tarea" || accion() == "crear_nota" {
                    input { class: "input", placeholder: if accion() == "crear_tarea" { if idioma == Idioma::Es { "Título de la tarea" } else { "Task title" } } else { if idioma == Idioma::Es { "Título de la nota" } else { "Note title" } }, value: "{titulo_accion}", oninput: move |e| titulo_accion.set(e.value()) }
                    input { class: "input", placeholder: if accion() == "crear_tarea" { if idioma == Idioma::Es { "Detalle (opcional)" } else { "Details (optional)" } } else { if idioma == Idioma::Es { "Contenido de la nota" } else { "Note body" } }, value: "{cuerpo_accion}", oninput: move |e| cuerpo_accion.set(e.value()) }
                }
                if accion() == "crear_tarea" {
                    select { class: "input", value: "{prioridad_accion}", onchange: move |e| prioridad_accion.set(e.value()),
                        option { value: "baja", if idioma == Idioma::Es { "Prioridad baja" } else { "Low priority" } }
                        option { value: "media", if idioma == Idioma::Es { "Prioridad media" } else { "Medium priority" } }
                        option { value: "alta", if idioma == Idioma::Es { "Prioridad alta" } else { "High priority" } }
                    }
                }
                if accion() == "mover_etapa" {
                    select { class: "input", value: "{etapa_accion}", onchange: move |e| etapa_accion.set(e.value()),
                        for etapa in occb_core::EtapaOportunidad::todas() {
                            option { value: "{etapa.como_texto()}", "{etiqueta_etapa(*etapa, idioma)}" }
                        }
                    }
                }
            }
            button { class: "btn-primary", onclick: crear, "{t.guardar}" }
        }
        for regla in reglas.read().iter() {
            div { class: "panel automation-card", key: "{regla.id}",
                div { class: "panel-head",
                    span { class: "panel-title", "{regla.nombre}" }
                    span { class: if regla.activa { "pill" } else { "pill muted" }, if regla.activa { if idioma == Idioma::Es { "Activa" } else { "Active" } } else { if idioma == Idioma::Es { "Inactiva" } else { "Inactive" } } }
                }
                p { class: "muted", "{regla.disparador} · postulacion" }
                p { class: "automation-json", "{regla.condiciones}" }
                p { class: "automation-json", "{regla.acciones}" }
                div { class: "cal-bar",
                    button { class: "btn-mini",
                        onclick: {
                            let id = regla.id;
                            let activa = !regla.activa;
                            move |_| { spawn(async move {
                                match cliente::alternar_automatizacion(id, activa).await {
                                    Ok(_) => recargar(),
                                    Err(m) => aviso.set(m),
                                }
                            }); }
                        },
                        if regla.activa { "{t.desactivar}" } else { "{t.activar}" }
                    }
                    button { class: "btn-mini",
                        onclick: {
                            let id = regla.id;
                            move |_| { spawn(async move {
                                regla_elegida.set(Some(id));
                                match cliente::ejecuciones_automatizacion(id).await {
                                    Ok(lista) => ejecuciones.set(lista),
                                    Err(m) => aviso.set(m),
                                }
                            }); }
                        },
                        "{t.historial}"
                    }
                    button { class: "btn-mini", onclick: {
                        let id=regla.id;
                        move |_| { spawn(async move {
                            match cliente::eliminar_automatizacion(id).await {
                                Ok(()) => recargar(),
                                Err(m) => aviso.set(m),
                            }
                        }); }
                    }, "✕" }
                }
                if regla_elegida() == Some(regla.id) {
                    for ejecucion in ejecuciones.read().iter() {
                        div { class: "mini-row", key: "{ejecucion.id}",
                            span { class: if ejecucion.ok { "pill" } else { "pill error-pill" }, if ejecucion.ok { "✓" } else { "!" } }
                            span { class: "mini-name", "{ejecucion.disparador} · {ejecucion.detalle}" }
                            span { class: "mini-amt", "{fmt_dia_hora(ejecucion.creado_en)}" }
                        }
                    }
                }
            }
        }
    }
}

/// Chat del espacio: conversaciones personales y herramientas seguras.
#[component]
fn VistaAgentes() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut conversaciones = use_signal(Vec::<occb_core::Conversacion>::new);
    let mut actual = use_signal(|| None::<Uuid>);
    let mut mensajes = use_signal(Vec::<occb_core::Mensaje>::new);
    let mut fuentes = use_signal(Vec::<occb_core::ResultadoBusqueda>::new);
    let mut pregunta = use_signal(String::new);
    let mut aviso = use_signal(String::new);
    let mut ocupado = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            match cliente::listar_conversaciones().await {
                Ok(lista) => {
                    if actual().is_none()
                        && let Some(c) = lista.first()
                    {
                        actual.set(Some(c.id));
                        if let Ok(ms) = cliente::listar_mensajes(c.id).await { mensajes.set(ms); }
                    }
                    conversaciones.set(lista);
                }
                Err(m) => aviso.set(m),
            }
        });
    });

    let crear = move |_| {
        spawn(async move {
            match cliente::crear_conversacion("").await {
                Ok(c) => {
                    actual.set(Some(c.id));
                    mensajes.set(Vec::new());
                    fuentes.set(Vec::new());
                    conversaciones.write().insert(0, c);
                    aviso.set(String::new());
                }
                Err(m) => aviso.set(m),
            }
        });
    };

    let mut enviar = move || {
        let texto = pregunta.read().trim().to_string();
        if texto.is_empty() || ocupado() { return; }
        ocupado.set(true);
        aviso.set(String::new());
        spawn(async move {
            let id = match actual() {
                Some(id) => id,
                None => match cliente::crear_conversacion("").await {
                    Ok(c) => {
                        let id = c.id;
                        actual.set(Some(id));
                        conversaciones.write().insert(0, c);
                        id
                    }
                    Err(m) => { aviso.set(m); ocupado.set(false); return; }
                },
            };
            match cliente::preguntar(id, &texto).await {
                Ok(respuesta) => {
                    if respuesta.conversacion_id != id {
                        aviso.set(if idioma == Idioma::Es { "La respuesta llegó a otra conversación.".into() } else { "The response belongs to another conversation.".into() });
                        ocupado.set(false);
                        return;
                    }
                    pregunta.set(String::new());
                    fuentes.set(respuesta.fuentes);
                    match cliente::listar_mensajes(id).await {
                        Ok(lista) => mensajes.set(lista),
                        Err(m) => {
                            aviso.set(m);
                            mensajes.write().push(occb_core::Mensaje {
                                id: Uuid::new_v4(), conversacion_id: id, rol: "agente".into(),
                                contenido: respuesta.respuesta, creado_en: chrono::Utc::now(),
                            });
                        }
                    }
                    if let Ok(lista) = cliente::listar_conversaciones().await { conversaciones.set(lista); }
                }
                Err(m) => aviso.set(m),
            }
            ocupado.set(false);
        });
    };

    rsx! {
        BarraSuperior {
            titulo: t.agentes.to_string(),
            subtitulo: if idioma == Idioma::Es { "Consulta y actúa sobre los registros escolares".to_string() } else { "Ask and act on school records".to_string() },
        }
        if !aviso.read().is_empty() { p { class: "form-error", "{aviso}" } }
        div { class: "chat-layout",
            div { class: "panel chat-history",
                button { class: "btn-primary block", onclick: crear, "{t.nueva_conversacion}" }
                if conversaciones.read().is_empty() {
                    p { class: "muted", "{t.sin_conversaciones}" }
                }
                for c in conversaciones.read().iter() {
                    div { class: if actual() == Some(c.id) { "chat-conversation selected" } else { "chat-conversation" }, key: "{c.id}",
                        button { class: "chat-conversation-title", onclick: {
                            let id = c.id;
                            move |_| { spawn(async move {
                                actual.set(Some(id));
                                match cliente::listar_mensajes(id).await {
                                    Ok(lista) => mensajes.set(lista),
                                    Err(m) => aviso.set(m),
                                }
                                fuentes.set(Vec::new());
                            }); }
                        }, "{c.titulo}" }
                        button { class: "btn-mini", title: "✕", onclick: {
                            let id = c.id;
                            move |_| { spawn(async move {
                                match cliente::eliminar_conversacion(id).await {
                                    Ok(()) => {
                                        conversaciones.write().retain(|x| x.id != id);
                                        if actual() == Some(id) { actual.set(None); mensajes.set(Vec::new()); }
                                    }
                                    Err(m) => aviso.set(m),
                                }
                            }); }
                        }, "✕" }
                    }
                }
            }
            div { class: "panel chat-main",
                div { class: "chat-messages",
                    if mensajes.read().is_empty() {
                        p { class: "muted", if idioma == Idioma::Es { "Pregunta por alumnos, apoderados, colegios o admisión." } else { "Ask about students, guardians, schools or admissions." } }
                    }
                    for m in mensajes.read().iter() {
                        div { class: if m.rol == "usuario" { "chat-message user-message" } else { "chat-message agent-message" }, key: "{m.id}",
                            div { class: "chat-role", if m.rol == "usuario" { if idioma == Idioma::Es { "Tú" } else { "You" } } else { "{t.agentes}" } }
                            p { "{m.contenido}" }
                        }
                    }
                    if ocupado() { p { class: "muted", "{t.pensando}" } }
                    if !fuentes.read().is_empty() {
                        div { class: "chat-sources",
                            div { class: "panel-title", if idioma == Idioma::Es { "Fuentes" } else { "Sources" } }
                            for f in fuentes.read().iter() {
                                Link { class: "search-result", key: "{f.tipo}-{f.id}", to: ruta_busqueda(&f.tipo, &f.id),
                                    span { class: "search-result-type", "{etiqueta_tipo_busqueda(&f.tipo, idioma)}" }
                                    span { class: "mini-name", "{f.titulo}" }
                                    span { class: "mini-amt", "{f.detalle}" }
                                }
                            }
                        }
                    }
                }
                div { class: "chat-compose",
                    textarea {
                        class: "input",
                        placeholder: "{t.pregunta}",
                        value: "{pregunta}",
                        oninput: move |e| pregunta.set(e.value()),
                    }
                    button { class: "btn-primary", disabled: ocupado(), onclick: move |_| enviar(), "{t.enviar}" }
                }
            }
        }
    }
}

/// Timeline visible en fichas, con autor y detalle seguro de la actividad.
#[component]
fn Timeline(tipo: String, id: Uuid) -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut filas = use_signal(Vec::<occb_core::Actividad>::new);
    let mut id_actual = use_signal(|| id);
    if id_actual() != id {
        id_actual.set(id);
    }
    use_effect(move || {
        let tipo = tipo.clone();
        let id = id_actual();
        spawn(async move {
            if let Ok(lista) = cliente::timeline(&tipo, id).await {
                filas.set(lista);
            }
        });
    });
    rsx! {
        div { class: "panel timeline",
            div { class: "panel-head",
                span { class: "panel-title", "{t.timeline}" }
            }
            if filas.read().is_empty() {
                p { class: "muted", if idioma == Idioma::Es { "Sin actividad registrada" } else { "No activity yet" } }
            }
            for a in filas.read().iter() {
                div { class: "timeline-item", key: "{a.id}",
                    span { class: "timeline-dot", "•" }
                    div {
                        div { class: "timeline-text",
                            b { "{a.autor}" }
                            " "
                            match a.accion.as_str() {
                                "creado" => rsx! { "{t.creado}" },
                                "actualizado" => rsx! { "{t.actualizado}" },
                                _ => rsx! { "{t.eliminado}" },
                            }
                            " · "
                            "{titulo_actividad(a)}"
                        }
                        div { class: "timeline-date muted", "{fmt_dia_hora(a.creado_en)}" }
                    }
                }
            }
        }
    }
}

fn titulo_actividad(actividad: &occb_core::Actividad) -> &str {
    actividad
        .detalle
        .get("titulo")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Registro")
}

/// Selector y guardado de vistas personales. `on_cargar` aplica los filtros
/// almacenados en la pantalla que lo contiene.
#[component]
fn SelectorVistas(
    objeto: String,
    modo: String,
    filtros: serde_json::Value,
    on_cargar: EventHandler<occb_core::VistaGuardada>,
) -> Element {
    let t = textos(IDIOMA());
    let mut vistas = use_signal(Vec::<occb_core::VistaGuardada>::new);
    let mut seleccionada = use_signal(|| None::<Uuid>);
    let mut nombre = use_signal(String::new);
    let mut aviso = use_signal(String::new);
    let objeto_cargar = objeto.clone();
    use_effect(move || {
        let objeto = objeto_cargar.clone();
        spawn(async move {
            if let Ok(lista) = cliente::listar_vistas(&objeto).await {
                vistas.set(lista);
            }
        });
    });
    let guardar = move |_| {
        let nombre_vista = nombre.read().trim().to_string();
        if nombre_vista.is_empty() {
            aviso.set(if IDIOMA() == Idioma::Es {
                "Escribe un nombre para la vista.".into()
            } else {
                "Enter a name for the view.".into()
            });
            return;
        }
        let (objeto, modo, filtros) = (objeto.clone(), modo.clone(), filtros.clone());
        spawn(async move {
            match cliente::crear_vista(
                &objeto,
                &nombre_vista,
                &modo,
                filtros,
                serde_json::json!({}),
            )
            .await
            {
                Ok(v) => {
                    aviso.set(String::new());
                    nombre.set(String::new());
                    vistas.write().push(v);
                }
                Err(m) => aviso.set(m),
            }
        });
    };
    rsx! {
        div { class: "saved-view-controls",
            select {
                class: "input saved-view-select",
                onchange: move |e| {
                    if let Ok(id) = Uuid::parse_str(&e.value())
                        && let Some(v) = vistas.read().iter().find(|v| v.id == id).cloned()
                    {
                        seleccionada.set(Some(id));
                        on_cargar.call(v);
                    } else {
                        seleccionada.set(None);
                    }
                },
                option { value: "", "{t.vista_guardada}" }
                for v in vistas.read().iter() {
                    option { value: "{v.id}", "{v.nombre}" }
                }
            }
            if let Some(id) = seleccionada() {
                button {
                    class: "btn-mini",
                    title: if IDIOMA() == Idioma::Es { "Eliminar vista" } else { "Delete view" },
                    onclick: move |_| {
                        spawn(async move {
                            match cliente::eliminar_vista(id).await {
                                Ok(()) => {
                                    vistas.write().retain(|v| v.id != id);
                                    seleccionada.set(None);
                                }
                                Err(m) => aviso.set(m),
                            }
                        });
                    },
                    "✕"
                }
            }
            input {
                class: "input saved-view-name",
                placeholder: "{t.nombre_vista}",
                value: "{nombre}",
                oninput: move |e| nombre.set(e.value()),
            }
            button { class: "btn-mini", onclick: guardar, "{t.guardar_vista}" }
        }
        if !aviso.read().is_empty() { p { class: "form-error", "{aviso}" } }
    }
}

/// Etiqueta de estado de alumno según idioma.
fn etiqueta_estado(estado: EstadoAlumno, idioma: Idioma) -> &'static str {
    match (estado, idioma) {
        (EstadoAlumno::Postulante, Idioma::Es) => "Postulante",
        (EstadoAlumno::Matriculado, Idioma::Es) => "Matriculado",
        (EstadoAlumno::Trasladado, Idioma::Es) => "Trasladado",
        (EstadoAlumno::Retirado, Idioma::Es) => "Retirado",
        (EstadoAlumno::Postulante, Idioma::En) => "Applicant",
        (EstadoAlumno::Matriculado, Idioma::En) => "Enrolled",
        (EstadoAlumno::Trasladado, Idioma::En) => "Transferred",
        (EstadoAlumno::Retirado, Idioma::En) => "Withdrawn",
    }
}

/// Iniciales para avatares.
fn iniciales_de(nombres: &str, apellidos: &str) -> String {
    format!(
        "{}{}",
        nombres.chars().next().unwrap_or('?'),
        apellidos.chars().next().unwrap_or('?')
    )
}

/// Vista de alumnos: tabla viva con búsqueda, filtro de estado, alta y baja.
#[component]
fn VistaAlumnos() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut busqueda = use_signal(String::new);
    let mut estado_f = use_signal(String::new);
    let mut alumnos = use_signal(Vec::<Alumno>::new);
    let mut listo = use_signal(|| false);
    let mut aviso = use_signal(String::new);
    let mut mostrar_form = use_signal(|| false);
    let mut rut = use_signal(String::new);
    let mut nombres = use_signal(String::new);
    let mut apellidos = use_signal(String::new);
    let mut curso = use_signal(String::new);

    use_effect(move || {
        let b = busqueda.read().clone();
        let e = estado_f.read().clone();
        spawn(async move {
            match cliente::listar_alumnos(&b, &e).await {
                Ok(lista) => {
                    alumnos.set(lista);
                    listo.set(true);
                }
                Err(_) if alumnos.read().is_empty() => {
                    alumnos.set(datos_semilla_alumnos());
                    listo.set(true);
                }
                Err(_) => {}
            }
        });
    });

    let total = alumnos.read().len();

    let crear = move |_| {
        let datos = CrearAlumno {
            rut: rut.read().clone(),
            nombres: nombres.read().clone(),
            apellidos: apellidos.read().clone(),
            fecha_nacimiento: None,
            curso: curso.read().clone(),
            colegio_rbd: None,
            estado: EstadoAlumno::Postulante,
        };
        spawn(async move {
            match cliente::crear_alumno(&datos).await {
                Ok(_) => {
                    aviso.set(String::new());
                    rut.set(String::new());
                    nombres.set(String::new());
                    apellidos.set(String::new());
                    curso.set(String::new());
                    mostrar_form.set(false);
                    if let Ok(lista) = cliente::listar_alumnos("", "").await {
                        alumnos.set(lista);
                    }
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    rsx! {
        BarraSuperior {
            titulo: t.alumnos.to_string(),
            subtitulo: format!("{total} · {}", t.tabla),
        }
        div { class: "toolbar",
            input {
                class: "input",
                placeholder: "{t.buscar}…",
                value: "{busqueda}",
                oninput: move |e| busqueda.set(e.value()),
            }
            select {
                class: "input",
                value: "{estado_f}",
                onchange: move |e| estado_f.set(e.value()),
                option { value: "", if idioma == Idioma::Es { "Todos" } else { "All" } }
                for e in EstadoAlumno::todas() {
                    option { value: "{e.como_texto()}", "{etiqueta_estado(*e, idioma)}" }
                }
            }
            span { class: "pill", "{t.tabla}" }
        }
        SelectorVistas {
            objeto: "alumnos".to_string(),
            modo: "tabla".to_string(),
            filtros: serde_json::json!({ "busqueda": busqueda(), "estado": estado_f() }),
            on_cargar: move |v: occb_core::VistaGuardada| {
                if let Some(q) = v.filtros.get("busqueda").and_then(|x| x.as_str()) {
                    busqueda.set(q.to_string());
                }
                if let Some(e) = v.filtros.get("estado").and_then(|x| x.as_str()) {
                    estado_f.set(e.to_string());
                }
            },
        }
        if mostrar_form() {
        div { class: "panel form",
            label { if idioma == Idioma::Es { "Matricular alumno" } else { "Enroll student" } }
            div { class: "row",
                div { class: "campo",
                    label { "RUT *" }
                    input { class: "input", placeholder: "12345678-5", value: "{rut}", oninput: move |e| rut.set(e.value()) }
                }
                div { class: "campo",
                    label { if idioma == Idioma::Es { "Nombres *" } else { "First names *" } }
                    input { class: "input", placeholder: "Martín", value: "{nombres}", oninput: move |e| nombres.set(e.value()) }
                }
                div { class: "campo",
                    label { if idioma == Idioma::Es { "Apellidos *" } else { "Last names *" } }
                    input { class: "input", placeholder: "González", value: "{apellidos}", oninput: move |e| apellidos.set(e.value()) }
                }
                div { class: "campo",
                    label { if idioma == Idioma::Es { "Curso" } else { "Grade" } }
                    input { class: "input", placeholder: "8° Básico A", value: "{curso}", oninput: move |e| curso.set(e.value()) }
                }
                div { class: "campo campo-btn",
                    button { class: "btn-primary", onclick: crear, "{t.nueva}" }
                }
            }
            if !aviso.read().is_empty() {
                p { class: "muted small", "{aviso}" }
            }
        }
        }
        div { class: "tbl-wrap",
            if !listo() {
                p { class: "muted", style: "padding: 12px;", "…" }
            } else {
                table { class: "tbl",
                    thead {
                        tr {
                            th { class: "check", "☐" }
                            th { "RUT" }
                            th { if idioma == Idioma::Es { "Nombre" } else { "Name" } }
                            th { if idioma == Idioma::Es { "Curso" } else { "Grade" } }
                            th { if idioma == Idioma::Es { "Estado" } else { "Status" } }
                            th { "" }
                        }
                    }
                    tbody {
                        for a in alumnos.read().iter() {
                            tr { key: "{a.id}",
                                td { class: "check", "☐" }
                                td { class: "muted", "{a.rut}" }
                                td {
                                    span { class: "chip",
                                        span { class: "mini-avatar", "{iniciales_de(&a.nombres, &a.apellidos)}" }
                                        Link { to: Ruta::DetalleAlumno { id: a.id }, class: "link",
                                            "{a.nombres} {a.apellidos}"
                                        }
                                        EstrellaFavorito { tipo: "alumno".to_string(), id: a.id }
                                    }
                                }
                                td { "{a.curso}" }
                                td { span { class: "pill", "{etiqueta_estado(a.estado, idioma)}" } }
                                td {
                                    button {
                                        class: "btn-mini",
                                        title: "✕",
                                        onclick: {
                                            let id = a.id;
                                            move |_| {
                                                spawn(async move {
                                                    if cliente::eliminar_alumno(id).await.is_ok()
                                                        && let Ok(lista) =
                                                            cliente::listar_alumnos("", "").await
                                                    {
                                                        alumnos.set(lista);
                                                    }
                                                });
                                            }
                                        },
                                        "✕"
                                    }
                                }
                            }
                        }
                    }
                    tfoot {
                        tr {
                            td { colspan: "6",
                                button {
                                    class: "ghost-new",
                                    onclick: move |_| mostrar_form.set(true),
                                    "+ Agregar nuevo"
                                }
                            }
                        }
                        tr { class: "tbl-foot",
                            td { colspan: "3", "Calcular ▾" }
                            td { colspan: "3", "Contar todo ", b { "{total}" } }
                        }
                    }
                }
            }
        }
    }
}

/// Ficha de alumno: datos editables + apoderados vinculados + colegio.
#[component]
fn DetalleAlumno(id: Uuid) -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut alumno = use_signal(|| None::<Alumno>);
    let mut vinculados = use_signal(Vec::<cliente::ApoderadoVinculado>::new);
    let mut todos_apoderados = use_signal(Vec::<Apoderado>::new);
    let mut aviso = use_signal(String::new);
    let mut listo = use_signal(|| false);
    let mut editando = use_signal(|| false);
    let mut rut = use_signal(String::new);
    let mut nombres = use_signal(String::new);
    let mut apellidos = use_signal(String::new);
    let mut curso = use_signal(String::new);
    let mut estado_sel = use_signal(String::new);
    let mut vinc_apoderado = use_signal(String::new);
    let mut vinc_texto = use_signal(String::new);
    let mut tr_historial = use_signal(Vec::<Traslado>::new);
    let tr_region = use_signal(String::new);
    let tr_comuna = use_signal(String::new);
    let mut tr_busqueda = use_signal(String::new);
    let mut tr_resultados = use_signal(Vec::<Colegio>::new);
    let mut tr_destino = use_signal(String::new);
    let mut tr_motivo = use_signal(String::new);
    let mut colegio_nombre = use_signal(String::new);
    let mut rel_tareas = use_signal(Vec::<Tarea>::new);
    let mut rel_notas = use_signal(Vec::<Nota>::new);
    let mut rel_eventos = use_signal(Vec::<Evento>::new);
    let mut rel_correos = use_signal(Vec::<Correo>::new);

    // Sigue al id de la ruta: si cambia, recarga.
    let mut id_actual = use_signal(|| id);
    if id_actual() != id {
        id_actual.set(id);
    }

    let recargar_vinculos = move |pid: Uuid| {
        spawn(async move {
            if let Ok(lista) = cliente::apoderados_de_alumno(pid).await {
                vinculados.set(lista);
            }
        });
    };

    let recargar_historial = move |pid: Uuid| {
        spawn(async move {
            if let Ok(lista) = cliente::traslados_de_alumno(pid).await {
                tr_historial.set(lista);
            }
        });
    };

    use_effect(move || {
        let pid = id_actual();
        spawn(async move {
            match cliente::obtener_alumno(pid).await {
                Ok(a) => {
                    rut.set(a.rut.clone());
                    nombres.set(a.nombres.clone());
                    apellidos.set(a.apellidos.clone());
                    curso.set(a.curso.clone());
                    estado_sel.set(a.estado.como_texto().to_string());
                    alumno.set(Some(a.clone()));
                    recargar_vinculos(pid);
                    recargar_historial(pid);
                    spawn(async move {
                        if let Ok(lista) = cliente::tareas_de_alumno(pid).await {
                            rel_tareas.set(lista);
                        }
                        if let Ok(lista) = cliente::notas_de_alumno(pid).await {
                            rel_notas.set(lista);
                        }
                        if let Ok(lista) = cliente::eventos_de_alumno(pid).await {
                            rel_eventos.set(lista);
                        }
                        if let Ok(lista) = cliente::correos_de_alumno(pid).await {
                            rel_correos.set(lista);
                        }
                    });
                    if let Some(rbd) = a.colegio_rbd.clone() {
                        let rbd_clon = rbd.clone();
                        spawn(async move {
                            if let Ok(c) = cliente::obtener_colegio(&rbd_clon).await {
                                colegio_nombre.set(c.nombre.clone());
                            }
                        });
                    } else {
                        colegio_nombre.set(String::new());
                    }
                    listo.set(true);
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
        spawn(async move {
            if let Ok(lista) = cliente::listar_apoderados("").await {
                todos_apoderados.set(lista);
            }
        });
    });

    let guardar = move |_| {
        let pid = id_actual();
        let estado = EstadoAlumno::desde_texto(&estado_sel.read().to_lowercase())
            .unwrap_or(EstadoAlumno::Postulante);
        let datos = CrearAlumno {
            rut: rut.read().clone(),
            nombres: nombres.read().clone(),
            apellidos: apellidos.read().clone(),
            fecha_nacimiento: None,
            curso: curso.read().clone(),
            colegio_rbd: alumno.read().as_ref().and_then(|a| a.colegio_rbd.clone()),
            estado,
        };
        spawn(async move {
            match cliente::actualizar_alumno(pid, &datos).await {
                Ok(a) => {
                    aviso.set(String::new());
                    alumno.set(Some(a));
                    editando.set(false);
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    let vincular = move |_| {
        let pid = id_actual();
        let aid = Uuid::parse_str(&vinc_apoderado.read()).ok();
        let texto = vinc_texto.read().clone();
        spawn(async move {
            match aid {
                Some(aid) => match cliente::vincular_apoderado(pid, aid, &texto).await {
                    Ok(_) => {
                        aviso.set(String::new());
                        vinc_apoderado.set(String::new());
                        vinc_texto.set(String::new());
                        recargar_vinculos(pid);
                    }
                    Err(mensaje) => aviso.set(mensaje),
                },
                None => aviso.set(
                    if idioma == Idioma::Es { "Elija un apoderado.".to_string() } else { "Pick a guardian.".to_string() },
                ),
            }
        });
    };

    let buscar_colegios = move |_| {
        spawn(async move {
            let region = region_codigo_de(&tr_region);
            let comuna = tr_comuna.read().clone();
            let texto = tr_busqueda.read().clone();
            match cliente::listar_colegios(region, &comuna, &texto).await {
                Ok(lista) => tr_resultados.set(lista),
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    let trasladar = move |_| {
        let pid = id_actual();
        let destino = tr_destino.read().clone();
        let motivo = tr_motivo.read().clone();
        spawn(async move {
            if destino.trim().is_empty() {
                aviso.set(
                    if idioma == Idioma::Es { "Elija el colegio destino.".to_string() } else { "Pick the destination school.".to_string() },
                );
                return;
            }
            match cliente::trasladar_alumno(pid, destino.trim(), &motivo).await {
                Ok(a) => {
                    aviso.set(String::new());
                    alumno.set(Some(a));
                    tr_destino.set(String::new());
                    tr_motivo.set(String::new());
                    tr_resultados.set(vec![]);
                    recargar_historial(pid);
                    if let Ok(c) = cliente::obtener_colegio(destino.trim()).await {
                        colegio_nombre.set(c.nombre.clone());
                    }
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    let titulo = alumno
        .read()
        .as_ref()
        .map(|a| format!("{} {}", a.nombres, a.apellidos))
        .unwrap_or_default();
    rsx! {
        VistaAlumnos {}
        div { class: "record-panel",
        Link { to: Ruta::VistaAlumnos {}, class: "record-close", "✕ {t.alumnos}" }
        h2 { class: "record-title", {if titulo.is_empty() { "…".to_string() } else { titulo.clone() }} EstrellaFavorito { tipo: "alumno".to_string(), id } }
        p { class: "page-sub", {(if idioma == Idioma::Es { "Ficha de alumno" } else { "Student record" }).to_string()} }
        if !aviso.read().is_empty() && !listo() {
            div { class: "panel",
                p { class: "muted", "{aviso}" }
            }
        } else {
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title", if idioma == Idioma::Es { "General" } else { "General" } }
                    button { class: "btn-mini", onclick: move |_| editando.set(!editando()), "✎" }
                }
                if let Some(a) = alumno.read().as_ref() {
                    div { class: "mini-row", span { class: "muted", "RUT" } span { class: "mini-name", "{a.rut}" } }
                    div { class: "mini-row", span { class: "muted", if idioma == Idioma::Es { "Curso" } else { "Grade" } } span { class: "mini-name", "{a.curso}" } }
                    div { class: "mini-row", span { class: "muted", if idioma == Idioma::Es { "Estado" } else { "Status" } } span { class: "pill", "{etiqueta_estado(a.estado, idioma)}" } }
                    if let Some(rbd) = a.colegio_rbd.clone() {
                        div { class: "mini-row", span { class: "muted", if idioma == Idioma::Es { "Colegio" } else { "School" } } span { class: "mini-name",
                            if colegio_nombre.read().is_empty() { "{rbd}" } else { "{colegio_nombre}" }
                        } }
                    }
                }
            }
            if editando() {
            div { class: "panel form",
                label { if idioma == Idioma::Es { "Editar (se guarda en la API)" } else { "Edit (saved to the API)" } }
                div { class: "row",
                    div { class: "campo",
                        label { "RUT *" }
                        input { class: "input", placeholder: "12345678-5", value: "{rut}", oninput: move |e| rut.set(e.value()) }
                    }
                    div { class: "campo",
                        label { if idioma == Idioma::Es { "Nombres *" } else { "First names *" } }
                        input { class: "input", value: "{nombres}", oninput: move |e| nombres.set(e.value()) }
                    }
                    div { class: "campo",
                        label { if idioma == Idioma::Es { "Apellidos *" } else { "Last names *" } }
                        input { class: "input", value: "{apellidos}", oninput: move |e| apellidos.set(e.value()) }
                    }
                }
                div { class: "row",
                    div { class: "campo",
                        label { if idioma == Idioma::Es { "Curso" } else { "Grade" } }
                        input { class: "input", value: "{curso}", oninput: move |e| curso.set(e.value()) }
                    }
                    div { class: "campo",
                        label { if idioma == Idioma::Es { "Estado" } else { "Status" } }
                        select {
                            class: "input",
                            value: "{estado_sel}",
                            onchange: move |e| estado_sel.set(e.value()),
                            for e in EstadoAlumno::todas() {
                                option { value: "{e.como_texto()}", "{etiqueta_estado(*e, idioma)}" }
                            }
                        }
                    }
                    div { class: "campo campo-btn",
                        button { class: "btn-primary", onclick: guardar, "{t.guardar}" }
                    }
                }
                if !aviso.read().is_empty() {
                    p { class: "muted small", "{aviso}" }
                }
            }
            }
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title",
                        "{vinculados.read().len()} · {t.apoderados}"
                    }
                    span { class: "link", "↗" }
                }
                for v in vinculados.read().iter() {
                    div { class: "mini-row", key: "{v.id}",
                        span { class: "mini-name",
                            Link { to: Ruta::DetalleApoderado { id: v.id }, class: "link",
                                "{v.nombre_completo()}"
                            }
                        }
                        span { class: "mini-amt", "{v.vinculo} · {v.rut}" }
                        button {
                            class: "btn-mini",
                            title: "✕",
                            onclick: {
                                let aid = v.id;
                                let pid = id_actual();
                                move |_| {
                                    spawn(async move {
                                        if cliente::desvincular_apoderado(pid, aid).await.is_ok() {
                                            recargar_vinculos(pid);
                                        }
                                    });
                                }
                            },
                            "✕"
                        }
                    }
                }
                div { class: "row",
                    select {
                        class: "input",
                        value: "{vinc_apoderado}",
                        onchange: move |e| vinc_apoderado.set(e.value()),
                        option { value: "", "—" }
                        for a in todos_apoderados.read().iter() {
                            option { value: "{a.id}", "{a.nombres} {a.apellidos} · {a.rut}" }
                        }
                    }
                    input {
                        class: "input",
                        placeholder: if idioma == Idioma::Es { "Vínculo (Madre/Padre…)" } else { "Relation" },
                        value: "{vinc_texto}",
                        oninput: move |e| vinc_texto.set(e.value()),
                    }
                    button { class: "btn-primary", onclick: vincular, if idioma == Idioma::Es { "Vincular" } else { "Link" } }
                }
            }
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title",
                        if idioma == Idioma::Es { "Traslado de colegio" } else { "School transfer" }
                    }
                    span { class: "link", "↗" }
                }
                div { class: "mini-row",
                    span { class: "muted", if idioma == Idioma::Es { "Actual" } else { "Current" } }
                    span { class: "mini-name",
                        if colegio_nombre.read().is_empty() {
                            if idioma == Idioma::Es { "Sin colegio" } else { "No school" }
                        } else {
                            "{colegio_nombre}"
                        }
                    }
                }
                for h in tr_historial.read().iter() {
                    div { class: "mini-row", key: "{h.id}",
                        match h.origen_rbd.clone() {
                            Some(o) => rsx! {
                                span { class: "mini-name", "{o} → {h.destino_rbd}" }
                            },
                            None => rsx! {
                                span { class: "mini-name", "→ {h.destino_rbd}" }
                            },
                        }
                        span { class: "mini-amt", "{h.fecha.date_naive().to_string()} {h.motivo}" }
                    }
                }
                div { class: "row",
                    input {
                        class: "input",
                        placeholder: if idioma == Idioma::Es { "Buscar destino…" } else { "Search destination…" },
                        value: "{tr_busqueda}",
                        oninput: move |e| tr_busqueda.set(e.value()),
                    }
                    SelectorTerritorio {
                        region_sel: tr_region,
                        comuna_sel: tr_comuna,
                    }
                    div { class: "campo campo-btn",
                        button { class: "btn-primary", onclick: buscar_colegios, if idioma == Idioma::Es { "Buscar" } else { "Search" } }
                    }
                }
                for c in tr_resultados.read().iter().take(8) {
                    div { class: "mini-row", key: "{c.rbd}",
                        span { class: "mini-name", "{c.nombre}" }
                        span { class: "mini-amt", "{c.rbd}" }
                        button {
                            class: "btn-mini",
                            onclick: {
                                let rbd = c.rbd.clone();
                                move |_| tr_destino.set(rbd.clone())
                            },
                            if idioma == Idioma::Es { "Elegir" } else { "Pick" }
                        }
                    }
                }
                if !tr_destino.read().is_empty() {
                    div { class: "row",
                        input {
                            class: "input",
                            placeholder: if idioma == Idioma::Es { "Motivo (opcional)" } else { "Reason (optional)" },
                            value: "{tr_motivo}",
                            oninput: move |e| tr_motivo.set(e.value()),
                        }
                        div { class: "campo campo-btn",
                            button { class: "btn-primary", onclick: trasladar, if idioma == Idioma::Es { "Trasladar a {tr_destino}" } else { "Transfer to {tr_destino}" } }
                        }
                    }
                }
            }
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title",
                        "{rel_tareas.read().len()} · {t.tareas}"
                    }
                    span { class: "link", "↗" }
                }
                for ta in rel_tareas.read().iter() {
                    div { class: "mini-row", key: "{ta.id}",
                        span { class: "mini-name", "{ta.titulo}" }
                        span { class: "mini-amt", "{etiqueta_tarea(ta.estado, idioma)}" }
                    }
                }
            }
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title",
                        "{rel_notas.read().len()} · {t.notas}"
                    }
                    span { class: "link", "↗" }
                }
                for n in rel_notas.read().iter() {
                    div { class: "mini-row", key: "{n.id}",
                        span { class: "mini-name", "{n.titulo}" }
                        span { class: "mini-amt", "{n.cuerpo.chars().take(40).collect::<String>()}" }
                    }
                }
            }
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title",
                        "{rel_eventos.read().len()} · {t.calendario}"
                    }
                    span { class: "link", "↗" }
                }
                for e in rel_eventos.read().iter() {
                    div { class: "mini-row", key: "{e.id}",
                        span { class: "mini-name", "{e.titulo}" }
                        span { class: "mini-amt", "{fmt_dia_hora(e.inicio)}" }
                    }
                }
            }
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title",
                        "{rel_correos.read().len()} · {t.correos}"
                    }
                    span { class: "link", "↗" }
                }
                for c in rel_correos.read().iter() {
                    div { class: "mini-row", key: "{c.id}",
                        span { class: "mini-name",
                            if c.asunto.is_empty() { "(sin asunto)" } else { "{c.asunto}" }
                        }
                        span { class: "mini-amt", "{c.remitente}" }
                    }
                }
            }
            Timeline { tipo: "alumno".to_string(), id }
        }
        }
    }
}
/// Vista de apoderados: tabla viva con búsqueda, alta y baja.
#[component]
fn VistaApoderados() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut busqueda = use_signal(String::new);
    let mut apoderados = use_signal(Vec::<Apoderado>::new);
    let mut listo = use_signal(|| false);
    let mut aviso = use_signal(String::new);
    let mut mostrar_form = use_signal(|| false);
    let mut rut = use_signal(String::new);
    let mut nombres = use_signal(String::new);
    let mut apellidos = use_signal(String::new);
    let mut telefono = use_signal(String::new);
    let mut region_sel = use_signal(String::new);
    let mut comuna_sel = use_signal(String::new);

    use_effect(move || {
        let b = busqueda.read().clone();
        spawn(async move {
            match cliente::listar_apoderados(&b).await {
                Ok(lista) => {
                    apoderados.set(lista);
                    listo.set(true);
                }
                Err(_) if apoderados.read().is_empty() => {
                    apoderados.set(datos_semilla_apoderados());
                    listo.set(true);
                }
                Err(_) => {}
            }
        });
    });

    let total = apoderados.read().len();

    let crear = move |_| {
        let datos = CrearApoderado {
            rut: rut.read().clone(),
            nombres: nombres.read().clone(),
            apellidos: apellidos.read().clone(),
            telefono: telefono.read().clone(),
            correo: String::new(),
            direccion: String::new(),
            region_codigo: region_codigo_de(&region_sel),
            comuna_codigo: comuna_codigo_de(&comuna_sel),
        };
        spawn(async move {
            match cliente::crear_apoderado(&datos).await {
                Ok(_) => {
                    aviso.set(String::new());
                    rut.set(String::new());
                    nombres.set(String::new());
                    apellidos.set(String::new());
                    telefono.set(String::new());
                    region_sel.set(String::new());
                    comuna_sel.set(String::new());
                    mostrar_form.set(false);
                    if let Ok(lista) = cliente::listar_apoderados("").await {
                        apoderados.set(lista);
                    }
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    rsx! {
        BarraSuperior {
            titulo: t.apoderados.to_string(),
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
        }
        SelectorVistas {
            objeto: "apoderados".to_string(),
            modo: "tabla".to_string(),
            filtros: serde_json::json!({ "busqueda": busqueda() }),
            on_cargar: move |v: occb_core::VistaGuardada| {
                if let Some(q) = v.filtros.get("busqueda").and_then(|x| x.as_str()) {
                    busqueda.set(q.to_string());
                }
            },
        }
        if mostrar_form() {
        div { class: "panel form",
            label { if idioma == Idioma::Es { "Nuevo apoderado" } else { "New guardian" } }
            div { class: "row",
                div { class: "campo",
                    label { "RUT *" }
                    input { class: "input", placeholder: "12345678-5", value: "{rut}", oninput: move |e| rut.set(e.value()) }
                }
                div { class: "campo",
                    label { if idioma == Idioma::Es { "Nombres *" } else { "First names *" } }
                    input { class: "input", value: "{nombres}", oninput: move |e| nombres.set(e.value()) }
                }
                div { class: "campo",
                    label { if idioma == Idioma::Es { "Apellidos *" } else { "Last names *" } }
                    input { class: "input", value: "{apellidos}", oninput: move |e| apellidos.set(e.value()) }
                }
                div { class: "campo",
                    label { if idioma == Idioma::Es { "Teléfono" } else { "Phone" } }
                    input { class: "input", value: "{telefono}", oninput: move |e| telefono.set(e.value()) }
                }
                SelectorTerritorio {
                    region_sel: region_sel,
                    comuna_sel: comuna_sel,
                }
                div { class: "campo campo-btn",
                    button { class: "btn-primary", onclick: crear, "{t.nueva}" }
                }
            }
            if !aviso.read().is_empty() {
                p { class: "muted small", "{aviso}" }
            }
        }
        }
        div { class: "tbl-wrap",
            if !listo() {
                p { class: "muted", style: "padding: 12px;", "…" }
            } else {
                table { class: "tbl",
                    thead {
                        tr {
                            th { class: "check", "☐" }
                            th { "RUT" }
                            th { if idioma == Idioma::Es { "Nombre" } else { "Name" } }
                            th { if idioma == Idioma::Es { "Teléfono" } else { "Phone" } }
                            th { "" }
                        }
                    }
                    tbody {
                        for a in apoderados.read().iter() {
                            tr { key: "{a.id}",
                                td { class: "check", "☐" }
                                td { class: "muted", "{a.rut}" }
                                td {
                                    span { class: "chip",
                                        span { class: "mini-avatar", "{iniciales_de(&a.nombres, &a.apellidos)}" }
                                        Link { to: Ruta::DetalleApoderado { id: a.id }, class: "link",
                                            "{a.nombres} {a.apellidos}"
                                        }
                                        EstrellaFavorito { tipo: "apoderado".to_string(), id: a.id }
                                    }
                                }
                                td { "{a.telefono}" }
                                td {
                                    button {
                                        class: "btn-mini",
                                        title: "✕",
                                        onclick: {
                                            let id = a.id;
                                            move |_| {
                                                spawn(async move {
                                                    if cliente::eliminar_apoderado(id).await.is_ok()
                                                        && let Ok(lista) =
                                                            cliente::listar_apoderados("").await
                                                    {
                                                        apoderados.set(lista);
                                                    }
                                                });
                                            }
                                        },
                                        "✕"
                                    }
                                }
                            }
                        }
                    }
                    tfoot {
                        tr {
                            td { colspan: "5",
                                button {
                                    class: "ghost-new",
                                    onclick: move |_| mostrar_form.set(true),
                                    "+ Agregar nuevo"
                                }
                            }
                        }
                        tr { class: "tbl-foot",
                            td { colspan: "2", "Calcular ▾" }
                            td { colspan: "3", "Contar todo ", b { "{total}" } }
                        }
                    }
                }
            }
        }
    }
}

/// Ficha de apoderado: datos editables + alumnos vinculados.
#[component]
fn DetalleApoderado(id: Uuid) -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut apoderado = use_signal(|| None::<Apoderado>);
    let mut vinculados = use_signal(Vec::<cliente::AlumnoVinculado>::new);
    let mut rel_eventos = use_signal(Vec::<Evento>::new);
    let mut rel_correos = use_signal(Vec::<Correo>::new);
    let mut aviso = use_signal(String::new);
    let mut listo = use_signal(|| false);
    let mut editando = use_signal(|| false);
    let mut rut = use_signal(String::new);
    let mut nombres = use_signal(String::new);
    let mut apellidos = use_signal(String::new);
    let mut telefono = use_signal(String::new);
    let mut region_sel = use_signal(String::new);
    let mut comuna_sel = use_signal(String::new);

    // Sigue al id de la ruta: si cambia, recarga.
    let mut id_actual = use_signal(|| id);
    if id_actual() != id {
        id_actual.set(id);
    }

    let recargar_vinculos = move |pid: Uuid| {
        spawn(async move {
            if let Ok(lista) = cliente::alumnos_de_apoderado(pid).await {
                vinculados.set(lista);
            }
            if let Ok(lista) = cliente::eventos_de_apoderado(pid).await {
                rel_eventos.set(lista);
            }
            if let Ok(lista) = cliente::correos_de_apoderado(pid).await {
                rel_correos.set(lista);
            }
        });
    };

    use_effect(move || {
        let pid = id_actual();
        spawn(async move {
            match cliente::obtener_apoderado(pid).await {
                Ok(a) => {
                    rut.set(a.rut.clone());
                    nombres.set(a.nombres.clone());
                    apellidos.set(a.apellidos.clone());
                    telefono.set(a.telefono.clone());
                    region_sel.set(a.region_codigo.map(|r| r.to_string()).unwrap_or_default());
                    comuna_sel.set(a.comuna_codigo.clone().unwrap_or_default());
                    apoderado.set(Some(a));
                    recargar_vinculos(pid);
                    listo.set(true);
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    });

    let guardar = move |_| {
        let pid = id_actual();
        let datos = CrearApoderado {
            rut: rut.read().clone(),
            nombres: nombres.read().clone(),
            apellidos: apellidos.read().clone(),
            telefono: telefono.read().clone(),
            correo: String::new(),
            direccion: String::new(),
            region_codigo: region_codigo_de(&region_sel),
            comuna_codigo: comuna_codigo_de(&comuna_sel),
        };
        spawn(async move {
            match cliente::actualizar_apoderado(pid, &datos).await {
                Ok(a) => {
                    aviso.set(String::new());
                    apoderado.set(Some(a));
                    editando.set(false);
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    let titulo = apoderado
        .read()
        .as_ref()
        .map(|a| format!("{} {}", a.nombres, a.apellidos))
        .unwrap_or_default();
    rsx! {
        VistaApoderados {}
        div { class: "record-panel",
        Link { to: Ruta::VistaApoderados {}, class: "record-close", "✕ {t.apoderados}" }
        h2 { class: "record-title", {if titulo.is_empty() { "…".to_string() } else { titulo.clone() }} EstrellaFavorito { tipo: "apoderado".to_string(), id } }
        p { class: "page-sub", {(if idioma == Idioma::Es { "Ficha de apoderado" } else { "Guardian record" }).to_string()} }
        if !aviso.read().is_empty() && !listo() {
            div { class: "panel",
                p { class: "muted", "{aviso}" }
            }
        } else {
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title", if idioma == Idioma::Es { "General" } else { "General" } }
                    button { class: "btn-mini", onclick: move |_| editando.set(!editando()), "✎" }
                }
                if let Some(a) = apoderado.read().as_ref() {
                    div { class: "mini-row", span { class: "muted", "RUT" } span { class: "mini-name", "{a.rut}" } }
                    div { class: "mini-row", span { class: "muted", "📞" } span { class: "mini-name", "{a.telefono}" } }
                }
            }
            if editando() {
            div { class: "panel form",
                label { if idioma == Idioma::Es { "Editar (se guarda en la API)" } else { "Edit (saved to the API)" } }
                div { class: "row",
                    div { class: "campo",
                        label { "RUT *" }
                        input { class: "input", value: "{rut}", oninput: move |e| rut.set(e.value()) }
                    }
                    div { class: "campo",
                        label { if idioma == Idioma::Es { "Nombres *" } else { "First names *" } }
                        input { class: "input", value: "{nombres}", oninput: move |e| nombres.set(e.value()) }
                    }
                    div { class: "campo",
                        label { if idioma == Idioma::Es { "Apellidos *" } else { "Last names *" } }
                        input { class: "input", value: "{apellidos}", oninput: move |e| apellidos.set(e.value()) }
                    }
                    div { class: "campo",
                        label { if idioma == Idioma::Es { "Teléfono" } else { "Phone" } }
                        input { class: "input", value: "{telefono}", oninput: move |e| telefono.set(e.value()) }
                    }
                    SelectorTerritorio {
                        region_sel: region_sel,
                        comuna_sel: comuna_sel,
                    }
                    div { class: "campo campo-btn",
                        button { class: "btn-primary", onclick: guardar, "{t.guardar}" }
                    }
                }
                if !aviso.read().is_empty() {
                    p { class: "muted small", "{aviso}" }
                }
            }
            }
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title",
                        "{vinculados.read().len()} · {t.alumnos}"
                    }
                    span { class: "link", "↗" }
                }
                for v in vinculados.read().iter() {
                    div { class: "mini-row", key: "{v.id}",
                        span { class: "mini-name",
                            Link { to: Ruta::DetalleAlumno { id: v.id }, class: "link",
                                "{v.nombre_completo()}"
                            }
                        }
                        span { class: "mini-amt", "{v.rut} · {v.curso} · {v.vinculo}" }
                    }
                }
            }
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title",
                        "{rel_eventos.read().len()} · {t.calendario}"
                    }
                    span { class: "link", "↗" }
                }
                for e in rel_eventos.read().iter() {
                    div { class: "mini-row", key: "{e.id}",
                        span { class: "mini-name", "{e.titulo}" }
                        span { class: "mini-amt", "{fmt_dia_hora(e.inicio)}" }
                    }
                }
            }
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title",
                        "{rel_correos.read().len()} · {t.correos}"
                    }
                    span { class: "link", "↗" }
                }
                for c in rel_correos.read().iter() {
                    div { class: "mini-row", key: "{c.id}",
                        span { class: "mini-name",
                            if c.asunto.is_empty() { "(sin asunto)" } else { "{c.asunto}" }
                        }
                        span { class: "mini-amt", "{c.remitente}" }
                    }
                }
            }
            Timeline { tipo: "apoderado".to_string(), id }
        }
        }
    }
}

/// Etiqueta de estado de tarea según idioma.
fn etiqueta_tarea(estado: EstadoTarea, idioma: Idioma) -> &'static str {
    match (estado, idioma) {
        (EstadoTarea::Pendiente, Idioma::Es) => "Pendiente",
        (EstadoTarea::EnCurso, Idioma::Es) => "En curso",
        (EstadoTarea::Hecha, Idioma::Es) => "Hecha",
        (EstadoTarea::Pendiente, Idioma::En) => "To do",
        (EstadoTarea::EnCurso, Idioma::En) => "In progress",
        (EstadoTarea::Hecha, Idioma::En) => "Done",
    }
}

/// Vista de tareas: tablero Por Estado con alta y avance persistido.
#[component]
fn VistaTareas() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut tareas = use_signal(Vec::<Tarea>::new);
    let mut aviso = use_signal(String::new);
    let mut mostrar_form = use_signal(|| false);
    let mut titulo = use_signal(String::new);
    let mut estado_nuevo = use_signal(|| EstadoTarea::Pendiente);

    use_effect(move || {
        spawn(async move {
            match cliente::listar_tareas("", "").await {
                Ok(lista) => tareas.set(lista),
                Err(_) if tareas.read().is_empty() => tareas.set(datos_semilla_tareas()),
                Err(_) => {}
            }
        });
    });

    let crear = move |_| {
        let datos = CrearTarea {
            titulo: titulo.read().clone(),
            detalle: String::new(),
            estado: estado_nuevo(),
            prioridad: "media".into(),
            vencimiento: None,
            alumno_id: None,
            apoderado_id: None,
            oportunidad_id: None,
        };
        spawn(async move {
            match cliente::crear_tarea(&datos).await {
                Ok(_) => {
                    aviso.set(String::new());
                    titulo.set(String::new());
                    estado_nuevo.set(EstadoTarea::Pendiente);
                    mostrar_form.set(false);
                    if let Ok(lista) = cliente::listar_tareas("", "").await {
                        tareas.set(lista);
                    }
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    rsx! {
        BarraSuperior {
            titulo: t.tareas.to_string(),
            subtitulo: format!(
                "{} · {}",
                tareas.read().len(),
                if idioma == Idioma::Es { "pulsa ▶ para avanzar" } else { "click ▶ to advance" }
            ),
        }
        if mostrar_form() {
        div { class: "panel form",
            label {
                if idioma == Idioma::Es { "Nueva tarea · " } else { "New task · " }
                "{etiqueta_tarea(estado_nuevo(), idioma)}"
            }
            div { class: "row",
                input {
                    class: "input",
                    placeholder: if idioma == Idioma::Es { "Título *" } else { "Title *" },
                    value: "{titulo}",
                    oninput: move |e| titulo.set(e.value()),
                }
                button { class: "btn-primary", onclick: crear, "{t.nueva}" }
            }
            if !aviso.read().is_empty() {
                p { class: "muted small", "{aviso}" }
            }
        }
        }
        div { class: "kanban",
            for estado in EstadoTarea::todas() {
                div { class: "col", key: "{estado:?}",
                    div { class: "col-head",
                        span { class: "pill", "{etiqueta_tarea(*estado, idioma)}" }
                        span { class: "count",
                            "{tareas.read().iter().filter(|x| &x.estado == estado).count()}"
                        }
                    }
                    for ta in tareas.read().iter().filter(|x| &x.estado == estado).cloned().collect::<Vec<_>>() {
                        div { class: "opp-card", key: "{ta.id}",
                            div { class: "opp-name", "{ta.titulo}" }
                            if !ta.detalle.is_empty() {
                                div { class: "opp-sub", "{ta.detalle}" }
                            }
                            div { class: "opp-foot",
                                span { class: "opp-prob", "{ta.prioridad}" }
                                button {
                                    class: "btn-mini",
                                    title: if idioma == Idioma::Es { "Eliminar" } else { "Delete" },
                                    onclick: {
                                        let id = ta.id;
                                        move |_| {
                                            spawn(async move {
                                                if cliente::eliminar_tarea(id).await.is_ok()
                                                    && let Ok(lista) =
                                                        cliente::listar_tareas("", "").await
                                                {
                                                    tareas.set(lista);
                                                }
                                            });
                                        }
                                    },
                                    "✕"
                                }
                                button {
                                    class: "btn-mini",
                                    title: if idioma == Idioma::Es { "Avanzar" } else { "Advance" },
                                    onclick: {
                                        let id = ta.id;
                                        let siguiente = ta.estado.siguiente().como_texto();
                                        move |_| {
                                            spawn(async move {
                                                match cliente::marcar_tarea(id, siguiente).await {
                                                    Ok(_) => {
                                                        if let Ok(lista) = cliente::listar_tareas("", "").await {
                                                            tareas.set(lista);
                                                        }
                                                    }
                                                    Err(mensaje) => aviso.set(mensaje),
                                                }
                                            });
                                        }
                                    },
                                    "▶"
                                }
                            }
                        }
                    }
                    button {
                        class: "ghost-new",
                        onclick: {
                            let e = *estado;
                            move |_| {
                                estado_nuevo.set(e);
                                mostrar_form.set(true);
                            }
                        },
                        "+ Nuevo"
                    }
                }
            }
        }
    }
}

/// Vista de notas: lista con búsqueda, alta, edición inline y baja.
#[component]
fn VistaNotas() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut busqueda = use_signal(String::new);
    let mut notas = use_signal(Vec::<Nota>::new);
    let mut listo = use_signal(|| false);
    let mut aviso = use_signal(String::new);
    let mut mostrar_form = use_signal(|| false);
    let mut titulo = use_signal(String::new);
    let mut cuerpo = use_signal(String::new);
    let mut editando = use_signal(|| None::<Uuid>);
    let mut cuerpo_ed = use_signal(String::new);

    use_effect(move || {
        let b = busqueda.read().clone();
        spawn(async move {
            match cliente::listar_notas(&b).await {
                Ok(lista) => {
                    notas.set(lista);
                    listo.set(true);
                }
                Err(_) if notas.read().is_empty() => {
                    notas.set(datos_semilla_notas());
                    listo.set(true);
                }
                Err(_) => {}
            }
        });
    });

    let total = notas.read().len();

    let crear = move |_| {
        let datos = CrearNota {
            titulo: titulo.read().clone(),
            cuerpo: cuerpo.read().clone(),
            alumno_id: None,
            apoderado_id: None,
            oportunidad_id: None,
        };
        spawn(async move {
            match cliente::crear_nota(&datos).await {
                Ok(_) => {
                    aviso.set(String::new());
                    titulo.set(String::new());
                    cuerpo.set(String::new());
                    mostrar_form.set(false);
                    if let Ok(lista) = cliente::listar_notas("").await {
                        notas.set(lista);
                    }
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    rsx! {
        BarraSuperior {
            titulo: t.notas.to_string(),
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
        }
        if mostrar_form() {
        div { class: "panel form",
            label { if idioma == Idioma::Es { "Nueva nota" } else { "New note" } }
            div { class: "row",
                input {
                    class: "input",
                    placeholder: if idioma == Idioma::Es { "Título" } else { "Title" },
                    value: "{titulo}",
                    oninput: move |e| titulo.set(e.value()),
                }
                button { class: "btn-primary", onclick: crear, "{t.nueva}" }
            }
            div { class: "row",
                input {
                    class: "input",
                    placeholder: if idioma == Idioma::Es { "Contenido *" } else { "Body *" },
                    value: "{cuerpo}",
                    oninput: move |e| cuerpo.set(e.value()),
                }
            }
            if !aviso.read().is_empty() {
                p { class: "muted small", "{aviso}" }
            }
        }
        }
        div { class: "tbl-wrap",
            if !listo() {
                p { class: "muted", style: "padding: 12px;", "…" }
            } else {
                table { class: "tbl",
                    thead {
                        tr {
                            th { class: "check", "☐" }
                            th { if idioma == Idioma::Es { "Título" } else { "Title" } }
                            th { if idioma == Idioma::Es { "Contenido" } else { "Body" } }
                            th { if idioma == Idioma::Es { "Creada" } else { "Created" } }
                            th { "" }
                        }
                    }
                    tbody {
                        for n in notas.read().iter() {
                            tr { key: "{n.id}",
                                td { class: "check", "☐" }
                                td { class: "strong", "{n.titulo}" }
                                td {
                                    if editando() == Some(n.id) {
                                        input {
                                            class: "input",
                                            value: "{cuerpo_ed}",
                                            oninput: move |e| cuerpo_ed.set(e.value()),
                                        }
                                    } else {
                                        span { class: "muted", "{n.cuerpo}" }
                                    }
                                }
                                td { class: "muted small", "{hace_tiempo(n.creado_en, idioma)}" }
                                td {
                                    if editando() == Some(n.id) {
                                        button {
                                            class: "btn-mini",
                                            onclick: {
                                                let id = n.id;
                                                let titulo_n = n.titulo.clone();
                                                move |_| {
                                                    let datos = CrearNota {
                                                        titulo: titulo_n.clone(),
                                                        cuerpo: cuerpo_ed.read().clone(),
                                                        alumno_id: None,
                                                        apoderado_id: None,
                                                        oportunidad_id: None,
                                                    };
                                                    spawn(async move {
                                                        match cliente::actualizar_nota(id, &datos).await {
                                                            Ok(_) => {
                                                                editando.set(None);
                                                                if let Ok(lista) = cliente::listar_notas("").await {
                                                                    notas.set(lista);
                                                                }
                                                            }
                                                            Err(mensaje) => aviso.set(mensaje),
                                                        }
                                                    });
                                                }
                                            },
                                            if idioma == Idioma::Es { "Guardar" } else { "Save" }
                                        }
                                    } else {
                                        button {
                                            class: "btn-mini",
                                            title: "✎",
                                            onclick: {
                                                let (id, c) = (n.id, n.cuerpo.clone());
                                                move |_| {
                                                    cuerpo_ed.set(c.clone());
                                                    editando.set(Some(id));
                                                }
                                            },
                                            "✎"
                                        }
                                    }
                                    button {
                                        class: "btn-mini",
                                        title: "✕",
                                        onclick: {
                                            let id = n.id;
                                            move |_| {
                                                spawn(async move {
                                                    if cliente::eliminar_nota(id).await.is_ok()
                                                        && let Ok(lista) =
                                                            cliente::listar_notas("").await
                                                    {
                                                        notas.set(lista);
                                                    }
                                                });
                                            }
                                        },
                                        "✕"
                                    }
                                }
                            }
                        }
                    }
                    tfoot {
                        tr {
                            td { colspan: "5",
                                button {
                                    class: "ghost-new",
                                    onclick: move |_| mostrar_form.set(true),
                                    "+ Agregar nuevo"
                                }
                            }
                        }
                        tr { class: "tbl-foot",
                            td { colspan: "2", "Calcular ▾" }
                            td { colspan: "3", "Contar todo ", b { "{total}" } }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn VistaAjustes() -> Element {
    let t = textos(IDIOMA());
    let mut claves = use_signal(Vec::<serde_json::Value>::new);
    let mut webhooks = use_signal(Vec::<serde_json::Value>::new);
    let mut nombre_clave = use_signal(String::new);
    let mut clave_nueva = use_signal(String::new);
    let mut nombre_webhook = use_signal(String::new);
    let mut url_webhook = use_signal(String::new);
    let mut evento_webhook = use_signal(|| "registro.creado".to_string());
    let mut entregas_hook = use_signal(Vec::<serde_json::Value>::new);
    let mut csv_objeto = use_signal(|| "alumnos".to_string());
    let mut csv_texto = use_signal(String::new);
    let mut csv_resultado = use_signal(|| serde_json::Value::Null);
    let mut aviso = use_signal(String::new);
    let recargar_integraciones = move || {
        spawn(async move {
            if let Ok(lista) = cliente::listar_claves_api().await { claves.set(lista); }
            if let Ok(lista) = cliente::listar_webhooks().await { webhooks.set(lista); }
        });
    };
    use_effect(move || {
        spawn(async move {
            if let Ok(lista) = cliente::listar_claves_api().await { claves.set(lista); }
            if let Ok(lista) = cliente::listar_webhooks().await { webhooks.set(lista); }
        });
    });
    let crear_clave = move |_| {
        let nombre = nombre_clave.read().trim().to_string();
        spawn(async move {
            match cliente::crear_clave_api(&nombre).await {
                Ok(c) => {
                    clave_nueva.set(c.get("clave").and_then(|x|x.as_str()).unwrap_or("").to_string());
                    nombre_clave.set(String::new());
                    recargar_integraciones();
                    aviso.set(String::new());
                }
                Err(m) => aviso.set(m),
            }
        });
    };
    let crear_webhook_btn = move |_| {
        let (nombre,url,eventos) = (nombre_webhook.read().trim().to_string(),url_webhook.read().trim().to_string(),vec![evento_webhook.read().clone()]);
        spawn(async move {
            match cliente::crear_webhook(&nombre,&url,eventos).await {
                Ok(_) => { nombre_webhook.set(String::new()); url_webhook.set(String::new()); recargar_integraciones(); aviso.set(String::new()); }
                Err(m) => aviso.set(m),
            }
        });
    };
    let previsualizar = move |_| {
        let (obj,contenido) = (csv_objeto.read().clone(),csv_texto.read().clone());
        spawn(async move { match cliente::importar_csv(&obj,&contenido,false).await { Ok(r)=>{csv_resultado.set(r);aviso.set(String::new());},Err(m)=>aviso.set(m) } });
    };
    let aplicar_csv = move |_| {
        let (obj,contenido) = (csv_objeto.read().clone(),csv_texto.read().clone());
        spawn(async move { match cliente::importar_csv(&obj,&contenido,true).await { Ok(r)=>{csv_resultado.set(r);aviso.set(String::new());},Err(m)=>aviso.set(m) } });
    };
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
                    "El frontal lee y escribe en la API; el CRUD pasa solo por funciones seguras con RLS. Ver /migrations."
                } else {
                    "The frontend reads and writes through the API; CRUD goes only through secure functions with RLS. See /migrations."
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
        if !aviso.read().is_empty() { p { class: "form-error", "{aviso}" } }
        div { class: "panel integration-panel",
            div { class: "panel-head", span { class: "panel-title", "{t.claves_api}" } }
            div { class: "row",
                input { class: "input", placeholder: "{t.nombre}", value: "{nombre_clave}", oninput: move |e| nombre_clave.set(e.value()) }
                button { class: "btn-primary", onclick: crear_clave, "{t.nueva_clave}" }
            }
            if !clave_nueva.read().is_empty() {
                p { class: "form-error secret-once", if IDIOMA() == Idioma::Es { "Copia esta clave ahora; no volverá a mostrarse:" } else { "Copy this key now; it will not be shown again:" }, code { "{clave_nueva}" } }
            }
            for k in claves.read().iter() {
                div { class: "mini-row", key: "{k.get(\"id\").and_then(|x|x.as_str()).unwrap_or(\"\")}",
                    span { class: "mini-name", "{k.get(\"nombre\").and_then(|x|x.as_str()).unwrap_or(\"API\")}" }
                    span { class: "mini-amt", if k.get("revocada").and_then(|x|x.as_bool()).unwrap_or(false) { "Revocada" } else { "Activa" } }
                    if !k.get("revocada").and_then(|x|x.as_bool()).unwrap_or(false) {
                        button { class: "btn-mini", onclick: {
                            let id=Uuid::parse_str(k.get("id").and_then(|x|x.as_str()).unwrap_or("")).ok();
                            move |_| if let Some(id)=id { spawn(async move { if let Err(m)=cliente::revocar_clave_api(id).await { aviso.set(m); } else { recargar_integraciones(); } }); }
                        }, "Revocar" }
                    }
                }
            }
        }
        div { class: "panel integration-panel",
            div { class: "panel-head", span { class: "panel-title", "{t.webhooks}" } }
            div { class: "row",
                input { class: "input", placeholder: "{t.nombre_webhook}", value: "{nombre_webhook}", oninput: move |e| nombre_webhook.set(e.value()) }
                input { class: "input", placeholder: "https://…", value: "{url_webhook}", oninput: move |e| url_webhook.set(e.value()) }
                select { class: "input", value: "{evento_webhook}", onchange: move |e| evento_webhook.set(e.value()),
                    option { value: "registro.creado", if IDIOMA()==Idioma::Es { "Registros creados" } else { "Records created" } }
                    option { value: "registro.actualizado", if IDIOMA()==Idioma::Es { "Registros actualizados" } else { "Records updated" } }
                    option { value: "postulacion.creado", if IDIOMA()==Idioma::Es { "Postulación creada" } else { "Application created" } }
                    option { value: "postulacion.actualizado", if IDIOMA()==Idioma::Es { "Postulación actualizada" } else { "Application updated" } }
                }
                button { class: "btn-primary", onclick: crear_webhook_btn, "{t.guardar}" }
            }
            for w in webhooks.read().iter() {
                {
                    let wid=Uuid::parse_str(w.get("id").and_then(|x|x.as_str()).unwrap_or("")).ok();
                    rsx! { div { class: "mini-row", key: "{w.get(\"id\").and_then(|x|x.as_str()).unwrap_or(\"\")}",
                        span { class: "mini-name", "{w.get(\"nombre\").and_then(|x|x.as_str()).unwrap_or(\"Webhook\")}" }
                        span { class: "mini-amt", "{w.get(\"url\").and_then(|x|x.as_str()).unwrap_or(\"\")}" }
                        if let Some(id)=wid {
                            button { class: "btn-mini", onclick: move |_| { spawn(async move { match cliente::probar_webhook(id).await { Ok(_)=>{ if let Ok(e)=cliente::entregas_webhook(id).await {entregas_hook.set(e);} }, Err(m)=>aviso.set(m) } }); }, "{t.probar}" }
                            button { class: "btn-mini", onclick: move |_| { spawn(async move { let _=cliente::alternar_webhook(id,false).await; recargar_integraciones(); }); }, "Desactivar" }
                            button { class: "btn-mini", onclick: move |_| { spawn(async move { let _=cliente::eliminar_webhook(id).await; recargar_integraciones(); }); }, "✕" }
                        }
                    } }
                }
            }
            for e in entregas_hook.read().iter() {
                div { class: "mini-row", key: "{e.get(\"id\").and_then(|x|x.as_str()).unwrap_or(\"\")}",
                    span { class: "mini-name", "{e.get(\"evento\").and_then(|x|x.as_str()).unwrap_or(\"entrega\")}" }
                    span { class: "mini-amt", "{e.get(\"ultimo_error\").and_then(|x|x.as_str()).unwrap_or(\"Pendiente\")}" }
                }
            }
        }
        div { class: "panel integration-panel",
            div { class: "panel-head", span { class: "panel-title", "{t.importar}" } }
            select { class: "input", value: "{csv_objeto}", onchange: move |e| csv_objeto.set(e.value()),
                option { value: "alumnos", "{t.alumnos}" }
                option { value: "apoderados", "{t.apoderados}" }
            }
            label { "{t.csv_archivo}" }
            input { class: "input", r#type: "file", accept: ".csv,text/csv", onchange: move |e| {
                if let Some(archivo)=e.files().first().cloned() {
                    spawn(async move { match archivo.read_string().await { Ok(s)=>csv_texto.set(s), Err(_)=>aviso.set(if IDIOMA()==Idioma::Es { "No se pudo leer el archivo.".into() } else { "Could not read file.".into() }) } });
                }
            } }
            textarea { class: "input csv-preview", value: "{csv_texto}", oninput: move |e| csv_texto.set(e.value()), placeholder: "rut,nombres,apellidos…" }
            div { class: "cal-bar",
                button { class: "btn-mini", onclick: previsualizar, "{t.previsualizar}" }
                button { class: "btn-primary", onclick: aplicar_csv, "{t.aplicar_importacion}" }
            }
            if !csv_resultado().is_null() { pre { class: "csv-result", "{csv_resultado}" } }
        }
    }
}

/// Pantalla de acceso con login y registro reales (sesión solo en memoria).
/// Al entrar, vuelve al destino pendiente (`VOLVER`) o al panel.
#[component]
fn VistaAcceso() -> Element {
    let t = textos(IDIOMA());
    let nav = use_navigator();
    if cliente::SESION().is_some() {
        nav.replace(Ruta::Panel {});
        return rsx! {
            div { class: "login-wrap",
                p { class: "muted", "…" }
            }
        };
    }
    let mut es_registro = use_signal(|| false);
    let mut nombre = use_signal(String::new);
    let mut correo = use_signal(String::new);
    let mut clave = use_signal(String::new);
    let mut aviso = use_signal(String::new);
    let mut ocupado = use_signal(|| false);

    let enviar = move |_| {
        let n = nombre.read().clone();
        let c = correo.read().trim().to_string();
        let k = clave.read().clone();
        let registro = *es_registro.read();
        if registro && n.trim().is_empty() {
            aviso.set(if IDIOMA() == Idioma::Es {
                "El nombre es obligatorio.".into()
            } else {
                "Name is required.".into()
            });
            return;
        }
        if c.is_empty() || k.is_empty() {
            aviso.set(if IDIOMA() == Idioma::Es {
                "Escribe tu correo y tu clave.".into()
            } else {
                "Type your email and password.".into()
            });
            return;
        }
        ocupado.set(true);
        aviso.set(String::new());
        spawn(async move {
            let resultado = if registro {
                cliente::registrar(n.trim(), c.trim(), &k).await
            } else {
                cliente::iniciar_sesion(c.trim(), &k).await
            };
            ocupado.set(false);
            match resultado {
                Ok(()) => {
                    let destino = VOLVER.write().take().unwrap_or(Ruta::Panel {});
                    nav.replace(destino);
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

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
                if *es_registro.read() {
                    label { "{t.nombre}" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{nombre}",
                        oninput: move |e| nombre.set(e.value()),
                    }
                }
                label { "{t.correo}" }
                input {
                    class: "input",
                    r#type: "email",
                    value: "{correo}",
                    oninput: move |e| correo.set(e.value()),
                }
                label { "{t.contrasena}" }
                input {
                    class: "input",
                    r#type: "password",
                    value: "{clave}",
                    oninput: move |e| clave.set(e.value()),
                }
                if *es_registro.read() {
                    p { class: "muted", "{t.clave_min}" }
                }
                if !aviso.read().is_empty() {
                    p { class: "form-error", "{aviso}" }
                }
                button {
                    class: "btn-primary block",
                    disabled: ocupado(),
                    onclick: enviar,
                    if *es_registro.read() { "{t.crear_cuenta}" } else { "{t.acceder}" }
                }
                button {
                    class: "btn-mini",
                    onclick: move |_| {
                        es_registro.set(!es_registro());
                        aviso.set(String::new());
                    },
                    if *es_registro.read() { "{t.tengo_cuenta}" } else { "{t.sin_cuenta}" }
                }
            }
        }
    }
}

/// Vista de miembros: lista del espacio; el admin puede cambiar roles.
/// Sin sesión no se llega aquí (el guardia redirige a acceso).
#[component]
fn VistaMiembros() -> Element {
    let t = textos(IDIOMA());
    let mut miembros = use_signal(Vec::<cliente::Miembro>::new);
    let mut aviso = use_signal(String::new);
    let es_admin = cliente::SESION().map(|s| s.rol == "admin").unwrap_or(false);

    use_effect(move || {
        spawn(async move {
            match cliente::listar_miembros().await {
                Ok(lista) => miembros.set(lista),
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    });

    rsx! {
        BarraSuperior {
            titulo: t.miembros.to_string(),
            subtitulo: t.miembros_sub.to_string(),
        }
        if !aviso.read().is_empty() {
            p { class: "form-error", "{aviso}" }
        }
        div { class: "panel",
            for m in miembros.read().iter() {
                div { class: "mini-row", key: "{m.id}",
                    span { class: "avatar", "{m.nombre.chars().next().unwrap_or('·')}" }
                    span { class: "mini-name", "{m.nombre}" }
                    span { class: "mini-amt", "{m.correo} · {m.rol}" }
                    if es_admin && m.rol == "miembro" {
                        button {
                            class: "btn-mini",
                            onclick: {
                                let id = m.id;
                                move |_| {
                                    spawn(async move {
                                        match cliente::cambiar_rol(id, "admin").await {
                                            Ok(cambiado) => {
                                                aviso.set(String::new());
                                                if let Some(f) = miembros.write().iter_mut().find(|x| x.id == id) {
                                                    f.rol = cambiado.rol;
                                                }
                                            }
                                            Err(mensaje) => aviso.set(mensaje),
                                        }
                                    });
                                }
                            },
                            "{t.hacer_admin}"
                        }
                    }
                    if es_admin && m.rol == "admin" {
                        button {
                            class: "btn-mini",
                            onclick: {
                                let id = m.id;
                                move |_| {
                                    spawn(async move {
                                        match cliente::cambiar_rol(id, "miembro").await {
                                            Ok(cambiado) => {
                                                aviso.set(String::new());
                                                if let Some(f) = miembros.write().iter_mut().find(|x| x.id == id) {
                                                    f.rol = cambiado.rol;
                                                }
                                            }
                                            Err(mensaje) => aviso.set(mensaje),
                                        }
                                    });
                                }
                            },
                            "{t.quitar_admin}"
                        }
                    }
                }
            }
        }
    }
}

/// Convención horaria del frontal: los `datetime-local` se guardan y muestran
/// en UTC (van y vuelven idénticos; el colegio opera en un solo huso).
fn a_campo_local(fecha: chrono::DateTime<chrono::Utc>) -> String {
    fecha.format("%Y-%m-%dT%H:%M").to_string()
}

/// Hora corta (`15:04`) para eventos y correos.
fn fmt_hora(fecha: chrono::DateTime<chrono::Utc>) -> String {
    fecha.format("%H:%M").to_string()
}

/// Día y hora (`20/11 15:04`) para relacionados y listas.
fn fmt_dia_hora(fecha: chrono::DateTime<chrono::Utc>) -> String {
    fecha.format("%d/%m %H:%M").to_string()
}

/// Fecha completa (`20/11/2026`) para el día seleccionado.
fn fmt_fecha(fecha: chrono::NaiveDate) -> String {
    fecha.format("%d/%m/%Y").to_string()
}

/// Lee un `datetime-local` (`YYYY-MM-DDTHH:MM`) como UTC.
fn desde_campo_local(valor: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::NaiveDateTime::parse_from_str(&format!("{valor}:00"), "%Y-%m-%dT%H:%M:%S")
        .ok()
        .map(|ingenuo| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ingenuo, chrono::Utc))
}

/// Recarga los eventos del mes indicado en las señales dadas.
fn recargar_mes(
    ano: Signal<i32>,
    mes: Signal<u32>,
    mut eventos: Signal<Vec<Evento>>,
    mut aviso: Signal<String>,
) {
    let (a, m) = (ano(), mes());
    spawn(async move {
        match cliente::eventos_mes(a, m).await {
            Ok(lista) => {
                aviso.set(String::new());
                eventos.set(lista);
            }
            Err(mensaje) => aviso.set(mensaje),
        }
    });
}

/// Mueve el mes visible y deja el día 1 seleccionado.
fn mover_mes(
    delta: i32,
    mut ano: Signal<i32>,
    mut mes: Signal<u32>,
    mut dia_sel: Signal<chrono::NaiveDate>,
    hoy: chrono::NaiveDate,
) {
    let (mut a, mut m) = (ano(), mes());
    if delta < 0 {
        if m == 1 {
            a -= 1;
            m = 12;
        } else {
            m -= 1;
        }
    } else if m == 12 {
        a += 1;
        m = 1;
    } else {
        m += 1;
    }
    ano.set(a);
    mes.set(m);
    dia_sel.set(chrono::NaiveDate::from_ymd_opt(a, m, 1).unwrap_or(hoy));
}

/// Nombres de mes según idioma.
fn nombre_mes(mes: u32, idioma: Idioma) -> &'static str {
    match (mes, idioma) {
        (1, Idioma::Es) => "enero",
        (2, Idioma::Es) => "febrero",
        (3, Idioma::Es) => "marzo",
        (4, Idioma::Es) => "abril",
        (5, Idioma::Es) => "mayo",
        (6, Idioma::Es) => "junio",
        (7, Idioma::Es) => "julio",
        (8, Idioma::Es) => "agosto",
        (9, Idioma::Es) => "septiembre",
        (10, Idioma::Es) => "octubre",
        (11, Idioma::Es) => "noviembre",
        (12, Idioma::Es) => "diciembre",
        (1, Idioma::En) => "January",
        (2, Idioma::En) => "February",
        (3, Idioma::En) => "March",
        (4, Idioma::En) => "April",
        (5, Idioma::En) => "May",
        (6, Idioma::En) => "June",
        (7, Idioma::En) => "July",
        (8, Idioma::En) => "August",
        (9, Idioma::En) => "September",
        (10, Idioma::En) => "October",
        (11, Idioma::En) => "November",
        (12, Idioma::En) => "December",
        _ => "?",
    }
}

/// Vista de calendario: rejilla mensual navegable con alta/edición/baja.
#[component]
fn VistaCalendario() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let hoy = chrono::Utc::now().date_naive();
    let ano = use_signal(|| hoy.year());
    let mes = use_signal(|| hoy.month());
    let eventos = use_signal(Vec::<Evento>::new);
    let mut aviso = use_signal(String::new);
    let mut dia_sel = use_signal(|| hoy);
    let mut mostrar_form = use_signal(|| false);
    let mut editando = use_signal(|| None::<Uuid>);
    let mut f_titulo = use_signal(String::new);
    let mut f_inicio = use_signal(String::new);
    let mut f_fin = use_signal(String::new);
    let mut f_lugar = use_signal(String::new);
    let mut f_descripcion = use_signal(String::new);
    let mut f_alumno = use_signal(String::new);
    let mut f_apoderado = use_signal(String::new);
    let mut f_oportunidad = use_signal(String::new);
    let mut alumnos = use_signal(Vec::<Alumno>::new);
    let mut apoderados = use_signal(Vec::<Apoderado>::new);
    let mut oportunidades = use_signal(Vec::<Oportunidad>::new);

    let recargar = move || recargar_mes(ano, mes, eventos, aviso);
    use_effect(move || {
        recargar();
    });
    use_effect(move || {
        spawn(async move {
            if let Ok(l) = cliente::listar_alumnos("", "").await {
                alumnos.set(l);
            }
            if let Ok(l) = cliente::listar_apoderados("").await {
                apoderados.set(l);
            }
            if let Ok(l) = cliente::listar_oportunidades().await {
                oportunidades.set(l);
            }
        });
    });

    let nuevo = move |_| {
        editando.set(None);
        f_titulo.set(String::new());
        let base = dia_sel().and_hms_opt(9, 0, 0).unwrap_or(
            hoy.and_hms_opt(9, 0, 0)
                .unwrap_or(chrono::NaiveDateTime::MIN),
        );
        let inicio_utc =
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(base, chrono::Utc);
        f_inicio.set(a_campo_local(inicio_utc));
        f_fin.set(a_campo_local(
            inicio_utc + chrono::Duration::hours(1),
        ));
        f_lugar.set(String::new());
        f_descripcion.set(String::new());
        f_alumno.set(String::new());
        f_apoderado.set(String::new());
        f_oportunidad.set(String::new());
        aviso.set(String::new());
        mostrar_form.set(true);
    };

    let guardar = move |_| {
        let (inicio, fin) = (
            desde_campo_local(&f_inicio.read()),
            desde_campo_local(&f_fin.read()),
        );
        let (Some(i), Some(f)) = (inicio, fin) else {
            aviso.set(if idioma == Idioma::Es {
                "Fecha no válida.".into()
            } else {
                "Invalid date.".into()
            });
            return;
        };
        let datos = CrearEvento {
            titulo: f_titulo.read().clone(),
            inicio: i,
            fin: f,
            lugar: f_lugar.read().clone(),
            descripcion: f_descripcion.read().clone(),
            alumno_id: uuid::Uuid::parse_str(&f_alumno.read()).ok(),
            apoderado_id: uuid::Uuid::parse_str(&f_apoderado.read()).ok(),
            oportunidad_id: uuid::Uuid::parse_str(&f_oportunidad.read()).ok(),
        };
        let eid = *editando.read();
        spawn(async move {
            let resultado: Result<Evento, String> = if let Some(id) = eid {
                cliente::actualizar_evento(id, &datos).await
            } else {
                cliente::crear_evento(&datos).await
            };
            match resultado {
                Ok(_) => {
                    mostrar_form.set(false);
                    recargar();
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    // Rejilla: lunes primero; celdas vacías antes del día 1.
    let primero = chrono::NaiveDate::from_ymd_opt(ano(), mes(), 1).unwrap_or(hoy);
    let (siguiente_ano, siguiente_mes) = if mes() == 12 {
        (ano() + 1, 1)
    } else {
        (ano(), mes() + 1)
    };
    let dias_mes = chrono::NaiveDate::from_ymd_opt(siguiente_ano, siguiente_mes, 1)
        .unwrap_or(primero)
        .pred_opt()
        .unwrap_or(primero)
        .day();
    let desplazamiento = primero.weekday().num_days_from_monday() as usize;
    let dias_semana = if idioma == Idioma::Es {
        ["lun", "mar", "mié", "jue", "vie", "sáb", "dom"]
    } else {
        ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
    };

    let eventos_del_dia: Vec<Evento> = eventos
        .read()
        .iter()
        .filter(|e| e.inicio.date_naive() <= dia_sel() && e.fin.date_naive() >= dia_sel())
        .cloned()
        .collect();

    rsx! {
        BarraSuperior {
            titulo: t.calendario.to_string(),
            subtitulo: format!("{} {}", nombre_mes(mes(), idioma), ano()),
        }
        if !aviso.read().is_empty() {
            p { class: "form-error", "{aviso}" }
        }
        div { class: "panel",
            div { class: "cal-bar",
                button { class: "btn-mini", onclick: move |_| mover_mes(-1, ano, mes, dia_sel, hoy), "←" }
                span { class: "cal-titulo", "{nombre_mes(mes(), idioma)} {ano()}" }
                button { class: "btn-mini", onclick: move |_| mover_mes(1, ano, mes, dia_sel, hoy), "→" }
                button { class: "btn-mini", onclick: nuevo, "{t.nuevo_evento}" }
            }
            div { class: "cal-grid",
                for d in dias_semana {
                    div { class: "cal-nombre", "{d}" }
                }
                for i in 0..(desplazamiento + dias_mes as usize) {
                    if i < desplazamiento {
                        div { class: "cal-dia vacio" }
                    } else {
                        {
                            let dia = chrono::NaiveDate::from_ymd_opt(ano(), mes(), (i - desplazamiento + 1) as u32).unwrap_or(hoy);
                            let del_dia: Vec<(Uuid, String)> = eventos.read().iter().filter(|e| e.inicio.date_naive() <= dia && e.fin.date_naive() >= dia).take(3).map(|e| (e.id, e.titulo.clone())).collect();
                            let clase = if dia == dia_sel() { "cal-dia sel" } else { "cal-dia" };
                            rsx! {
                                div {
                                    class: "{clase}",
                                    key: "{dia}",
                                    "data-dia": "{dia}",
                                    onclick: move |_| dia_sel.set(dia),
                                    span { class: "cal-num", "{dia.day()}" }
                                    for (eid, titulo) in del_dia.iter() {
                                        div { class: "cal-punto", key: "{eid}", "{titulo}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        div { class: "panel",
            div { class: "panel-head",
                span { class: "panel-title",
                    "{fmt_fecha(dia_sel())} · {eventos_del_dia.len()}"
                }
            }
            if eventos_del_dia.is_empty() {
                p { class: "muted", "{t.sin_eventos_dia}" }
            }
            for e in eventos_del_dia {
                div { class: "mini-row", key: "{e.id}",
                    span { class: "mini-name",
                        "{fmt_hora(e.inicio)}-{fmt_hora(e.fin)} {e.titulo}"
                    }
                    span { class: "mini-amt", "{e.lugar}" }
                    button {
                        class: "btn-mini",
                        onclick: {
                            let ev = e.clone();
                            move |_| {
                                editando.set(Some(ev.id));
                                f_titulo.set(ev.titulo.clone());
                                f_inicio.set(a_campo_local(ev.inicio));
                                f_fin.set(a_campo_local(ev.fin));
                                f_lugar.set(ev.lugar.clone());
                                f_descripcion.set(ev.descripcion.clone());
                                f_alumno.set(ev.alumno_id.map(|v| v.to_string()).unwrap_or_default());
                                f_apoderado.set(ev.apoderado_id.map(|v| v.to_string()).unwrap_or_default());
                                f_oportunidad.set(ev.oportunidad_id.map(|v| v.to_string()).unwrap_or_default());
                                mostrar_form.set(true);
                            }
                        },
                        "✎"
                    }
                    button {
                        class: "btn-mini",
                        onclick: {
                            let id = e.id;
                            move |_| {
                                spawn(async move {
                                    match cliente::eliminar_evento(id).await {
                                        Ok(()) => recargar(),
                                        Err(mensaje) => aviso.set(mensaje),
                                    }
                                });
                            }
                        },
                        "✕"
                    }
                }
            }
        }
        if *mostrar_form.read() {
            div { class: "panel form",
                div { class: "campo",
                    label { "{t.nuevo_evento}" }
                    input {
                        class: "input",
                        value: "{f_titulo}",
                        oninput: move |e| f_titulo.set(e.value()),
                    }
                }
                div { class: "campo",
                    label { "{t.inicio}" }
                    input {
                        class: "input",
                        r#type: "datetime-local",
                        value: "{f_inicio}",
                        oninput: move |e| f_inicio.set(e.value()),
                    }
                }
                div { class: "campo",
                    label { "{t.fin}" }
                    input {
                        class: "input",
                        r#type: "datetime-local",
                        value: "{f_fin}",
                        oninput: move |e| f_fin.set(e.value()),
                    }
                }
                div { class: "campo",
                    label { "{t.lugar}" }
                    input {
                        class: "input",
                        value: "{f_lugar}",
                        oninput: move |e| f_lugar.set(e.value()),
                    }
                }
                div { class: "campo",
                    label { "{t.descripcion}" }
                    input {
                        class: "input",
                        value: "{f_descripcion}",
                        oninput: move |e| f_descripcion.set(e.value()),
                    }
                }
                div { class: "campo",
                    label { "{t.alumnos}" }
                    select {
                        class: "input",
                        value: "{f_alumno}",
                        onchange: move |e| f_alumno.set(e.value()),
                        option { value: "", "—" }
                        for a in alumnos.read().iter() {
                            option { value: "{a.id}", "{a.nombres} {a.apellidos}" }
                        }
                    }
                }
                div { class: "campo",
                    label { "{t.apoderados}" }
                    select {
                        class: "input",
                        value: "{f_apoderado}",
                        onchange: move |e| f_apoderado.set(e.value()),
                        option { value: "", "—" }
                        for a in apoderados.read().iter() {
                            option { value: "{a.id}", "{a.nombres} {a.apellidos}" }
                        }
                    }
                }
                div { class: "campo",
                    label { "{t.oportunidades}" }
                    select {
                        class: "input",
                        value: "{f_oportunidad}",
                        onchange: move |e| f_oportunidad.set(e.value()),
                        option { value: "", "—" }
                        for o in oportunidades.read().iter() {
                            option { value: "{o.id}", "{o.nombre}" }
                        }
                    }
                }
                div { class: "campo campo-btn",
                    button { class: "btn-primary", onclick: guardar, "{t.guardar}" }
                    button {
                        class: "btn-mini",
                        onclick: move |_| mostrar_form.set(false),
                        "✕"
                    }
                }
            }
        }
    }
}

/// Vista de correos: bandeja con no leídos, lector y redacción vinculada.
#[component]
fn VistaCorreos() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut correos = use_signal(Vec::<Correo>::new);
    let mut busqueda = use_signal(String::new);
    let mut solo_no_leidos = use_signal(|| false);
    let mut seleccionado = use_signal(|| None::<Uuid>);
    let mut aviso = use_signal(String::new);
    let mut mostrar_form = use_signal(|| false);
    let mut c_asunto = use_signal(String::new);
    let mut c_cuerpo = use_signal(String::new);
    let mut c_remitente = use_signal(String::new);
    let mut c_alumno = use_signal(String::new);
    let mut c_apoderado = use_signal(String::new);
    let mut c_oportunidad = use_signal(String::new);
    let mut alumnos = use_signal(Vec::<Alumno>::new);
    let mut apoderados = use_signal(Vec::<Apoderado>::new);
    let mut oportunidades = use_signal(Vec::<Oportunidad>::new);

    let recargar = move || {
        let (b, nl) = (busqueda.peek().clone(), solo_no_leidos());
        spawn(async move {
            match cliente::listar_correos(&b, nl).await {
                Ok(lista) => {
                    aviso.set(String::new());
                    correos.set(lista);
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };
    use_effect(move || {
        solo_no_leidos();
        recargar();
    });
    use_effect(move || {
        spawn(async move {
            if let Ok(l) = cliente::listar_alumnos("", "").await {
                alumnos.set(l);
            }
            if let Ok(l) = cliente::listar_apoderados("").await {
                apoderados.set(l);
            }
            if let Ok(l) = cliente::listar_oportunidades().await {
                oportunidades.set(l);
            }
        });
    });

    let mut abrir = move |id: Uuid| {
        seleccionado.set(Some(id));
        spawn(async move {
            match cliente::marcar_correo(id, true).await {
                Ok(marcado) => {
                    if let Some(f) = correos.write().iter_mut().find(|x| x.id == id) {
                        f.leido = marcado.leido;
                    }
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    let alternar_leido = move |_| {
        let Some(id) = *seleccionado.read() else {
            return;
        };
        let actual = correos
            .read()
            .iter()
            .find(|x| x.id == id)
            .map(|x| x.leido)
            .unwrap_or(true);
        spawn(async move {
            match cliente::marcar_correo(id, !actual).await {
                Ok(marcado) => {
                    if let Some(f) = correos.write().iter_mut().find(|x| x.id == id) {
                        f.leido = marcado.leido;
                    }
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    let eliminar = move |_| {
        let Some(id) = *seleccionado.read() else {
            return;
        };
        spawn(async move {
            match cliente::eliminar_correo(id).await {
                Ok(()) => {
                    seleccionado.set(None);
                    recargar();
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    let enviar = move |_| {
        let datos = CrearCorreo {
            asunto: c_asunto.read().clone(),
            cuerpo: c_cuerpo.read().clone(),
            remitente: c_remitente.read().clone(),
            alumno_id: uuid::Uuid::parse_str(&c_alumno.read()).ok(),
            apoderado_id: uuid::Uuid::parse_str(&c_apoderado.read()).ok(),
            oportunidad_id: uuid::Uuid::parse_str(&c_oportunidad.read()).ok(),
        };
        spawn(async move {
            match cliente::crear_correo(&datos).await {
                Ok(_) => {
                    c_asunto.set(String::new());
                    c_cuerpo.set(String::new());
                    c_remitente.set(String::new());
                    c_alumno.set(String::new());
                    c_apoderado.set(String::new());
                    c_oportunidad.set(String::new());
                    mostrar_form.set(false);
                    recargar();
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

    let en_lector: Option<Correo> = seleccionado()
        .and_then(|id| correos.read().iter().find(|x| x.id == id).cloned());

    rsx! {
        BarraSuperior {
            titulo: t.correos.to_string(),
            subtitulo: format!("{} · {}", correos.read().len(), t.correos),
        }
        if !aviso.read().is_empty() {
            p { class: "form-error", "{aviso}" }
        }
        div { class: "panel",
            div { class: "cal-bar",
                input {
                    class: "input",
                    value: "{busqueda}",
                    oninput: move |e| busqueda.set(e.value()),
                }
                button { class: "btn-mini", onclick: move |_| recargar(), "{t.filtrar}" }
                button {
                    class: if solo_no_leidos() { "btn-mini active" } else { "btn-mini" },
                    onclick: move |_| solo_no_leidos.set(!solo_no_leidos()),
                    "{t.solo_no_leidos}"
                }
                button {
                    class: "btn-mini",
                    onclick: move |_| mostrar_form.set(true),
                    "{t.redactar}"
                }
            }
            for c in correos.read().iter() {
                div {
                    class: if c.leido { "mini-row" } else { "mini-row unread" },
                    key: "{c.id}",
                    "data-correo": "{c.id}",
                    onclick: {
                        let id = c.id;
                        move |_| abrir(id)
                    },
                    span { class: "avatar", "{c.remitente.chars().next().unwrap_or('·')}" }
                    span { class: "mini-name",
                        if c.asunto.is_empty() { "(sin asunto)" } else { "{c.asunto}" }
                    }
                    span { class: "mini-amt", "{c.remitente} · {fmt_dia_hora(c.recibido_en)}" }
                }
            }
        }
        if let Some(lector) = en_lector {
            div { class: "panel",
                div { class: "panel-head",
                    span { class: "panel-title", "{lector.asunto}" }
                    span { class: "mini-amt", "{lector.remitente}" }
                }
                p { "{lector.cuerpo}" }
                p { class: "muted",
                    "{fmt_dia_hora(lector.recibido_en)}"
                }
                div { class: "cal-bar",
                    button { class: "btn-mini", onclick: alternar_leido,
                        if lector.leido { "{t.marcar_no_leido}" } else { "{t.marcar_leido}" }
                    }
                    button { class: "btn-mini", onclick: eliminar, "✕" }
                }
            }
        }
        if *mostrar_form.read() {
            div { class: "panel form",
                div { class: "campo",
                    label { "{t.asunto}" }
                    input {
                        class: "input",
                        value: "{c_asunto}",
                        oninput: move |e| c_asunto.set(e.value()),
                    }
                }
                div { class: "campo",
                    label { "{t.remitente}" }
                    input {
                        class: "input",
                        r#type: "email",
                        value: "{c_remitente}",
                        oninput: move |e| c_remitente.set(e.value()),
                    }
                }
                div { class: "campo",
                    label { "{t.descripcion}" }
                    input {
                        class: "input",
                        value: "{c_cuerpo}",
                        oninput: move |e| c_cuerpo.set(e.value()),
                    }
                }
                div { class: "campo",
                    label { "{t.alumnos}" }
                    select {
                        class: "input",
                        value: "{c_alumno}",
                        onchange: move |e| c_alumno.set(e.value()),
                        option { value: "", "—" }
                        for a in alumnos.read().iter() {
                            option { value: "{a.id}", "{a.nombres} {a.apellidos}" }
                        }
                    }
                }
                div { class: "campo",
                    label { "{t.apoderados}" }
                    select {
                        class: "input",
                        value: "{c_apoderado}",
                        onchange: move |e| c_apoderado.set(e.value()),
                        option { value: "", "—" }
                        for a in apoderados.read().iter() {
                            option { value: "{a.id}", "{a.nombres} {a.apellidos}" }
                        }
                    }
                }
                div { class: "campo campo-btn",
                    button { class: "btn-primary", onclick: enviar, "{t.guardar}" }
                    button {
                        class: "btn-mini",
                        onclick: move |_| mostrar_form.set(false),
                        "✕"
                    }
                }
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
