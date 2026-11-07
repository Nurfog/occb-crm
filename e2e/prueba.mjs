// Prueba E2E de OCCB CRM en Chrome headless vía CDP (sin dependencias).
// Uso: node e2e/prueba.mjs
// Requiere: frontal en :8080, API en :3001 y Chrome en /opt/google/chrome/chrome.
// Cubre: panel vivo, colegios (API+ficha), kanban de admisión persistido,
// acceso en español, edición en fichas y territorio chileno.

const CHROME = process.env.CHROME_BIN || '/opt/google/chrome/chrome';
const FRONTAL = process.env.FRONTAL_URL || 'http://127.0.0.1:8080';
const API = process.env.API_URL || 'http://127.0.0.1:3001';
const SELLO = `E2E-${Date.now().toString(36)}`;

const { spawn } = await import('node:child_process');
const net = await import('node:net');

// No reutiliza sesiones Chrome huérfanas de ejecuciones previas.
const reservaPuerto = net.createServer();
await new Promise((resolve, reject) => {
  reservaPuerto.once('error', reject);
  reservaPuerto.listen(0, '127.0.0.1', resolve);
});
const PUERTO_CD = reservaPuerto.address().port;
await new Promise((resolve) => reservaPuerto.close(resolve));

function dormir(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function esperarUrl(url, intentos = 40) {
  for (let i = 0; i < intentos; i++) {
    try {
      const r = await fetch(url);
      if (r.ok) return true;
    } catch {}
    await dormir(500);
  }
  throw new Error(`Sin respuesta de ${url}`);
}

// --- Lanza Chrome ---
const chrome = spawn(
  CHROME,
  [
    '--headless=new',
    `--remote-debugging-port=${PUERTO_CD}`,
    '--no-sandbox',
    '--disable-dev-shm-usage',
    '--disable-gpu',
    '--window-size=1280,900',
    `--user-data-dir=/tmp/opencode/e2e-chrome-${PUERTO_CD}`,
    'about:blank',
  ],
  { stdio: 'ignore', detached: true },
);
chrome.unref();
await esperarUrl(`http://127.0.0.1:${PUERTO_CD}/json/version`);

// --- Conexión CDP ---
const objetivos = await (await fetch(`http://127.0.0.1:${PUERTO_CD}/json`)).json();
const pagina = objetivos.find((t) => t.type === 'page');
const ws = new WebSocket(pagina.webSocketDebuggerUrl, { maxPayload: 64 * 1024 * 1024 });
await new Promise((res, rej) => {
  ws.onopen = res;
  ws.onerror = rej;
});
let id = 0;
const pendientes = new Map();
ws.onmessage = (ev) => {
  const msg = JSON.parse(ev.data);
  if (msg.id && pendientes.has(msg.id)) {
    pendientes.get(msg.id)(msg);
    pendientes.delete(msg.id);
  }
};
function enviar(metodo, params = {}) {
  return new Promise((resolve) => {
    const miId = ++id;
    pendientes.set(miId, resolve);
    ws.send(JSON.stringify({ id: miId, method: metodo, params }));
  });
}
async function evaluar(expresion, esperar = true) {
  const r = await enviar('Runtime.evaluate', {
    expression: expresion,
    awaitPromise: esperar,
    returnByValue: true,
  });
  return r.result?.result?.value;
}
async function esperarCondicion(js, timeoutMs = 25000) {
  const t0 = Date.now();
  while (Date.now() - t0 < timeoutMs) {
    try {
      if (await evaluar(`(${js})`)) return true;
    } catch {}
    await dormir(500);
  }
  return false;
}
async function ir(ruta) {
  await enviar('Page.navigate', { url: FRONTAL + ruta });
  for (let i = 0; i < 30; i++) {
    await dormir(500);
    if (await evaluar(`document.readyState === 'complete'`)) break;
  }
  // El HTML inicial viene vacío: espera render real del WASM.
  // (Dioxus no pinta dentro de #main; se mide el cuerpo: acceso ≈110, resto >200.)
  await esperarCondicion(`(document.body.textContent.length || 0) > 80`);
  await dormir(2000);
  // La sesión vive solo en memoria: cada carga pierde la sesión y el
  // guardia redirige a /acceso (siempre, en ruta protegida). Se espera esa
  // redirección antes de entrar: si aún no aparece, el WASM sigue
  // arrancando, no que ya estemos dentro. Tras entrar, la app vuelve
  // sola al destino (VOLVER).
  if (!ruta.startsWith('/acceso')) {
    let visto = false;
    for (let i = 0; i < 12; i++) {
      await dormir(1000);
      if ((await texto()).includes('Bienvenido de nuevo')) {
        visto = true;
        break;
      }
    }
    if (visto) {
      await loginUI(E2E_CORREO, E2E_CLAVE);
      await dormir(1500);
    }
  }
}
async function loginUI(correo, clave) {
  await esperarCondicion(`!!document.querySelector('.login-card input[type="email"]')`);
  await evaluar(`(() => {
    const set = (sel, v) => {
      const el = document.querySelector(sel);
      el.focus(); el.value = v;
      el.dispatchEvent(new Event('input', { bubbles: true }));
    };
    set('.login-card input[type="email"]', ${JSON.stringify(correo)});
    set('.login-card input[type="password"]', ${JSON.stringify(clave)});
  })()`);
  await evaluar(
    `[...document.querySelectorAll('.login-card button')].find(b => b.textContent.trim() === 'Acceder').click()`,
  );
  const t0 = Date.now();
  while (Date.now() - t0 < 20000) {
    const t1 = await texto();
    if (!t1.includes('Bienvenido de nuevo')) return true;
    await dormir(500);
  }
  return false;
}
async function texto() {
  return evaluar(`document.body.textContent`);
}

let fallos = 0;
function afirma(cond, nombre) {
  console.log(`${cond ? 'PASS' : 'FAIL'} — ${nombre}`);
  if (!cond) fallos++;
}

// --- 0. Acceso: usuarios dedicados + ayudantes con Bearer ---
// Usa filas propias (correos E2E) y nunca toca datos de usuarios reales.
const E2E_CORREO = `e2e-${SELLO}@test.cl`;
const E2E_CLAVE = 'Clave1234';
const ADMIN_CORREO = 'admin-e2e@test.cl';
async function asegurarAcceso(nombre, correo, clave) {
  let r = await fetch(`${API}/api/acceso/registro`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ nombre, correo, clave }),
  });
  let d = await r.json();
  if (!r.ok && d.mensaje !== 'Ya existe un usuario con ese correo.') {
    throw new Error(`No se pudo asegurar ${correo}: ${d.mensaje}`);
  }
  if (!r.ok) {
    r = await fetch(`${API}/api/acceso/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ correo, clave }),
    });
    d = await r.json();
    if (!r.ok) throw new Error(`No se pudo entrar con ${correo}: ${d.mensaje}`);
  }
  return d;
}
async function renovarSesion(ses) {
  const r = await fetch(`${API}/api/acceso/renovar`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ sesion_id: ses.sesion_id, renovacion: ses.renovacion }),
  });
  const d = await r.json();
  if (!r.ok) {
    const f = await asegurarAcceso('E2E API', ses.correo, E2E_CLAVE);
    ses.acceso = f.acceso;
    ses.sesion_id = f.sesion_id;
    ses.renovacion = f.renovacion;
    return;
  }
  ses.acceso = d.acceso;
  ses.renovacion = d.renovacion;
}
function conSesion(ses, correo) {
  ses.correo = correo;
  return async (url, opts = {}) => {
    const h = { ...(opts.headers || {}), Authorization: `Bearer ${ses.acceso}` };
    let r = await fetch(url, { ...opts, headers: h });
    if (r.status === 401) {
      await renovarSesion(ses);
      r = await fetch(url, {
        ...opts,
        headers: { ...(opts.headers || {}), Authorization: `Bearer ${ses.acceso}` },
      });
    }
    return r;
  };
}
// El admin primero (en volumen limpio, el primero nace admin).
const adminIni = await asegurarAcceso('Admin E2E', ADMIN_CORREO, E2E_CLAVE);
const SES_ADMIN = {
  acceso: adminIni.acceso,
  sesion_id: adminIni.sesion_id,
  renovacion: adminIni.renovacion,
};
const apiAdmin = conSesion(SES_ADMIN, ADMIN_CORREO);
const e2eIni = await asegurarAcceso('E2E API', E2E_CORREO, E2E_CLAVE);
const SES = {
  acceso: e2eIni.acceso,
  sesion_id: e2eIni.sesion_id,
  renovacion: e2eIni.renovacion,
};
const api = conSesion(SES, E2E_CORREO);
afirma(!!e2eIni.acceso && !!e2eIni.sesion_id && !!e2eIni.renovacion, 'registro/login entrega acceso + sesión + renovación');
afirma(e2eIni.usuario?.rol === 'miembro', 'usuario E2E dedicado nace miembro');

// --- 0b. Acceso: validaciones y protección ---
const dupli = await (
  await fetch(`${API}/api/acceso/registro`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ nombre: 'X', correo: E2E_CORREO, clave: E2E_CLAVE }),
  })
).json();
afirma(dupli.codigo === 'datos_invalidos', 'correo duplicado se rechaza');
const corta = await (
  await fetch(`${API}/api/acceso/registro`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ nombre: 'X', correo: `corto-${SELLO}@test.cl`, clave: '123' }),
  })
).json();
afirma(corta.codigo === 'datos_invalidos', 'clave corta se rechaza');
const malaClave = await (
  await fetch(`${API}/api/acceso/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ correo: E2E_CORREO, clave: 'otra-clave' }),
  })
).json();
afirma(malaClave.codigo === 'datos_invalidos', 'clave errónea se rechaza');
const sinToken = await fetch(`${API}/api/personas?limite=1`);
afirma(sinToken.status === 401, 'ruta protegida sin token devuelve 401');
const prohibido = await api(`${API}/api/miembros/${e2eIni.usuario.id}/rol`, {
  method: 'PATCH',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ rol: 'admin' }),
});
afirma(prohibido.status === 403, 'miembro no puede cambiar roles (403)');
const promo = await apiAdmin(`${API}/api/miembros/${e2eIni.usuario.id}/rol`, {
  method: 'PATCH',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ rol: 'admin' }),
});
const promoD = await promo.json();
afirma(promo.ok && promoD.rol === 'admin', 'admin promueve a miembro E2E');
await apiAdmin(`${API}/api/miembros/${e2eIni.usuario.id}/rol`, {
  method: 'PATCH',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ rol: 'miembro' }),
});

