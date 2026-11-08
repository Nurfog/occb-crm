//! Claves de API y webhooks personales del espacio.
use axum::{Json, extract::{Path,State}, http::StatusCode};
use hmac::{Hmac,Mac};
use serde::Deserialize;
use sha2::{Digest,Sha256};
use sqlx::{PgPool,Row};
use uuid::Uuid;
use validator::Validate;
use crate::{auth::UsuarioActual,errores::ErrorApi};

#[derive(Debug,Deserialize,Validate)]
pub struct CuerpoClave {
    #[validate(length(min=1,max=80,message="El nombre es obligatorio (máximo 80 caracteres)."))]
    pub nombre:String,
    #[serde(default="alcances_todos")]
    pub alcances:Vec<String>,
}
fn alcances_todos()->Vec<String>{vec!["*".into()]}
#[derive(Debug,Deserialize,Validate)]
pub struct CuerpoWebhook {
    #[validate(length(min=1,max=80,message="El nombre es obligatorio (máximo 80 caracteres)."))]
    pub nombre:String,
    pub url:String,
    pub eventos:Vec<String>,
}
#[derive(Debug,Deserialize)]
pub struct CuerpoActiva{pub activa:bool}

pub async fn listar_claves(State(pool):State<PgPool>,axum::Extension(u):axum::Extension<UsuarioActual>)->Result<Json<Vec<serde_json::Value>>,ErrorApi>{
 let fs=sqlx::query("select * from listar_claves_api($1)").bind(u.id).fetch_all(&pool).await?;
 Ok(Json(fs.iter().map(|f|serde_json::json!({"id":f.get::<Uuid,_>("id"),"nombre":f.get::<String,_>("nombre"),"alcances":f.get::<Vec<String>,_>("alcances"),"revocada":f.get::<bool,_>("revocada"),"creado_en":f.get::<chrono::DateTime<chrono::Utc>,_>("creado_en"),"ultimo_uso_en":f.get::<Option<chrono::DateTime<chrono::Utc>>,_>("ultimo_uso_en")})).collect()))
}
/// La clave en claro se devuelve únicamente en esta respuesta de creación.
pub async fn crear_clave(State(pool):State<PgPool>,axum::Extension(u):axum::Extension<UsuarioActual>,Json(c):Json<CuerpoClave>)->Result<(StatusCode,Json<serde_json::Value>),ErrorApi>{
 c.validate().map_err(|e|ErrorApi::Validacion(e.to_string()))?;
 let clave=format!("occb_{}",Uuid::new_v4().simple()); let hash=format!("{:x}",Sha256::digest(clave.as_bytes()));
 let f=sqlx::query("select * from crear_clave_api($1,$2,$3,$4)").bind(u.id).bind(c.nombre.trim()).bind(hash).bind(&c.alcances).fetch_one(&pool).await?;
 Ok((StatusCode::CREATED,Json(serde_json::json!({"id":f.get::<Uuid,_>("id"),"nombre":f.get::<String,_>("nombre"),"alcances":f.get::<Vec<String>,_>("alcances"),"clave":clave,"creado_en":f.get::<chrono::DateTime<chrono::Utc>,_>("creado_en")}))))
}
pub async fn revocar_clave(State(pool):State<PgPool>,axum::Extension(u):axum::Extension<UsuarioActual>,Path(id):Path<Uuid>)->Result<StatusCode,ErrorApi>{
 sqlx::query("select revocar_clave_api($1,$2)").bind(u.id).bind(id).fetch_optional(&pool).await?.ok_or_else(||ErrorApi::NoEncontrado("Clave API no encontrada.".into()))?; Ok(StatusCode::NO_CONTENT)
}
fn url_webhook_valida(url:&str)->bool{
 let Ok(u)=reqwest::Url::parse(url) else{return false};
 if u.scheme()=="https" {return u.host_str().is_some();}
 u.scheme()=="http"&&matches!(u.host_str(),Some("localhost"|"127.0.0.1"|"::1"|"host.docker.internal"))
}
pub async fn listar_webhooks(State(pool):State<PgPool>,axum::Extension(u):axum::Extension<UsuarioActual>)->Result<Json<Vec<serde_json::Value>>,ErrorApi>{
 let fs=sqlx::query("select * from listar_webhooks($1)").bind(u.id).fetch_all(&pool).await?;
 Ok(Json(fs.iter().map(|f|serde_json::json!({"id":f.get::<Uuid,_>("id"),"nombre":f.get::<String,_>("nombre"),"url":f.get::<String,_>("url"),"eventos":f.get::<Vec<String>,_>("eventos"),"activa":f.get::<bool,_>("activa"),"creado_en":f.get::<chrono::DateTime<chrono::Utc>,_>("creado_en")})).collect()))
}
/// El secreto de firma se muestra una sola vez junto con la creación.
pub async fn crear_webhook(State(pool):State<PgPool>,axum::Extension(u):axum::Extension<UsuarioActual>,Json(c):Json<CuerpoWebhook>)->Result<(StatusCode,Json<serde_json::Value>),ErrorApi>{
 c.validate().map_err(|e|ErrorApi::Validacion(e.to_string()))?;
 if !url_webhook_valida(&c.url){return Err(ErrorApi::Validacion("El webhook debe usar HTTPS (HTTP solo en localhost).".into()));}
 let f=sqlx::query("select * from crear_webhook($1,$2,$3,$4)").bind(u.id).bind(c.nombre.trim()).bind(c.url.trim()).bind(&c.eventos).fetch_one(&pool).await?;
 Ok((StatusCode::CREATED,Json(serde_json::json!({"id":f.get::<Uuid,_>("id"),"nombre":f.get::<String,_>("nombre"),"url":f.get::<String,_>("url"),"eventos":f.get::<Vec<String>,_>("eventos"),"activa":f.get::<bool,_>("activa"),"secreto":f.get::<String,_>("secreto"),"creado_en":f.get::<chrono::DateTime<chrono::Utc>,_>("creado_en")}))))
}
pub async fn alternar_webhook(State(pool):State<PgPool>,axum::Extension(u):axum::Extension<UsuarioActual>,Path(id):Path<Uuid>,Json(c):Json<CuerpoActiva>)->Result<Json<serde_json::Value>,ErrorApi>{
 let f=sqlx::query("select * from alternar_webhook($1,$2,$3)").bind(u.id).bind(id).bind(c.activa).fetch_optional(&pool).await?.ok_or_else(||ErrorApi::NoEncontrado("Webhook no encontrado.".into()))?;
 Ok(Json(serde_json::json!({"id":f.get::<Uuid,_>("id"),"activa":f.get::<bool,_>("activa")})))
}
pub async fn eliminar_webhook(State(pool):State<PgPool>,axum::Extension(u):axum::Extension<UsuarioActual>,Path(id):Path<Uuid>)->Result<StatusCode,ErrorApi>{
 sqlx::query("select eliminar_webhook($1,$2)").bind(u.id).bind(id).fetch_optional(&pool).await?.ok_or_else(||ErrorApi::NoEncontrado("Webhook no encontrado.".into()))?; Ok(StatusCode::NO_CONTENT)
}
pub async fn probar_webhook(State(pool):State<PgPool>,axum::Extension(u):axum::Extension<UsuarioActual>,Path(id):Path<Uuid>)->Result<Json<serde_json::Value>,ErrorApi>{
 let entrega:Uuid=sqlx::query_scalar("select probar_webhook($1,$2)").bind(u.id).bind(id).fetch_one(&pool).await?;
 Ok(Json(serde_json::json!({"entrega_id":entrega,"mensaje":"Entrega de prueba encolada."})))
}
pub async fn entregas(State(pool):State<PgPool>,axum::Extension(u):axum::Extension<UsuarioActual>,Path(id):Path<Uuid>)->Result<Json<Vec<serde_json::Value>>,ErrorApi>{
 let fs=sqlx::query("select * from listar_entregas_webhook($1,$2,50)").bind(u.id).bind(id).fetch_all(&pool).await?;
 Ok(Json(fs.iter().map(|f|serde_json::json!({"id":f.get::<Uuid,_>("id"),"evento":f.get::<String,_>("evento"),"intentos":f.get::<i16,_>("intentos"),"ok":f.get::<Option<bool>,_>("ok"),"ultimo_error":f.get::<String,_>("ultimo_error"),"creado_en":f.get::<chrono::DateTime<chrono::Utc>,_>("creado_en"),"ultimo_intento_en":f.get::<Option<chrono::DateTime<chrono::Utc>>,_>("ultimo_intento_en")})).collect()))
}

