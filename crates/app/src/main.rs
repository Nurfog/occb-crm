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
    Colegio, CrearOportunidad, CrearPersona, EtapaOportunidad, Idioma, Oportunidad,
    Persona, calcular_resumen, datos_semilla_colegios, datos_semilla_oportunidades,
    datos_semilla_personas, textos,
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
        #[route("/personas")]
        VistaPersonas {},
        #[route("/personas/:id")]
        DetallePersona { id: Uuid },
        #[route("/colegios/:rbd")]
        DetalleColegio { rbd: String },
        #[route("/oportunidades")]
        VistaOportunidades {},
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
                Link { to: Ruta::VistaPersonas {}, class: "side-item",
                    span { class: "ico ico-personas", "○" }
                    span { "{t.personas}" }
                }
                Link { to: Ruta::VistaOportunidades {}, class: "side-item active",
                    span { class: "ico ico-admision", "◇" }
                    span { "{t.oportunidades}" }
                }
                div { class: "side-item dim",
                    span { class: "ico ico-notas", "☰" }
                    span { "{t.notas}" }
                }
                div { class: "side-item dim",
                    span { class: "ico ico-tareas", "✓" }
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

/// Pantalla de acceso (demo sin autenticación real en esta fase).
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
