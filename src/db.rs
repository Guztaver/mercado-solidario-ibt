use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::{BTreeMap, HashMap};

pub type Form = HashMap<String, String>;
pub type Result<T> = std::result::Result<T, String>;

pub fn today() -> NaiveDate {
    chrono::Utc::now()
        .with_timezone(&chrono_tz::America::Sao_Paulo)
        .date_naive()
}
pub fn init(c: &Connection) -> rusqlite::Result<()> {
    c.busy_timeout(std::time::Duration::from_secs(10))?;
    c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;")?;
    c.execute_batch(include_str!("schema.sql"))?;
    let has_barcode: bool = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM pragma_table_info('products') WHERE name='barcode')",
        [],
        |row| row.get(0),
    )?;
    if !has_barcode {
        c.execute("ALTER TABLE products ADD COLUMN barcode TEXT", [])?;
    }
    c.execute_batch(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_products_barcode ON products(barcode) WHERE barcode IS NOT NULL AND barcode<>'';
         INSERT OR IGNORE INTO schema_version(version) VALUES(2);",
    )
}
pub fn text(f: &Form, key: &str, max: usize) -> Result<String> {
    let v = f.get(key).map(|s| s.trim()).unwrap_or("");
    if v.is_empty()
        || v.chars().count() > max
        || v.chars().any(|x| x.is_control() && x != '\n' && x != '\r')
    {
        return Err(format!(
            "Preencha corretamente o campo {key} (até {max} caracteres)."
        ));
    }
    Ok(v.to_owned())
}
pub fn num(f: &Form, key: &str, min: i64, max: i64) -> Result<i64> {
    let n = text(f, key, 24)?
        .parse::<i64>()
        .map_err(|_| format!("{key}: informe um número inteiro."))?;
    if !(min..=max).contains(&n) {
        return Err(format!("{key}: valor fora do limite ({min} a {max})."));
    }
    Ok(n)
}
pub fn money(s: &str) -> Result<i64> {
    let s = s.trim().replace(',', ".");
    let parts: Vec<_> = s.split('.').collect();
    if parts.len() > 2 || parts[0].is_empty() || !parts[0].chars().all(|c| c.is_ascii_digit()) {
        return Err("Informe um valor positivo, por exemplo 12,50.".into());
    }
    let fraction = parts.get(1).copied().unwrap_or("");
    if fraction.len() > 2 || !fraction.chars().all(|c| c.is_ascii_digit()) {
        return Err("Use no máximo duas casas decimais.".into());
    }
    let whole = parts[0].parse::<i64>().map_err(|_| "Valor inválido.")?;
    if whole > 1_000_000 {
        return Err("Valor acima do limite.".into());
    }
    let frac = format!("{fraction:0<2}")
        .parse::<i64>()
        .map_err(|_| "Valor inválido.")?;
    Ok(whole * 100 + frac)
}
pub fn amount(f: &Form, key: &str) -> Result<i64> {
    money(&text(f, key, 24)?)
}
pub fn date(f: &Form, key: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(&text(f, key, 10)?, "%Y-%m-%d").map_err(|_| "Data inválida.".into())
}
pub fn month_index(m: &str) -> Result<i32> {
    let d = NaiveDate::parse_from_str(&format!("{m}-01"), "%Y-%m-%d")
        .map_err(|_| "Mês inválido.".to_string())?;
    Ok(d.year() * 12 + d.month0() as i32)
}
pub fn eligible(start: &str, group: &str, month: &str) -> bool {
    let Ok(s) = month_index(start) else {
        return false;
    };
    let Ok(m) = month_index(month) else {
        return false;
    };
    let offset = m - s;
    (0..6).contains(&offset) && offset % 2 == if group == "A" { 0 } else { 1 }
}
pub fn rows(
    c: &Connection,
    sql: &str,
    args: &[&dyn rusqlite::ToSql],
) -> rusqlite::Result<Vec<Vec<String>>> {
    let mut stmt = c.prepare(sql)?;
    let n = stmt.column_count();
    stmt.query_map(args, |r| {
        (0..n)
            .map(|i| {
                Ok(match r.get_ref(i)? {
                    rusqlite::types::ValueRef::Null => String::new(),
                    rusqlite::types::ValueRef::Integer(v) => v.to_string(),
                    rusqlite::types::ValueRef::Real(v) => v.to_string(),
                    rusqlite::types::ValueRef::Text(v) => String::from_utf8_lossy(v).into_owned(),
                    _ => String::new(),
                })
            })
            .collect()
    })?
    .collect()
}
pub fn audit(
    c: &Connection,
    user: i64,
    action: &str,
    entity: &str,
    id: i64,
    detail: &str,
) -> rusqlite::Result<()> {
    c.execute(
        "INSERT INTO audit_log(user_id,action,entity,entity_id,detail) VALUES(?1,?2,?3,?4,?5)",
        params![user, action, entity, id, detail],
    )?;
    Ok(())
}
pub fn apply(c: &mut Connection, user: i64, role: &str, action: &str, f: &Form) -> Result<String> {
    let tx = c
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(db_err)?;
    let message = match action {
        "cycle" => {
            admin(role)?;
            let name = text(f, "name", 100)?;
            let start = text(f, "start_month", 7)?;
            let index = month_index(&start)?;
            let overlaps:i64=tx.query_row("SELECT count(*) FROM cycles WHERE abs((CAST(substr(start_month,1,4) AS INTEGER)*12+CAST(substr(start_month,6,2) AS INTEGER)-1)-?1)<6",[index],|r|r.get(0)).map_err(db_err)?;
            if overlaps > 0 {
                return Err("O ciclo se sobrepõe a outro ciclo de seis meses.".into());
            }
            tx.execute(
                "INSERT INTO cycles(name,start_month,capacity) VALUES(?1,?2,100)",
                params![name, start],
            )
            .map_err(db_err)?;
            audit(&tx, user, "create", "cycle", tx.last_insert_rowid(), &name).map_err(db_err)?;
            "Ciclo criado. Grupo A no primeiro mês; grupo B no segundo.".into()
        }
        "family" => {
            insert_family(&tx, user, f)?;
            "Família cadastrada e vinculada ao ciclo.".into()
        }
        "enroll" => {
            let family = num(f, "family_id", 1, i64::MAX)?;
            enroll(&tx, user, family, f)?;
            "Família inscrita no ciclo.".into()
        }
        "enrollment" => {
            admin(role)?;
            let id = num(f, "enrollment_id", 1, i64::MAX)?;
            let credit = amount(f, "credit")?;
            if credit <= 0 {
                return Err("Crédito deve ser positivo.".into());
            }
            let criteria = text(f, "criteria", 500)?;
            let active = num(f, "active", 0, 1)?;
            let used: i64 = tx
                .query_row(
                    "SELECT COALESCE(max(total_credits),0) FROM checkouts WHERE enrollment_id=?1",
                    [id],
                    |r| r.get(0),
                )
                .map_err(db_err)?;
            if credit < used {
                return Err("O crédito não pode ser menor que uma retirada já realizada.".into());
            }
            if tx
                .execute(
                    "UPDATE enrollments SET monthly_credit=?1,criteria=?2,active=?3 WHERE id=?4",
                    params![credit, criteria, active, id],
                )
                .map_err(db_err)?
                != 1
            {
                return Err("Inscrição não encontrada.".into());
            }
            audit(
                &tx,
                user,
                "update",
                "enrollment",
                id,
                &format!("Crédito={credit}; ativo={active}; {criteria}"),
            )
            .map_err(db_err)?;
            "Inscrição atualizada.".into()
        }
        "product" => {
            let name = text(f, "name", 100)?;
            let unit = text(f, "unit", 30)?;
            let barcode = f.get("barcode").map(|value| value.trim()).unwrap_or("");
            if !barcode.is_empty()
                && (!(8..=14).contains(&barcode.len())
                    || !barcode.bytes().all(|byte| byte.is_ascii_digit()))
            {
                return Err("O código de barras deve conter de 8 a 14 números.".into());
            }
            if !barcode.is_empty()
                && tx
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM products WHERE barcode=?1)",
                        [barcode],
                        |row| row.get::<_, bool>(0),
                    )
                    .map_err(db_err)?
            {
                return Err("Este código de barras já está cadastrado em outro produto.".into());
            }
            let minimum = num(f, "minimum", 0, 1000000)?;
            let price = amount(f, "price")?;
            if price <= 0 {
                return Err("O preço em créditos deve ser positivo.".into());
            }
            tx.execute(
                "INSERT INTO products(name,barcode,unit,minimum) VALUES(?1,?2,?3,?4)",
                params![name, if barcode.is_empty() { None } else { Some(barcode) }, unit, minimum],
            )
            .map_err(db_err)?;
            let id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO prices(product_id,effective_date,credits) VALUES(?1,?2,?3)",
                params![id, today().to_string(), price],
            )
            .map_err(db_err)?;
            audit(&tx, user, "create", "product", id, &name).map_err(db_err)?;
            "Produto cadastrado.".into()
        }
        "price" => {
            admin(role)?;
            let id = num(f, "product_id", 1, i64::MAX)?;
            let price = amount(f, "price")?;
            let effective = date(f, "effective_date")?;
            if price <= 0 || effective < today() {
                return Err("Use preço positivo e vigência de hoje em diante.".into());
            }
            tx.execute(
                "INSERT INTO prices(product_id,effective_date,credits) VALUES(?1,?2,?3)",
                params![id, effective.to_string(), price],
            )
            .map_err(db_err)?;
            audit(
                &tx,
                user,
                "create",
                "price",
                tx.last_insert_rowid(),
                &format!("produto={id}; créditos={price}; data={effective}"),
            )
            .map_err(db_err)?;
            "Nova vigência registrada. O histórico foi preservado.".into()
        }
        "donation" => {
            let id = num(f, "product_id", 1, i64::MAX)?;
            let qty = num(f, "quantity", 1, 1000000)?;
            let received = date(f, "received_on")?;
            let expires = date(f, "expires_on")?;
            if received > today() || expires < received || expires < today() {
                return Err("A entrada não pode ser futura e a validade deve ser igual ou posterior à entrada e a hoje.".into());
            }
            let donor = text(f, "donor", 120)?;
            let code = text(f, "code", 80)?;
            let value = amount(f, "value")?;
            tx.execute(
                "INSERT INTO donations(donor,received_on,user_id) VALUES(?1,?2,?3)",
                params![donor, received.to_string(), user],
            )
            .map_err(db_err)?;
            let donation = tx.last_insert_rowid();
            tx.execute("INSERT INTO lots(donation_id,product_id,code,expires_on,received,remaining,value_cents) VALUES(?1,?2,?3,?4,?5,?5,?6)",params![donation,id,code,expires.to_string(),qty,value]).map_err(db_err)?;
            let lot = tx.last_insert_rowid();
            movement(&tx, lot, None, "donation", qty, "Doação recebida", user)?;
            audit(
                &tx,
                user,
                "create",
                "donation",
                donation,
                &format!("produto={id}; quantidade={qty}"),
            )
            .map_err(db_err)?;
            "Doação registrada e estoque atualizado.".into()
        }
        "discard" => {
            admin(role)?;
            let lot = num(f, "lot_id", 1, i64::MAX)?;
            let qty = num(f, "quantity", 1, 1000000)?;
            let reason = text(f, "reason", 300)?;
            if tx
                .execute(
                    "UPDATE lots SET remaining=remaining-?1 WHERE id=?2 AND remaining>=?1",
                    params![qty, lot],
                )
                .map_err(db_err)?
                != 1
            {
                return Err("Saldo do lote insuficiente.".into());
            }
            movement(&tx, lot, None, "discard", -qty, &reason, user)?;
            audit(&tx, user, "discard", "lot", lot, &reason).map_err(db_err)?;
            "Descarte registrado no histórico.".into()
        }
        "checkout" => {
            let enrollment = num(f, "enrollment_id", 1, i64::MAX)?;
            let request = text(f, "request_key", 128)?;
            let mut cart = BTreeMap::new();
            for (k, v) in f {
                if let Some(id) = k.strip_prefix("qty_") {
                    let qty = v.parse::<i64>().map_err(|_| "Quantidade inválida.")?;
                    if !(0..=1000000).contains(&qty) {
                        return Err("Quantidade inválida.".into());
                    }
                    if qty > 0 {
                        cart.insert(id.parse::<i64>().map_err(|_| "Produto inválido.")?, qty);
                    }
                }
            }
            let id = checkout(&tx, user, enrollment, &request, &cart, today())?;
            format!("Entrega #{id} concluída. Estoque e créditos debitados.")
        }
        "import" => {
            admin(role)?;
            let source = text(f, "csv", 200000)?;
            let mut reader = csv::ReaderBuilder::new()
                .delimiter(b';')
                .from_reader(source.as_bytes());
            if reader
                .headers()
                .map_err(|_| "Cabeçalho CSV inválido.")?
                .iter()
                .collect::<Vec<_>>()
                != vec![
                    "nome",
                    "referencia",
                    "membros",
                    "grupo",
                    "credito",
                    "criterios",
                ]
            {
                return Err(
                    "Cabeçalho esperado: nome;referencia;membros;grupo;credito;criterios".into(),
                );
            }
            let mut n = 0;
            for record in reader.records() {
                let r = record.map_err(|_| "CSV inválido. Nenhuma linha foi importada.")?;
                let mut entry = Form::new();
                for (key, val) in [
                    "name",
                    "reference",
                    "members",
                    "group_code",
                    "credit",
                    "criteria",
                ]
                .into_iter()
                .zip(r.iter())
                {
                    entry.insert(key.into(), val.into());
                }
                entry.insert("cycle_id".into(), text(f, "cycle_id", 20)?);
                insert_family(&tx, user, &entry)?;
                n += 1;
                if n > 100 {
                    return Err("Limite de 100 famílias por importação.".into());
                }
            }
            format!("{n} famílias importadas. Duplicidades são bloqueadas pela referência.")
        }
        _ => return Err("Ação desconhecida.".into()),
    };
    tx.commit().map_err(db_err)?;
    Ok(message)
}
fn insert_family(c: &Connection, user: i64, f: &Form) -> Result<()> {
    let name = text(f, "name", 120)?;
    let reference = text(f, "reference", 50)?.to_uppercase();
    let members = num(f, "members", 1, 30)?;
    c.execute(
        "INSERT INTO families(name,reference,members) VALUES(?1,?2,?3)",
        params![name, reference, members],
    )
    .map_err(db_err)?;
    let id = c.last_insert_rowid();
    enroll(c, user, id, f)?;
    audit(c, user, "create", "family", id, &reference).map_err(db_err)?;
    Ok(())
}
fn enroll(c: &Connection, user: i64, family: i64, f: &Form) -> Result<()> {
    let cycle = num(f, "cycle_id", 1, i64::MAX)?;
    let group = text(f, "group_code", 1)?;
    if group != "A" && group != "B" {
        return Err("Grupo inválido.".into());
    }
    let credit = amount(f, "credit")?;
    if credit <= 0 {
        return Err("O crédito deve ser maior que zero.".into());
    }
    let criteria = text(f, "criteria", 500)?;
    let(capacity,count,group_count):(i64,i64,i64)=c.query_row("SELECT capacity,(SELECT count(*) FROM enrollments WHERE cycle_id=?1),(SELECT count(*) FROM enrollments WHERE cycle_id=?1 AND group_code=?2) FROM cycles WHERE id=?1",params![cycle,group],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(db_err)?;
    if count >= capacity || group_count >= capacity / 2 {
        return Err("Limite do ciclo (100) ou do grupo (50) atingido.".into());
    }
    c.execute("INSERT INTO enrollments(family_id,cycle_id,group_code,monthly_credit,criteria) VALUES(?1,?2,?3,?4,?5)",params![family,cycle,group,credit,criteria]).map_err(db_err)?;
    audit(
        c,
        user,
        "create",
        "enrollment",
        c.last_insert_rowid(),
        &format!("família={family}; ciclo={cycle}; grupo={group}"),
    )
    .map_err(db_err)?;
    Ok(())
}
fn movement(
    c: &Connection,
    lot: i64,
    item: Option<i64>,
    kind: &str,
    qty: i64,
    reason: &str,
    user: i64,
) -> Result<()> {
    c.execute("INSERT INTO stock_movements(lot_id,checkout_item_id,kind,quantity,reason,user_id) VALUES(?1,?2,?3,?4,?5,?6)",params![lot,item,kind,qty,reason,user]).map_err(db_err)?;
    Ok(())
}
pub fn checkout(
    c: &Connection,
    user: i64,
    enrollment: i64,
    request: &str,
    cart: &BTreeMap<i64, i64>,
    day: NaiveDate,
) -> Result<i64> {
    let old = c
        .query_row(
            "SELECT id,enrollment_id FROM checkouts WHERE request_key=?1",
            [request],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
        )
        .optional()
        .map_err(db_err)?;
    if let Some((id, e)) = old {
        return if e == enrollment {
            Ok(id)
        } else {
            Err("Identificador de entrega já utilizado.".into())
        };
    }
    if cart.is_empty() || cart.len() > 200 {
        return Err("Selecione pelo menos um produto (máximo 200).".into());
    }
    let(start,group,credit,active):(String,String,i64,i64)=c.query_row("SELECT cy.start_month,e.group_code,e.monthly_credit,e.active FROM enrollments e JOIN cycles cy ON cy.id=e.cycle_id WHERE e.id=?1",[enrollment],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(db_err)?;
    let month = day.format("%Y-%m").to_string();
    if active != 1 || !eligible(&start, &group, &month) {
        return Err("Família inativa ou fora do mês de atendimento do seu grupo.".into());
    }
    let used: i64 = c
        .query_row(
            "SELECT count(*) FROM checkouts WHERE enrollment_id=?1 AND month=?2",
            params![enrollment, month],
            |r| r.get(0),
        )
        .map_err(db_err)?;
    if used > 0 {
        return Err("Esta família já realizou a retirada neste mês.".into());
    }
    let mut items = Vec::new();
    let mut total = 0i64;
    for (&product, &qty) in cart {
        if !(1..=1000000).contains(&qty) {
            return Err("Quantidade inválida.".into());
        }
        let price:Option<(i64,i64)>=c.query_row("SELECT pr.id,pr.credits FROM prices pr JOIN products p ON p.id=pr.product_id WHERE pr.product_id=?1 AND p.active=1 AND pr.effective_date<=?2 ORDER BY pr.effective_date DESC LIMIT 1",params![product,day.to_string()],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(db_err)?;
        let Some((price_id, unit)) = price else {
            return Err("Produto sem preço vigente.".into());
        };
        let stock:i64=c.query_row("SELECT COALESCE(sum(remaining),0) FROM lots WHERE product_id=?1 AND expires_on>=?2",params![product,day.to_string()],|r|r.get(0)).map_err(db_err)?;
        if stock < qty {
            return Err(format!(
                "Estoque disponível insuficiente para o produto #{product}."
            ));
        }
        total = total
            .checked_add(unit.checked_mul(qty).ok_or("Total acima do limite.")?)
            .ok_or("Total acima do limite.")?;
        items.push((product, qty, price_id, unit));
    }
    if total > credit {
        return Err(format!(
            "Créditos insuficientes: necessário {}, disponível {}.",
            display_money(total),
            display_money(credit)
        ));
    }
    c.execute("INSERT INTO checkouts(enrollment_id,month,total_credits,user_id,request_key) VALUES(?1,?2,?3,?4,?5)",params![enrollment,month,total,user,request]).map_err(db_err)?;
    let checkout_id = c.last_insert_rowid();
    for (product, qty, price_id, unit) in items {
        c.execute("INSERT INTO checkout_items(checkout_id,product_id,price_id,quantity,unit_credits) VALUES(?1,?2,?3,?4,?5)",params![checkout_id,product,price_id,qty,unit]).map_err(db_err)?;
        let item = c.last_insert_rowid();
        let mut needed = qty;
        let lots=rows(c,"SELECT id,remaining FROM lots WHERE product_id=?1 AND expires_on>=?2 AND remaining>0 ORDER BY expires_on,id",&[&product,&day.to_string()]).map_err(db_err)?;
        for lot in lots {
            let id = lot[0].parse::<i64>().map_err(|_| "Lote inválido.")?;
            let remaining = lot[1].parse::<i64>().map_err(|_| "Saldo inválido.")?;
            let take = needed.min(remaining);
            if take == 0 {
                break;
            }
            if c.execute(
                "UPDATE lots SET remaining=remaining-?1 WHERE id=?2 AND remaining>=?1",
                params![take, id],
            )
            .map_err(db_err)?
                != 1
            {
                return Err("Estoque alterado. Refaça a entrega.".into());
            }
            c.execute(
                "INSERT INTO allocations(checkout_item_id,lot_id,quantity) VALUES(?1,?2,?3)",
                params![item, id, take],
            )
            .map_err(db_err)?;
            movement(
                c,
                id,
                Some(item),
                "checkout",
                -take,
                "Entrega à família",
                user,
            )?;
            needed -= take;
        }
        if needed != 0 {
            return Err("Não foi possível reservar todos os lotes.".into());
        }
    }
    audit(
        c,
        user,
        "checkout",
        "checkout",
        checkout_id,
        &format!("inscrição={enrollment}; créditos={total}"),
    )
    .map_err(db_err)?;
    Ok(checkout_id)
}
pub fn display_money(n: i64) -> String {
    format!("{},{:02}", n / 100, n.abs() % 100)
}
fn admin(role: &str) -> Result<()> {
    if role == "admin" {
        Ok(())
    } else {
        Err("Esta ação exige um administrador.".into())
    }
}
pub fn db_err(e: rusqlite::Error) -> String {
    eprintln!("Database operation failed: {e}");
    match e {
        rusqlite::Error::SqliteFailure(_, Some(ref s)) if s.contains("UNIQUE") => {
            "Registro duplicado. Confira referência, ciclo ou vigência do preço.".into()
        }
        rusqlite::Error::QueryReturnedNoRows => "Registro não encontrado.".into(),
        _ => "Não foi possível salvar. Confira os dados e tente novamente.".into(),
    }
}
