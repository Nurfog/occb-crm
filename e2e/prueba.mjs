// Prueba E2E de OCCB CRM en Chrome headless vía CDP (sin dependencias).
// Uso: node e2e/prueba.mjs
// Requiere: frontal en :8080, API en :3001 y Chrome en /opt/google/chrome/chrome.
// Cubre: panel vivo, colegios (API+ficha), kanban de admisión persistido,
// acceso en español, edición en fichas y territorio chileno.

const CHROME = process.env.CHROME_BIN || '/opt/google/chrome/chrome';
const PUERTO_CD = 9222;
const FRONTAL = process.env.FRONTAL_URL || 'http://127.0.0.1:8080';
const API = process.env.API_URL || 'http://127.0.0.1:3001';
const SELLO = `E2E-${Date.now().toString(36)}`;

const { spawn } = await import('node:child_process');

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
  await esperarCondicion(`(document.getElementById('main')?.innerText.length || 0) > 200`);
  await dormir(2000);
}
async function texto() {
  return evaluar(`document.body.innerText`);
}

let fallos = 0;
function afirma(cond, nombre) {
  console.log(`${cond ? 'PASS' : 'FAIL'} — ${nombre}`);
  if (!cond) fallos++;
}

// --- 1. Panel con datos vivos ---
await ir('/');
let t = await texto();
afirma(t.includes('● API :3001'), 'panel indica API en línea (no demo local)');
afirma(t.includes('Colegios') && t.includes('RBD'), 'panel muestra colegios de la API');

// --- 2. Alta de colegio (API) visible en su ficha ---
const RBD_UI = `E2E-${SELLO}`;
const NOMBRE_UI = `Colegio UI ${SELLO}`;
await fetch(`${API}/api/colegios`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ rbd: RBD_UI, nombre: NOMBRE_UI, region: 13, comuna: '13101' }),
});
await ir(`/colegios/${encodeURIComponent(RBD_UI)}`);
t = await texto();
afirma(t.includes(NOMBRE_UI) && t.includes('Ficha de colegio'), 'ficha de colegio con datos del directorio');

// --- 3. Baja de colegio deja de verse ---
await fetch(`${API}/api/colegios/${encodeURIComponent(RBD_UI)}`, { method: 'DELETE' });
await ir(`/colegios/${encodeURIComponent(RBD_UI)}`);
t = await texto();
afirma(t.includes('Colegio no encontrado'), 'colegio eliminado muestra aviso en su ficha');