// --- 0c. Acceso: rotación y reuso que revoca ---
const rot1 = await (
  await fetch(`${API}/api/acceso/renovar`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ sesion_id: SES.sesion_id, renovacion: SES.renovacion }),
  })
).json();
afirma(!!rot1.acceso && rot1.acceso !== SES.acceso, 'renovar rota el acceso');
const renVieja = SES.renovacion;
SES.acceso = rot1.acceso;
SES.renovacion = rot1.renovacion;
const reuso = await (
  await fetch(`${API}/api/acceso/renovar`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ sesion_id: SES.sesion_id, renovacion: renVieja }),
  })
).json();
afirma(reuso.mensaje === 'Sesión revocada por seguridad.', 'reuso de renovación revoca');
const trasReuso = await (
  await fetch(`${API}/api/acceso/renovar`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ sesion_id: SES.sesion_id, renovacion: SES.renovacion }),
  })
).json();
afirma(trasReuso.codigo === 'no_encontrado', 'tras el reuso no queda ninguna sesión');
const deNuevo = await asegurarAcceso('E2E API', E2E_CORREO, E2E_CLAVE);
SES.acceso = deNuevo.acceso;
SES.sesion_id = deNuevo.sesion_id;
SES.renovacion = deNuevo.renovacion;

// --- 1. Panel con datos vivos ---
await ir('/');
await esperarCondicion(`document.body.textContent.includes('API :3001')`);
let t = await texto();
afirma(t.includes('● API :3001'), 'panel indica API en línea (no demo local)');
afirma(t.includes('Colegios') && t.includes('RBD'), 'panel muestra colegios de la API');

// --- 2. Alta de colegio (API) visible en su ficha ---
const RBD_UI = `E2E-${SELLO}`;
const NOMBRE_UI = `Colegio UI ${SELLO}`;
await api(`${API}/api/colegios`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ rbd: RBD_UI, nombre: NOMBRE_UI, region: 13, comuna: '13101' }),
});
await ir(`/colegios/${encodeURIComponent(RBD_UI)}`);
await esperarCondicion(`document.body.textContent.includes(${JSON.stringify(NOMBRE_UI)})`);
t = await texto();
afirma(t.includes(NOMBRE_UI) && t.includes('Ficha de colegio'), 'ficha de colegio con datos del directorio');

// --- 3. Baja de colegio deja de verse ---
await api(`${API}/api/colegios/${encodeURIComponent(RBD_UI)}`, { method: 'DELETE' });
await ir(`/colegios/${encodeURIComponent(RBD_UI)}`);
t = await texto();
afirma(t.includes('Colegio no encontrado'), 'colegio eliminado muestra aviso en su ficha');

