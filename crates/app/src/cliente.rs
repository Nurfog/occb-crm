//! Cliente HTTP del frontal hacia la API OCCB.
//! Todas las operaciones devuelven `Result<_, String>` con mensajes en español.
//! La API es la única que habla con PostgreSQL (vía funciones seguras).
//!
//! Sesión solo en memoria (`SESION`, nunca se persiste): al recargar la página
//! hay que entrar de nuevo. Cada petición lleva `Bearer`; ante un 401 se
//! renueva una vez y se reintenta.

use dioxus::prelude::{GlobalSignal, Signal};
use occb_core::{
    Alumno, Apoderado, Colegio, Correo, CrearAlumno, CrearApoderado, CrearColegio, CrearCorreo,
    CrearEvento, CrearNota, CrearOportunidad, CrearPersona, CrearTarea, Evento, Nota, Oportunidad,
    Persona, ResumenPanel, Tarea, Traslado,
};
use uuid::Uuid;

/// Sesión en memoria: acceso corto + renovación opaca + identidad.
#[derive(Debug, Clone)]
pub struct Sesion {
    pub acceso: String,
    pub sesion_id: Uuid,
    pub renovacion: String,
    pub nombre: String,
    #[allow(dead_code)]
    pub correo: String,
    pub rol: String,
}

/// Sesión global (opcional): `None` = sin entrar.
pub static SESION: GlobalSignal<Option<Sesion>> = Signal::global(|| None);

/// Miembro del espacio (lista y cambio de rol).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Miembro {
    pub id: Uuid,
    pub nombre: String,
    pub correo: String,
    pub rol: String,
}

/// Respuesta de registro/login/renovación.
#[derive(Debug, serde::Deserialize)]
struct RespuestaSesion {
    acceso: String,
    sesion_id: Uuid,
    renovacion: String,
    usuario: UsuarioSesion,
}

/// Usuario que trae la respuesta de sesión.
#[derive(Debug, serde::Deserialize)]
struct UsuarioSesion {
    nombre: String,
    correo: String,
    rol: String,
}

/// Crea una cuenta (el primero del espacio nace admin) y guarda la sesión.
pub async fn registrar(nombre: &str, correo: &str, clave: &str) -> Result<(), String> {
    let respuesta: RespuestaSesion = enviar_json(
        "POST",
        "/api/acceso/registro",
        &serde_json::json!({ "nombre": nombre, "correo": correo, "clave": clave }),
    )
    .await?;
    guardar_sesion(respuesta);
    Ok(())
}

/// Entra con correo y clave y guarda la sesión.
pub async fn iniciar_sesion(correo: &str, clave: &str) -> Result<(), String> {
    let respuesta: RespuestaSesion = enviar_json(
        "POST",
        "/api/acceso/login",
        &serde_json::json!({ "correo": correo, "clave": clave }),
    )
    .await?;
    guardar_sesion(respuesta);
    Ok(())
}

/// Cierra la sesión en la API y la olvida en memoria (siempre limpia).
pub async fn cerrar_sesion() -> Result<(), String> {
    if let Some(s) = SESION() {
        let _: serde_json::Value = enviar_json(
            "POST",
            "/api/acceso/salir",
            &serde_json::json!({ "sesion_id": s.sesion_id }),
        )
        .await
        .unwrap_or(serde_json::Value::Null);
    }
    *SESION.write() = None;
    Ok(())
}

/// Lista los miembros del espacio (exige sesión).
pub async fn listar_miembros() -> Result<Vec<Miembro>, String> {
    obtener_json("/api/miembros").await
}

/// Cambia el rol de un miembro (solo admin; 403 si no lo es).
pub async fn cambiar_rol(id: Uuid, rol: &str) -> Result<Miembro, String> {
    enviar_json(
        "PATCH",
        &format!("/api/miembros/{id}/rol"),
        &serde_json::json!({ "rol": rol }),
    )
    .await
}

/// Guarda la sesión de una respuesta de acceso.
fn guardar_sesion(respuesta: RespuestaSesion) {
    *SESION.write() = Some(Sesion {
        acceso: respuesta.acceso,
        sesion_id: respuesta.sesion_id,
        renovacion: respuesta.renovacion,
        nombre: respuesta.usuario.nombre,
        correo: respuesta.usuario.correo,
        rol: respuesta.usuario.rol,
    });
}