/// Worker continuo con timeout, firma HMAC-SHA256 y hasta tres intentos.
/// Un error de red no afecta a las operaciones del API.
pub async fn ejecutar_worker_webhooks(pool:PgPool)->!{
 let cliente=match reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).redirect(reqwest::redirect::Policy::none()).build(){
  Ok(c)=>c,Err(e)=>{tracing::error!(?e,"No se pudo inicializar cliente de webhooks");loop{tokio::time::sleep(std::time::Duration::from_secs(30)).await}}
 };
 loop{
  let filas=sqlx::query("select * from reclamar_entregas_webhook(10)").fetch_all(&pool).await;
  match filas{
   Ok(filas) if !filas.is_empty()=>{
    for f in filas{
     let id:Uuid=f.get("id"); let url:String=f.get("url"); let secreto:String=f.get("secreto");
     let evento:String=f.get("evento"); let carga:serde_json::Value=f.get("carga");
     let bytes=serde_json::to_vec(&carga).unwrap_or_default();
     let firma=firmar(&secreto,&bytes);
     let resultado=cliente.post(&url).header("X-OCCB-Evento",evento).header("X-OCCB-Firma",format!("sha256={firma}"))
       .header("Content-Type","application/json").body(bytes).send().await;
     let (ok,error)=match resultado{
      Ok(r) if r.status().is_success()=>(true,String::new()),
      Ok(r)=>(false,format!("El receptor respondió HTTP {}.",r.status().as_u16())),
      Err(_)=>(false,"No se pudo conectar con el receptor.".to_string()),
     };
     if let Err(e)=sqlx::query("select registrar_resultado_webhook($1,$2,$3)").bind(id).bind(ok).bind(error).execute(&pool).await{
      tracing::error!(?e,entrega=%id,"No se pudo guardar el resultado del webhook");
     }
    }
   }
   Ok(_)=>tokio::time::sleep(std::time::Duration::from_secs(2)).await,
   Err(e)=>{tracing::warn!(?e,"No se pudieron reclamar entregas webhook");tokio::time::sleep(std::time::Duration::from_secs(5)).await;}
  }
 }
}

fn firmar(secreto:&str,carga:&[u8])->String{
 let Ok(mut mac)=Hmac::<sha2::Sha256>::new_from_slice(secreto.as_bytes()) else{return String::new()};
 mac.update(carga);
 mac.finalize().into_bytes().iter().map(|b|format!("{b:02x}")).collect()
}