// --- 4. Kanban de admisión: avanzar etapa persiste (fila propia) ---
const ORDEN_ES = ['Contacto', 'Postulante', 'Evaluado', 'Aceptado', 'Matriculado'];
const A_TEXTO_DB = { Contacto: 'contacto', Postulante: 'postulante', Evaluado: 'evaluado', Aceptado: 'aceptado', Matriculado: 'matriculado' };
const NOMBRE_OPP = `Kanban ${SELLO}`;
const oppCreada = await (await api(`${API}/api/oportunidades`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ nombre: NOMBRE_OPP, importe: 50000 }),
})).json();
await ir('/oportunidades');
await esperarCondicion(`[...document.querySelectorAll('.opp-card button')].some(b => b.textContent.includes('▶'))`);
const antes = await evaluar(`
  (() => {
    const nombre = ${JSON.stringify(NOMBRE_OPP)};
    const tarjeta = [...document.querySelectorAll('.opp-card')].find(c => (c.querySelector('.opp-name')?.textContent || '').includes(nombre));
    const columna = tarjeta.closest('.col');
    const boton = [...tarjeta.querySelectorAll('button')].find(b => b.textContent.includes('▶'));
    boton.click();
    return {
      nombre,
      etapa: columna.querySelector('.stage').textContent.trim(),
    };
  })()
`);
console.log('  kanban antes:', JSON.stringify(antes));
const idxAntes = ORDEN_ES.indexOf(antes.etapa);
const esperadoEs = ORDEN_ES[Math.min(idxAntes + 1, ORDEN_ES.length - 1)];
await esperarCondicion(`(() => {
  const tarjeta = [...document.querySelectorAll('.opp-card')].find(c => (c.querySelector('.opp-name')?.textContent || '').includes(${JSON.stringify(antes.nombre)}));
  if (!tarjeta) return false;
  return tarjeta.closest('.col').querySelector('.stage').textContent.trim() === ${JSON.stringify(esperadoEs)};
})()`);
const despues = await evaluar(`
  (() => {
    const tarjeta = [...document.querySelectorAll('.opp-card')].find(c => (c.querySelector('.opp-name')?.textContent || '').includes(${JSON.stringify(antes.nombre)}));
    if (!tarjeta) return null;
    return tarjeta.closest('.col').querySelector('.stage').textContent.trim();
  })()
`);
afirma(despues === esperadoEs, `kanban mueve tarjeta en el frontal (${antes.etapa} → ${despues})`);
const lista = await (await api(`${API}/api/oportunidades?limite=200`)).json();
const tocada = lista.find((o) => o.nombre === antes.nombre);
afirma(
  tocada?.etapa === A_TEXTO_DB[esperadoEs],
  `avance kanban persiste en PG (${antes.nombre}: ${tocada?.etapa})`,
);
await api(`${API}/api/oportunidades/${oppCreada.id}`, { method: 'DELETE' });

// --- 5. Acceso renderiza en español ---
await ir('/acceso');
t = await texto();
afirma(t.includes('Bienvenido de nuevo'), 'pantalla de acceso en español');

// --- 6. Edición en ficha de persona persiste (fila propia) ---
const propia = await (await api(`${API}/api/personas`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ nombre: 'Edit', apellido: `E2E ${SELLO}`, correo: `edit-${SELLO}@test.cl` }),
})).json();
await ir(`/personas/${propia.id}`);
// Abre edición (✎) antes de rellenar.
await esperarCondicion(`[...document.querySelectorAll('.record-panel .btn-mini')].some(b => b.textContent.includes('✎'))`);
await evaluar(`[...document.querySelectorAll('.record-panel .btn-mini')].find(b => b.textContent.includes('✎')).click()`);
await dormir(800);
await evaluar(`
  (() => {
    const puesto = [...document.querySelectorAll('.panel.form input')][4];
    puesto.focus();
    puesto.value = 'Puesto E2E ${SELLO}';
    puesto.dispatchEvent(new Event('input', { bubbles: true }));
  })()
`);
await evaluar(`[...document.querySelectorAll('.panel.form button')].find(b => b.textContent.includes('Guardar')).click()`);
await dormir(2500);
const persEdit = await (await api(`${API}/api/personas/${propia.id}`)).json();
afirma(persEdit.puesto === `Puesto E2E ${SELLO}`, 'edición de persona en ficha persiste en PG');
await api(`${API}/api/personas/${propia.id}`, { method: 'DELETE' });

// --- 7. Territorio: la cascada filtra ---
await ir('/personas');
await esperarCondicion(`document.querySelectorAll('.panel.form select').length >= 2`);
await evaluar(`document.querySelector('.ghost-new').click()`);
await dormir(800);
await evaluar(`
  (() => {
    const region = document.querySelectorAll('.panel.form select')[0];
    region.value = '13';
    region.dispatchEvent(new Event('change', { bubbles: true }));
  })()
`);
await dormir(1000);
const comunasRM = await evaluar(`
  [...document.querySelectorAll('.panel.form select')[1].options].map(o => o.textContent)
`);
afirma(comunasRM.includes('Santiago'), 'cascada muestra comunas de RM');
afirma(!comunasRM.includes('Iquique'), 'cascada filtra fuera comunas de otras regiones');

// --- 8. Guardado con región y comuna persiste códigos ---
const NOMBRE_T = `Territorio ${SELLO}`;
await evaluar(`
  (() => {
    const inputs = document.querySelectorAll('.panel.form input');
    const set = (el, v) => { el.focus(); el.value = v; el.dispatchEvent(new Event('input', { bubbles: true })); };
    set(inputs[0], ${JSON.stringify(NOMBRE_T)});
    set(inputs[1], 'E2E');
    set(inputs[2], ${JSON.stringify(`terr-${SELLO}@test.cl`)});
    const comuna = document.querySelectorAll('.panel.form select')[1];
    comuna.value = '13101';
    comuna.dispatchEvent(new Event('change', { bubbles: true }));
  })()
`);
await evaluar(`[...document.querySelectorAll('.panel.form button')].find(b => b.textContent.includes('Crear')).click()`);
await dormir(2500);
const filaT = await (await api(`${API}/api/personas?busqueda=${encodeURIComponent(NOMBRE_T)}`)).json();
afirma(
  filaT.length === 1 && filaT[0].region_codigo === 13 && filaT[0].comuna_codigo === '13101',
  'persona guardada con región 13 y comuna 13101',
);

// --- 9. Comuna de otra región se rechaza ---
const mala = await (
  await api(`${API}/api/personas`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ nombre: 'X', apellido: 'Y', correo: `mala-${SELLO}@test.cl`, region_codigo: 13, comuna_codigo: '01101' }),
  })
).json();
afirma(mala.codigo === 'datos_invalidos', 'comuna ajena a la región se rechaza');
await api(`${API}/api/personas/${filaT[0].id}`, { method: 'DELETE' });

// --- 10. Alta de alumno con RUT válido por UI ---
await ir('/alumnos');
await esperarCondicion(`!!document.querySelector('.ghost-new')`);
// Limpieza de corridas interrumpidas.
try {
  const previo = await (await api(`${API}/api/alumnos?busqueda=19654321-K`)).json();
  for (const r of previo) await api(`${API}/api/alumnos/${r.id}`, { method: 'DELETE' });
} catch {}
await evaluar(`document.querySelector('.ghost-new').click()`);
await dormir(800);
await evaluar(`
  (() => {
    const inputs = document.querySelectorAll('.panel.form input');
    const set = (el, v) => { el.focus(); el.value = v; el.dispatchEvent(new Event('input', { bubbles: true })); };
    set(inputs[0], '19654321-K');
    set(inputs[1], 'E2E Alumno');
    set(inputs[2], ${JSON.stringify(SELLO)});
    set(inputs[3], '8° Básico A');
  })()
`);
await evaluar(`[...document.querySelectorAll('.panel.form button')].find(b => b.textContent.includes('Crear')).click()`);
await dormir(2500);
const filaA = await (await api(`${API}/api/alumnos?busqueda=${encodeURIComponent(`E2E Alumno ${SELLO}`)}`)).json();
afirma(filaA.length === 1 && filaA[0].rut === '19654321-K', 'alumno con RUT válido persiste en PG');

