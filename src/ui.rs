use crate::{User, db};
use leptos::prelude::*;
use rusqlite::Connection;
use std::collections::HashMap;
type R<T> = rusqlite::Result<T>;
fn av(v: impl IntoView + 'static) -> AnyView {
    v.into_any()
}
fn field(
    name: &'static str,
    label: &'static str,
    kind: &'static str,
    value: impl Into<String>,
) -> AnyView {
    let value = value.into();
    av(
        view! {<label class="field"><span>{label}</span><input name=name type=kind value=value required min=if kind=="number"{"0"}else{""} maxlength="200" step=if kind=="number"{"1"}else{"any"}/></label>},
    )
}
fn credit(name: &'static str, label: &'static str, value: &str) -> AnyView {
    av(
        view! {<label class="field"><span>{label}</span><input name=name type="text" inputmode="decimal" value=value.to_owned() required maxlength="20" placeholder="0,00"/></label>},
    )
}
fn select(name: &'static str, label: &'static str, options: Vec<(String, String)>) -> AnyView {
    av(
        view! {<label class="field"><span>{label}</span><select name=name required><option value="" disabled selected>"Selecione…"</option>{options.into_iter().map(|(v,t)|view!{<option value=v>{t}</option>}).collect_view()}</select></label>},
    )
}
fn memo(name: &'static str, label: &'static str, placeholder: &'static str) -> AnyView {
    av(
        view! {<label class="field full"><span>{label}</span><textarea name=name required maxlength="200000" rows="3" placeholder=placeholder></textarea></label>},
    )
}
fn form(action: &str, csrf: &str, fields: Vec<AnyView>, button: &str) -> AnyView {
    let action = format!("/acoes/{action}");
    let csrf = csrf.to_owned();
    let button = button.to_owned();
    av(
        view! {<form method="post" action=action class="form-grid"><input type="hidden" name="csrf" value=csrf/>{fields}<div class="form-footer full"><button class="button" type="submit">{button}<span aria-hidden="true">" ↗"</span></button><span class="form-note">"Os dados ficam registrados no histórico."</span></div></form>},
    )
}
fn panel(title: &str, sub: &str, content: AnyView) -> AnyView {
    let title = title.to_owned();
    let sub = sub.to_owned();
    av(
        view! {<section class="panel"><div class="panel-head"><div><h2>{title}</h2><p>{sub}</p></div></div>{content}</section>},
    )
}
fn disclosure(title: &str, content: AnyView) -> AnyView {
    let title = title.to_owned();
    av(
        view! {<details class="panel disclosure"><summary>{title}<span aria-hidden="true">"＋"</span></summary><div class="details-content">{content}</div></details>},
    )
}
fn table(headers: Vec<&'static str>, rows: Vec<Vec<String>>) -> AnyView {
    let empty = rows.is_empty();
    av(
        view! {<div class="table-wrap"><table><thead><tr>{headers.into_iter().map(|h|view!{<th scope="col">{h}</th>}).collect_view()}</tr></thead><tbody>{rows.into_iter().map(|r|view!{<tr>{r.into_iter().map(|s|view!{<td>{s}</td>}).collect_view()}</tr>}).collect_view()}</tbody></table>{empty.then(||view!{<div class="empty"><span class="empty-icon">"◇"</span><h3>"Ainda não há registros"</h3><p>"Os registros aparecerão aqui depois do primeiro lançamento."</p></div>})}</div>},
    )
}
fn opts(c: &Connection, sql: &str) -> R<Vec<(String, String)>> {
    Ok(db::rows(c, sql, &[])?
        .into_iter()
        .map(|r| (r[0].clone(), r[1].clone()))
        .collect())
}
fn cycles(c: &Connection) -> R<Vec<(String, String)>> {
    opts(
        c,
        "SELECT id,name||' · '||start_month FROM cycles ORDER BY start_month DESC",
    )
}
fn products(c: &Connection) -> R<Vec<(String, String)>> {
    opts(
        c,
        "SELECT id,name||' · '||unit FROM products WHERE active=1 ORDER BY name",
    )
}
fn scalar(c: &Connection, sql: &str) -> R<i64> {
    c.query_row(sql, [], |r| r.get(0))
}
fn formatted(mut rows: Vec<Vec<String>>, indices: &[usize]) -> Vec<Vec<String>> {
    for row in &mut rows {
        for &i in indices {
            row[i] = db::display_money(row[i].parse().unwrap_or(0));
        }
    }
    rows
}
fn title(page: &str) -> (&'static str, &'static str) {
    match page {
        "familias" => (
            "Famílias & créditos",
            "Cadastros organizados. Mais tempo para acolher.",
        ),
        "estoque" => ("Estoque", "Cada item acompanhado, da doação à entrega."),
        "doacoes" => (
            "Entrada de doações",
            "Generosidade que chega, cuidado que se multiplica.",
        ),
        "atendimento" => ("Atendimento", "Uma entrega simples, segura e acolhedora."),
        "relatorios" => (
            "Relatórios",
            "Transparência para acompanhar o impacto do mercado.",
        ),
        "ciclos" => (
            "Ciclos de atendimento",
            "Seis meses de cuidado. Dois grupos intercalados.",
        ),
        "configuracoes" => (
            "Equipe & acesso",
            "Um acesso individual para cada voluntário.",
        ),
        "historico" => (
            "Histórico da família",
            "Créditos, produtos e entregas em um só lugar.",
        ),
        "recibo" => (
            "Comprovante de entrega",
            "Rastreabilidade de cada produto entregue.",
        ),
        _ => (
            "Visão geral",
            "Bem-vindo ao Mercado Solidário. Vamos cuidar de quem precisa.",
        ),
    }
}

