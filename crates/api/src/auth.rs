//! Acceso con JWT: claves argon2, sesiones con rotación y middleware.
//!
//! - Acceso corto (15 min) en `Authorization: Bearer`.
//! - Renovación opaca en cookie `HttpOnly` (sha256 en PG, rota en cada uso).
//! - El secreto sale de `APP_SECRETO` (con valor de desarrollo si falta).

use std::{collections::{HashMap,VecDeque},net::SocketAddr,sync::{Mutex,OnceLock}};
use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::{ConnectInfo,Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};

use crate::errores::ErrorApi;

/// Segundos de vida del token de acceso (15 minutos).
const VIDA_ACCESO_SEGS: u64 = 15 * 60;
/// Días de vida de la sesión renovable.
const VIDA_SESION_DIAS: i32 = 30;
/// Nombre de la cookie de renovación.
const COOKIE_RENOVAR: &str = "occb_renovar";

/// Reclamos del JWT de acceso.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Reclamos {
    sub: String,
    nombre: String,
    rol: String,
    exp: u64,
    /// Identificador único: dos accesos nunca comparten el token.
    jti: String,
}

/// Usuario autenticado que el middleware deja en las extensiones.
#[derive(Debug, Clone)]
pub struct UsuarioActual {
    pub id: uuid::Uuid,
    pub nombre: String,
    pub rol: String,
    /// `None` para JWT; una clave API lleva alcances explícitos.
    pub alcances: Option<Vec<String>>,
}

fn ahora_segs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Secreto JWT (de entorno o desarrollo).
fn secreto() -> &'static [u8] {
    static SECRETO: OnceLock<Vec<u8>> = OnceLock::new();
    SECRETO.get_or_init(|| {
        std::env::var("APP_SECRETO")
            .map(|s| s.into_bytes())
            .unwrap_or_else(|_| {
                tracing::warn!("APP_SECRETO sin definir: usando secreto de desarrollo");
                b"occb-desarrollo-cambieme".to_vec()
            })
    })
    .as_slice()
}

/// Hash argon2 de una clave en claro.
pub fn hash_clave(clave: &str) -> Result<String, ErrorApi> {
    let sal = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(clave.as_bytes(), &sal)
        .map(|h| h.to_string())
        .map_err(|_| ErrorApi::Validacion("No se pudo procesar la clave.".into()))
}

/// Comprueba una clave contra su hash.
pub fn verificar_clave(clave: &str, hash: &str) -> bool {
    let Ok(analizado) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(clave.as_bytes(), &analizado)
        .is_ok()
}

/// sha256 hexadecimal (para guardar tokens de renovación, nunca en claro).
fn huella(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

/// Memoria local de intentos por IP: máximo 60 solicitudes de login por minuto.
fn login_limitado(ip: std::net::IpAddr) -> bool {
    static INTENTOS: OnceLock<Mutex<HashMap<std::net::IpAddr,VecDeque<SystemTime>>>> = OnceLock::new();
    let ahora=SystemTime::now();
    let mut mapa=INTENTOS.get_or_init(||Mutex::new(HashMap::new())).lock().unwrap_or_else(|e|e.into_inner());
    let cola=mapa.entry(ip).or_default();
    while cola.front().is_some_and(|t|ahora.duration_since(*t).unwrap_or_default().as_secs()>=60){cola.pop_front();}
    if cola.len()>=60{return true;}
    cola.push_back(ahora);
    false
}

/// Emite un JWT de acceso para el usuario.
fn emitir_acceso(id: &uuid::Uuid, nombre: &str, rol: &str) -> Result<String, ErrorApi> {
    let reclamos = Reclamos {
        sub: id.to_string(),
        nombre: nombre.into(),
        rol: rol.into(),
        exp: ahora_segs() + VIDA_ACCESO_SEGS,
        jti: uuid::Uuid::new_v4().to_string(),
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &reclamos,
        &jsonwebtoken::EncodingKey::from_secret(secreto()),
    )
    .map_err(|_| ErrorApi::Validacion("No se pudo emitir la sesión.".into()))
}

/// Decodifica y valida un JWT de acceso.
fn leer_acceso(token: &str) -> Result<UsuarioActual, ErrorApi> {
    let datos = jsonwebtoken::decode::<Reclamos>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secreto()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| ErrorApi::NoEncontrado("Sesión inválida o expirada.".into()))?;
    let id = datos
        .claims
        .sub
        .parse()
        .map_err(|_| ErrorApi::NoEncontrado("Sesión inválida.".into()))?;
    Ok(UsuarioActual {
        id,
        nombre: datos.claims.nombre,
        rol: datos.claims.rol,
        alcances: None,
    })
}

