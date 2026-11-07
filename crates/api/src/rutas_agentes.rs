//! Conversaciones con proveedor LLM compatible con OpenAI y herramientas seguras.

use axum::{Json, extract::{Path, State}, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{auth::UsuarioActual, bd, errores::ErrorApi};

#[derive(Debug, Deserialize)]
pub struct CuerpoConversacion { pub titulo: Option<String> }
#[derive(Debug, Deserialize)]
pub struct CuerpoPregunta { pub pregunta: String }

#[derive(Debug, Serialize)]
pub struct RespuestaAgente {
    pub conversacion_id: Uuid,
    pub respuesta: String,
    pub fuentes: Vec<occb_core::ResultadoBusqueda>,
}

#[derive(Clone)]
struct ConfigLlm { url: String, clave: String, modelo: String }

fn config_llm() -> Option<ConfigLlm> {
    let url=std::env::var("LLM_URL").ok()?;
    let clave=std::env::var("LLM_CLAVE").ok()?;
    let modelo=std::env::var("LLM_MODELO").ok()?;
    if url.trim().is_empty() || clave.trim().is_empty() || modelo.trim().is_empty() { return None; }
    let base=url.trim_end_matches('/');
    Some(ConfigLlm {
        url: if base.starts_with("mock://") || base.ends_with("/chat/completions") { base.to_string() } else { format!("{base}/chat/completions") },
        clave, modelo,
    })
}

fn error_no_configurado() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::SERVICE_UNAVAILABLE,Json(serde_json::json!({
        "codigo":"no_configurado","mensaje":"El asistente no está configurado. El resto del espacio sigue disponible."
    })))
}

pub async fn listar(
    State(pool):State<PgPool>,
    axum::Extension(usuario):axum::Extension<UsuarioActual>,
)->Result<Json<Vec<occb_core::Conversacion>>,ErrorApi>{
    let fs=sqlx::query("select * from listar_conversaciones($1,50)").bind(usuario.id).fetch_all(&pool).await?;
    Ok(Json(fs.iter().map(fila_a_conversacion).collect()))
}

pub async fn crear(
    State(pool):State<PgPool>,
    axum::Extension(usuario):axum::Extension<UsuarioActual>,
    Json(c):Json<CuerpoConversacion>,
)->Result<(StatusCode,Json<occb_core::Conversacion>),ErrorApi>{
    let f=sqlx::query("select * from crear_conversacion($1,$2)")
        .bind(usuario.id).bind(c.titulo.unwrap_or_default()).fetch_one(&pool).await?;
    Ok((StatusCode::CREATED,Json(fila_a_conversacion(&f))))
}