#[component]
fn Shell(
    user: User,
    page: String,
    notice: Option<(bool, String)>,
    children: Children,
) -> impl IntoView {
    let (heading, subtitle) = title(&page);
    let nav = vec![
        ("painel", "◫", "Visão geral"),
        ("familias", "♧", "Famílias & créditos"),
        ("estoque", "▦", "Estoque"),
        ("doacoes", "↓", "Doações"),
        ("atendimento", "↗", "Atendimento"),
        ("relatorios", "▤", "Relatórios"),
        ("ciclos", "◷", "Ciclos"),
        ("configuracoes", "⚙", "Equipe & acesso"),
    ];
    let initial = user.name.chars().next().unwrap_or('V').to_string();
    view! {
        <html lang="pt-BR"><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/><meta name="color-scheme" content="light"/><title>{format!("{heading} · Mercado Solidário IBT")}</title><link rel="stylesheet" href="/assets/app.css"/><script src="/assets/app.js" defer></script></head>
        <body><a class="skip" href="#conteudo">"Ir para o conteúdo"</a><div class="app-shell">
            <aside class="sidebar"><a href="/" class="brand"><span class="brand-symbol" aria-hidden="true">"✳"</span><span>"mercado"<strong>"solidário"<small>"IBT · ITABUNA"</small></strong></span></a><div class="sidebar-caption">"ESPAÇO DO VOLUNTÁRIO"</div><nav aria-label="Navegação principal">{nav.into_iter().map(|(id,icon,label)|{let active=page==id;view!{<a href=if id=="painel"{"/".to_string()}else{format!("/{id}")} class=if active{"nav-link active"}else{"nav-link"} aria-current=if active{Some("page")}else{None}><span class="nav-icon" aria-hidden="true">{icon}</span>{label}{active.then(||view!{<span class="active-dot"></span>})}</a>}}).collect_view()}</nav>
                <div class="sidebar-bottom"><div class="mission"><span>"PEQUENOS GESTOS"</span><p>"Juntos, a gente"<br/><strong>"alimenta esperança."</strong></p><span class="mission-flower" aria-hidden="true">"✳"</span></div><div class="profile"><div class="avatar">{initial}</div><div><strong>{user.name}</strong><small>{if user.role=="admin"{"Coordenação"}else{"Voluntário(a)"}}</small></div><form method="post" action="/acoes/logout"><input type="hidden" name="csrf" value=user.csrf/><button class="logout" title="Sair" aria-label="Sair">"↪"</button></form></div></div>
            </aside>
            <div class="workspace"><header class="topbar"><span class="breadcrumb">"Mercado Solidário"<span>" / "</span><strong>{heading}</strong></span><div class="topbar-right"><span class="live-dot"></span>"Sistema conectado"<span class="date-label">{db::today().format("%d.%m.%Y").to_string()}</span></div></header>
                <main id="conteudo"><div class="page-heading"><div><div class="eyebrow">"AÇÃO SOCIAL · IGREJA BATISTA TEOSÓPOLIS"</div><h1>{heading}</h1><p>{subtitle}</p></div><a class="button button-light" href="/atendimento">"Iniciar atendimento ↗"</a></div>
                    {notice.map(|(ok,message)|view!{<div class=if ok{"notice success"}else{"notice danger"} role="status">{message}</div>})}
                    {children()}
                    <footer>"Mercado Solidário IBT"<span>"Feito para servir. Construído em comunidade."</span></footer>
                </main>
            </div></div></body></html>
    }
}
pub fn render(c: &Connection, u: &User, page: &str, q: &HashMap<String, String>) -> R<String> {
    let content = match page {
        "painel" => dashboard(c)?,
        "familias" => families(c, u)?,
        "estoque" => stock(c, u)?,
        "doacoes" => donations(c, u)?,
        "atendimento" => checkout(c, u)?,
        "relatorios" => reports(c, q)?,
        "ciclos" => cycle_page(c, u)?,
        "configuracoes" => settings(c, u)?,
        "historico" => history(c, q)?,
        "recibo" => receipt(c, q)?,
        _ => av(
            view! {<div class="empty"><h2>"Página não encontrada"</h2><a href="/">"Voltar ao painel"</a></div>},
        ),
    };
    let notice = q
        .get("ok")
        .map(|s| (true, s.clone()))
        .or_else(|| q.get("erro").map(|s| (false, s.clone())));
    let u = u.clone();
    let page = page.to_owned();
    Ok(view! {<!DOCTYPE html><Shell user=u page=page notice=notice>{content}</Shell>}.to_html())
}
pub fn login(nonce: &str, error: Option<&str>) -> String {
    let nonce = nonce.to_owned();
    let error = error.map(str::to_owned);
    view!{<!DOCTYPE html><html lang="pt-BR"><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/><title>"Entrar · Mercado Solidário IBT"</title><link rel="stylesheet" href="/assets/app.css"/></head><body class="login-body"><main class="login-layout"><section class="login-story"><a class="brand" href="/login"><span class="brand-symbol">"✳"</span><span>"mercado"<strong>"solidário"<small>"IBT · ITABUNA"</small></strong></span></a><div><div class="eyebrow">"O CUIDADO COMEÇA AQUI"</div><h1>"Generosidade"<br/>"que transforma"<br/><em>"todos os dias."</em></h1><p>"Um espaço para organizar as doações, acolher famílias e fazer o bem chegar mais longe."</p><div class="login-seal">"✳"</div></div><small>"IGREJA BATISTA TEOSÓPOLIS · AÇÃO SOCIAL"</small></section><section class="login-panel"><div class="login-card"><div class="eyebrow">"ESPAÇO DO VOLUNTÁRIO"</div><h2>"Que bom ter você aqui."</h2><p>"Entre com seu acesso para começar."</p>{error.map(|e|view!{<div class="notice danger" role="alert">{e}</div>})}<form method="post" action="/login"><input name="csrf" type="hidden" value=nonce/><label class="field"><span>"E-mail"</span><input type="email" name="email" required autocomplete="username" maxlength="180" placeholder="voce@exemplo.com"/></label><label class="field"><span>"Senha"</span><input type="password" name="password" required autocomplete="current-password" maxlength="200" placeholder="Sua senha de acesso"/></label><button class="button" type="submit">"Entrar no mercado ↗"</button></form><p class="login-help">"Precisa de acesso? Fale com a coordenação do Mercado Solidário."</p><div class="login-security">"◈ Acesso protegido · Dados da comunidade em segurança"</div></div></section></main></body></html>}.to_html()
}
pub fn error_page() -> String {
    view!{<!DOCTYPE html><html lang="pt-BR"><head><meta charset="utf-8"/><title>"Erro"</title></head><body><h1>"Não foi possível carregar a página."</h1><p>"Tente novamente. Os dados já salvos estão preservados."</p><a href="/">"Voltar"</a></body></html>}.to_html()
}

