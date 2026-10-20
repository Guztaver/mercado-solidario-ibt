use super::*;
use std::collections::BTreeMap;
fn setup() -> Connection {
    let c = Connection::open_in_memory().unwrap();
    db::init(&c).unwrap();
    c.execute_batch("INSERT INTO users(id,name,email,password_hash,role) VALUES(1,'Teste','teste@ibt.test','unused','admin');
    INSERT INTO cycles VALUES(1,'Teste','2026-01',100);
    INSERT INTO families(id,name,reference,members) VALUES(1,'Família Teste','IBT-001',4);
    INSERT INTO enrollments(id,family_id,cycle_id,group_code,monthly_credit,criteria) VALUES(1,1,1,'A',10000,'Teste');
    INSERT INTO products(id,name,unit,minimum) VALUES(1,'Arroz','1kg',5),(2,'Feijão','1kg',5);
    INSERT INTO prices(id,product_id,effective_date,credits) VALUES(1,1,'2026-01-01',500),(2,2,'2026-01-01',800);
    INSERT INTO donations(id,donor,received_on,user_id) VALUES(1,'Teste','2026-01-01',1);
    INSERT INTO lots(id,donation_id,product_id,code,expires_on,received,remaining,value_cents) VALUES
    (1,1,1,'vencido','2025-12-31',10,10,5000),
    (2,1,1,'primeiro','2026-02-01',2,2,1000),
    (3,1,1,'segundo','2026-04-01',10,10,5000),
    (4,1,2,'feijao','2026-05-01',10,10,8000);
    INSERT INTO stock_movements(lot_id,kind,quantity,reason,user_id) SELECT id,'donation',received,'Teste',1 FROM lots;").unwrap();
    c
}
fn day() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()
}
fn run(c: &mut Connection, key: &str, cart: BTreeMap<i64, i64>) -> db::Result<i64> {
    let tx = c
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .unwrap();
    let id = db::checkout(&tx, 1, 1, key, &cart, day())?;
    tx.commit().unwrap();
    Ok(id)
}
#[test]
fn checkout_uses_fefo_and_never_expired_stock() {
    let mut c = setup();
    run(&mut c, "one", BTreeMap::from([(1, 3)])).unwrap();
    let balances = db::rows(&c, "SELECT remaining FROM lots ORDER BY id", &[]).unwrap();
    assert_eq!(balances, vec![vec!["10"], vec!["0"], vec!["9"], vec!["10"]]);
    assert_eq!(
        c.query_row("SELECT total_credits FROM checkouts", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1500
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM allocations", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert_eq!(c.query_row("SELECT count(*) FROM (SELECT l.id FROM lots l JOIN stock_movements sm ON sm.lot_id=l.id GROUP BY l.id HAVING l.remaining<>sum(sm.quantity))",[],|r|r.get::<_,i64>(0)).unwrap(),0);
}
#[test]
fn duplicate_is_blocked_and_retry_is_idempotent() {
    let mut c = setup();
    let id = run(&mut c, "one", BTreeMap::from([(1, 1)])).unwrap();
    assert_eq!(run(&mut c, "one", BTreeMap::from([(1, 1)])).unwrap(), id);
    assert!(
        run(&mut c, "different", BTreeMap::from([(1, 1)]))
            .unwrap_err()
            .contains("já realizou")
    );
    assert_eq!(
        c.query_row("SELECT sum(quantity) FROM checkout_items", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
}
#[test]
fn insufficient_credit_and_stock_leave_everything_unchanged() {
    let mut c = setup();
    c.execute("UPDATE enrollments SET monthly_credit=100", [])
        .unwrap();
    assert!(run(&mut c, "credit", BTreeMap::from([(1, 1), (2, 1)])).is_err());
    c.execute("UPDATE enrollments SET monthly_credit=100000", [])
        .unwrap();
    assert!(run(&mut c, "stock", BTreeMap::from([(1, 13)])).is_err());
    assert_eq!(
        c.query_row("SELECT sum(remaining) FROM lots", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        32
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM checkouts", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn failure_mid_transaction_rolls_back_stock_and_credits() {
    let mut c = setup();
    c.execute_batch("CREATE TRIGGER fail_item BEFORE INSERT ON checkout_items WHEN NEW.product_id=2 BEGIN SELECT RAISE(ABORT,'simulated failure'); END;").unwrap();
    assert!(run(&mut c, "rollback", BTreeMap::from([(1, 1), (2, 1)])).is_err());
    assert_eq!(
        c.query_row("SELECT sum(remaining) FROM lots", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        32
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM checkouts", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM allocations", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn groups_alternate_for_exactly_six_months() {
    assert!(db::eligible("2026-11", "A", "2026-11"));
    assert!(!db::eligible("2026-11", "B", "2026-11"));
    assert!(db::eligible("2026-11", "B", "2027-04"));
    assert!(!db::eligible("2026-11", "A", "2027-05"));
    assert!(!db::eligible("2026-11", "A", "2026-09"));
    assert!(!db::eligible("invalid", "A", "2026-11"));
}
#[test]
fn wrong_month_inactive_and_empty_cart_are_blocked() {
    let mut c = setup();
    c.execute("UPDATE enrollments SET group_code='B'", [])
        .unwrap();
    assert!(run(&mut c, "group", BTreeMap::from([(1, 1)])).is_err());
    c.execute("UPDATE enrollments SET group_code='A',active=0", [])
        .unwrap();
    assert!(run(&mut c, "inactive", BTreeMap::from([(1, 1)])).is_err());
    c.execute("UPDATE enrollments SET active=1", []).unwrap();
    assert!(run(&mut c, "empty", BTreeMap::new()).is_err());
}
#[test]
fn future_prices_do_not_change_current_checkout_or_receipt() {
    let mut c = setup();
    c.execute(
        "INSERT INTO prices(product_id,effective_date,credits) VALUES(1,'2026-02-01',9999)",
        [],
    )
    .unwrap();
    run(&mut c, "price", BTreeMap::from([(1, 1)])).unwrap();
    assert_eq!(
        c.query_row("SELECT unit_credits FROM checkout_items", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        500
    );
}
#[test]
fn money_has_no_float_rounding_or_negative_values() {
    assert_eq!(db::money("12,50").unwrap(), 1250);
    assert_eq!(db::money("0.01").unwrap(), 1);
    assert_eq!(db::money("1.5").unwrap(), 150);
    for value in [
        "-1",
        "1.001",
        "NaN",
        "1e9",
        "1.000,00",
        "99999999999999999",
        "",
        ".1",
    ] {
        assert!(db::money(value).is_err(), "{value}");
    }
}
#[test]
fn unauthorized_mutations_are_rejected() {
    let mut c = setup();
    assert!(
        db::apply(&mut c, 1, "volunteer", "cycle", &db::Form::new())
            .unwrap_err()
            .contains("administrador")
    );
}
#[test]
fn pages_escape_untrusted_family_names() {
    let c = setup();
    c.execute("UPDATE families SET name='<script>alert(1)</script>'", [])
        .unwrap();
    let u = User {
        id: 1,
        name: "Teste".into(),
        role: "admin".into(),
        csrf: "test".into(),
    };
    let html = ui::render(&c, &u, "familias", &HashMap::new()).unwrap();
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;"));
}
#[test]
fn all_pages_render_against_real_schema() {
    let c = setup();
    let u = User {
        id: 1,
        name: "Teste".into(),
        role: "admin".into(),
        csrf: "test".into(),
    };
    for page in [
        "painel",
        "familias",
        "estoque",
        "doacoes",
        "atendimento",
        "relatorios",
        "ciclos",
        "configuracoes",
        "historico",
        "recibo",
    ] {
        let html = ui::render(&c, &u, page, &HashMap::new()).unwrap();
        assert!(html.contains("<!DOCTYPE html>"), "{page}");
    }
}
#[test]
fn two_connections_cannot_duplicate_a_withdrawal() {
    let c = setup();
    // Use an empty, nonexistent destination for a consistent SQLite snapshot.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("race.db");
    c.execute("VACUUM INTO ?1", [path.to_str().unwrap()])
        .unwrap();
    let initialized = Connection::open(&path).unwrap();
    db::init(&initialized).unwrap();
    drop(initialized);
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let mut handles = Vec::new();
    for i in 0..2 {
        let path = path.clone();
        let barrier = barrier.clone();
        handles.push(std::thread::spawn(move || {
            let mut c = Connection::open(path).unwrap();
            c.busy_timeout(std::time::Duration::from_secs(10)).unwrap();
            barrier.wait();
            run(&mut c, &format!("race-{i}"), BTreeMap::from([(1, 1)]))
        }));
    }
    let outcomes: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(outcomes.iter().filter(|r| r.is_ok()).count(), 1);
    let c = Connection::open(path).unwrap();
    assert_eq!(
        c.query_row("SELECT sum(quantity) FROM checkout_items", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn csv_import_rolls_back_every_row_when_one_reference_is_duplicate() {
    let mut c = setup();
    let f = db::Form::from([
        ("cycle_id".into(), "1".into()),
        ("csv".into(), "nome;referencia;membros;grupo;credito;criterios\nNova;IBT-002;2;A;100,00;Teste\nDuplicada;IBT-001;3;B;100,00;Teste".into()),
    ]);
    assert!(db::apply(&mut c, 1, "admin", "import", &f).is_err());
    assert_eq!(
        c.query_row("SELECT count(*) FROM families", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM enrollments", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn group_capacity_is_enforced_without_creating_an_orphan_family() {
    let mut c = setup();
    for i in 2..=50 {
        c.execute(
            "INSERT INTO families(id,name,reference,members) VALUES(?1,'Teste',?2,1)",
            params![i, format!("REF-{i}")],
        )
        .unwrap();
        c.execute("INSERT INTO enrollments(family_id,cycle_id,group_code,monthly_credit,criteria) VALUES(?1,1,'A',10000,'Teste')",[i]).unwrap();
    }
    let f = db::Form::from([
        ("name".into(), "Excedente".into()),
        ("reference".into(), "REF-51".into()),
        ("members".into(), "1".into()),
        ("cycle_id".into(), "1".into()),
        ("group_code".into(), "A".into()),
        ("credit".into(), "100,00".into()),
        ("criteria".into(), "Teste".into()),
    ]);
    assert!(
        db::apply(&mut c, 1, "admin", "family", &f)
            .unwrap_err()
            .contains("Limite")
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM families", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        50
    );
}

#[test]
fn password_hashes_and_sessions_do_not_store_plaintext_secrets() {
    let password = "SomenteTeste123456";
    let hash = hash_password(password).unwrap();
    assert!(hash.starts_with("$argon2id$"));
    assert!(
        Argon2::default()
            .verify_password(password.as_bytes(), &PasswordHash::new(&hash).unwrap())
            .is_ok()
    );
    assert!(
        Argon2::default()
            .verify_password(b"wrong password", &PasswordHash::new(&hash).unwrap())
            .is_err()
    );
    let token = random();
    assert_eq!(token.len(), 64);
    assert_ne!(digest(&token), token);
}