pub async fn eliminar(
    State(pool):State<PgPool>,
    axum::Extension(usuario):axum::Extension<UsuarioActual>,
    Path(id):Path<Uuid>,
)->Result<StatusCode,ErrorApi>{
    sqlx::query("select eliminar_conversacion($1,$2)").bind(usuario.id).bind(id).fetch_optional(&pool).await?
        .ok_or_else(||ErrorApi::NoEncontrado("Conversación no encontrada.".into()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn mensajes(
    State(pool):State<PgPool>,
    axum::Extension(usuario):axum::Extension<UsuarioActual>,
    Path(id):Path<Uuid>,
)->Result<Json<Vec<occb_core::Mensaje>>,ErrorApi>{
    let fs=sqlx::query("select * from listar_mensajes($1,$2,200)").bind(usuario.id).bind(id).fetch_all(&pool).await?;
    Ok(Json(fs.iter().map(fila_a_mensaje).collect()))
}

pub async fn preguntar(
    State(pool):State<PgPool>,
    axum::Extension(usuario):axum::Extension<UsuarioActual>,
    Path(id):Path<Uuid>,
    Json(c):Json<CuerpoPregunta>,
)->Result<Json<RespuestaAgente>,(StatusCode,Json<serde_json::Value>)>{
    let Some(config)=config_llm() else { return Err(error_no_configurado()); };
    let pregunta=c.pregunta.trim().to_string();
    if pregunta.is_empty() || pregunta.len()>8000 {
        return Err((StatusCode::BAD_REQUEST,Json(serde_json::json!({"codigo":"datos_invalidos","mensaje":"La pregunta debe tener entre 1 y 8000 caracteres."}))));
    }
    // La comprobación de propiedad ocurre antes de persistir o llamar al proveedor.
    let conversacion=sqlx::query("select * from obtener_conversacion($1,$2)")
        .bind(usuario.id).bind(id).fetch_optional(&pool).await
        .map_err(error_llm)?
        .ok_or_else(||(StatusCode::NOT_FOUND,Json(serde_json::json!({"codigo":"no_encontrado","mensaje":"Conversación no encontrada."}))))?;
    let titulo_actual:String=conversacion.get("titulo");
    if titulo_actual=="Nueva conversación" {
        let titulo=pregunta.chars().take(80).collect::<String>();
        sqlx::query("select * from actualizar_titulo_conversacion($1,$2,$3)")
            .bind(usuario.id).bind(id).bind(titulo).fetch_one(&pool).await.map_err(error_llm)?;
    }
    sqlx::query("select * from guardar_mensaje($1,$2,'usuario',$3)")
        .bind(usuario.id).bind(id).bind(&pregunta).fetch_one(&pool).await.map_err(error_llm)?;
    let historial=sqlx::query("select * from listar_mensajes($1,$2,40)")
        .bind(usuario.id).bind(id).fetch_all(&pool).await.map_err(error_llm)?;
    let mut mensajes=vec![serde_json::json!({
        "role":"system",
        "content":"Eres el asistente del espacio escolar OCCB. Responde en español, de forma breve y honesta. Solo accede o modifica datos mediante las herramientas disponibles. No inventes resultados; cita los registros consultados usando sus ids y títulos. No expongas secretos ni claves."
    })];
    for fila in &historial {
        let rol:String=fila.get("rol");
        let contenido:String=fila.get("contenido");
        mensajes.push(serde_json::json!({"role":if rol=="agente"{"assistant"}else{"user"},"content":contenido}));
    }
    let herramientas=definiciones_herramientas();
    let mut fuentes=Vec::new();
    let mut respuesta=None;
    for _ in 0..5 {
        let peticion=serde_json::json!({"model":config.modelo,"messages":mensajes,"tools":herramientas,"tool_choice":"auto","temperature":0.2});
        let salida=if config.url=="mock://occb-e2e" && cfg!(debug_assertions) {
            respuesta_mock(&mensajes)
        } else {
            let http=reqwest::Client::builder().timeout(std::time::Duration::from_secs(30)).build()
                .map_err(|_|error_proveedor())?.post(&config.url).bearer_auth(&config.clave).json(&peticion).send().await
                .map_err(|_|error_proveedor())?;
            if !http.status().is_success() { tracing::warn!(status=%http.status(),"Proveedor LLM respondió con error"); return Err(error_proveedor()); }
            http.json::<serde_json::Value>().await.map_err(|_|error_proveedor())?
        };
        let Some(message)=salida.get("choices").and_then(|x|x.get(0)).and_then(|x|x.get("message")) else { return Err(error_proveedor()); };
        let calls=message.get("tool_calls").and_then(|x|x.as_array()).cloned().unwrap_or_default();
        if calls.is_empty() {
            respuesta=message.get("content").and_then(|x|x.as_str()).map(str::to_string);
            break;
        }
        mensajes.push(message.clone());
        for call in calls {
            let call_id=call.get("id").and_then(|x|x.as_str()).unwrap_or_default();
            let nombre=call.pointer("/function/name").and_then(|x|x.as_str()).unwrap_or_default();
            let argumentos=call.pointer("/function/arguments").and_then(|x|x.as_str()).and_then(|s|serde_json::from_str::<serde_json::Value>(s).ok()).unwrap_or(serde_json::json!({}));
            let resultado=ejecutar_herramienta(&pool,usuario.id,nombre,&argumentos,&mut fuentes).await;
            mensajes.push(serde_json::json!({"role":"tool","tool_call_id":call_id,"content":resultado.to_string()}));
        }
    }
    let respuesta=respuesta.unwrap_or_else(||"No pude completar la consulta dentro del límite de herramientas.".into());
    let respuesta=respuesta.chars().take(20000).collect::<String>();
    sqlx::query("select * from guardar_mensaje($1,$2,'agente',$3)")
        .bind(usuario.id).bind(id).bind(&respuesta).fetch_one(&pool).await.map_err(error_llm)?;
    if fuentes.len()>10 { fuentes.truncate(10); }
    Ok(Json(RespuestaAgente{conversacion_id:id,respuesta,fuentes}))
}

/// Proveedor simulado solo para binarios de desarrollo. Permite comprobar el
/// ciclo de herramientas sin Internet ni credenciales reales; está desactivado
/// en compilaciones release aunque alguien configure esta URL.
fn respuesta_mock(mensajes:&[serde_json::Value])->serde_json::Value{
    if let Some(tool)=mensajes.iter().rev().find(|m|m.get("role").and_then(|x|x.as_str())==Some("tool")) {
        let contenido=tool.get("content").and_then(|x|x.as_str()).unwrap_or("");
        if let Ok(d)=serde_json::from_str::<serde_json::Value>(contenido) {
            if let Some(valor)=d.get("valor_tuberia") {
                return serde_json::json!({"choices":[{"message":{"role":"assistant","content":format!("El valor real de la tubería es ${valor}.")}}]});
            }
            if d.get("creada").and_then(|x|x.as_bool())==Some(true) {
                return serde_json::json!({"choices":[{"message":{"role":"assistant","content":format!("Creé la nota solicitada (id {}).",d.get("id").and_then(|x|x.as_str()).unwrap_or(""))}}]});
            }
        }
        return serde_json::json!({"choices":[{"message":{"role":"assistant","content":"Consulté la herramienta del espacio."}}]});
    }
    let pregunta=mensajes.iter().rev().find(|m|m.get("role").and_then(|x|x.as_str())==Some("user"))
        .and_then(|m|m.get("content")).and_then(|x|x.as_str()).unwrap_or("").to_lowercase();
    if pregunta.contains("tubería")||pregunta.contains("tuberia")||pregunta.contains("total") {
        serde_json::json!({"choices":[{"message":{"role":"assistant","tool_calls":[{"id":"mock-panel-1","type":"function","function":{"name":"resumen_panel","arguments":"{}"}}]}}]})
    } else if pregunta.contains("crear una nota")||pregunta.contains("crea una nota") {
        serde_json::json!({"choices":[{"message":{"role":"assistant","tool_calls":[{"id":"mock-nota-1","type":"function","function":{"name":"crear_nota","arguments":"{\"titulo\":\"Nota creada por asistente\",\"cuerpo\":\"Nota de verificación del proveedor simulado.\"}"}}]}}]})
    } else {
        serde_json::json!({"choices":[{"message":{"role":"assistant","content":"Puedo buscar registros escolares, consultar el panel y crear notas o tareas vinculadas."}}]})
    }
}

fn definiciones_herramientas()->serde_json::Value{
    serde_json::json!([
      {"type":"function","function":{"name":"buscar_registros","description":"Busca alumnos, apoderados, colegios, postulaciones, tareas, notas, eventos y correos.","parameters":{"type":"object","properties":{"q":{"type":"string"}},"required":["q"]}}},
      {"type":"function","function":{"name":"resumen_panel","description":"Consulta las cifras reales del panel escolar.","parameters":{"type":"object","properties":{},"additionalProperties":false}}},
      {"type":"function","function":{"name":"crear_nota","description":"Crea una nota escolar usando la función segura del sistema.","parameters":{"type":"object","properties":{"titulo":{"type":"string"},"cuerpo":{"type":"string"},"alumno_id":{"type":"string"},"apoderado_id":{"type":"string"},"oportunidad_id":{"type":"string"}},"required":["titulo","cuerpo"]}}},
      {"type":"function","function":{"name":"crear_tarea","description":"Crea una tarea escolar vinculada a una postulación, alumno o apoderado.","parameters":{"type":"object","properties":{"titulo":{"type":"string"},"detalle":{"type":"string"},"prioridad":{"type":"string","enum":["baja","media","alta"]},"alumno_id":{"type":"string"},"apoderado_id":{"type":"string"},"oportunidad_id":{"type":"string"}},"required":["titulo"]}}}
    ])
}

async fn ejecutar_herramienta(
    pool:&PgPool,usuario_id:Uuid,nombre:&str,args:&serde_json::Value,fuentes:&mut Vec<occb_core::ResultadoBusqueda>
)->serde_json::Value{
    match nombre{
      "buscar_registros"=>{
        let q=args.get("q").and_then(|x|x.as_str()).unwrap_or("");
        match sqlx::query("select * from buscar_espacio($1)").bind(q).fetch_all(pool).await{
          Ok(fs)=>{let lista:Vec<_>=fs.iter().map(|f|occb_core::ResultadoBusqueda{tipo:f.get("tipo"),id:f.get("id"),titulo:f.get("titulo"),detalle:f.get("detalle"),puntaje:f.get("puntaje")}).collect();
            for x in &lista { if !fuentes.iter().any(|y|y.id==x.id&&y.tipo==x.tipo){fuentes.push(x.clone());} }
            serde_json::to_value(lista).unwrap_or(serde_json::Value::Null)}
          Err(_)=>serde_json::json!({"error":"No se pudo realizar la búsqueda."})
        }
      }
      "resumen_panel"=>{
        match sqlx::query("select total_colegios,total_personas,total_oportunidades,valor_tuberia,valor_ganado,tasa_exito::float8 tasa_exito from panel_resumen()")
          .fetch_one(pool).await{
          Ok(f)=>serde_json::json!({"colegios":f.get::<i64,_>("total_colegios"),"personas":f.get::<i64,_>("total_personas"),"postulaciones":f.get::<i64,_>("total_oportunidades"),"valor_tuberia":f.get::<i64,_>("valor_tuberia"),"matriculado":f.get::<i64,_>("valor_ganado"),"tasa_exito":f.get::<f64,_>("tasa_exito")}),
          Err(_)=>serde_json::json!({"error":"No se pudo consultar el panel."})
        }
      }
      "crear_nota"=>{
        let titulo=args.get("titulo").and_then(|x|x.as_str()).unwrap_or("");
        let cuerpo=args.get("cuerpo").and_then(|x|x.as_str()).unwrap_or("");
        let alumno=args.get("alumno_id").and_then(|x|x.as_str()).and_then(|x|Uuid::parse_str(x).ok());
        let apoderado=args.get("apoderado_id").and_then(|x|x.as_str()).and_then(|x|Uuid::parse_str(x).ok());
        let opp=args.get("oportunidad_id").and_then(|x|x.as_str()).and_then(|x|Uuid::parse_str(x).ok());
        match bd::transaccion_con_actor(pool,usuario_id).await{
          Ok(mut tx)=>match sqlx::query("select * from crear_nota($1,$2,$3,$4,$5)").bind(titulo).bind(cuerpo).bind(alumno).bind(apoderado).bind(opp).fetch_one(&mut *tx).await{
            Ok(f)=>{let id:Uuid=f.get("id"); let _=tx.commit().await; fuentes.push(occb_core::ResultadoBusqueda{tipo:"nota".into(),id:id.to_string(),titulo:titulo.into(),detalle:cuerpo.chars().take(100).collect(),puntaje:1.0}); serde_json::json!({"creada":true,"id":id,"titulo":titulo})},
            Err(_)=>serde_json::json!({"error":"No se pudo crear la nota; revisa el contenido y los vínculos."})
          },
          Err(_)=>serde_json::json!({"error":"No se pudo iniciar la operación."})
        }
      }
      "crear_tarea"=>{
        let titulo=args.get("titulo").and_then(|x|x.as_str()).unwrap_or("");
        let detalle=args.get("detalle").and_then(|x|x.as_str()).unwrap_or("");
        let prioridad=args.get("prioridad").and_then(|x|x.as_str()).unwrap_or("media");
        let alumno=args.get("alumno_id").and_then(|x|x.as_str()).and_then(|x|Uuid::parse_str(x).ok());
        let apoderado=args.get("apoderado_id").and_then(|x|x.as_str()).and_then(|x|Uuid::parse_str(x).ok());
        let opp=args.get("oportunidad_id").and_then(|x|x.as_str()).and_then(|x|Uuid::parse_str(x).ok());
        match bd::transaccion_con_actor(pool,usuario_id).await{
          Ok(mut tx)=>match sqlx::query("select * from crear_tarea($1,$2,'pendiente',$3,null,$4,$5,$6)").bind(titulo).bind(detalle).bind(prioridad).bind(alumno).bind(apoderado).bind(opp).fetch_one(&mut *tx).await{
            Ok(f)=>{let id:Uuid=f.get("id"); let _=tx.commit().await; fuentes.push(occb_core::ResultadoBusqueda{tipo:"tarea".into(),id:id.to_string(),titulo:titulo.into(),detalle:detalle.chars().take(100).collect(),puntaje:1.0}); serde_json::json!({"creada":true,"id":id,"titulo":titulo})},
            Err(_)=>serde_json::json!({"error":"No se pudo crear la tarea; revisa prioridad y vínculos."})
          },
          Err(_)=>serde_json::json!({"error":"No se pudo iniciar la operación."})
        }
      }
      _=>serde_json::json!({"error":"Herramienta no disponible."})
    }
}

fn error_llm(error:sqlx::Error)->(StatusCode,Json<serde_json::Value>){
    tracing::error!(?error,"Error de conversación en base de datos");
    (StatusCode::INTERNAL_SERVER_ERROR,Json(serde_json::json!({"codigo":"error_interno","mensaje":"No se pudo guardar la conversación."})))
}
fn error_proveedor()->(StatusCode,Json<serde_json::Value>){
    (StatusCode::BAD_GATEWAY,Json(serde_json::json!({"codigo":"proveedor_no_disponible","mensaje":"El proveedor del asistente no está disponible."})))
}

fn fila_a_conversacion(f:&sqlx::postgres::PgRow)->occb_core::Conversacion{
    occb_core::Conversacion{id:f.get("id"),usuario_id:f.get("usuario_id"),titulo:f.get("titulo"),creado_en:f.get("creado_en"),actualizado_en:f.get("actualizado_en")}
}
fn fila_a_mensaje(f:&sqlx::postgres::PgRow)->occb_core::Mensaje{
    occb_core::Mensaje{id:f.get("id"),conversacion_id:f.get("conversacion_id"),rol:f.get("rol"),contenido:f.get("contenido"),creado_en:f.get("creado_en")}
}