/// Middleware que exige `Authorization: Bearer <jwt>` válido.
pub async fn exigir_sesion(
    State(pool): State<sqlx::PgPool>,
    request: Request,
    next: Next,
) -> Response {
    let (mut partes, cuerpo) = request.into_parts();
    let bearer = partes
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    let usuario = if let Some(token) = bearer {
        leer_acceso(token.trim())
    } else if let Some(clave) = partes.headers.get("x-clave-api").and_then(|v| v.to_str().ok()) {
        use sqlx::Row;
        let hash = format!("{:x}", Sha256::digest(clave.trim().as_bytes()));
        match sqlx::query("select * from validar_clave_api($1)")
            .bind(hash)
            .fetch_optional(&pool)
            .await
        {
            Ok(Some(f)) => Ok(UsuarioActual {
                id: f.get("usuario_id"),
                nombre: f.get("nombre"),
                rol: f.get("rol"),
                alcances: Some(f.get("alcances")),
            }),
            Ok(None) => Err(ErrorApi::NoEncontrado("Clave API inválida o revocada.".into())),
            Err(e) => {
                tracing::error!(?e, "No se pudo validar clave API");
                Err(ErrorApi::NoEncontrado("Clave API inválida.".into()))
            }
        }
    } else {
        Err(ErrorApi::NoEncontrado("Falta autorización.".into()))
    };
    match usuario {
        Ok(usuario) => {
            if let Some(alcances) = &usuario.alcances
                && !alcances.iter().any(|a| a == "*" || a == "escritura")
                && !matches!(partes.method.clone(), axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS)
            {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"codigo":"prohibido","mensaje":"La clave API no tiene alcance de escritura."})),
                ).into_response();
            }
            partes.extensions.insert(usuario);
            next.run(Request::from_parts(partes, cuerpo)).await
        }
        Err(ErrorApi::NoEncontrado(mensaje)) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"codigo": "sin_sesion", "mensaje": mensaje})),
        )
            .into_response(),
        Err(otro) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"codigo": "no_autorizado", "mensaje": otro.to_string()})),
        )
            .into_response(),
    }
}

/// Cabecera que borra la cookie de renovación.
fn cookie_borrar() -> String {
    format!("{COOKIE_RENOVAR}=; HttpOnly; SameSite=Lax; Path=/api/acceso; Max-Age=0")
}

/// Lee el token de renovación de la cookie (si viene).
fn leer_cookie(headers: &axum::http::HeaderMap, sesion_id: &uuid::Uuid) -> Option<(uuid::Uuid, String)> {
    let cruda = headers
        .get_all(header::COOKIE)
        .into_iter()
        .filter_map(|v| v.to_str().ok())
        .collect::<Vec<_>>()
        .join("; ");
    // Formato propio: <sesion_id>.<token>
    for parte in cruda.split(';') {
        let parte = parte.trim();
        let (nombre, valor) = parte.split_once('=')?;
        if nombre.trim() == COOKIE_RENOVAR {
            let (sid, token) = valor.trim().split_once('.')?;
            if sid == sesion_id.to_string() {
                return Some((*sesion_id, token.into()));
            }
        }
    }
    None
}

/// Cuerpo de registro/login.
#[derive(Debug, Deserialize)]
pub struct CuerpoAcceso {
    pub nombre: Option<String>,
    pub correo: String,
    pub clave: String,
}

/// Respuesta pública de usuario (sin hash).
#[derive(Debug, Serialize)]
pub struct UsuarioPublico {
    pub id: uuid::Uuid,
    pub nombre: String,
    pub correo: String,
    pub rol: String,
}

