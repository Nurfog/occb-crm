//! Frontal OCCB en Dioxus, todo en español con i18n ES/EN.
//!
//! Buenas prácticas: textos centralizados en `occb-core::i18n`, un módulo
//! `cliente` para hablar con la API y componentes pequeños por vista.
//! Las vistas leen y escriben en la API (`:3001`); si no responde,
//! muestran datos de demostración locales.

mod cliente;

use dioxus::prelude::*;
use uuid::Uuid;
use occb_core::{
    Alumno, Apoderado, Colegio, CrearAlumno, CrearApoderado, CrearNota, CrearOportunidad,
    CrearPersona, CrearTarea, EstadoAlumno, EstadoTarea, EtapaOportunidad, Idioma, Nota,
    Oportunidad, Persona, Tarea, Traslado, calcular_resumen, datos_semilla_alumnos, datos_semilla_apoderados,
    datos_semilla_colegios, datos_semilla_notas, datos_semilla_oportunidades,
    datos_semilla_personas, datos_semilla_tareas, textos,
};

/// Hoja de estilos del frontal.
const CSS: Asset = asset!("/assets/main.css");

/// Idioma global de la interfaz (español por defecto, conmutable a inglés).
static IDIOMA: GlobalSignal<Idioma> = Signal::global(|| Idioma::Es);

/// Destino pendiente tras entrar (el guardia lo fija antes de mandar a acceso).
static VOLVER: GlobalSignal<Option<Ruta>> = Signal::global(|| None);

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
        #[route("/miembros")]
        VistaMiembros {},
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
    let nav = use_navigator();
    let sesion = cliente::SESION();
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
            div { class: "search",
                span { class: "search-icon", "⌕" }
                span { "{t.buscar}" }
                span { style: "margin-left: auto; display: flex; gap: 2px;",
                    kbd { "⌘" }
                    kbd { "K" }
                }
            }
            div { class: "side-section",
                div { class: "side-title", "{t.favoritos}" }
                Link { to: Ruta::VistaOportunidades {}, class: "side-item",
                    span { "★" }
                    span { "Q3" }
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
                                td { class: "strong", "{o.nombre}" }
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
                                    "{op.nombre}"
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
        h2 { class: "record-title", {if titulo.is_empty() { "…".to_string() } else { titulo }} }
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
        }
        }
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
        h2 { class: "record-title", {if titulo.is_empty() { "…".to_string() } else { titulo }} }
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
        h2 { class: "record-title", {if titulo.is_empty() { "…".to_string() } else { titulo }} }
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