// --- 11. RUT inválido se rechaza ---
const rutMalo = await (
  await api(`${API}/api/alumnos`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ rut: '19654321-9', nombres: 'X', apellidos: 'Y' }),
  })
).json();
afirma(rutMalo.codigo === 'datos_invalidos', 'RUT con DV malo se rechaza');

// --- 12. Apoderado + vínculo en ficha de alumno ---
const RUT_APO = '20123456-5';
const apo = await (await api(`${API}/api/apoderados`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ rut: RUT_APO, nombres: 'Vínc', apellidos: `E2E ${SELLO}` }),
})).json();
afirma(!!apo.id, 'apoderado con RUT válido se crea');
await ir(`/alumnos/${filaA[0].id}`);
await esperarCondicion(
  `[...document.querySelectorAll('.panel select')].some(s => [...s.options].some(o => o.value === ${JSON.stringify(apo.id)}))`,
);
await evaluar(`
  (() => {
    const btn = [...document.querySelectorAll('.panel button')].find(b => b.textContent.includes('Vincular'));
    const row = btn.closest('.row');
    const sel = row.querySelector('select');
    sel.value = ${JSON.stringify(apo.id)};
    sel.dispatchEvent(new Event('change', { bubbles: true }));
    const inp = row.querySelector('input');
    inp.focus(); inp.value = 'Padre';
    inp.dispatchEvent(new Event('input', { bubbles: true }));
  })()
`);
await evaluar(`[...document.querySelectorAll('.panel button')].find(b => b.textContent.includes('Vincular')).click()`);
await dormir(2500);
const vinc = await (await api(`${API}/api/alumnos/${filaA[0].id}/apoderados`)).json();
afirma(vinc.length === 1 && vinc[0].vinculo === 'Padre', 'vínculo apoderado persiste en PG');
t = await texto();
afirma(t.includes('Padre'), 'vínculo visible en ficha de alumno');
await api(`${API}/api/alumnos/${filaA[0].id}`, { method: 'DELETE' });
await api(`${API}/api/apoderados/${apo.id}`, { method: 'DELETE' });

// --- 13. Traslado por UI: cambia colegio y registra historial ---
const alumT = await (await api(`${API}/api/alumnos?busqueda=12345678-5`)).json();
const idAT = alumT[0].id;
const histAntes = await (await api(`${API}/api/alumnos/${idAT}/traslados`)).json();
const cands = await (await api(`${API}/api/colegios?region=13&limite=2`)).json();
const destinoT = cands.find((c) => c.rbd !== alumT[0].colegio_rbd) || cands[0];
await ir(`/alumnos/${idAT}`);
await esperarCondicion(`document.querySelectorAll('.record-panel select').length >= 2`);
await evaluar(`
  (() => {
    const sels = document.querySelectorAll('.record-panel select');
    sels[0].value = '13';
    sels[0].dispatchEvent(new Event('change', { bubbles: true }));
  })()
`);
await dormir(1000);
await evaluar(`
  (() => {
    const sels = document.querySelectorAll('.record-panel select');
    const opc = [...sels[1].options].find(o => o.value === ${JSON.stringify(destinoT.comuna)});
    if (opc) { sels[1].value = ${JSON.stringify(destinoT.comuna)}; sels[1].dispatchEvent(new Event('change', { bubbles: true })); }
    const inputs = document.querySelectorAll('.record-panel input');
    const b = [...inputs].find(i => (i.placeholder || '').includes('destino'));
    if (b) { b.focus(); b.value = ${JSON.stringify(destinoT.nombre.slice(0, 12))}; b.dispatchEvent(new Event('input', { bubbles: true })); }
  })()
`);
await evaluar(`[...document.querySelectorAll('.record-panel button')].find(b => b.textContent.includes('Buscar')).click()`);
await dormir(2500);
await esperarCondicion(`[...document.querySelectorAll('.record-panel .mini-row')].some(f => f.textContent.includes(${JSON.stringify(destinoT.rbd)}) && [...f.querySelectorAll('button')].some(b => b.textContent.includes('Elegir')))`);
await evaluar(`
  (() => {
    const filas = [...document.querySelectorAll('.record-panel .mini-row')];
    const fila = filas.find(f => f.textContent.includes(${JSON.stringify(destinoT.rbd)}) && [...f.querySelectorAll('button')].some(b => b.textContent.includes('Elegir')));
    if (fila) [...fila.querySelectorAll('button')].find(b => b.textContent.includes('Elegir')).click();
  })()
`);
await dormir(800);
await evaluar(`
  (() => {
    const inputs = document.querySelectorAll('.record-panel input');
    const m = [...inputs].find(i => (i.placeholder || '').includes('Motivo'));
    if (m) { m.focus(); m.value = 'E2E traslado'; m.dispatchEvent(new Event('input', { bubbles: true })); }
  })()
`);
await evaluar(`[...document.querySelectorAll('.record-panel button')].find(b => b.textContent.includes('Trasladar')).click()`);
await dormir(3000);
const alumDesp = await (await api(`${API}/api/alumnos/${idAT}`)).json();
afirma(alumDesp.colegio_rbd === destinoT.rbd, `traslado cambia colegio a ${destinoT.rbd}`);
const histDesp = await (await api(`${API}/api/alumnos/${idAT}/traslados`)).json();
afirma(
  histDesp.length === histAntes.length + 1 && histDesp[0].motivo === 'E2E traslado',
  'traslado registra historial con motivo',
);

// --- 14. Tarea por UI: alta y avance persisten ---
const TIT_T = `Tarea ${SELLO}`;
await ir('/tareas');
await esperarCondicion(`!!document.querySelector('.ghost-new')`);
await evaluar(`document.querySelector('.ghost-new').click()`);
await dormir(800);
await evaluar(`
  (() => {
    const input = document.querySelector('.panel.form input');
    input.focus(); input.value = ${JSON.stringify(TIT_T)};
    input.dispatchEvent(new Event('input', { bubbles: true }));
  })()
`);
await evaluar(`[...document.querySelectorAll('.panel.form button')].find(b => b.textContent.includes('Crear')).click()`);
await dormir(2500);
const filaTT = await (await api(`${API}/api/tareas?busqueda=${encodeURIComponent(TIT_T)}`)).json();
afirma(filaTT.length === 1 && filaTT[0].estado === 'pendiente', 'tarea creada por UI persiste en PG');
await evaluar(`
  (() => {
    const tarjeta = [...document.querySelectorAll('.opp-card')].find(c => (c.querySelector('.opp-name')?.textContent || '').includes(${JSON.stringify(TIT_T)}));
    [...tarjeta.querySelectorAll('button')].find(b => b.textContent.includes('▶')).click();
  })()
`);
await dormir(3000);
const tareaMov = await (await api(`${API}/api/tareas?busqueda=${encodeURIComponent(TIT_T)}`)).json();
afirma(tareaMov[0]?.estado === 'en_curso', 'avance de tarea persiste en PG');
await api(`${API}/api/tareas/${filaTT[0].id}`, { method: 'DELETE' });

