// End-to-end HTTP checks against an isolated SQLite database. No production data.
import { spawn } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { resolve, join } from 'node:path';
import assert from 'node:assert/strict';
const root = resolve(import.meta.dirname, '..');
const dir = join(root, 'target', `smoke-${Date.now()}`);
mkdirSync(dir, {recursive: true});
const binary = process.env.SMOKE_BINARY || join(root, 'target', 'debug', `mercado-solidario${process.platform === 'win32' ? '.exe' : ''}`);
const base = 'http://127.0.0.1:13101';
const child = spawn(binary, [], {cwd: root, env: {...process.env, DATABASE_PATH: join(dir, 'test.db'), BIND_ADDR: '127.0.0.1:13101', COOKIE_SECURE: 'false', ADMIN_EMAIL: 'admin@ibt.test', ADMIN_PASSWORD: 'SomenteTesteLocal123!'}, stdio: ['ignore', 'pipe', 'pipe']});
let logs = '';
child.stdout.on('data', chunk => {logs += chunk;});
child.stderr.on('data', chunk => {logs += chunk;});
const cookies = new Map();
async function req(path, body) {
  const response = await fetch(base + path, {method: body ? 'POST' : 'GET', redirect: 'manual', headers: {'Cookie': [...cookies].map(([k,v]) => `${k}=${v}`).join('; '), ...(body ? {'Content-Type': 'application/x-www-form-urlencoded'} : {})}, ...(body ? {body: new URLSearchParams(body)} : {})});
  for (const cookie of response.headers.getSetCookie()) {const [k,v] = cookie.split(';')[0].split('='); cookies.set(k,v);}
  return {status: response.status, html: await response.text(), location: response.headers.get('location'), headers: response.headers};
}
let assertions = 0;
function check(value, message) {assert.ok(value, message); assertions++;}
try {
  for (let i=0;i<120;i++) {try {const r=await fetch(base+'/healthz');if(r.ok)break;}catch{} await new Promise(r=>setTimeout(r,250));if(child.exitCode!==null)throw Error(logs);}
  check((await req('/')).location === '/login', 'Anonymous dashboard redirects to login');
  let r = await req('/login'); const nonce=r.html.match(/name="csrf"[^>]*value="([^"]+)"/)[1];
  const loginCookie = cookies.get('ms_login');
  const icon = await req('/favicon.ico');
  check(icon.status === 200 && icon.headers.get('content-type').includes('image/svg+xml') && !icon.location, 'Browser favicon request never redirects to login');
  check(cookies.get('ms_login') === loginCookie, 'Favicon does not overwrite the login nonce');
  r = await req('/missing-asset.txt');
  check(r.status === 404 && !r.location && cookies.get('ms_login') === loginCookie, 'Missing assets do not redirect or reset login state');
  r = await req('/login');
  check(r.html.match(/name="csrf"[^>]*value="([^"]+)"/)[1] === nonce, 'Opening a second login tab preserves the first form');
  r = await req('/login', {csrf:'invalid-token', email:'admin@ibt.test', password:'SomenteTesteLocal123!'});
  check(r.status === 403 && r.html.includes('name="password"') && r.html.includes('name="csrf"'), 'Invalid login CSRF is rejected with a usable recovery form');
  check((await req('/login',{csrf:nonce,email:'admin@ibt.test',password:'SomenteTesteLocal123!'})).location==='/', 'Login sets a session');
  r=await req('/');check(r.status===200&&r.html.includes('Visão geral'), 'SSR dashboard renders');
  const csrf=r.html.match(/name="csrf"[^>]*value="([^"]+)"/)[1];
  check(r.headers.get('content-security-policy').includes("script-src 'self'"), 'CSP blocks inline scripts');
  check((await req('/acoes/product',{csrf:'wrong'})).status===403, 'CSRF is required');
  const today=new Date().toLocaleDateString('en-CA',{timeZone:'America/Sao_Paulo'});
  const month=today.slice(0,7);
  const later=new Date(Date.now()+86400000*45).toISOString().slice(0,10);
  const save=async(action,data)=>{const r=await req('/acoes/'+action,{csrf,...data});check(r.status===303&&r.location.includes('?ok='), `${action}: ${decodeURIComponent(r.location||r.html)}`);return r;};
  await save('cycle',{name:'Ciclo de teste',start_month:month});
  await save('family',{name:'Família Teste',reference:'QA-001',members:'4',cycle_id:'1',group_code:'A',credit:'100,00',criteria:'Dados fictícios para teste'});
  r=await req('/acoes/family',{csrf,name:'Duplicada',reference:'QA-001',members:'1',cycle_id:'1',group_code:'A',credit:'100',criteria:'Teste'});check(r.location.includes('?erro='),'Duplicate family reference rejected');
  await save('product',{name:'Arroz QA',unit:'1 kg',minimum:'5',price:'5,00'});
  await save('product',{name:'Feijão QA',unit:'1 kg',minimum:'5',price:'3,50'});
  await save('donation',{product_id:'1',donor:'Doador fictício',quantity:'10',value:'50,00',code:'QA-ARROZ',received_on:today,expires_on:later});
  await save('donation',{product_id:'2',donor:'Doador fictício',quantity:'10',value:'35,00',code:'QA-FEIJAO',received_on:today,expires_on:later});
  r=await req('/atendimento');check(r.html.includes('QA-001'), 'Eligible family visible at checkout');
  const key=r.html.match(/name="request_key"[^>]*value="([^"]+)"/)[1];
  await save('checkout',{enrollment_id:'1',request_key:key,qty_1:'2',qty_2:'3'});
  r=await req('/atendimento');check(!r.html.includes('value="1" data-credit='),'Served family removed from eligible list');
  r=await req('/recibo?id=1');check(r.status===200&&r.html.includes('20,50')&&r.html.includes('QA-ARROZ'), 'Receipt preserves total and lot traceability');
  r=await req('/acoes/checkout',{csrf,enrollment_id:'1',request_key:'another-request',qty_1:'1'});check(r.location.includes('?erro='), 'Second monthly withdrawal blocked');
  r=await req('/exportar?month='+month);check(r.status===200&&r.html.includes('QA-001')&&r.html.includes('1000')&&r.html.includes('1050'), 'Monthly CSV includes both checkout lines');
  for (const path of ['familias','estoque','doacoes','atendimento','relatorios','ciclos','configuracoes','historico?family=1','cartao?family=1']) {r=await req('/'+path);check(r.status===200,'Page renders: '+path);if(path.startsWith('cartao'))check(r.html.includes('class="family-card"')&&r.html.includes('QA-001')&&r.html.includes('Imprimir cartão'),'Family card renders printable identity');}
  await save('user',{name:'Voluntário QA',email:'voluntario@ibt.test',password:'VoluntarioTeste123!',role:'volunteer'});
  check((await req('/acoes/logout',{csrf})).location==='/login', 'Logout redirects to login');
  check((await req('/familias')).location==='/login', 'Logout revokes session');
  r=await req('/login');const nonce2=r.html.match(/name="csrf"[^>]*value="([^"]+)"/)[1];
  await req('/login',{csrf:nonce2,email:'voluntario@ibt.test',password:'VoluntarioTeste123!'});
  r=await req('/');const csrf2=r.html.match(/name="csrf"[^>]*value="([^"]+)"/)[1];
  r=await req('/acoes/cycle',{csrf:csrf2,name:'Proibido',start_month:'2028-01'});check(r.location.includes('?erro='),'Volunteer cannot create cycles');
  console.log(`PASS: ${assertions} HTTP checks, including login, CSRF, CRUD, atomic checkout, duplicate withdrawal, receipt, CSV, roles and logout.`);
} catch (error) {console.error(logs);throw error;} finally {if(child.exitCode===null){const closed=new Promise(r=>child.once('close',r));child.kill();await closed;}}