fn dashboard(c: &Connection) -> R<AnyView> {
    let month = db::today().format("%Y-%m").to_string();
    let day = db::today().to_string();
    let registered = scalar(c, "SELECT count(*) FROM families")?;
    let served: i64 = c.query_row(
        "SELECT count(*) FROM checkouts WHERE month=?1",
        [&month],
        |r| r.get(0),
    )?;
    let available: i64 = c.query_row(
        "SELECT COALESCE(sum(remaining),0) FROM lots WHERE expires_on>=?1",
        [&day],
        |r| r.get(0),
    )?;
    let donated:i64=c.query_row("SELECT COALESCE(sum(l.received),0) FROM lots l JOIN donations d ON d.id=l.donation_id WHERE substr(d.received_on,1,7)=?1",[&month],|r|r.get(0))?;
    let stats = vec![
        (
            "FAMÍLIAS CADASTRADAS",
            registered,
            "acolhimento com organização",
            "♧",
        ),
        (
            "ENTREGAS NESTE MÊS",
            served,
            "famílias atendidas com cuidado",
            "↗",
        ),
        (
            "ITENS DISPONÍVEIS",
            available,
            "unidades dentro da validade",
            "▦",
        ),
        (
            "DOAÇÕES NESTE MÊS",
            donated,
            "unidades recebidas pela comunidade",
            "↓",
        ),
    ];
    let pending=db::rows(c,"SELECT cy.start_month,e.group_code,(SELECT count(*) FROM checkouts co WHERE co.enrollment_id=e.id AND co.month=?1) FROM enrollments e JOIN cycles cy ON cy.id=e.cycle_id WHERE e.active=1",&[&month])?.into_iter().filter(|r|db::eligible(&r[0],&r[1],&month)&&r[2]=="0").count();
    let alerts = db::rows(
        c,
        "SELECT p.name,COALESCE(sum(CASE WHEN l.expires_on>=?1 THEN l.remaining ELSE 0 END),0),p.minimum FROM products p LEFT JOIN lots l ON l.product_id=p.id WHERE p.active=1 GROUP BY p.id HAVING COALESCE(sum(CASE WHEN l.expires_on>=?1 THEN l.remaining ELSE 0 END),0)<=p.minimum ORDER BY p.name",
        &[&day],
    )?;
    let expiry = db::rows(
        c,
        "SELECT p.name,l.code,l.remaining,l.expires_on,CASE WHEN l.expires_on<?1 THEN 'Vencido · bloqueado' ELSE 'Validade próxima' END FROM lots l JOIN products p ON p.id=l.product_id WHERE l.remaining>0 AND l.expires_on<=date(?1,'+30 days') ORDER BY l.expires_on LIMIT 50",
        &[&day],
    )?;
    let recent = db::rows(
        c,
        "SELECT co.id,f.name,co.month,co.total_credits FROM checkouts co JOIN enrollments e ON e.id=co.enrollment_id JOIN families f ON f.id=e.family_id ORDER BY co.id DESC LIMIT 6",
        &[],
    )?;
    Ok(av(view! {
        <section class="welcome-banner"><div><span class="eyebrow">"CADA ENTREGA FAZ A DIFERENÇA"</span><h2>"Cuidar fica mais simples"<br/>"quando estamos juntos."</h2><p>{format!("{pending} famílias elegíveis aguardam atendimento neste mês.")}</p><a href="/atendimento" class="button button-white">"Abrir atendimento ↗"</a></div><div class="banner-art" aria-hidden="true"><span class="orbit orbit-one"></span><span class="orbit orbit-two"></span><span class="art-flower">"✳"</span><span class="art-word">"partilhar"</span><span class="art-tag">"CUIDADO QUE CIRCULA"</span></div></section>
        <div class="stat-grid">{stats.into_iter().enumerate().map(|(i,(label,n,sub,icon))|view!{<article class="stat-card"><div class="stat-top"><span>{label}</span><span class=format!("stat-icon tone-{i}")>{icon}</span></div><strong class="stat-number">{n}</strong><p>{sub}</p></article>}).collect_view()}</div>
        <div class="two-columns"><section class="panel"><div class="panel-head"><div><h2>"Atenção ao estoque"</h2><p>"Produtos no limite mínimo de reposição"</p></div><a class="text-link" href="/estoque">"Ver estoque ↗"</a></div>{table(vec!["Produto","Disponível","Mínimo"],alerts)}</section><section class="panel"><div class="panel-head"><div><h2>"Últimas entregas"</h2><p>"O cuidado chegando às famílias"</p></div><a class="text-link" href="/relatorios">"Ver todas ↗"</a></div>{table(vec!["Entrega","Família","Mês","Créditos"],formatted(recent,&[3]))}</section></div>
        {panel("De olho na validade","Lotes que vencem em até 30 dias e lotes vencidos. Produtos vencidos nunca entram no checkout.",table(vec!["Produto","Lote","Unidades","Validade","Situação"],expiry))}
        <div class="community-note"><span>"✳"</span><div><strong>"Mais organização. Mais acolhimento."</strong><p>"Cada entrada e cada entrega ajudam a contar a história de uma comunidade que cuida."</p></div><a href="/doacoes" class="text-link">"Registrar uma doação ↗"</a></div>
    }))
}
fn families(c: &Connection, u: &User) -> R<AnyView> {
    let rows = db::rows(
        c,
        "SELECT f.id,f.reference,f.name,f.members,COALESCE(cy.name,'Sem ciclo'),COALESCE(e.id,''),COALESCE(e.group_code,''),COALESCE(e.monthly_credit,0),COALESCE(cy.start_month,''),COALESCE(e.active,0),COALESCE((SELECT sum(total_credits) FROM checkouts co WHERE co.enrollment_id=e.id AND co.month=?1),0) FROM families f LEFT JOIN enrollments e ON e.family_id=f.id LEFT JOIN cycles cy ON cy.id=e.cycle_id ORDER BY f.name,cy.start_month DESC",
        &[&db::today().format("%Y-%m").to_string()],
    )?;
    let month = db::today().format("%Y-%m").to_string();
    let view = av(
        view! {<div class="table-tools"><label>"Buscar família"<input type="search" data-table-search placeholder="Nome ou referência…" aria-label="Buscar família"/></label><span>{format!("{} inscrições na lista",rows.len())}</span></div><div class="table-wrap"><table data-searchable><thead><tr><th>"Referência"</th><th>"Família"</th><th>"Ciclo / grupo"</th><th>"Crédito mensal"</th><th>"Saldo do mês"</th><th>"Situação"</th><th>"Histórico"</th></tr></thead><tbody>{rows.into_iter().map(|r|{
            let eligible=r[9]=="1"&&db::eligible(&r[8],&r[6],&month);let used=r[10].parse::<i64>().unwrap_or(0);let credit=r[7].parse::<i64>().unwrap_or(0);
            let status=if r[9]!="1"{"Inativa"}else if !eligible{"Fora da vez"}else if used>0{"Atendida"}else{"Pode retirar"};
            view!{<tr><td><span class="reference">{r[1].clone()}</span></td><td><strong>{r[2].clone()}</strong><small class="cell-sub">{format!("{} integrantes",r[3])}</small></td><td>{format!("{} · {}",r[4],r[6])}</td><td>{db::display_money(credit)}</td><td>{if eligible{db::display_money(credit-used)}else{"—".into()}}</td><td><span class=if eligible&&used==0{"badge green"}else{"badge"}>{status}</span></td><td><a class="text-link" href=format!("/historico?family={}",r[0])>"Consultar ↗"</a></td></tr>}
        }).collect_view()}</tbody></table></div>},
    );
    let new = form(
        "family",
        &u.csrf,
        vec![
            field("name", "Responsável pela família", "text", ""),
            field("reference", "Referência única (ex.: IBT-001)", "text", ""),
            field("members", "Número de integrantes", "number", "1"),
            select("cycle_id", "Ciclo de atendimento", cycles(c)?),
            select(
                "group_code",
                "Grupo",
                vec![
                    ("A".into(), "A · primeiro mês e alternados".into()),
                    ("B".into(), "B · segundo mês e alternados".into()),
                ],
            ),
            credit("credit", "Créditos por mês de atendimento", "100,00"),
            memo(
                "criteria",
                "Critérios de concessão do crédito",
                "Registre os critérios aprovados pela coordenação.",
            ),
        ],
        "Cadastrar família",
    );
    let renew = form(
        "enroll",
        &u.csrf,
        vec![
            select(
                "family_id",
                "Família existente",
                opts(
                    c,
                    "SELECT id,reference||' · '||name FROM families ORDER BY name",
                )?,
            ),
            select("cycle_id", "Novo ciclo", cycles(c)?),
            select(
                "group_code",
                "Grupo",
                vec![("A".into(), "A".into()), ("B".into(), "B".into())],
            ),
            credit("credit", "Créditos mensais", "100,00"),
            memo(
                "criteria",
                "Critérios",
                "Critérios de concessão no novo ciclo.",
            ),
        ],
        "Inscrever no ciclo",
    );
    let admin = if u.role == "admin" {
        let enrollment_opts = opts(
            c,
            "SELECT e.id,f.reference||' · '||f.name||' / '||cy.name FROM enrollments e JOIN families f ON f.id=e.family_id JOIN cycles cy ON cy.id=e.cycle_id ORDER BY cy.start_month DESC,f.name",
        )?;
        av(view! {
            {disclosure("Ajustar crédito ou suspender uma inscrição",form("enrollment",&u.csrf,vec![select("enrollment_id","Inscrição",enrollment_opts),credit("credit","Novo crédito mensal","100,00"),select("active","Situação",vec![("1".into(),"Ativa".into()),("0".into(),"Suspensa".into())]),memo("criteria","Critérios / justificativa","Justifique a alteração.")],"Salvar inscrição"))}
            {disclosure("Importar famílias de uma planilha (CSV)",form("import",&u.csrf,vec![select("cycle_id","Ciclo de destino",cycles(c)?),memo("csv","Cole o CSV separado por ponto e vírgula","nome;referencia;membros;grupo;credito;criterios\nFamília Exemplo;IBT-001;4;A;100,00;Critério aprovado")],"Validar e importar"))}
        })
    } else {
        av(())
    };
    Ok(av(
        view! {{panel("Cadastro rápido","Use uma referência interna única. Evite registrar documentos pessoais desnecessários.",new)}{panel("Famílias cadastradas","O saldo é mensal e disponível apenas nos meses do grupo. Uma retirada por família por mês.",view)}{disclosure("Inscrever uma família em um novo ciclo",renew)}{admin}},
    ))
}
fn stock(c: &Connection, u: &User) -> R<AnyView> {
    let day = db::today().to_string();
    let rows = db::rows(
        c,
        "SELECT p.id,p.name,p.unit,COALESCE((SELECT pr.credits FROM prices pr WHERE pr.product_id=p.id AND pr.effective_date<=?1 ORDER BY pr.effective_date DESC LIMIT 1),0),COALESCE((SELECT sum(remaining) FROM lots l WHERE l.product_id=p.id AND l.expires_on>=?1),0),p.minimum,COALESCE((SELECT sum(remaining) FROM lots l WHERE l.product_id=p.id AND l.expires_on<?1),0) FROM products p ORDER BY p.name",
        &[&day],
    )?;
    let lots = db::rows(
        c,
        "SELECT l.id,p.name,l.code,l.expires_on,l.received,l.remaining,l.value_cents,d.donor FROM lots l JOIN products p ON p.id=l.product_id JOIN donations d ON d.id=l.donation_id WHERE l.remaining>0 ORDER BY l.expires_on,l.id",
        &[],
    )?;
    let new = form(
        "product",
        &u.csrf,
        vec![
            field("name", "Nome do produto", "text", ""),
            field("unit", "Unidade / embalagem", "text", "unidade"),
            field("minimum", "Estoque mínimo (unidades)", "number", "5"),
            credit("price", "Preço em créditos por unidade", "5,00"),
        ],
        "Cadastrar produto",
    );
    let admin = if u.role == "admin" {
        av(view! {
            {disclosure("Agendar novo preço em créditos",form("price",&u.csrf,vec![select("product_id","Produto",products(c)?),credit("price","Novo preço em créditos","5,00"),field("effective_date","Início da vigência","date",&day)],"Registrar vigência"))}
            {disclosure("Registrar descarte ou perda",form("discard",&u.csrf,vec![select("lot_id","Lote",opts(c,"SELECT l.id,p.name||' · lote '||l.code||' · saldo '||l.remaining FROM lots l JOIN products p ON p.id=l.product_id WHERE l.remaining>0 ORDER BY l.expires_on")?),field("quantity","Quantidade descartada","number","1"),memo("reason","Motivo do descarte","Ex.: vencimento, embalagem danificada ou perda registrada.")],"Registrar descarte"))}
        })
    } else {
        av(())
    };
    let prices = db::rows(
        c,
        "SELECT p.name,pr.effective_date,pr.credits FROM prices pr JOIN products p ON p.id=pr.product_id ORDER BY pr.effective_date DESC,p.name LIMIT 200",
        &[],
    )?;
    Ok(av(view! {
        {panel("Produtos & disponibilidade","O estoque disponível exclui lotes vencidos. Créditos são valores sociais, separados do valor das doações.",table(vec!["Código","Produto","Embalagem","Créditos / un.","Disponível","Mínimo","Vencidos"],formatted(rows,&[3])))}
        {disclosure("Cadastrar um novo produto",new)}
        {panel("Lotes em estoque","A saída prioriza o lote com validade mais próxima (FEFO). O valor informado é o valor total do lote em reais.",table(vec!["ID","Produto","Lote","Validade","Recebido","Saldo","Valor total R$","Doador"],formatted(lots,&[6])))}
        {admin}{disclosure("Histórico de preços · até 200 vigências",table(vec!["Produto","Vigência","Créditos / unidade"],formatted(prices,&[2])))}
    }))
}
fn donations(c: &Connection, u: &User) -> R<AnyView> {
    let day = db::today().to_string();
    let new = form(
        "donation",
        &u.csrf,
        vec![
            select("product_id", "Produto recebido", products(c)?),
            field("donor", "Doador / origem", "text", "Doação anônima"),
            field("quantity", "Quantidade de embalagens", "number", "1"),
            credit("value", "Valor total do lote (R$)", "0,00"),
            field("code", "Identificação do lote", "text", ""),
            field("received_on", "Data de recebimento", "date", &day),
            field("expires_on", "Data de validade", "date", ""),
        ],
        "Registrar doação",
    );
    let rows = db::rows(
        c,
        "SELECT d.id,d.received_on,d.donor,p.name,l.code,l.received,l.expires_on,l.value_cents FROM donations d JOIN lots l ON l.donation_id=d.id JOIN products p ON p.id=l.product_id ORDER BY d.id DESC LIMIT 200",
        &[],
    )?;
    Ok(av(
        view! {<div class="info-strip"><span>"↓"</span><p>"Cada doação conta. Registre um produto e um lote por lançamento para manter a validade e a origem rastreáveis."</p></div>{panel("Nova doação","Cadastre primeiro o produto em Estoque. Quantidades são sempre unidades inteiras da embalagem cadastrada.",new)}{panel("Doações recebidas","Últimos 200 lançamentos, com o valor total estimado de cada lote.",table(vec!["Entrada","Recebimento","Origem","Produto","Lote","Unidades","Validade","Valor R$"],formatted(rows,&[7])))}},
    ))
}
fn checkout(c: &Connection, u: &User) -> R<AnyView> {
    let month = db::today().format("%Y-%m").to_string();
    let day = db::today().to_string();
    let families=db::rows(c,"SELECT e.id,f.reference,f.name,e.group_code,e.monthly_credit,cy.start_month FROM enrollments e JOIN families f ON f.id=e.family_id JOIN cycles cy ON cy.id=e.cycle_id WHERE e.active=1 AND NOT EXISTS(SELECT 1 FROM checkouts co WHERE co.enrollment_id=e.id AND co.month=?1) ORDER BY f.name",&[&month])?.into_iter().filter(|r|db::eligible(&r[5],&r[3],&month)).collect::<Vec<_>>();
    let available = db::rows(
        c,
        "SELECT p.id,p.name,p.unit,pr.credits,COALESCE((SELECT sum(remaining) FROM lots l WHERE l.product_id=p.id AND l.expires_on>=?1),0) FROM products p JOIN prices pr ON pr.id=(SELECT id FROM prices WHERE product_id=p.id AND effective_date<=?1 ORDER BY effective_date DESC LIMIT 1) WHERE p.active=1 ORDER BY p.name",
        &[&day],
    )?;
    let csrf = u.csrf.clone();
    let request = crate::random();
    let ready = !families.is_empty() && available.iter().any(|r| r[4] != "0");
    let count = families.len();
    let cart = av(
        view! {<form method="post" action="/acoes/checkout" id="checkout-form"><input type="hidden" name="csrf" value=csrf/><input type="hidden" name="request_key" value=request/>
            <div class="checkout-family"><label class="field"><span>"1. Selecione a família"</span><select name="enrollment_id" id="checkout-family" required><option value="" disabled selected>"Selecione uma família elegível…"</option>{families.into_iter().map(|r|view!{<option value=r[0].clone() data-credit=r[4].clone()>{format!("{} · {} · grupo {} · {} créditos",r[1],r[2],r[3],db::display_money(r[4].parse().unwrap_or(0)))}</option>}).collect_view()}</select></label><p class="form-note">{format!("{count} famílias disponíveis em {month}. Famílias atendidas ou fora da vez não aparecem nesta lista.")}</p></div>
            <div class="table-wrap"><table><thead><tr><th>"2. Separe os produtos"</th><th>"Créditos / un."</th><th>"Disponível"</th><th>"Quantidade"</th></tr></thead><tbody>{available.into_iter().map(|r|{let stock=r[4].clone();let empty=stock=="0";view!{<tr><td><strong>{r[1].clone()}</strong><small class="cell-sub">{r[2].clone()}</small></td><td>{db::display_money(r[3].parse().unwrap_or(0))}</td><td><span class=if empty{"badge"}else{"badge green"}>{stock.clone()}</span></td><td><input class="qty" type="number" name=format!("qty_{}",r[0]) aria-label=format!("Quantidade de {}",r[1]) min="0" max=stock step="1" value="0" data-price=r[3].clone() disabled=empty/></td></tr>}}).collect_view()}</tbody></table></div>
            <div class="checkout-total"><div><span>"TOTAL DA ENTREGA"</span><strong id="cart-total" aria-live="polite">"0,00 créditos"</strong><small id="cart-balance">"Escolha uma família para ver o saldo."</small></div><button class="button" type="submit" disabled=!ready data-confirm="Confirma a entrega? Estoque e créditos serão debitados juntos e a família ficará marcada como atendida neste mês.">"Confirmar entrega ↗"</button></div>
        </form>},
    );
    let recent = db::rows(
        c,
        "SELECT co.id,f.name,co.total_credits,co.created_at FROM checkouts co JOIN enrollments e ON e.id=co.enrollment_id JOIN families f ON f.id=e.family_id WHERE co.month=?1 ORDER BY co.id DESC",
        &[&month],
    )?;
    let deliveries = av(
        view! {<div class="table-wrap"><table><thead><tr><th>"Entrega"</th><th>"Família"</th><th>"Créditos"</th><th>"Comprovante"</th></tr></thead><tbody>{recent.into_iter().map(|r|view!{<tr><td>{format!("#{}",r[0])}</td><td>{r[1].clone()}</td><td>{db::display_money(r[2].parse().unwrap_or(0))}</td><td><a class="text-link" href=format!("/recibo?id={}",r[0])>"Abrir comprovante ↗"</a></td></tr>}).collect_view()}</tbody></table></div>},
    );
    Ok(av(
        view! {<div class="steps"><span><b>"01"</b>" Identifique a família"</span><span><b>"02"</b>" Separe os produtos"</span><span><b>"03"</b>" Confirme a entrega"</span></div>{panel("Nova entrega","Confira os produtos antes de confirmar. Produtos vencidos e saldos insuficientes são bloqueados automaticamente.",cart)}{panel("Entregas do mês","Uma retirada por família no mês de atendimento. Horários detalhados no comprovante.",deliveries)}},
    ))
}
fn reports(c: &Connection, q: &HashMap<String, String>) -> R<AnyView> {
    let month = q
        .get("month")
        .filter(|m| db::month_index(m).is_ok())
        .cloned()
        .unwrap_or_else(|| db::today().format("%Y-%m").to_string());
    let incoming = db::rows(
        c,
        "SELECT p.name,COALESCE((SELECT sum(l.received) FROM lots l JOIN donations d ON d.id=l.donation_id WHERE l.product_id=p.id AND substr(d.received_on,1,7)=?1),0),COALESCE((SELECT sum(i.quantity) FROM checkout_items i JOIN checkouts co ON co.id=i.checkout_id WHERE i.product_id=p.id AND co.month=?1),0),COALESCE((SELECT -sum(sm.quantity) FROM stock_movements sm JOIN lots l ON l.id=sm.lot_id WHERE l.product_id=p.id AND sm.kind='discard' AND strftime('%Y-%m',sm.created_at,'-3 hours')=?1),0),COALESCE((SELECT sum(l.remaining) FROM lots l WHERE l.product_id=p.id),0) FROM products p ORDER BY p.name",
        &[&month],
    )?;
    let families = db::rows(
        c,
        "SELECT f.reference,f.name,co.id,co.month,co.total_credits,u.name FROM checkouts co JOIN enrollments e ON e.id=co.enrollment_id JOIN families f ON f.id=e.family_id JOIN users u ON u.id=co.user_id WHERE co.month=?1 ORDER BY f.name",
        &[&month],
    )?;
    let values:(i64,i64)=c.query_row("SELECT COALESCE((SELECT sum(l.value_cents) FROM lots l JOIN donations d ON d.id=l.donation_id WHERE substr(d.received_on,1,7)=?1),0),COALESCE((SELECT sum(total_credits) FROM checkouts WHERE month=?1),0)",[&month],|r|Ok((r.get(0)?,r.get(1)?)))?;
    let reconciliation = db::rows(
        c,
        "SELECT p.name,l.code,l.remaining,COALESCE(sum(sm.quantity),0),l.remaining-COALESCE(sum(sm.quantity),0) FROM lots l JOIN products p ON p.id=l.product_id LEFT JOIN stock_movements sm ON sm.lot_id=l.id GROUP BY l.id HAVING l.remaining<>COALESCE(sum(sm.quantity),0)",
        &[],
    )?;
    let integrity = reconciliation.is_empty();
    let family_count = families.len();
    let month_clone = month.clone();
    Ok(av(view! {
        <div class="report-controls"><form method="get" action="/relatorios"><label class="field"><span>"Mês de referência"</span><input type="month" name="month" value=month_clone required/></label><button class="button button-light" type="submit">"Aplicar mês"</button></form><a class="button" href=format!("/exportar?month={month}")>"Exportar entregas CSV ↓"</a></div>
        <div class="stat-grid report-stats"><article class="stat-card"><span class="eyebrow">"DOAÇÕES RECEBIDAS · VALOR ESTIMADO"</span><strong class="stat-number">{format!("R$ {}",db::display_money(values.0))}</strong><p>"Valor dos lotes recebidos no mês"</p></article><article class="stat-card"><span class="eyebrow">"CRÉDITOS UTILIZADOS"</span><strong class="stat-number">{db::display_money(values.1)}</strong><p>"Valor social das entregas no mês"</p></article><article class="stat-card"><span class="eyebrow">"FAMÍLIAS ATENDIDAS"</span><strong class="stat-number">{family_count}</strong><p>"Retiradas concluídas no mês"</p></article></div>
        {panel("Balanço de produtos","Entradas pelo recebimento; saídas e descartes do mês selecionado. O saldo físico inclui vencidos e representa a posição atual.",table(vec!["Produto","Recebido no mês","Entregue no mês","Descartado no mês","Saldo físico atual"],incoming))}
        {panel("Atendimento por família","Créditos utilizados no mês selecionado e responsável pela entrega.",table(vec!["Referência","Família","Entrega","Mês","Créditos","Voluntário"],formatted(families,&[4])))}
        <div class=if integrity{"notice success"}else{"notice danger"}>{if integrity{"Conferência dos registros: todos os saldos dos lotes correspondem à soma dos movimentos. Faça também a contagem física para verificar a posição real."}else{"Atenção: há divergência entre os saldos e os movimentos registrados."}}</div>
        {(!integrity).then(||panel("Divergências de registros","Confira a trilha de auditoria antes de ajustar o estoque.",table(vec!["Produto","Lote","Saldo","Movimentos","Diferença"],reconciliation)))}
    }))
}
fn cycle_page(c: &Connection, u: &User) -> R<AnyView> {
    let rows = db::rows(
        c,
        "SELECT cy.name,cy.start_month,strftime('%Y-%m',cy.start_month||'-01','+5 months'),(SELECT count(*) FROM enrollments e WHERE e.cycle_id=cy.id AND e.group_code='A'),(SELECT count(*) FROM enrollments e WHERE e.cycle_id=cy.id AND e.group_code='B'),cy.capacity FROM cycles cy ORDER BY cy.start_month DESC",
        &[],
    )?;
    let new = if u.role == "admin" {
        panel(
            "Novo ciclo semestral",
            "O grupo A é atendido no primeiro, terceiro e quinto meses; o grupo B, no segundo, quarto e sexto. Limite de 50 famílias por grupo.",
            form(
                "cycle",
                &u.csrf,
                vec![
                    field("name", "Nome do ciclo", "text", ""),
                    field(
                        "start_month",
                        "Primeiro mês",
                        "month",
                        db::today().format("%Y-%m").to_string(),
                    ),
                ],
                "Criar ciclo",
            ),
        )
    } else {
        av(())
    };
    Ok(av(
        view! {<div class="info-strip"><span>"◷"</span><p>"O crédito é uma cota mensal para os meses de atendimento do grupo. Não acumula de um mês para o outro. A regra pode ser ajustada no código se a coordenação adotar outro modelo."</p></div>{panel("Ciclos cadastrados","Os ciclos duram seis meses e não podem se sobrepor.",table(vec!["Ciclo","Primeiro mês","Último mês","Grupo A / 50","Grupo B / 50","Capacidade"],rows))}{new}},
    ))
}
fn settings(c: &Connection, u: &User) -> R<AnyView> {
    let password = form(
        "password",
        &u.csrf,
        vec![
            field("old_password", "Senha atual", "password", ""),
            field(
                "password",
                "Nova senha (mínimo 12 caracteres)",
                "password",
                "",
            ),
        ],
        "Alterar minha senha",
    );
    let admin = if u.role == "admin" {
        let users = db::rows(
            c,
            "SELECT name,email,CASE role WHEN 'admin' THEN 'Administrador' ELSE 'Voluntário' END FROM users ORDER BY name",
            &[],
        )?;
        let log = db::rows(
            c,
            "SELECT datetime(a.created_at,'-3 hours'),u.name,a.action,a.entity,a.entity_id,a.detail FROM audit_log a JOIN users u ON u.id=a.user_id ORDER BY a.id DESC LIMIT 100",
            &[],
        )?;
        av(view! {
            {panel("Pessoas da equipe","Administradores gerenciam ciclos, créditos, preços, descartes e acessos. Voluntários cadastram famílias, produtos, doações e entregas.",table(vec!["Nome","E-mail","Perfil"],users))}
            {disclosure("Criar um acesso para a equipe",form("user",&u.csrf,vec![field("name","Nome","text",""),field("email","E-mail","email",""),field("password","Senha inicial (mínimo 12 caracteres)","password",""),select("role","Perfil",vec![("volunteer".into(),"Voluntário".into()),("admin".into(),"Administrador".into())])],"Criar acesso"))}
            {disclosure("Trilha de auditoria · últimos 100 eventos",table(vec!["Data (Brasília)","Responsável","Ação","Registro","ID","Detalhes"],log))}
        })
    } else {
        av(())
    };
    Ok(av(
        view! {{admin}{panel("Minha senha","Ao alterar a senha, suas sessões serão encerradas e será necessário entrar novamente.",password)}{panel("Encerrar acesso","Saia ao terminar, especialmente em computadores e celulares compartilhados.",form("logout",&u.csrf,vec![],"Sair da minha conta"))}},
    ))
}
fn history(c: &Connection, q: &HashMap<String, String>) -> R<AnyView> {
    let id = q
        .get("family")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let family = db::rows(
        c,
        "SELECT reference,name,members FROM families WHERE id=?1",
        &[&id],
    )?;
    let rows = db::rows(
        c,
        "SELECT cy.name,e.group_code,co.id,co.month,p.name,i.quantity,i.unit_credits,i.quantity*i.unit_credits FROM checkouts co JOIN enrollments e ON e.id=co.enrollment_id JOIN cycles cy ON cy.id=e.cycle_id JOIN checkout_items i ON i.checkout_id=co.id JOIN products p ON p.id=i.product_id WHERE e.family_id=?1 ORDER BY co.id DESC,p.name",
        &[&id],
    )?;
    Ok(av(
        view! {{panel("Cadastro da família","Dados de referência da família.",table(vec!["Referência","Responsável","Integrantes"],family))}{panel("Histórico de consumo","Entregas de todos os ciclos, com o preço praticado na data do atendimento.",table(vec!["Ciclo","Grupo","Entrega","Mês","Produto","Quantidade","Créditos / un.","Total"],formatted(rows,&[6,7])))}<a class="text-link" href="/familias">"← Voltar às famílias"</a>},
    ))
}
fn receipt(c: &Connection, q: &HashMap<String, String>) -> R<AnyView> {
    let id = q.get("id").and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
    let rows = db::rows(
        c,
        "SELECT co.id,f.reference,f.name,co.month,datetime(co.created_at,'-3 hours'),u.name,co.total_credits FROM checkouts co JOIN enrollments e ON e.id=co.enrollment_id JOIN families f ON f.id=e.family_id JOIN users u ON u.id=co.user_id WHERE co.id=?1",
        &[&id],
    )?;
    let items = db::rows(
        c,
        "SELECT p.name,i.quantity,i.unit_credits,i.quantity*i.unit_credits FROM checkout_items i JOIN products p ON p.id=i.product_id WHERE i.checkout_id=?1",
        &[&id],
    )?;
    let lots = db::rows(
        c,
        "SELECT p.name,l.code,l.expires_on,a.quantity FROM allocations a JOIN checkout_items i ON i.id=a.checkout_item_id JOIN lots l ON l.id=a.lot_id JOIN products p ON p.id=i.product_id WHERE i.checkout_id=?1 ORDER BY p.name,l.expires_on",
        &[&id],
    )?;
    Ok(av(
        view! {{panel("Entrega concluída","Horário de Brasília. Este comprovante confirma a baixa do estoque e o débito dos créditos.",table(vec!["Entrega","Referência","Família","Mês","Data","Responsável","Créditos"],formatted(rows,&[6])))}{panel("Produtos entregues","Preços registrados no momento da entrega.",table(vec!["Produto","Quantidade","Créditos / un.","Total"],formatted(items,&[2,3])))}{disclosure("Lotes utilizados nesta entrega",table(vec!["Produto","Lote","Validade","Quantidade"],lots))}<button class="button button-light" type="button" data-print>"Imprimir comprovante"</button>},
    ))
}