/// Renueva el acceso con la renovación guardada; si falla, olvida la sesión.
async fn renovar_sesion() -> Result<(), String> {
    let (sid, ren) = SESION()
        .map(|s| (s.sesion_id, s.renovacion))
        .ok_or_else(|| "Accede de nuevo para continuar.".to_string())?;
    let respuesta = reqwest::Client::new()
        .post(format!("{}{}", base(), "/api/acceso/renovar"))
        .json(&serde_json::json!({ "sesion_id": sid, "renovacion": ren }))
        .send()
        .await
        .map_err(|_| "No se pudo conectar con la API. ¿Está en marcha?".to_string())?;
    if !respuesta.status().is_success() {
        *SESION.write() = None;
        return Err("Tu sesión expiró. Accede de nuevo.".to_string());
    }
    let renovada: RespuestaSesion = respuesta
        .json()
        .await
        .map_err(|_| "La API devolvió una respuesta inesperada.".to_string())?;
    if let Some(s) = SESION.write().as_mut() {
        s.acceso = renovada.acceso;
        s.renovacion = renovada.renovacion;
    }
    Ok(())
}

/// URL base de la API, inyectada en compilación (`API_BASE_URL`).
/// Por defecto apunta al desarrollo local.
fn base() -> String {
    option_env!("API_BASE_URL")
        .unwrap_or("http://127.0.0.1:3001")
        .to_string()
}

/// Ejecuta una petición con `Bearer` si hay sesión; ante un 401 renueva
/// una vez y reintenta. Sin sesión, la API responde 401 en rutas protegidas.
async fn ejecutar(
    metodo: &str,
    ruta: &str,
    cuerpo: Option<serde_json::Value>,
) -> Result<reqwest::Response, String> {
    for intento in 0..2 {
        let cliente = reqwest::Client::new();
        let url = format!("{}{ruta}", base());
        let mut peticion = match metodo {
            "GET" => cliente.get(&url),
            "POST" => cliente.post(&url),
            "PATCH" => cliente.patch(&url),
            "DELETE" => cliente.delete(&url),
            _ => return Err("Método no soportado por el cliente.".to_string()),
        };
        if let Some(token) = SESION().map(|s| s.acceso) {
            peticion = peticion.header("Authorization", format!("Bearer {token}"));
        }
        if let Some(c) = cuerpo.clone() {
            peticion = peticion.json(&c);
        }
        let respuesta = peticion
            .send()
            .await
            .map_err(|_| "No se pudo conectar con la API. ¿Está en marcha?".to_string())?;
        if respuesta.status() == reqwest::StatusCode::UNAUTHORIZED && intento == 0 {
            renovar_sesion().await?;
            continue;
        }
        return Ok(respuesta);
    }
    Err("No se pudo completar la petición.".to_string())
}

/// GET que espera JSON; traduce errores HTTP al mensaje en español de la API.
async fn obtener_json<T>(ruta: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    leer_cuerpo(ejecutar("GET", ruta, None).await?).await
}

/// Lee el cuerpo esperando éxito; si la API responde error, usa su `mensaje`.
async fn leer_cuerpo<T>(respuesta: reqwest::Response) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    if respuesta.status().is_success() {
        respuesta
            .json::<T>()
            .await
            .map_err(|_| "La API devolvió una respuesta inesperada.".to_string())
    } else {
        let cuerpo: serde_json::Value = respuesta.json().await.unwrap_or_default();
        Err(cuerpo
            .get("mensaje")
            .and_then(|m| m.as_str())
            .unwrap_or("La API devolvió un error.")
            .to_string())
    }
}

/// Envía JSON (POST/PATCH) y lee la respuesta con el mismo criterio.
async fn enviar_json<T>(metodo: &str, ruta: &str, cuerpo: &impl serde::Serialize) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let valor =
        serde_json::to_value(cuerpo).map_err(|_| "No se pudo armar la petición.".to_string())?;
    leer_cuerpo(ejecutar(metodo, ruta, Some(valor)).await?).await
}

