mod db;
#[cfg(test)]
mod tests;
mod ui;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{
    Router,
    extract::Request,
    extract::{DefaultBodyLimit, Form, Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use rand::RngCore;
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
struct App {
    db: Arc<Mutex<Connection>>,
    secure: bool,
}
#[derive(Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub role: String,
    pub csrf: String,
}
fn random() -> String {
    let mut b = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut b);
    hex::encode(b)
}
fn digest(s: &str) -> String {
    hex::encode(Sha256::digest(s.as_bytes()))
}
fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
fn cookie(h: &HeaderMap, name: &str) -> Option<String> {
    h.get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|s| {
            s.trim()
                .strip_prefix(&format!("{name}="))
                .map(str::to_string)
        })
}
fn user(c: &Connection, h: &HeaderMap) -> Option<User> {
    let token = cookie(h, "ms_session")?;
    c.query_row("SELECT u.id,u.name,u.role,s.csrf FROM sessions s JOIN users u ON u.id=s.user_id WHERE token_hash=?1 AND expires_at>?2 AND u.active=1",params![digest(&token),now()],|r|Ok(User{id:r.get(0)?,name:r.get(1)?,role:r.get(2)?,csrf:r.get(3)?})).optional().ok().flatten()
}
fn set_cookie(response: &mut Response, name: &str, value: &str, age: i64, secure: bool) {
    let value = format!(
        "{name}={value}; Path=/; Max-Age={age}; HttpOnly; SameSite=Strict{}",
        if secure { "; Secure" } else { "" }
    );
    if let Ok(v) = value.parse() {
        response.headers_mut().append(header::SET_COOKIE, v);
    }
}
fn hash_password(password: &str) -> db::Result<String> {
    if password.len() < 12 || password.len() > 200 {
        return Err("A senha deve ter entre 12 e 200 caracteres.".into());
    }
    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|s| s.to_string())
        .map_err(|_| "Não foi possível proteger a senha.".into())
}
async fn blocking(f: impl FnOnce() -> Response + Send + 'static) -> Response {
    tokio::task::spawn_blocking(f).await.unwrap_or_else(|e| {
        eprintln!("Request worker failed: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Falha interna. Tente novamente.",
        )
            .into_response()
    })
}
async fn security(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    for (k, v) in [
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        ("referrer-policy", "same-origin"),
        ("cache-control", "no-store"),
        (
            "permissions-policy",
            "camera=(), microphone=(), geolocation=()",
        ),
        (
            "content-security-policy",
            "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'none'; form-action 'self'",
        ),
    ] {
        response
            .headers_mut()
            .insert(axum::http::HeaderName::from_static(k), v.parse().unwrap());
    }
    response
}
async fn page(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    path: Option<Path<String>>,
) -> Response {
    blocking(move || {
        let c = app.db.lock().unwrap();
        let Some(u) = user(&c, &headers) else {
            return Redirect::to("/login").into_response();
        };
        let page = path.map(|p| p.0).unwrap_or_else(|| "painel".into());
        match ui::render(&c, &u, &page, &q) {
            Ok(html) => Html(html).into_response(),
            Err(e) => {
                eprintln!("Page rendering failed: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, Html(ui::error_page())).into_response()
            }
        }
    })
    .await
}
async fn login_page(State(app): State<App>) -> Response {
    let nonce = random();
    let mut response = Html(ui::login(&nonce, None)).into_response();
    set_cookie(&mut response, "ms_login", &nonce, 600, app.secure);
    response
}
async fn login(State(app): State<App>, headers: HeaderMap, Form(f): Form<db::Form>) -> Response {
    blocking(move||{
        let nonce=f.get("csrf").cloned().unwrap_or_default();
        if nonce.len()!=64 || cookie(&headers,"ms_login").as_deref()!=Some(&nonce){return (StatusCode::FORBIDDEN,"Sessão do formulário expirada. Reabra /login.").into_response();}
        let email=f.get("email").map(|s|s.trim().to_lowercase()).unwrap_or_default();
        let password=f.get("password").cloned().unwrap_or_default();
        if email.len()>180 || password.len()>200{return (StatusCode::BAD_REQUEST,"Dados inválidos.").into_response();}
        let c=app.db.lock().unwrap(); let key=digest(&email);
        let locked=c.query_row("SELECT locked_until FROM login_attempts WHERE key=?1",[&key],|r|r.get::<_,i64>(0)).unwrap_or(0);
        if locked>now(){return (StatusCode::TOO_MANY_REQUESTS,Html(ui::login(&nonce,Some("Muitas tentativas. Aguarde 15 minutos.")))).into_response();}
        let record=c.query_row("SELECT id,password_hash FROM users WHERE email=?1 AND active=1",[&email],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?))).optional().ok().flatten();
        let valid=record.as_ref().is_some_and(|(_,hash)|PasswordHash::new(hash).ok().is_some_and(|p|Argon2::default().verify_password(password.as_bytes(),&p).is_ok()));
        if !valid {
            let _=c.execute("INSERT INTO login_attempts(key,failures,locked_until) VALUES(?1,1,0) ON CONFLICT(key) DO UPDATE SET failures=CASE WHEN locked_until>0 AND locked_until<=?2 THEN 1 ELSE failures+1 END,locked_until=CASE WHEN failures>=4 AND (locked_until=0 OR locked_until>?2) THEN ?2+900 ELSE 0 END",params![key,now()]);
            return (StatusCode::UNAUTHORIZED,Html(ui::login(&nonce,Some("E-mail ou senha incorretos.")))).into_response();
        }
        let id=record.unwrap().0; let token=random(); let csrf=random();
        let result=c.execute("DELETE FROM sessions WHERE expires_at<=?1",[now()]).and_then(|_|c.execute("DELETE FROM login_attempts WHERE key=?1",[key])).and_then(|_|c.execute("INSERT INTO sessions(token_hash,user_id,csrf,expires_at) VALUES(?1,?2,?3,?4)",params![digest(&token),id,csrf,now()+28800]));
        if result.is_err(){return (StatusCode::INTERNAL_SERVER_ERROR,"Não foi possível abrir a sessão.").into_response();}
        let _=db::audit(&c,id,"login","user",id,"Autenticação");
        let mut response=Redirect::to("/").into_response(); set_cookie(&mut response,"ms_session",&token,28800,app.secure);set_cookie(&mut response,"ms_login","",0,app.secure);response
    }).await
}
async fn action(
    State(app): State<App>,
    headers: HeaderMap,
    Path(action): Path<String>,
    Form(f): Form<db::Form>,
) -> Response {
    blocking(move || {
        let mut c = app.db.lock().unwrap();
        let Some(u) = user(&c, &headers) else {
            return Redirect::to("/login").into_response();
        };
        if f.get("csrf") != Some(&u.csrf) {
            return (
                StatusCode::FORBIDDEN,
                "Sessão do formulário inválida. Atualize a página.",
            )
                .into_response();
        }
        if action == "logout" {
            if let Some(token) = cookie(&headers, "ms_session") {
                let _ = c.execute("DELETE FROM sessions WHERE token_hash=?1", [digest(&token)]);
            }
            let mut response = Redirect::to("/login").into_response();
            set_cookie(&mut response, "ms_session", "", 0, app.secure);
            return response;
        }
        let outcome = if action == "user" {
            create_user(&mut c, &u, &f)
        } else if action == "password" {
            change_password(&mut c, &u, &f)
        } else {
            db::apply(&mut c, u.id, &u.role, &action, &f)
        };
        let back = match action.as_str() {
            "family" | "enroll" | "enrollment" | "import" => "familias",
            "product" | "price" | "discard" => "estoque",
            "donation" => "doacoes",
            "cycle" => "ciclos",
            "user" | "password" => "configuracoes",
            _ => "atendimento",
        };
        match outcome {
            Ok(m) => {
                Redirect::to(&format!("/{back}?ok={}", urlencoding::encode(&m))).into_response()
            }
            Err(e) => {
                Redirect::to(&format!("/{back}?erro={}", urlencoding::encode(&e))).into_response()
            }
        }
    })
    .await
}
fn create_user(c: &mut Connection, u: &User, f: &db::Form) -> db::Result<String> {
    if u.role != "admin" {
        return Err("Somente administradores podem criar usuários.".into());
    }
    let name = db::text(f, "name", 100)?;
    let email = db::text(f, "email", 180)?.to_lowercase();
    let role = db::text(f, "role", 20)?;
    if !email.contains('@') || !["admin", "volunteer"].contains(&role.as_str()) {
        return Err("E-mail ou perfil inválido.".into());
    }
    let hash = hash_password(f.get("password").map(String::as_str).unwrap_or(""))?;
    let tx = c.transaction().map_err(db::db_err)?;
    tx.execute(
        "INSERT INTO users(name,email,password_hash,role) VALUES(?1,?2,?3,?4)",
        params![name, email, hash, role],
    )
    .map_err(db::db_err)?;
    db::audit(&tx, u.id, "create", "user", tx.last_insert_rowid(), &email).map_err(db::db_err)?;
    tx.commit().map_err(db::db_err)?;
    Ok("Usuário criado.".into())
}
fn change_password(c: &mut Connection, u: &User, f: &db::Form) -> db::Result<String> {
    let old = db::text(f, "old_password", 200)?;
    let hash: String = c
        .query_row("SELECT password_hash FROM users WHERE id=?1", [u.id], |r| {
            r.get(0)
        })
        .map_err(db::db_err)?;
    if !PasswordHash::new(&hash).ok().is_some_and(|p| {
        Argon2::default()
            .verify_password(old.as_bytes(), &p)
            .is_ok()
    }) {
        return Err("Senha atual incorreta.".into());
    }
    let hash = hash_password(f.get("password").map(String::as_str).unwrap_or(""))?;
    let tx = c.transaction().map_err(db::db_err)?;
    tx.execute(
        "UPDATE users SET password_hash=?1 WHERE id=?2",
        params![hash, u.id],
    )
    .map_err(db::db_err)?;
    tx.execute("DELETE FROM sessions WHERE user_id=?1", [u.id])
        .map_err(db::db_err)?;
    db::audit(
        &tx,
        u.id,
        "password",
        "user",
        u.id,
        "Senha alterada; sessões encerradas",
    )
    .map_err(db::db_err)?;
    tx.commit().map_err(db::db_err)?;
    Ok("Senha alterada. Entre novamente.".into())
}
async fn export(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    blocking(move||{
        let c=app.db.lock().unwrap();if user(&c,&headers).is_none(){return Redirect::to("/login").into_response();}
        let month=q.get("month").cloned().unwrap_or_else(||db::today().format("%Y-%m").to_string());if db::month_index(&month).is_err(){return (StatusCode::BAD_REQUEST,"Mês inválido.").into_response();}
        let rows=db::rows(&c,"SELECT co.id,f.reference,f.name,co.month,p.name,i.quantity,i.unit_credits,i.quantity*i.unit_credits,co.created_at FROM checkouts co JOIN enrollments e ON e.id=co.enrollment_id JOIN families f ON f.id=e.family_id JOIN checkout_items i ON i.checkout_id=co.id JOIN products p ON p.id=i.product_id WHERE co.month=?1 ORDER BY co.id,p.name",&[&month]);
        let Ok(rows)=rows else{return (StatusCode::INTERNAL_SERVER_ERROR,"Erro ao exportar.").into_response()};
        let mut writer=csv::WriterBuilder::new().delimiter(b';').from_writer(Vec::new());
        let _=writer.write_record(["entrega","referencia","familia","mes","produto","quantidade","credito_unitario_centavos","total_centavos","data_utc"]);
        for row in rows {let safe:Vec<_>=row.into_iter().map(|s|if s.trim_start().starts_with(['=','+','-','@','\t','\r']){format!("'{s}")}else{s}).collect();let _=writer.write_record(safe);}
        let Ok(bytes)=writer.into_inner()else{return (StatusCode::INTERNAL_SERVER_ERROR,"Erro ao exportar.").into_response()};
        ([(header::CONTENT_TYPE,"text/csv; charset=utf-8".to_string()),(header::CONTENT_DISPOSITION,format!("attachment; filename=balanco-{month}.csv"))],bytes).into_response()
    }).await
}
#[tokio::main]
async fn main() {
    let path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "data/mercado.db".into());
    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent).expect("Criar diretório do banco");
    }
    let c = Connection::open(&path).expect("Abrir SQLite");
    db::init(&c).expect("Inicializar SQLite");
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("backup") {
        let dest = args
            .get(2)
            .expect("Uso: mercado-solidario backup /caminho/backup.db");
        c.execute("VACUUM INTO ?1", [dest])
            .expect("Backup consistente do SQLite");
        println!("Backup criado.");
        return;
    }
    let count: i64 = c
        .query_row("SELECT count(*) FROM users", [], |r| r.get(0))
        .unwrap();
    if count == 0 {
        let password = std::env::var("ADMIN_PASSWORD")
            .expect("Defina ADMIN_PASSWORD (mínimo 12 caracteres) no primeiro início");
        let email = std::env::var("ADMIN_EMAIL").expect("Defina ADMIN_EMAIL no primeiro início");
        let hash = hash_password(&password).expect("Senha inicial inválida");
        c.execute("INSERT INTO users(name,email,password_hash,role) VALUES('Coordenação IBT',?1,?2,'admin')",params![email.to_lowercase(),hash]).expect("Criar administrador");
    }
    let app = App {
        db: Arc::new(Mutex::new(c)),
        secure: std::env::var("COOKIE_SECURE").unwrap_or_else(|_| "true".into()) != "false",
    };
    let router = Router::new()
        .route("/", get(page))
        .route("/login", get(login_page).post(login))
        .route("/acoes/{action}", post(action))
        .route("/exportar", get(export))
        .route(
            "/assets/app.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("../static/app.css"),
                )
            }),
        )
        .route(
            "/assets/app.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../static/app.js"),
                )
            }),
        )
        .route(
            "/healthz",
            get(|State(app): State<App>| async move {
                blocking(move || {
                    match app
                        .db
                        .lock()
                        .unwrap()
                        .query_row("SELECT 1", [], |r| r.get::<_, i64>(0))
                    {
                        Ok(1) => (StatusCode::OK, "ok").into_response(),
                        _ => (StatusCode::SERVICE_UNAVAILABLE, "unavailable").into_response(),
                    }
                })
                .await
            }),
        )
        .route("/{page}", get(page))
        .layer(DefaultBodyLimit::max(256 * 1024))
        .layer(middleware::from_fn(security))
        .with_state(app);
    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3100".into());
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .expect("Porta indisponível");
    println!("Mercado Solidário em {bind}");
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .unwrap();
}