/// POST /api/acceso/registro — crea usuario (el primero es admin) + sesión.
pub async fn registro(
    State(pool): State<PgPool>,
    Json(cuerpo): Json<CuerpoAcceso>,
) -> Result<(StatusCode, axum::http::HeaderMap, Json<serde_json::Value>), ErrorApi> {
    let nombre = cuerpo.nombre.unwrap_or_default();
    if nombre.trim().is_empty() {
        return Err(ErrorApi::Validacion("El nombre es obligatorio.".into()));
    }
    if cuerpo.clave.len() < 8 {
        return Err(ErrorApi::Validacion("La clave debe tener al menos 8 caracteres.".into()));
    }
    let hash = hash_clave(&cuerpo.clave)?;
    let fila = sqlx::query("select * from registrar_usuario($1, $2, $3)")
        .bind(nombre.trim())
        .bind(cuerpo.correo.trim())
        .bind(&hash)
        .fetch_one(&pool)
        .await?;
    responder_sesion(&pool, fila.get("id"), fila.get("nombre"), fila.get("correo"), fila.get("rol"))
        .await
}

/// POST /api/acceso/login — verifica clave + sesión.
pub async fn login(
    ConnectInfo(direccion): ConnectInfo<SocketAddr>,
    State(pool): State<PgPool>,
    Json(cuerpo): Json<CuerpoAcceso>,
) -> Result<(StatusCode, axum::http::HeaderMap, Json<serde_json::Value>), ErrorApi> {
    if login_limitado(direccion.ip()) {
        return Err(ErrorApi::DemasiadasSolicitudes("Espere un minuto antes de volver a intentar.".into()));
    }
    let fila = sqlx::query("select * from credencial_usuario($1)")
        .bind(cuerpo.correo.trim())
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::Validacion("Correo o clave incorrectos.".into()))?;
    let hash: String = fila.get("clave_hash");
    if !verificar_clave(&cuerpo.clave, &hash) {
        return Err(ErrorApi::Validacion("Correo o clave incorrectos.".into()));
    }
    responder_sesion(&pool, fila.get("id"), fila.get("nombre"), fila.get("correo"), fila.get("rol"))
        .await
}

/// Crea sesión en PG y arma acceso + cookie de renovación.
async fn responder_sesion(
    pool: &PgPool,
    id: uuid::Uuid,
    nombre: String,
    correo: String,
    rol: String,
) -> Result<(StatusCode, axum::http::HeaderMap, Json<serde_json::Value>), ErrorApi> {
    use sqlx::Row;
    let token = uuid::Uuid::new_v4().to_string().replace('-', "");
    let sesion: uuid::Uuid = sqlx::query("select * from crear_sesion($1, $2, $3)")
        .bind(id)
        .bind(huella(&token))
        .bind(VIDA_SESION_DIAS)
        .fetch_one(pool)
        .await?
        .get("id");
    let acceso = emitir_acceso(&id, &nombre, &rol)?;
    let mut cabeceras = axum::http::HeaderMap::new();
    cabeceras.insert(
        header::SET_COOKIE,
        format!("{COOKIE_RENOVAR}={sesion}.{token}; HttpOnly; SameSite=Lax; Path=/api/acceso; Max-Age={}", 30 * 24 * 3600)
            .parse()
            .map_err(|_| ErrorApi::Validacion("No se pudo iniciar sesión.".into()))?,
    );
    Ok((
        StatusCode::OK,
        cabeceras,
        Json(serde_json::json!({
            "acceso": acceso,
            "expira_en": VIDA_ACCESO_SEGS,
            "sesion_id": sesion,
            "renovacion": token,
            "usuario": { "id": id, "nombre": nombre, "correo": correo, "rol": rol },
        })),
    ))
}

