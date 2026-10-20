"""Generate the physical UML directly from the SQLite migration, without real data."""
from pathlib import Path
import sqlite3

root = Path(__file__).resolve().parents[1]
db = sqlite3.connect(':memory:')
db.executescript((root / 'src/schema.sql').read_text(encoding='utf-8'))
tables = [r[0] for r in db.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")]
groups = {
    'Comunidade e créditos': ['families', 'cycles', 'enrollments'],
    'Estoque e doações': ['products', 'prices', 'donations', 'lots'],
    'Atendimento e rastreabilidade': ['checkouts', 'checkout_items', 'allocations', 'stock_movements'],
    'Acesso e auditoria': ['users', 'sessions', 'audit_log', 'login_attempts', 'schema_version'],
}
lines = [
    '@startuml banco-de-dados', '!pragma layout smetana',
    'title Mercado Solidário IBT | Modelo UML do banco SQLite',
    'hide methods', 'hide circle', 'skinparam shadowing false',
    'skinparam backgroundColor #FAFCFA', 'skinparam defaultFontName SansSerif',
    'skinparam defaultFontSize 12', 'skinparam classAttributeIconSize 0',
    'skinparam classBackgroundColor #FFFFFF', 'skinparam classBorderColor #789580',
    'skinparam classHeaderBackgroundColor #E9F1E7', 'skinparam ArrowColor #6C8976',
    'skinparam packageBorderColor #C4D5C3', 'skinparam packageBackgroundColor #F3F7F1',
    'skinparam linetype ortho', 'left to right direction',
]
for label, members in groups.items():
    lines.append(f'package "{label}" {{')
    for table in members:
        assert table in tables
        columns = list(db.execute(f'PRAGMA table_info("{table}")'))
        fks = {r[3] for r in db.execute(f'PRAGMA foreign_key_list("{table}")')}
        lines.append(f'  class {table} <<table>> {{')
        for _, name, kind, required, default, primary in columns:
            tags = []
            if primary: tags.append('PK')
            if name in fks: tags.append('FK')
            if not required and not primary: tags.append('nullable')
            suffix = ' {' + ', '.join(tags) + '}' if tags else ''
            lines.append(f'    {name} : {kind}{suffix}')
        lines.append('  }')
    lines.append('}')
for child in tables:
    info = {r[1]:r for r in db.execute(f'PRAGMA table_info("{child}")')}
    for fk in db.execute(f'PRAGMA foreign_key_list("{child}")'):
        _, _, parent, column, target, *_ = fk
        parent_mult = '1' if info[column][3] else '0..1'
        lines.append(f'{parent} "{parent_mult}" -- "0..*" {child} : {column}')
lines += [
    'note bottom of enrollments',
    '  UNIQUE(family_id, cycle_id)',
    '  Até 100 famílias por ciclo e 50 por grupo (serviço).',
    '  Crédito mensal em centésimos de crédito.',
    'end note',
    'note bottom of checkouts',
    '  UNIQUE(enrollment_id, month)',
    '  UNIQUE(request_key): idempotência',
    '  BEGIN IMMEDIATE: débito e estoque atômicos.',
    'end note',
    'note bottom of prices',
    '  UNIQUE(product_id, effective_date)',
    '  Preços históricos preservados nos itens.',
    'end note',
    'legend bottom',
    '  PK = chave primária | FK = chave estrangeira | nullable = opcional',
    '  Valores em INTEGER: créditos em centésimos; doações em centavos de real.',
    '  Quantidades inteiras por embalagem. Datas ISO 8601. Auditoria em UTC.',
    '  lots.remaining = soma dos movimentos. Lotes vencidos bloqueados.',
    'endlegend', '@enduml',
]
(root / 'docs/banco-de-dados.puml').write_text('\n'.join(lines) + '\n', encoding='utf-8')
print(f'UML gerado: {len(tables)} tabelas, com colunas e relações extraídas da migração.')