/// DELETE que solo exige éxito (el 204 no trae cuerpo; el mensaje solo se lee si falla).
async fn borrar(ruta: &str) -> Result<(), String> {
    let respuesta = ejecutar("DELETE", ruta, None).await?;
    if respuesta.status().is_success() {
        Ok(())
    } else {
        let cuerpo: serde_json::Value = respuesta.json().await.unwrap_or_default();
        Err(cuerpo
            .get("mensaje")
            .and_then(|m| m.as_str())
            .unwrap_or("La API devolvió un error.")
            .to_string())
    }
}

/// Resumen del panel calculado en PostgreSQL.
pub async fn obtener_panel() -> Result<ResumenPanel, String> {
    obtener_json("/api/panel").await
}

/// Lista colegios del directorio con filtros (región, comuna, texto).
pub async fn listar_colegios(
    region: Option<i16>,
    comuna: &str,
    busqueda: &str,
) -> Result<Vec<Colegio>, String> {
    let mut ruta = String::from("/api/colegios?limite=100");
    if let Some(r) = region {
        ruta.push_str(&format!("&region={r}"));
    }
    if !comuna.trim().is_empty() {
        ruta.push_str(&format!("&comuna={}", comuna.trim()));
    }
    if !busqueda.trim().is_empty() {
        ruta.push_str(&format!("&busqueda={}", busqueda.trim()));
    }
    obtener_json(&ruta).await
}

/// Un colegio por RBD.
pub async fn obtener_colegio(rbd: &str) -> Result<Colegio, String> {
    obtener_json(&format!("/api/colegios/{rbd}")).await
}

/// Registra un colegio manualmente (lo usa el flujo de traslados, Fase E3).
#[allow(dead_code)]
pub async fn crear_colegio(datos: &CrearColegio) -> Result<Colegio, String> {
    enviar_json("POST", "/api/colegios", datos).await
}

/// Elimina un colegio manual del directorio (lo usa el flujo de traslados, Fase E3).
#[allow(dead_code)]
pub async fn eliminar_colegio(rbd: &str) -> Result<(), String> {
    borrar(&format!("/api/colegios/{rbd}")).await
}

/// Lista alumnos con filtros.
pub async fn listar_alumnos(busqueda: &str, estado: &str) -> Result<Vec<Alumno>, String> {
    let mut ruta = String::from("/api/alumnos?limite=100");
    if !busqueda.trim().is_empty() {
        ruta.push_str(&format!("&busqueda={}", busqueda.trim()));
    }
    if !estado.trim().is_empty() {
        ruta.push_str(&format!("&estado={}", estado.trim()));
    }
    obtener_json(&ruta).await
}

/// Un alumno por id.
pub async fn obtener_alumno(id: Uuid) -> Result<Alumno, String> {
    obtener_json(&format!("/api/alumnos/{id}")).await
}

/// Matricula un alumno (valida RUT en API y PG).
pub async fn crear_alumno(datos: &CrearAlumno) -> Result<Alumno, String> {
    enviar_json("POST", "/api/alumnos", datos).await
}

/// Actualiza un alumno.
pub async fn actualizar_alumno(id: Uuid, datos: &CrearAlumno) -> Result<Alumno, String> {
    enviar_json("PATCH", &format!("/api/alumnos/{id}"), datos).await
}

/// Elimina un alumno (desvincula en cascada).
pub async fn eliminar_alumno(id: Uuid) -> Result<(), String> {
    borrar(&format!("/api/alumnos/{id}")).await
}

/// Apoderado vinculado con su vínculo y permisos (algunos campos se muestran
/// en detalles futuros).
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApoderadoVinculado {
    pub id: Uuid,
    pub rut: String,
    pub nombres: String,
    pub apellidos: String,
    pub telefono: String,
    pub correo: String,
    pub vinculo: String,
    pub retiro_autorizado: bool,
    pub notificaciones: bool,
}

impl ApoderadoVinculado {
    pub fn nombre_completo(&self) -> String {
        format!("{} {}", self.nombres, self.apellidos)
    }
}

/// Apoderados vinculados a un alumno.
pub async fn apoderados_de_alumno(id: Uuid) -> Result<Vec<ApoderadoVinculado>, String> {
    obtener_json(&format!("/api/alumnos/{id}/apoderados")).await
}

/// Traslados (historial) de un alumno.
pub async fn traslados_de_alumno(id: Uuid) -> Result<Vec<Traslado>, String> {
    obtener_json(&format!("/api/alumnos/{id}/traslados")).await
}