/// POST /api/acceso/renovar — rota la sesión y devuelve acceso nuevo.
/// Acepta el token en el cuerpo (`renovacion`) o en la cookie.
pub async fn renovar(
    State(pool): State<PgPool>,
    cabeceras: axum::http::HeaderMap,
    Json(cuerpo): Json<serde_json::Value>,
) -> Result<(StatusCode, axum::http::HeaderMap, Json<serde_json::Value>), ErrorApi> {
    use sqlx::Row;
    let sid = cuerpo
        .get("sesion_id")
        .and_then(|v| v.as_str())
        .and_then(|v| v.parse::<uuid::Uuid>().ok())
        .ok_or_else(|| ErrorApi::Validacion("Sesión inválida.".into()))?;
    let token = cuerpo
        .get("renovacion")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .or_else(|| leer_cookie(&cabeceras, &sid).map(|(_, t)| t))
        .ok_or_else(|| ErrorApi::Validacion("Sesión inválida.".into()))?;
    let nuevo = uuid::Uuid::new_v4().to_string().replace('-', "");
    let fila = sqlx::query("select * from rotar_sesion($1, $2, $3)")
        .bind(sid)
        .bind(huella(&token))
        .bind(huella(&nuevo))
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Sesión no encontrada.".into()))?;
    // Fila NULL = reuso detectado: la función revocó todo sin RAISE (ver 015).
    let usuario_id: Option<uuid::Uuid> = fila.get("usuario_id");
    let usuario_id =
        usuario_id.ok_or_else(|| ErrorApi::NoEncontrado("Sesión revocada por seguridad.".into()))?;
    let f = sqlx::query("select nombre, rol from usuarios where id = $1")
        .bind(usuario_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Usuario no encontrado.".into()))?;
    let acceso = emitir_acceso(&usuario_id, f.get("nombre"), f.get("rol"))?;
    let mut cab = axum::http::HeaderMap::new();
    cab.insert(
        header::SET_COOKIE,
        format!("{COOKIE_RENOVAR}={sid}.{nuevo}; HttpOnly; SameSite=Lax; Path=/api/acceso; Max-Age={}", 30 * 24 * 3600)
            .parse()
            .map_err(|_| ErrorApi::Validacion("No se pudo renovar.".into()))?,
    );
    Ok((
        StatusCode::OK,
        cab,
        Json(serde_json::json!({
            "acceso": acceso,
            "expira_en": VIDA_ACCESO_SEGS,
            "sesion_id": sid,
            "renovacion": nuevo,
        })),
    ))
}

/// POST /api/acceso/salir — cierra la sesión indicada en el cuerpo.
pub async fn salir(
    State(pool): State<PgPool>,
    _cabeceras: axum::http::HeaderMap,
    Json(cuerpo): Json<serde_json::Value>,
) -> Result<(StatusCode, axum::http::HeaderMap, Json<serde_json::Value>), ErrorApi> {
    let mut cab = axum::http::HeaderMap::new();
    cab.insert(
        header::SET_COOKIE,
        cookie_borrar().parse().unwrap_or_else(|_| header::HeaderValue::from_static("")),
    );
    if let Some(sid) = cuerpo
        .get("sesion_id")
        .and_then(|v| v.as_str())
        .and_then(|v| v.parse::<uuid::Uuid>().ok())
    {
        let _ = sqlx::query("select cerrar_sesion($1)")
            .bind(sid)
            .fetch_optional(&pool)
            .await;
    }
    Ok((StatusCode::OK, cab, Json(serde_json::json!({ "salio": true }))))
}

/// GET /api/acceso/yo — usuario actual (exige sesión).
pub async fn yo(
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
) -> Json<UsuarioPublico> {
    // El correo se re-lee en miembros; aquí basta identidad del token.
    Json(UsuarioPublico {
        id: usuario.id,
        nombre: usuario.nombre,
        correo: String::new(),
        rol: usuario.rol,
    })
}

/// GET /api/miembros — lista (exige sesión).
pub async fn miembros(
    State(pool): State<PgPool>,
    axum::Extension(_usuario): axum::Extension<UsuarioActual>,
) -> Result<Json<Vec<UsuarioPublico>>, ErrorApi> {
    use sqlx::Row;
    let filas = sqlx::query("select * from listar_miembros()")
        .fetch_all(&pool)
        .await?;
    Ok(Json(
        filas
            .iter()
            .map(|f| UsuarioPublico {
                id: f.get("id"),
                nombre: f.get("nombre"),
                correo: f.get("correo"),
                rol: f.get::<String, _>("rol"),
            })
            .collect(),
    ))
}

/// PATCH /api/miembros/:id/rol — solo admin.
pub async fn cambiar_rol(
    State(pool): State<PgPool>,
    axum::Extension(usuario): axum::Extension<UsuarioActual>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(cuerpo): Json<serde_json::Value>,
) -> Result<Json<UsuarioPublico>, ErrorApi> {
    use sqlx::Row;
    if usuario.rol != "admin" {
        return Err(ErrorApi::Prohibido("Solo un admin puede hacer esto.".into()));
    }
    let rol = cuerpo
        .get("rol")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let fila = sqlx::query("select * from cambiar_rol($1, $2)")
        .bind(id)
        .bind(rol)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| ErrorApi::NoEncontrado("Usuario no encontrado.".into()))?;
    Ok(Json(UsuarioPublico {
        id: fila.get("id"),
        nombre: fila.get("nombre"),
        correo: fila.get("correo"),
        rol: fila.get::<String, _>("rol"),
    }))
}