// --- 15. Nota por UI: alta, edición y baja ---
const TIT_N = `Nota ${SELLO}`;
await ir('/notas');
await esperarCondicion(`!!document.querySelector('.ghost-new')`);
await evaluar(`document.querySelector('.ghost-new').click()`);
await dormir(800);
await evaluar(`
  (() => {
    const inputs = document.querySelectorAll('.panel.form input');
    const set = (el, v) => { el.focus(); el.value = v; el.dispatchEvent(new Event('input', { bubbles: true })); };
    set(inputs[0], ${JSON.stringify(TIT_N)});
    set(inputs[1], 'Contenido E2E');
  })()
`);
await evaluar(`[...document.querySelectorAll('.panel.form button')].find(b => b.textContent.includes('Crear')).click()`);
await dormir(2500);
let filaN = await (await api(`${API}/api/notas?busqueda=${encodeURIComponent(TIT_N)}`)).json();
afirma(filaN.length === 1, 'nota creada por UI persiste en PG');
await evaluar(`
  (() => {
    const fila = [...document.querySelectorAll('.tbl tbody tr')].find(tr => tr.textContent.includes(${JSON.stringify(TIT_N)}));
    [...fila.querySelectorAll('button')].find(b => b.textContent.includes('✎')).click();
  })()
`);
await dormir(800);
await evaluar(`
  (() => {
    const fila = [...document.querySelectorAll('.tbl tbody tr')].find(tr => tr.textContent.includes(${JSON.stringify(TIT_N)}));
    const input = fila.querySelector('input');
    input.focus(); input.value = 'Editada E2E';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    [...fila.querySelectorAll('button')].find(b => b.textContent.includes('Guardar')).click();
  })()
`);
await dormir(2500);
filaN = await (await api(`${API}/api/notas?busqueda=${encodeURIComponent(TIT_N)}`)).json();
afirma(filaN[0]?.cuerpo === 'Editada E2E', 'edición de nota persiste en PG');
await evaluar(`
  (() => {
    const fila = [...document.querySelectorAll('.tbl tbody tr')].find(tr => tr.textContent.includes(${JSON.stringify(TIT_N)}));
    [...fila.querySelectorAll('button')].find(b => b.textContent === '✕').click();
  })()
`);
await dormir(2500);
const filaNB = await (await api(`${API}/api/notas?busqueda=${encodeURIComponent(TIT_N)}`)).json();
afirma(filaNB.length === 0, 'baja de nota por UI persiste en PG');

// --- 16. Guardia: sin sesión se cae a acceso y tras entrar vuelve ---
await enviar('Page.navigate', { url: `${FRONTAL}/alumnos` });
await esperarCondicion(`(document.body.textContent.length || 0) > 80`);
await dormir(1500);
t = await texto();
afirma(t.includes('Bienvenido de nuevo'), 'ruta protegida sin sesión muestra acceso');
await loginUI(E2E_CORREO, E2E_CLAVE);
await esperarCondicion(`document.body.textContent.includes('Alumnos') && !document.body.textContent.includes('Bienvenido de nuevo')`);
t = await texto();
afirma(t.includes('Alumnos'), 'tras entrar vuelve al destino pedido');

// --- 17. Registro por UI crea cuenta y entra ---
await enviar('Page.navigate', { url: `${FRONTAL}/acceso` });
await esperarCondicion(`(document.body.textContent.length || 0) > 80`);
await dormir(1500);
const CORREO_UI = `ui-${SELLO}@test.cl`;
await evaluar(
  `[...document.querySelectorAll('.login-card button')].find(b => b.textContent.includes('Sin cuenta')).click()`,
);
await dormir(800);
await evaluar(`(() => {
  const set = (sel, v) => {
    const el = document.querySelector(sel);
    el.focus(); el.value = v;
    el.dispatchEvent(new Event('input', { bubbles: true }));
  };
  set('.login-card input[type="text"]', 'E2E UI');
  set('.login-card input[type="email"]', ${JSON.stringify(CORREO_UI)});
  set('.login-card input[type="password"]', 'Clave1234');
})()`);
await evaluar(
  `[...document.querySelectorAll('.login-card button')].find(b => b.textContent.trim() === 'Crear cuenta').click()`,
);
await esperarCondicion(`!document.body.textContent.includes('Bienvenido de nuevo')`);
await dormir(1500);
t = await texto();
afirma(t.includes('E2E UI'), 'registro por UI entra y muestra el nombre');

// --- 18. Miembros: el listado se ve, sin botones de rol para miembro ---
// (La API guarda el correo en minúsculas: se compara en minúsculas.)
const E2E_CORREO_MIN = E2E_CORREO.toLowerCase();
await ir('/miembros');
await esperarCondicion(`document.body.textContent.includes(${JSON.stringify(E2E_CORREO_MIN)})`);
t = await texto();
afirma(t.includes(E2E_CORREO_MIN), 'miembros muestra la lista del espacio');
afirma(!t.includes('Hacer admin') && !t.includes('Quitar admin'), 'miembro no ve botones de rol');

// --- 19. Eventos y correos por API: validaciones y mes ---
const EV_TIT = `Evento ${SELLO}`;
const evCreado = await (await api(`${API}/api/eventos`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ titulo: EV_TIT, inicio: '2026-11-20T15:00:00Z', fin: '2026-11-20T16:00:00Z', lugar: 'Oficina' }),
})).json();
afirma(!!evCreado.id, 'evento se crea por API');
const evMal = await (await api(`${API}/api/eventos`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ titulo: 'X', inicio: '2026-11-20T16:00:00Z', fin: '2026-11-20T15:00:00Z' }),
})).json();
afirma(evMal.codigo === 'datos_invalidos', 'evento con fin anterior se rechaza');
const evMes = await (await api(`${API}/api/eventos/mes?ano=2026&mes=11`)).json();
afirma(evMes.some((e) => e.id === evCreado.id), 'evento aparece en su mes');
const evMesMal = await (await api(`${API}/api/eventos/mes?ano=2026&mes=13`)).json();
afirma(evMesMal.codigo === 'datos_invalidos', 'mes 13 se rechaza');
const COR_ASUNTO = `Correo ${SELLO}`;
const corCreado = await (await api(`${API}/api/correos`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ asunto: COR_ASUNTO, cuerpo: 'Hola', remitente: 'colegio@test.cl' }),
})).json();
afirma(!!corCreado.id && corCreado.leido === false, 'correo nace no leído');
const corMal = await (await api(`${API}/api/correos`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ cuerpo: 'x', remitente: 'mal' }),
})).json();
afirma(corMal.codigo === 'datos_invalidos', 'remitente inválido se rechaza');
const corLeido = await (await api(`${API}/api/correos/${corCreado.id}/leido`, {
  method: 'PATCH',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({}),
})).json();
afirma(corLeido.leido === true, 'marcar leído persiste');
const soloLeidos = await (await api(`${API}/api/correos?no_leidos=true`)).json();
afirma(!soloLeidos.some((c) => c.id === corCreado.id), 'leído sale del filtro no leídos');
await api(`${API}/api/eventos/${evCreado.id}`, { method: 'DELETE' });
await api(`${API}/api/correos/${corCreado.id}`, { method: 'DELETE' });