/// Traslada un alumno a otro colegio registrando el historial.
pub async fn trasladar_alumno(id: Uuid, destino_rbd: &str, motivo: &str) -> Result<Alumno, String> {
    enviar_json(
        "POST",
        &format!("/api/alumnos/{id}/traslados"),
        &serde_json::json!({ "destino_rbd": destino_rbd, "motivo": motivo }),
    )
    .await
}
/// Vincula un apoderado existente a un alumno.
pub async fn vincular_apoderado(
    alumno_id: Uuid,
    apoderado_id: Uuid,
    vinculo: &str,
) -> Result<(), String> {
    let _: serde_json::Value = enviar_json(
        "POST",
        &format!("/api/alumnos/{alumno_id}/apoderados"),
        &serde_json::json!({ "apoderado_id": apoderado_id, "vinculo": vinculo }),
    )
    .await?;
    Ok(())
}

/// Desvincula un apoderado de un alumno.
pub async fn desvincular_apoderado(alumno_id: Uuid, apoderado_id: Uuid) -> Result<(), String> {
    borrar(&format!("/api/alumnos/{alumno_id}/apoderados/{apoderado_id}")).await
}

/// Lista apoderados.
pub async fn listar_apoderados(busqueda: &str) -> Result<Vec<Apoderado>, String> {
    let ruta = if busqueda.trim().is_empty() {
        "/api/apoderados?limite=100".to_string()
    } else {
        format!("/api/apoderados?busqueda={}&limite=100", busqueda.trim())
    };
    obtener_json(&ruta).await
}

/// Un apoderado por id.
pub async fn obtener_apoderado(id: Uuid) -> Result<Apoderado, String> {
    obtener_json(&format!("/api/apoderados/{id}")).await
}

/// Registra un apoderado (valida RUT en API y PG).
pub async fn crear_apoderado(datos: &CrearApoderado) -> Result<Apoderado, String> {
    enviar_json("POST", "/api/apoderados", datos).await
}

/// Actualiza un apoderado.
pub async fn actualizar_apoderado(id: Uuid, datos: &CrearApoderado) -> Result<Apoderado, String> {
    enviar_json("PATCH", &format!("/api/apoderados/{id}"), datos).await
}

/// Elimina un apoderado (desvincula en cascada).
pub async fn eliminar_apoderado(id: Uuid) -> Result<(), String> {
    borrar(&format!("/api/apoderados/{id}")).await
}

/// Alumno vinculado con su vínculo (algunos campos se muestran en detalles futuros).
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AlumnoVinculado {
    pub id: Uuid,
    pub rut: String,
    pub nombres: String,
    pub apellidos: String,
    pub curso: String,
    pub estado: String,
    pub vinculo: String,
}

impl AlumnoVinculado {
    pub fn nombre_completo(&self) -> String {
        format!("{} {}", self.nombres, self.apellidos)
    }
}

/// Alumnos vinculados a un apoderado.
pub async fn alumnos_de_apoderado(id: Uuid) -> Result<Vec<AlumnoVinculado>, String> {
    obtener_json(&format!("/api/apoderados/{id}/alumnos")).await
}

/// Lista de personas desde `listar_personas()`.
pub async fn listar_personas() -> Result<Vec<Persona>, String> {
    obtener_json("/api/personas?limite=100").await
}

/// Una persona por id (`obtener_persona()`).
pub async fn obtener_persona(id: Uuid) -> Result<Persona, String> {
    obtener_json(&format!("/api/personas/{id}")).await
}

/// Crea una persona vía `crear_persona()`.
pub async fn crear_persona(datos: &CrearPersona) -> Result<Persona, String> {
    enviar_json("POST", "/api/personas", datos).await
}

/// Elimina una persona vía `eliminar_persona()`.
pub async fn eliminar_persona(id: Uuid) -> Result<(), String> {
    borrar(&format!("/api/personas/{id}")).await
}

/// Actualiza una persona vía `actualizar_persona()`.
pub async fn actualizar_persona(id: Uuid, datos: &CrearPersona) -> Result<Persona, String> {
    enviar_json("PATCH", &format!("/api/personas/{id}"), datos).await
}

