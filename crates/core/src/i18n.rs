//! Internacionalización ES/EN.
//! El idioma por defecto es español; el frontal permite cambiar a inglés.

/// Idioma de la interfaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Idioma {
    /// Español (por defecto).
    #[default]
    Es,
    /// Inglés.
    En,
}

impl Idioma {
    /// Código corto para cabeceras y selectores.
    pub fn codigo(&self) -> &'static str {
        match self {
            Self::Es => "es",
            Self::En => "en",
        }
    }

    /// Cambia al otro idioma (botón ES/EN del frontal).
    pub fn alternar(&self) -> Self {
        match self {
            Self::Es => Self::En,
            Self::En => Self::Es,
        }
    }
}

/// Todos los textos de la interfaz en ambos idiomas.
/// Añadir una clave aquí obliga a traducirla (buena práctica i18n).
#[derive(Debug, Clone)]
pub struct Textos {
    pub nombre_app: &'static str,
    pub eslogan: &'static str,
    pub buscar: &'static str,
    pub favoritos: &'static str,
    pub espacio_trabajo: &'static str,
    pub panel: &'static str,
    pub personas: &'static str,
    pub empresas: &'static str,
    pub oportunidades: &'static str,
    pub notas: &'static str,
    pub tareas: &'static str,
    pub automatizaciones: &'static str,
    pub ajustes: &'static str,
    pub salir: &'static str,
    pub invitar: &'static str,
    pub invitar_subtitulo: &'static str,
    pub bienvenida_panel: &'static str,
    pub valor_tuberia: &'static str,
    pub ganado_trimestre: &'static str,
    pub cuentas: &'static str,
    pub motor_api: &'static str,
    pub nueva: &'static str,
    pub filtrar: &'static str,
    pub ordenar: &'static str,
    pub tabla: &'static str,
    pub kanban: &'static str,
    pub guardar: &'static str,
    pub continuar: &'static str,
    pub acceso_titulo: &'static str,
    pub acceso_subtitulo: &'static str,
    pub correo: &'static str,
    pub contrasena: &'static str,
    pub salud_tuberia: &'static str,
    pub recientes: &'static str,
    pub ver_todo: &'static str,
}

/// Devuelve los textos para el idioma pedido.
pub fn textos(idioma: Idioma) -> Textos {
    match idioma {
        Idioma::Es => Textos {
            nombre_app: "OCCB CRM",
            eslogan: "CRM · Rust",
            buscar: "Buscar ⌘K",
            favoritos: "Favoritos",
            espacio_trabajo: "Espacio de trabajo",
            panel: "Panel",
            personas: "Personas",
            empresas: "Empresas",
            oportunidades: "Oportunidades",
            notas: "Notas",
            tareas: "Tareas",
            automatizaciones: "Automatizaciones",
            ajustes: "Ajustes",
            salir: "Cerrar sesión",
            invitar: "Invitar",
            invitar_subtitulo: "API Axum :3001 · Dioxus web",
            bienvenida_panel: "Bienvenido: así va hoy tu embudo.",
            valor_tuberia: "VALOR EN TUBERÍA",
            ganado_trimestre: "GANADO ESTE TRIMESTRE",
            cuentas: "EMPRESAS",
            motor_api: "MOTOR API",
            nueva: "+ Nueva",
            filtrar: "Filtrar ▾",
            ordenar: "Ordenar ▾",
            tabla: "Tabla",
            kanban: "Kanban",
            guardar: "Guardar",
            continuar: "Continuar →",
            acceso_titulo: "Bienvenido de nuevo",
            acceso_subtitulo: "Accede a tu espacio de trabajo OCCB",
            correo: "Correo",
            contrasena: "Contraseña",
            salud_tuberia: "Salud de la tubería",
            recientes: "Empresas recientes",
            ver_todo: "Ver todo →",
        },
        Idioma::En => Textos {
            nombre_app: "OCCB CRM",
            eslogan: "Rust CRM",
            buscar: "Search ⌘K",
            favoritos: "Favorites",
            espacio_trabajo: "Workspace",
            panel: "Dashboard",
            personas: "People",
            empresas: "Companies",
            oportunidades: "Opportunities",
            notas: "Notes",
            tareas: "Tasks",
            automatizaciones: "Workflows",
            ajustes: "Settings",
            salir: "Sign out",
            invitar: "Invite",
            invitar_subtitulo: "Axum API :3001 · Dioxus web",
            bienvenida_panel: "Welcome back — here is your pipeline today.",
            valor_tuberia: "PIPELINE VALUE",
            ganado_trimestre: "WON THIS QUARTER",
            cuentas: "COMPANIES",
            motor_api: "API ENGINE",
            nueva: "+ New",
            filtrar: "Filter ▾",
            ordenar: "Sort ▾",
            tabla: "Table",
            kanban: "Kanban",
            guardar: "Save",
            continuar: "Continue →",
            acceso_titulo: "Welcome back",
            acceso_subtitulo: "Sign in to your OCCB workspace",
            correo: "Email",
            contrasena: "Password",
            salud_tuberia: "Pipeline health",
            recientes: "Recent companies",
            ver_todo: "View all →",
        },
    }
}