// --- 20. Evento por UI aparece en su día ---
const TIT_EV = `UI Evento ${SELLO}`;
const ahora = new Date();
const YM = `${ahora.getFullYear()}-${String(ahora.getMonth() + 1).padStart(2, '0')}-15`;
await ir('/calendario');
await esperarCondicion(`!!document.querySelector('.cal-grid')`);
await evaluar(`[...document.querySelectorAll('.cal-bar button')].find(b => b.textContent.includes('Nuevo evento')).click()`);
await dormir(800);
await evaluar(`
  (() => {
    const inputs = document.querySelectorAll('.panel.form input');
    const set = (el, v) => { el.focus(); el.value = v; el.dispatchEvent(new Event('input', { bubbles: true })); };
    set(inputs[0], ${JSON.stringify(TIT_EV)});
    set(inputs[1], ${JSON.stringify(`${YM}T10:00`)});
    set(inputs[2], ${JSON.stringify(`${YM}T11:00`)});
  })()
`);
await evaluar(`[...document.querySelectorAll('.panel.form button')].find(b => b.textContent.includes('Guardar')).click()`);
const evEnDia = await esperarCondicion(`(() => {
  const celda = document.querySelector(${JSON.stringify(`[data-dia="${YM}"]`)});
  return celda && celda.textContent.includes(${JSON.stringify(TIT_EV)});
})()`);
afirma(evEnDia, 'evento creado por UI aparece en su día');
const evUi = await (await api(`${API}/api/eventos?busqueda=${encodeURIComponent(TIT_EV)}`)).json();
afirma(evUi.length === 1, 'evento de UI persiste en PG');

// --- 21. Correo vinculado por UI aparece en ficha de apoderado + leído ---
const RUT_APO4 = '22222222-2';
const apo4 = await (await api(`${API}/api/apoderados`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ rut: RUT_APO4, nombres: 'Correo', apellidos: `E2E ${SELLO}` }),
})).json();
const ASUNTO_UI = `UI Asunto ${SELLO}`;
await ir('/correos');
await esperarCondicion(`!!document.querySelector('.cal-bar button')`);
await evaluar(`[...document.querySelectorAll('.cal-bar button')].find(b => b.textContent.includes('Redactar')).click()`);
await dormir(800);
await evaluar(`
  (() => {
    const inputs = document.querySelectorAll('.panel.form input');
    const set = (el, v) => { el.focus(); el.value = v; el.dispatchEvent(new Event('input', { bubbles: true })); };
    set(inputs[0], ${JSON.stringify(ASUNTO_UI)});
    set(inputs[1], 'apoderado@test.cl');
    set(inputs[2], 'Reunión el viernes');
    const sels = document.querySelectorAll('.panel.form select');
    sels[1].value = ${JSON.stringify(apo4.id)};
    sels[1].dispatchEvent(new Event('change', { bubbles: true }));
  })()
`);
await evaluar(`[...document.querySelectorAll('.panel.form button')].find(b => b.textContent.includes('Guardar')).click()`);
const corEnBandeja = await esperarCondicion(`(() => [...document.querySelectorAll('[data-correo]')].some(f => f.textContent.includes(${JSON.stringify(ASUNTO_UI)})))`);
afirma(corEnBandeja, 'correo redactado aparece en la bandeja');
// Abrirlo lo marca leído (reintenta el clic si el lector tarda).
let lectorOk = false;
for (let i = 0; i < 3 && !lectorOk; i++) {
  await evaluar(`[...document.querySelectorAll('[data-correo]')].find(f => f.textContent.includes(${JSON.stringify(ASUNTO_UI)})).click()`);
  lectorOk = await esperarCondicion(`document.body.textContent.includes('Reunión el viernes')`, 8000);
}
afirma(lectorOk, 'abrir correo muestra el lector');
let corUi = [];
for (let i = 0; i < 12; i++) {
  await dormir(1000);
  corUi = await (await api(`${API}/api/correos?busqueda=${encodeURIComponent(ASUNTO_UI)}`)).json();
  if (corUi.length === 1 && corUi[0].leido === true) break;
}
afirma(corUi.length === 1 && corUi[0].leido === true, 'abrir correo lo marca leído en PG');
// Vinculado: se ve en la ficha del apoderado.
await ir(`/apoderados/${apo4.id}`);
await esperarCondicion(`document.body.textContent.includes(${JSON.stringify(ASUNTO_UI)})`);
t = await texto();
afirma(t.includes(ASUNTO_UI), 'correo vinculado aparece en ficha de apoderado');
await api(`${API}/api/correos/${corUi[0].id}`, { method: 'DELETE' });
await api(`${API}/api/apoderados/${apo4.id}`, { method: 'DELETE' });
await api(`${API}/api/eventos/${evUi[0].id}`, { method: 'DELETE' });

// --- 22. Contador de no leídos en el menú ---
const corBadge = await (await api(`${API}/api/correos`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ asunto: `Badge ${SELLO}`, cuerpo: 'x', remitente: 'colegio@test.cl' }),
})).json();
await ir('/correos');
const badge = await esperarCondicion(`(() => {
  const link = [...document.querySelectorAll('.side-item')].find(e => e.textContent.includes('Correos'));
  return link && /[1-9]/.test(link.textContent);
})()`);
afirma(badge, 'menú muestra contador de no leídos');
await api(`${API}/api/correos/${corBadge.id}`, { method: 'DELETE' });

// --- 23. Búsqueda global, favoritos, vistas guardadas y timeline ---
const NOMBRE_F5 = `Global F5 ${SELLO}`;
const CORREO_F5 = `global-${SELLO}@test.cl`;
const CORREO_F5_MIN = CORREO_F5.toLowerCase();
const personaF5 = await (await api(`${API}/api/personas`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ nombre: NOMBRE_F5, apellido: 'Búsqueda', correo: CORREO_F5 }),
})).json();
const buscaNombre = await (await api(`${API}/api/buscar?q=${encodeURIComponent(NOMBRE_F5)}`)).json();
afirma(buscaNombre.some((r) => r.tipo === 'persona' && r.id === personaF5.id), 'búsqueda global encuentra por nombre');
const buscaCorreo = await (await api(`${API}/api/buscar?q=${encodeURIComponent(CORREO_F5)}`)).json();
afirma(buscaCorreo.some((r) => r.tipo === 'persona' && r.id === personaF5.id), 'búsqueda global encuentra por correo');
const buscaCorta = await (await api(`${API}/api/buscar?q=a`)).json();
afirma(Array.isArray(buscaCorta) && buscaCorta.length <= 20, 'búsqueda de un carácter es segura y acotada');

const favoritoF5 = await (await api(`${API}/api/favoritos/alternar`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ tipo: 'persona', id: personaF5.id }),
})).json();
const favoritosF5 = await (await api(`${API}/api/favoritos`)).json();
afirma(favoritoF5.agregado && favoritosF5.some((f) => f.id === personaF5.id), 'favorito personal persiste en PG');
const favoritosAjeno = await (await apiAdmin(`${API}/api/favoritos`)).json();
afirma(!favoritosAjeno.some((f) => f.id === personaF5.id), 'favoritos quedan aislados por usuario');