/// Lista de oportunidades desde `listar_oportunidades()`.
pub async fn listar_oportunidades() -> Result<Vec<Oportunidad>, String> {
    obtener_json("/api/oportunidades?limite=200").await
}

/// Crea una oportunidad vía `crear_oportunidad()`.
pub async fn crear_oportunidad(datos: &CrearOportunidad) -> Result<Oportunidad, String> {
    enviar_json("POST", "/api/oportunidades", datos).await
}

/// Actualiza una oportunidad vía `actualizar_oportunidad()`.
pub async fn actualizar_oportunidad(id: Uuid, datos: &CrearOportunidad) -> Result<Oportunidad, String> {
    enviar_json("PATCH", &format!("/api/oportunidades/{id}"), datos).await
}

/// Mueve una oportunidad de etapa vía `mover_etapa_oportunidad()`.
/// `etapa` es el texto que guarda PG: contacto, postulante, evaluado, aceptado o matriculado.
pub async fn mover_etapa(id: Uuid, etapa: &str) -> Result<Oportunidad, String> {
    enviar_json(
        "PATCH",
        &format!("/api/oportunidades/{id}/etapa"),
        &serde_json::json!({ "etapa": etapa }),
    )
    .await
}

/// Lista tareas con búsqueda y estado opcionales.
pub async fn listar_tareas(busqueda: &str, estado: &str) -> Result<Vec<Tarea>, String> {
    let mut ruta = String::from("/api/tareas?limite=100");
    if !busqueda.trim().is_empty() {
        ruta.push_str(&format!("&busqueda={}", busqueda.trim()));
    }
    if !estado.trim().is_empty() {
        ruta.push_str(&format!("&estado={}", estado.trim()));
    }
    obtener_json(&ruta).await
}

/// Crea una tarea vía `crear_tarea()`.
pub async fn crear_tarea(datos: &CrearTarea) -> Result<Tarea, String> {
    enviar_json("POST", "/api/tareas", datos).await
}

/// Avanza una tarea de estado vía `marcar_tarea()`.
pub async fn marcar_tarea(id: Uuid, estado: &str) -> Result<Tarea, String> {
    enviar_json(
        "PATCH",
        &format!("/api/tareas/{id}/estado"),
        &serde_json::json!({ "estado": estado }),
    )
    .await
}

/// Elimina una tarea.
pub async fn eliminar_tarea(id: Uuid) -> Result<(), String> {
    borrar(&format!("/api/tareas/{id}")).await
}

/// Tareas vinculadas a un alumno (para su ficha).
pub async fn tareas_de_alumno(alumno_id: Uuid) -> Result<Vec<Tarea>, String> {
    let todas = listar_tareas("", "").await?;
    Ok(todas.into_iter().filter(|t| t.alumno_id == Some(alumno_id)).collect())
}

/// Lista notas con búsqueda opcional.
pub async fn listar_notas(busqueda: &str) -> Result<Vec<Nota>, String> {
    let ruta = if busqueda.trim().is_empty() {
        "/api/notas?limite=100".to_string()
    } else {
        format!("/api/notas?busqueda={}&limite=100", busqueda.trim())
    };
    obtener_json(&ruta).await
}

/// Crea una nota vía `crear_nota()`.
pub async fn crear_nota(datos: &CrearNota) -> Result<Nota, String> {
    enviar_json("POST", "/api/notas", datos).await
}

/// Actualiza una nota.
pub async fn actualizar_nota(id: Uuid, datos: &CrearNota) -> Result<Nota, String> {
    enviar_json("PATCH", &format!("/api/notas/{id}"), datos).await
}

/// Elimina una nota.
pub async fn eliminar_nota(id: Uuid) -> Result<(), String> {
    borrar(&format!("/api/notas/{id}")).await
}

/// Notas vinculadas a un alumno (para su ficha).
pub async fn notas_de_alumno(alumno_id: Uuid) -> Result<Vec<Nota>, String> {
    let todas = listar_notas("").await?;
    Ok(todas.into_iter().filter(|n| n.alumno_id == Some(alumno_id)).collect())
}

/// No leídos global (contador del menú; lo actualizan correos y barra).
pub static NO_LEIDOS: GlobalSignal<i64> = Signal::global(|| 0);

