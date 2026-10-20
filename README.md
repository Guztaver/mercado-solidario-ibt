# Mercado Solidário IBT

Sistema de gestão de estoque e créditos sociais da Igreja Batista Teosópolis, em Itabuna/BA. Implementação baseada no projeto de intervenção fornecido, com **Rust, Leptos 0.8, Axum e SQLite**.

## Funcionalidades

- Interface em português, responsiva, renderizada pelo Leptos no servidor. Os formulários funcionam mesmo sem JavaScript; busca local, cálculo do carrinho e impressão são aprimoramentos opcionais.
- Acesso individual com senhas Argon2id, sessões de oito horas, cookies HttpOnly/Secure/SameSite, proteção CSRF e bloqueio temporário após cinco falhas por conta.
- Perfis de administrador e voluntário; trilha de auditoria de operações.
- Cadastro rápido de famílias por referência interna única e importação CSV transacional.
- Ciclos de seis meses, com até 100 famílias e dois grupos de até 50.
- Cadastro de produtos, preços sociais com vigência e histórico imutável de preços utilizados.
- Doações por lote, origem, quantidade, valor estimado em reais e validade.
- Alertas de estoque mínimo e de validade em até 30 dias. Lotes vencidos ficam indisponíveis para entregas.
- Checkout com vários produtos, cota mensal, retirada única por família no mês e alocação FEFO (vence primeiro, sai primeiro).
- Comprovantes, histórico por família, balanços mensais, descartes, exportação CSV e conferência de saldos versus movimentos.

## Regras adotadas

O documento define a alternância dos grupos, mas não detalha a fórmula de concessão dos créditos. A coordenação informa o valor e os critérios de cada inscrição. **O crédito é uma cota por mês elegível e não se acumula. Uma família faz uma única retirada naquele mês, mesmo que sobre crédito.**

O grupo A atende nos meses 1, 3 e 5 do ciclo; B, nos meses 2, 4 e 6. Ciclos não se sobrepõem. Famílias podem ser inscritas novamente em ciclos futuros, preservando todo o histórico. Inscrições suspensas continuam contando na capacidade histórica do ciclo.

Quantidades são inteiras por embalagem: cadastre “Arroz 1 kg”, “Óleo 900 ml” etc. Créditos usam centésimos inteiros e valores das doações usam centavos de real, sem cálculos financeiros com ponto flutuante. O valor de uma doação é o **valor total do lote**, não o preço unitário.

A validade inclui o próprio dia de vencimento. O calendário de atendimento usa America/Sao_Paulo; eventos são armazenados em UTC. O banco usa chaves estrangeiras, WAL, sincronização FULL e transações `BEGIN IMMEDIATE`. Não há edição silenciosa de entregas concluídas.

## Execução local

Requisitos: Rust 1.99 ou superior e compilador C para o SQLite embarcado; ou Docker com Compose.

```powershell
$env:ADMIN_EMAIL = "admin@mercadosolidario.local"
$env:ADMIN_PASSWORD = "defina-uma-senha-forte-com-12-ou-mais-caracteres"
$env:COOKIE_SECURE = "false" # somente para localhost HTTP
cargo run
```

Abra `http://127.0.0.1:3100`. O banco será criado em `data/mercado.db`. As variáveis de administrador só são usadas quando ainda não existe usuário no banco. Não existe senha padrão embutida.

Use um navegador externo para entrar. Caso um navegador embutido retorne “Sessão do formulário expirada”, abra o mesmo endereço no Chrome, Edge ou Firefox.

## Deploy com Docker e Cloudflare

1. Copie o projeto para uma pasta exclusiva no servidor.
2. Crie `.env` a partir de `.env.example`, com e-mail de acesso e senha forte. Restrinja a permissão para `600`.
3. Execute `docker compose up -d --build`.
4. Consulte o endereço HTTPS em `docker compose logs tunnel`.

A aplicação expõe apenas `127.0.0.1:13100` no host. O túnel acessa a aplicação pela rede interna do Compose. O SQLite persiste no volume `mercado_data`; os contêineres executam sem privilégios, com limites de memória e logs rotacionados.

O Compose fornecido usa um **Quick Tunnel**. O endereço `trycloudflare.com` é temporário e pode mudar se o túnel for recriado. Para domínio estável, configure um túnel nomeado da sua conta Cloudflare conforme [deploy/CLOUDFLARE.md](deploy/CLOUDFLARE.md).

## Verificações

```text
cargo test --locked
cargo fmt --check
```

Para repetir as 30 verificações HTTP em um banco separado, faça `cargo build --locked` e `node deploy/smoke.mjs` com Node.js 22 ou superior. Os dados de teste ficam em `target/smoke-*`; o script não acessa o banco de produção.

Os testes exercitam FEFO, bloqueio de vencidos, retirada duplicada, idempotência, falhas com rollback, saldo insuficiente, concorrência entre conexões SQLite, alternância, preço vigente, autorização e escape de HTML. O Dockerfile exige a aprovação dos testes antes de gerar a imagem final.

## Backup e restauração

Backup consistente, mesmo com WAL ativo:

```sh
docker exec mercado-solidario mkdir -p /data/backups
docker exec mercado-solidario mercado-solidario backup /data/backups/mercado-AAAA-MM-DD.db
docker cp mercado-solidario:/data/backups/mercado-AAAA-MM-DD.db ./mercado-AAAA-MM-DD.db
```

O arquivo de destino deve ser novo. Guarde uma cópia fora do servidor. O backup contém dados das famílias e hashes de acesso: mantenha acesso restrito.

Para restaurar: pare `app` e `tunnel`, copie o backup para o volume no lugar de `mercado.db`, remova apenas os arquivos auxiliares `mercado.db-wal` e `mercado.db-shm` desse volume com o serviço parado, confirme dono `10001:10001` e reinicie. Valide `PRAGMA integrity_check` e `PRAGMA foreign_key_check` antes de retomar atendimento. Faça o procedimento primeiro em uma cópia.

## Entrega

- [Manual da equipe](docs/MANUAL.md)
- [UML do banco em SVG](docs/banco-de-dados.svg)
- [UML editável em PlantUML](docs/banco-de-dados.puml)
- [Script SQL real](src/schema.sql)
- [Requisitos e decisões](docs/REQUISITOS.md)

Dados reais das planilhas não foram incluídos no documento e não são inventados pelo sistema. Importe-os após a conferência da coordenação. Homologação com Marley Mota, treinamento, contagem física e validação institucional dos critérios de crédito continuam sendo etapas humanas previstas no projeto de intervenção.