const NOMBRE_VISTA = `Vista ${SELLO}`;
const vistaF5 = await (await api(`${API}/api/vistas?objeto=personas`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ nombre: NOMBRE_VISTA, tipo: 'tabla', filtros: { busqueda: NOMBRE_F5 } }),
})).json();
const vistasF5 = await (await api(`${API}/api/vistas?objeto=personas`)).json();
afirma(!!vistaF5.id && vistasF5.some((v) => v.id === vistaF5.id), 'vista guardada queda asociada al usuario');
const vistasAjeno = await (await apiAdmin(`${API}/api/vistas?objeto=personas`)).json();
afirma(!vistasAjeno.some((v) => v.id === vistaF5.id), 'vistas quedan aisladas por usuario');

// Crear una modificación atribuida al actor del JWT y abrir la ficha desde ⌘K.
await api(`${API}/api/personas/${personaF5.id}`, {
  method: 'PATCH',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ nombre: NOMBRE_F5, apellido: 'Búsqueda', correo: CORREO_F5, telefono: '555-0115' }),
});
await enviar('Page.navigate', { url: `${FRONTAL}/personas` });
await esperarCondicion(`document.body.textContent.includes('Bienvenido de nuevo')`);
await loginUI(E2E_CORREO, E2E_CLAVE);
await esperarCondicion(`document.body.textContent.includes('Personas')`);
await evaluar(`document.querySelector('.shell').dispatchEvent(new KeyboardEvent('keydown', { key: 'k', metaKey: true, bubbles: true }))`);
const modalBusqueda = await esperarCondicion(`!!document.querySelector('.search-dialog input')`);
afirma(modalBusqueda, 'atajo ⌘K abre la búsqueda global');
await evaluar(`(() => {
  const i = document.querySelector('.search-dialog input');
  i.focus(); i.value = ${JSON.stringify(CORREO_F5)}; i.dispatchEvent(new Event('input', { bubbles: true }));
})()`);
const apareceResultado = await esperarCondicion(`!![...document.querySelectorAll('.search-result')].find(x => x.textContent.includes(${JSON.stringify(CORREO_F5_MIN)}))`);
if (!apareceResultado) {
  console.log('  DIAG búsqueda:', JSON.stringify(await evaluar(`({url: location.href, q: document.querySelector('.search-dialog input')?.value, resultados: [...document.querySelectorAll('.search-result')].map(x => x.textContent).slice(0, 8), body: document.querySelector('.search-dialog')?.textContent?.slice(0, 300)})`)));
}
afirma(apareceResultado, 'buscador global muestra resultados agrupados y navegables');
if (apareceResultado) {
  await evaluar(`[...document.querySelectorAll('.search-result')].find(x => x.textContent.includes(${JSON.stringify(CORREO_F5_MIN)})).click()`);
  await esperarCondicion(`document.body.textContent.includes('Ficha de persona')`);
} else {
  await ir(`/personas/${personaF5.id}`);
  await esperarCondicion(`document.body.textContent.includes('Ficha de persona')`);
}
const timelineVisible = await esperarCondicion(`document.body.textContent.includes('actualizó el registro')`);
afirma(timelineVisible, 'edición genera actividad visible en la ficha');
afirma((await texto()).includes('E2E API'), 'timeline muestra autor del JWT');

// La estrella y la vista cargan de nuevo desde servidor tras navegar.
await ir('/personas');
await esperarCondicion(`!![...document.querySelectorAll('tr')].find(x => x.textContent.includes(${JSON.stringify(NOMBRE_F5)}))`);
const estrellaPersistida = await evaluar(`(() => {
  const fila = [...document.querySelectorAll('tr')].find(x => x.textContent.includes(${JSON.stringify(NOMBRE_F5)}));
  return !!fila?.querySelector('.star-favorite.marked');
})()`);
afirma(estrellaPersistida, 'estrella refleja favorito tras volver a cargar');
const selectVista = await esperarCondicion(`!![...document.querySelectorAll('.saved-view-select option')].find(o => o.textContent === ${JSON.stringify(NOMBRE_VISTA)})`);
afirma(selectVista, 'selector presenta vistas personales guardadas');
await evaluar(`(() => {
  const s = document.querySelector('.saved-view-select');
  const o = [...s.options].find(x => x.textContent === ${JSON.stringify(NOMBRE_VISTA)});
  s.value = o.value; s.dispatchEvent(new Event('change', { bubbles: true }));
})()`);
const vistaAplica = await esperarCondicion(`document.querySelector('.toolbar input')?.value === ${JSON.stringify(NOMBRE_F5)}`);
afirma(vistaAplica, 'elegir vista aplica sus filtros guardados');

await api(`${API}/api/favoritos/alternar`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ tipo: 'persona', id: personaF5.id }),
});
await api(`${API}/api/vistas/${vistaF5.id}`, { method: 'DELETE' });
await api(`${API}/api/personas/${personaF5.id}`, { method: 'DELETE' });

// --- 24. Automatización escolar guiada, activación y ejecución fallida ---
const NOMBRE_REGLA = `Regla E2E ${SELLO}`;
const TITULO_AUTO = `Tarea automática ${SELLO}`;
await ir('/automatizaciones');
await esperarCondicion(`!!document.querySelector('.automation-form input')`);
await evaluar(`(() => {
  const form = document.querySelector('.automation-form');
  const inputs = form.querySelectorAll('input');
  const set = (el, v) => { el.focus(); el.value = v; el.dispatchEvent(new Event('input', { bubbles: true })); };
  set(inputs[0], ${JSON.stringify(NOMBRE_REGLA)});
  set(inputs[3], ${JSON.stringify(TITULO_AUTO)});
})()`);
await evaluar(`[...document.querySelectorAll('.automation-form button')].find(b => b.className.includes('btn-primary')).click()`);
const reglaVisible = await esperarCondicion(`document.body.textContent.includes(${JSON.stringify(NOMBRE_REGLA)})`);
afirma(reglaVisible, 'regla de admisión se crea desde el editor guiado');
const regla = (await (await api(`${API}/api/automatizaciones`)).json()).find((r) => r.nombre === NOMBRE_REGLA);
afirma(!!regla?.id, 'regla guiada persiste en PG');

const OPP_AUTO = `Postulación automática ${SELLO}`;
const oppAuto = await (await api(`${API}/api/oportunidades`, {
  method: 'POST', headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ nombre: OPP_AUTO, importe: 150000 }),
})).json();
let tareasAuto = [];
for (let i = 0; i < 12; i++) {
  await dormir(500);
  tareasAuto = await (await api(`${API}/api/tareas?busqueda=${encodeURIComponent(TITULO_AUTO)}`)).json();
  if (tareasAuto.length) break;
}
afirma(tareasAuto.length === 1 && tareasAuto[0].oportunidad_id === oppAuto.id, 'regla crea tarea vinculada a la postulación');