/// Recalcula el contador de no leídos desde la API.
pub async fn refrescar_no_leidos() {
    if let Ok(lista) = listar_correos("", true).await {
        *NO_LEIDOS.write() = lista.len() as i64;
    }
}

/// Lista eventos con búsqueda opcional.
pub async fn listar_eventos(busqueda: &str) -> Result<Vec<Evento>, String> {
    let ruta = if busqueda.trim().is_empty() {
        "/api/eventos?limite=100".to_string()
    } else {
        format!("/api/eventos?busqueda={}&limite=100", busqueda.trim())
    };
    obtener_json(&ruta).await
}

/// Eventos que tocan el mes indicado (para la rejilla del calendario).
pub async fn eventos_mes(ano: i32, mes: u32) -> Result<Vec<Evento>, String> {
    obtener_json(&format!("/api/eventos/mes?ano={ano}&mes={mes}")).await
}

/// Crea un evento vía `crear_evento()`.
pub async fn crear_evento(datos: &CrearEvento) -> Result<Evento, String> {
    enviar_json("POST", "/api/eventos", datos).await
}

/// Actualiza un evento.
pub async fn actualizar_evento(id: Uuid, datos: &CrearEvento) -> Result<Evento, String> {
    enviar_json("PATCH", &format!("/api/eventos/{id}"), datos).await
}

/// Elimina un evento.
pub async fn eliminar_evento(id: Uuid) -> Result<(), String> {
    borrar(&format!("/api/eventos/{id}")).await
}

/// Eventos vinculados a un alumno (para su ficha).
pub async fn eventos_de_alumno(alumno_id: Uuid) -> Result<Vec<Evento>, String> {
    let todos = listar_eventos("").await?;
    Ok(todos.into_iter().filter(|e| e.alumno_id == Some(alumno_id)).collect())
}

/// Eventos vinculados a un apoderado (para su ficha).
pub async fn eventos_de_apoderado(apoderado_id: Uuid) -> Result<Vec<Evento>, String> {
    let todos = listar_eventos("").await?;
    Ok(todos
        .into_iter()
        .filter(|e| e.apoderado_id == Some(apoderado_id))
        .collect())
}

/// Lista correos con búsqueda y filtro de no leídos opcionales.
pub async fn listar_correos(busqueda: &str, no_leidos: bool) -> Result<Vec<Correo>, String> {
    let mut ruta = String::from("/api/correos?limite=100");
    if no_leidos {
        ruta.push_str("&no_leidos=true");
    }
    if !busqueda.trim().is_empty() {
        ruta.push_str(&format!("&busqueda={}", busqueda.trim()));
    }
    obtener_json(&ruta).await
}

/// Registra un correo vía `crear_correo()`.
pub async fn crear_correo(datos: &CrearCorreo) -> Result<Correo, String> {
    let creado: Correo = enviar_json("POST", "/api/correos", datos).await?;
    *NO_LEIDOS.write() = NO_LEIDOS() + 1;
    Ok(creado)
}

/// Marca un correo como leído (o no leído) y ajusta el contador.
pub async fn marcar_correo(id: Uuid, leido: bool) -> Result<Correo, String> {
    let marcado: Correo = enviar_json(
        "PATCH",
        &format!("/api/correos/{id}/leido"),
        &serde_json::json!({ "leido": leido }),
    )
    .await?;
    refrescar_no_leidos().await;
    Ok(marcado)
}

/// Elimina un correo.
pub async fn eliminar_correo(id: Uuid) -> Result<(), String> {
    borrar(&format!("/api/correos/{id}")).await?;
    refrescar_no_leidos().await;
    Ok(())
}

/// Correos vinculados a un alumno (para su ficha).
pub async fn correos_de_alumno(alumno_id: Uuid) -> Result<Vec<Correo>, String> {
    let todos = listar_correos("", false).await?;
    Ok(todos.into_iter().filter(|c| c.alumno_id == Some(alumno_id)).collect())
}

/// Correos vinculados a un apoderado (para su ficha).
pub async fn correos_de_apoderado(apoderado_id: Uuid) -> Result<Vec<Correo>, String> {
    let todos = listar_correos("", false).await?;
    Ok(todos
        .into_iter()
        .filter(|c| c.apoderado_id == Some(apoderado_id))
        .collect())
}
