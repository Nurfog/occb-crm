//! Frontal OCCB en Dioxus, todo en español con i18n ES/EN.
//!
//! Buenas prácticas: textos centralizados en `occb-core::i18n`, un módulo
//! `cliente` para hablar con la API y componentes pequeños por vista.
//! Las vistas leen y escriben en la API (`:3001`); si no responde,
//! muestran datos de demostración locales.

mod cliente;

use dioxus::prelude::*;
use occb_core::{
    CrearEmpresa, CrearOportunidad, CrearPersona, Empresa, EtapaOportunidad, Idioma, Oportunidad,
    Persona, calcular_resumen, datos_semilla_empresas, datos_semilla_oportunidades,
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
        (EtapaOportunidad::Contacto, Idioma::Es) => "Contacto",
        (EtapaOportunidad::Reunion, Idioma::Es) => "Reunión",
        (EtapaOportunidad::Propuesta, Idioma::Es) => "Propuesta",
        (EtapaOportunidad::Cliente, Idioma::Es) => "Cliente",
        (EtapaOportunidad::Nueva, Idioma::En) => "New",
        (EtapaOportunidad::Contacto, Idioma::En) => "Contact",
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
                }
                Link { to: Ruta::VistaEmpresas {}, class: "side-item",
                    span { class: "ico", "□" }
                    span { "{t.empresas}" }
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

/// Carga las empresas de demostración (respaldo si la API no responde).
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

/// Recarga empresas desde la API; usa demo solo si aún no hay nada.
fn recargar_empresas(mut empresas: Signal<Vec<Empresa>>) {
    spawn(async move {
        match cliente::listar_empresas().await {
            Ok(lista) => empresas.set(lista),
            Err(_) if empresas.read().is_empty() => empresas.set(cargar_empresas_demo()),
            Err(_) => {}
        }
    });
}

/// Recarga personas desde la API; usa demo solo si aún no hay nada.
fn recargar_personas(mut personas: Signal<Vec<Persona>>, ids_empresas: Vec<uuid::Uuid>) {
    spawn(async move {
        match cliente::listar_personas().await {
            Ok(lista) => personas.set(lista),
            Err(_) if personas.read().is_empty() => {
                personas.set(cargar_personas_demo(&ids_empresas))
            }
            Err(_) => {}
        }
    });
}

/// Recarga oportunidades desde la API; usa demo solo si aún no hay nada.
fn recargar_oportunidades(
    mut oportunidades: Signal<Vec<Oportunidad>>,
    ids_empresas: Vec<uuid::Uuid>,
    ids_personas: Vec<uuid::Uuid>,
) {
    spawn(async move {
        match cliente::listar_oportunidades().await {
            Ok(lista) => oportunidades.set(lista),
            Err(_) if oportunidades.read().is_empty() => {
                if !ids_empresas.is_empty() && !ids_personas.is_empty() {
                    oportunidades.set(datos_semilla_oportunidades(&ids_empresas, &ids_personas));
                }
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
    let mut empresas = use_signal(Vec::<Empresa>::new);
    let mut personas = use_signal(Vec::<Persona>::new);
    let mut oportunidades = use_signal(Vec::<Oportunidad>::new);
    let mut en_linea = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            match cliente::listar_empresas().await {
                Ok(lista_e) => {
                    en_linea.set(true);
                    let per =
                        cliente::listar_personas().await.unwrap_or_default();
                    let opp =
                        cliente::listar_oportunidades().await.unwrap_or_default();
                    empresas.set(lista_e);
                    personas.set(per);
                    oportunidades.set(opp);
                }
                Err(_) => {
                    let demo_e = cargar_empresas_demo();
                    let ids: Vec<_> = demo_e.iter().map(|e| e.id).collect();
                    let demo_p = cargar_personas_demo(&ids);
                    let ids_p: Vec<_> = demo_p.iter().map(|p| p.id).collect();
                    let demo_o = datos_semilla_oportunidades(&ids, &ids_p);
                    empresas.set(demo_e);
                    personas.set(demo_p);
                    oportunidades.set(demo_o);
                }
            }
        });
    });

    let resumen = calcular_resumen(&empresas.read(), &personas.read(), &oportunidades.read());
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
                div { class: "card-value", "{resumen.total_empresas}" }
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

/// Vista de personas: lista viva con búsqueda, alta y baja.
#[component]
fn VistaPersonas() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut busqueda = use_signal(String::new);
    let personas = use_signal(Vec::<Persona>::new);
    let mut listo = use_signal(|| false);
    let mut aviso = use_signal(String::new);
    let mut nombre = use_signal(String::new);
    let mut apellido = use_signal(String::new);
    let mut correo = use_signal(String::new);

    use_effect(move || {
        recargar_personas(personas, vec![]);
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
            telefono: String::new(),
            puesto: String::new(),
            empresa_id: None,
            ciudad: String::new(),
        };
        spawn(async move {
            match cliente::crear_persona(&datos).await {
                Ok(_) => {
                    aviso.set(String::new());
                    nombre.set(String::new());
                    apellido.set(String::new());
                    correo.set(String::new());
                    recargar_personas(personas, vec![]);
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
                button { class: "btn-primary", onclick: crear, "{t.nueva}" }
            }
            if !aviso.read().is_empty() {
                p { class: "muted small", "{aviso}" }
            }
        }
        div { class: "panel",
            if !listo() {
                p { class: "muted", "…" }
            } else {
                table { class: "tbl",
                    thead {
                        tr {
                            th { if idioma == Idioma::Es { "Nombre" } else { "Name" } }
                            th { "{t.correo}" }
                            th { if idioma == Idioma::Es { "Puesto" } else { "Role" } }
                            th { if idioma == Idioma::Es { "Ciudad" } else { "City" } }
                            th { "" }
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
                                                        recargar_personas(personas, vec![]);
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
                }
            }
        }
    }
}

/// Vista de empresas: tarjetas vivas con alta, búsqueda y baja.
#[component]
fn VistaEmpresas() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let mut busqueda = use_signal(String::new);
    let empresas = use_signal(Vec::<Empresa>::new);
    let mut aviso = use_signal(String::new);
    let mut nombre = use_signal(String::new);
    let mut dominio = use_signal(String::new);
    let mut ciudad = use_signal(String::new);

    use_effect(move || {
        recargar_empresas(empresas);
    });

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

    let crear = move |_| {
        let datos = CrearEmpresa {
            nombre: nombre.read().clone(),
            dominio: dominio.read().clone(),
            sector: String::new(),
            empleados: 0,
            ciudad: ciudad.read().clone(),
        };
        spawn(async move {
            match cliente::crear_empresa(&datos).await {
                Ok(_) => {
                    aviso.set(String::new());
                    nombre.set(String::new());
                    dominio.set(String::new());
                    ciudad.set(String::new());
                    recargar_empresas(empresas);
                }
                Err(mensaje) => aviso.set(mensaje),
            }
        });
    };

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
        div { class: "panel form",
            label { if idioma == Idioma::Es { "Nueva empresa" } else { "New company" } }
            div { class: "row",
                input {
                    class: "input",
                    placeholder: if idioma == Idioma::Es { "Nombre *" } else { "Name *" },
                    value: "{nombre}",
                    oninput: move |e| nombre.set(e.value()),
                }
                input {
                    class: "input",
                    placeholder: "dominio.com",
                    value: "{dominio}",
                    oninput: move |e| dominio.set(e.value()),
                }
                input {
                    class: "input",
                    placeholder: if idioma == Idioma::Es { "Ciudad" } else { "City" },
                    value: "{ciudad}",
                    oninput: move |e| ciudad.set(e.value()),
                }
                button { class: "btn-primary", onclick: crear, "{t.nueva}" }
            }
            if !aviso.read().is_empty() {
                p { class: "muted small", "{aviso}" }
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
                        button {
                            class: "btn-mini",
                            title: "✕",
                            onclick: {
                                let id = e.id;
                                move |_| {
                                    spawn(async move {
                                        if cliente::eliminar_empresa(id).await.is_ok() {
                                            recargar_empresas(empresas);
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
    }
}

/// Vista kanban de oportunidades: datos vivos y ▶ que persiste en la API.
#[component]
fn VistaOportunidades() -> Element {
    let idioma = IDIOMA();
    let t = textos(idioma);
    let oportunidades = use_signal(Vec::<Oportunidad>::new);
    let mut aviso = use_signal(String::new);
    let mut nombre = use_signal(String::new);
    let mut importe_txt = use_signal(String::new);

    use_effect(move || {
        recargar_oportunidades(oportunidades, vec![], vec![]);
    });

    let total: i64 = oportunidades.read().iter().map(|o| o.importe).sum();

    let crear = move |_| {
        let datos = CrearOportunidad {
            nombre: nombre.read().clone(),
            importe: importe_txt.read().parse().unwrap_or(0),
            etapa: EtapaOportunidad::Nueva,
            empresa_id: None,
        };
        spawn(async move {
            match cliente::crear_oportunidad(&datos).await {
                Ok(_) => {
                    aviso.set(String::new());
                    nombre.set(String::new());
                    importe_txt.set(String::new());
                    recargar_oportunidades(oportunidades, vec![], vec![]);
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
        div { class: "panel form",
            label { if idioma == Idioma::Es { "Nueva oportunidad" } else { "New opportunity" } }
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
                button { class: "btn-primary", onclick: crear, "{t.nueva}" }
            }
            if !aviso.read().is_empty() {
                p { class: "muted small", "{aviso}" }
            }
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
                            div { class: "opp-foot",
                                span { class: "opp-amt", "${op.importe}" }
                                span { class: "opp-prob", "{etapa.probabilidad()}%" }
                                button {
                                    class: "btn-mini",
                                    title: if idioma == Idioma::Es { "Avanzar etapa" } else { "Advance stage" },
                                    onclick: {
                                        let id = op.id;
                                        let siguiente = op.etapa.siguiente().como_texto();
                                        move |_| {
                                            spawn(async move {
                                                match cliente::mover_etapa(id, siguiente).await {
                                                    Ok(_) => recargar_oportunidades(
                                                        oportunidades,
                                                        vec![],
                                                        vec![],
                                                    ),
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