// --- 4. Kanban de admisión: avanzar etapa persiste (fila propia) ---
const ORDEN_ES = ['Contacto', 'Postulante', 'Evaluado', 'Aceptado', 'Matriculado'];
const A_TEXTO_DB = { Contacto: 'contacto', Postulante: 'postulante', Evaluado: 'evaluado', Aceptado: 'aceptado', Matriculado: 'matriculado' };
const NOMBRE_OPP = `Kanban ${SELLO}`;
const oppCreada = await (await fetch(`${API}/api/oportunidades`, {
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
const lista = await (await fetch(`${API}/api/oportunidades?limite=200`)).json();
const tocada = lista.find((o) => o.nombre === antes.nombre);
afirma(
  tocada?.etapa === A_TEXTO_DB[esperadoEs],
  `avance kanban persiste en PG (${antes.nombre}: ${tocada?.etapa})`,
);
await fetch(`${API}/api/oportunidades/${oppCreada.id}`, { method: 'DELETE' });

// --- 5. Acceso renderiza en español ---
await ir('/acceso');
t = await texto();
afirma(t.includes('Bienvenido de nuevo'), 'pantalla de acceso en español');

// --- 6. Edición en ficha de persona persiste (fila propia) ---
const propia = await (await fetch(`${API}/api/personas`, {
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
const persEdit = await (await fetch(`${API}/api/personas/${propia.id}`)).json();
afirma(persEdit.puesto === `Puesto E2E ${SELLO}`, 'edición de persona en ficha persiste en PG');
await fetch(`${API}/api/personas/${propia.id}`, { method: 'DELETE' });

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
const filaT = await (await fetch(`${API}/api/personas?busqueda=${encodeURIComponent(NOMBRE_T)}`)).json();
afirma(
  filaT.length === 1 && filaT[0].region_codigo === 13 && filaT[0].comuna_codigo === '13101',
  'persona guardada con región 13 y comuna 13101',
);

// --- 9. Comuna de otra región se rechaza ---
const mala = await (
  await fetch(`${API}/api/personas`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ nombre: 'X', apellido: 'Y', correo: `mala-${SELLO}@test.cl`, region_codigo: 13, comuna_codigo: '01101' }),
  })
).json();
afirma(mala.codigo === 'datos_invalidos', 'comuna ajena a la región se rechaza');
await fetch(`${API}/api/personas/${filaT[0].id}`, { method: 'DELETE' });

// --- 10. Alta de alumno con RUT válido por UI ---
await ir('/alumnos');
await esperarCondicion(`!!document.querySelector('.ghost-new')`);
// Limpieza de corridas interrumpidas.
try {
  const previo = await (await fetch(`${API}/api/alumnos?busqueda=19654321-K`)).json();
  for (const r of previo) await fetch(`${API}/api/alumnos/${r.id}`, { method: 'DELETE' });
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
const filaA = await (await fetch(`${API}/api/alumnos?busqueda=${encodeURIComponent(`E2E Alumno ${SELLO}`)}`)).json();
afirma(filaA.length === 1 && filaA[0].rut === '19654321-K', 'alumno con RUT válido persiste en PG');

// --- 11. RUT inválido se rechaza ---
const rutMalo = await (
  await fetch(`${API}/api/alumnos`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ rut: '19654321-9', nombres: 'X', apellidos: 'Y' }),
  })
).json();
afirma(rutMalo.codigo === 'datos_invalidos', 'RUT con DV malo se rechaza');

// --- 12. Apoderado + vínculo en ficha de alumno ---
const RUT_APO = '20123456-5';
const apo = await (await fetch(`${API}/api/apoderados`, {
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
const vinc = await (await fetch(`${API}/api/alumnos/${filaA[0].id}/apoderados`)).json();
afirma(vinc.length === 1 && vinc[0].vinculo === 'Padre', 'vínculo apoderado persiste en PG');
t = await texto();
afirma(t.includes('Padre'), 'vínculo visible en ficha de alumno');
await fetch(`${API}/api/alumnos/${filaA[0].id}`, { method: 'DELETE' });
await fetch(`${API}/api/apoderados/${apo.id}`, { method: 'DELETE' });

// --- 13. Traslado por UI: cambia colegio y registra historial ---
const alumT = await (await fetch(`${API}/api/alumnos?busqueda=12345678-5`)).json();
const idAT = alumT[0].id;
const histAntes = await (await fetch(`${API}/api/alumnos/${idAT}/traslados`)).json();
const cands = await (await fetch(`${API}/api/colegios?region=13&limite=2`)).json();
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
const alumDesp = await (await fetch(`${API}/api/alumnos/${idAT}`)).json();
afirma(alumDesp.colegio_rbd === destinoT.rbd, `traslado cambia colegio a ${destinoT.rbd}`);
const histDesp = await (await fetch(`${API}/api/alumnos/${idAT}/traslados`)).json();
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
const filaTT = await (await fetch(`${API}/api/tareas?busqueda=${encodeURIComponent(TIT_T)}`)).json();
afirma(filaTT.length === 1 && filaTT[0].estado === 'pendiente', 'tarea creada por UI persiste en PG');
await evaluar(`
  (() => {
    const tarjeta = [...document.querySelectorAll('.opp-card')].find(c => (c.querySelector('.opp-name')?.textContent || '').includes(${JSON.stringify(TIT_T)}));
    [...tarjeta.querySelectorAll('button')].find(b => b.textContent.includes('▶')).click();
  })()
`);
await dormir(3000);
const tareaMov = await (await fetch(`${API}/api/tareas?busqueda=${encodeURIComponent(TIT_T)}`)).json();
afirma(tareaMov[0]?.estado === 'en_curso', 'avance de tarea persiste en PG');
await fetch(`${API}/api/tareas/${filaTT[0].id}`, { method: 'DELETE' });

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
let filaN = await (await fetch(`${API}/api/notas?busqueda=${encodeURIComponent(TIT_N)}`)).json();
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
filaN = await (await fetch(`${API}/api/notas?busqueda=${encodeURIComponent(TIT_N)}`)).json();
afirma(filaN[0]?.cuerpo === 'Editada E2E', 'edición de nota persiste en PG');
await evaluar(`
  (() => {
    const fila = [...document.querySelectorAll('.tbl tbody tr')].find(tr => tr.textContent.includes(${JSON.stringify(TIT_N)}));
    [...fila.querySelectorAll('button')].find(b => b.textContent === '✕').click();
  })()
`);
await dormir(2500);
const filaNB = await (await fetch(`${API}/api/notas?busqueda=${encodeURIComponent(TIT_N)}`)).json();
afirma(filaNB.length === 0, 'baja de nota por UI persiste en PG');

console.log(fallos === 0 ? 'E2E: TODO CORRECTO' : `E2E: ${fallos} FALLOS`);
ws.close();
try {
  process.kill(chrome.pid);
} catch {}
process.exit(fallos === 0 ? 0 : 1);