await ir('/automatizaciones');
await esperarCondicion(`document.body.textContent.includes(${JSON.stringify(NOMBRE_REGLA)})`);
await evaluar(`(() => {
  const card = [...document.querySelectorAll('.automation-card')].find(c => c.textContent.includes(${JSON.stringify(NOMBRE_REGLA)}));
  [...card.querySelectorAll('button')].find(b => b.textContent.includes('Desactivar')).click();
})()`);
await esperarCondicion(`(() => {
  const card = [...document.querySelectorAll('.automation-card')].find(c => c.textContent.includes(${JSON.stringify(NOMBRE_REGLA)}));
  return card && card.textContent.includes('Inactiva');
})()`);
const OPP_APAGADA = `Postulación sin automatización ${SELLO}`;
const oppApagada = await (await api(`${API}/api/oportunidades`, {
  method: 'POST', headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ nombre: OPP_APAGADA, importe: 160000 }),
})).json();
const sinTarea = await (await api(`${API}/api/tareas?busqueda=${encodeURIComponent(TITULO_AUTO)}`)).json();
afirma(sinTarea.length === 1, 'desactivar la regla evita nuevas tareas');

const REGLA_MALA = `Regla fallida ${SELLO}`;
const reglaMala = await (await api(`${API}/api/automatizaciones`, {
  method: 'POST', headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    nombre: REGLA_MALA, disparador: 'al_crear', condiciones: [],
    acciones: [{ tipo: 'crear_tarea', params: { titulo: `Tarea inválida ${SELLO}`, prioridad: 'urgente' } }],
  }),
})).json();
const OPP_MALA = `Postulación con regla fallida ${SELLO}`;
const oppMalaResp = await api(`${API}/api/oportunidades`, {
  method: 'POST', headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ nombre: OPP_MALA, importe: 1 }),
});
const oppMala = await oppMalaResp.json();
const histMala = await (await api(`${API}/api/automatizaciones/${reglaMala.id}/ejecuciones`)).json();
afirma(oppMalaResp.status === 201 && !!oppMala.id, 'error de regla no revierte la postulación');
afirma(histMala.some((x) => !x.ok), 'error de regla queda en historial de ejecuciones');

for (const o of [oppAuto, oppApagada, oppMala]) {
  await api(`${API}/api/oportunidades/${o.id}`, { method: 'DELETE' });
}
for (const tarea of tareasAuto) await api(`${API}/api/tareas/${tarea.id}`, { method: 'DELETE' });
await api(`${API}/api/automatizaciones/${regla.id}`, { method: 'DELETE' });
await api(`${API}/api/automatizaciones/${reglaMala.id}`, { method: 'DELETE' });

// --- 25. Agente: conversación privada y modo no configurado seguro ---
const conv = await (await api(`${API}/api/conversaciones`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({}),
})).json();
afirma(!!conv.id && conv.usuario_id === e2eIni.usuario.id, 'conversación queda asociada al usuario autenticado');
const sinLlm = await api(`${API}/api/conversaciones/${conv.id}/preguntar`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ pregunta: '¿Cuántas postulaciones hay?' }),
});
const sinLlmJson = await sinLlm.json();
afirma(sinLlm.status === 503 && sinLlmJson.codigo === 'no_configurado', 'LLM sin configurar no afecta ni expone errores al resto');
const conversacionAjena = await apiAdmin(`${API}/api/conversaciones/${conv.id}/mensajes`);
afirma(conversacionAjena.status === 404, 'conversaciones y mensajes se aíslan por usuario');
await api(`${API}/api/conversaciones/${conv.id}`, { method: 'DELETE' });

// El chat sigue disponible aunque falte la configuración del proveedor.
const convAntesUi = await (await api(`${API}/api/conversaciones`)).json();
await ir('/agentes');
await esperarCondicion(`!!document.querySelector('.chat-layout')`);
await evaluar(`[...document.querySelectorAll('.chat-history button')].find(b => b.textContent.includes('Nueva conversación')).click()`);
await dormir(500);
await evaluar(`(() => {
  const area = document.querySelector('.chat-compose textarea');
  area.focus(); area.value = '¿Cuántas postulaciones hay?';
  area.dispatchEvent(new Event('input', { bubbles: true }));
  [...document.querySelectorAll('.chat-compose button')].find(b => b.textContent.includes('Enviar')).click();
})()`);
const avisoSinLlm = await esperarCondicion(`document.body.textContent.includes('El asistente no está configurado')`);
afirma(avisoSinLlm, 'chat indica configuración ausente sin afectar el espacio');
const convDespuesUi = await (await api(`${API}/api/conversaciones`)).json();
const convUi = convDespuesUi.find((c) => !convAntesUi.some((x) => x.id === c.id));
afirma(!!convUi, 'chat UI crea y persiste conversación personal');
if (convUi) await api(`${API}/api/conversaciones/${convUi.id}`, { method: 'DELETE' });

// Opcional en el compose local: API de prueba en otro puerto con
// LLM_URL=mock://occb-e2e, misma base y APP_SECRETO.
if (process.env.LLM_MOCK_API_URL) {
  const API_MOCK = process.env.LLM_MOCK_API_URL;
  const apiMock = async (ruta, opciones = {}) => {
    let r = await fetch(`${API_MOCK}${ruta}`, {
      ...opciones,
      headers: { ...(opciones.headers || {}), Authorization: `Bearer ${SES.acceso}` },
    });
    if (r.status === 401) {
      await renovarSesion(SES);
      r = await fetch(`${API_MOCK}${ruta}`, {
        ...opciones,
        headers: { ...(opciones.headers || {}), Authorization: `Bearer ${SES.acceso}` },
      });
    }
    return r;
  };
  const convMock = await (await apiMock('/api/conversaciones', {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ titulo: 'Mock E2E' }),
  })).json();
  const panelReal = await (await api(`${API}/api/panel`)).json();
  const totalMock = await (await apiMock(`/api/conversaciones/${convMock.id}/preguntar`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ pregunta: '¿Cuál es el total real de la tubería?' }),
  })).json();
  afirma(totalMock.respuesta.includes(String(panelReal.valor_tuberia)), 'proveedor simulado usa el total real del panel');
  const notaMock = await (await apiMock(`/api/conversaciones/${convMock.id}/preguntar`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ pregunta: 'Crear una nota de prueba' }),
  })).json();
  const notasMock = await (await api(`${API}/api/notas?busqueda=Nota%20creada%20por%20asistente`)).json();
  afirma(notasMock.some((n) => notaMock.fuentes.some((f) => f.id === n.id)), 'asistente simulado crea nota por función segura y la cita');
  for (const n of notasMock) await api(`${API}/api/notas/${n.id}`, { method: 'DELETE' });
  await apiMock(`/api/conversaciones/${convMock.id}`, { method: 'DELETE' });
} else {
  console.log('SKIP — proveedor LLM simulado (define LLM_MOCK_API_URL para ejecutarlo)');
}

console.log(fallos === 0 ? 'E2E: TODO CORRECTO' : `E2E: ${fallos} FALLOS`);
ws.close();
try {
  process.kill(chrome.pid);
} catch {}
process.exit(fallos === 0 ? 0 : 1);
