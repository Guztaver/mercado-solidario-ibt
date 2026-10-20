# Relação com o projeto de intervenção

| Necessidade do documento | Implementação |
|---|---|
| Gestão autônoma, fora do Excel | Aplicação Rust, Leptos SSR e Axum |
| Banco relacional centralizado | SQLite persistente com chaves estrangeiras, WAL e transações |
| Cadastro rápido de famílias e critérios | Cadastro, referência única, inscrição, crédito e justificativa |
| 100 famílias por ciclo de seis meses; 50 intercaladas por mês | Dois grupos de 50; meses alternados a partir do início do ciclo |
| Entrada de doações com produto, quantidade, valor e validade | Doações e lotes, com valor total estimado em reais |
| Baixa automática e débito simultâneo | Transação imediata envolvendo entrega, itens, alocações, estoque e auditoria |
| Trava contra retirada duplicada | Restrição única por inscrição/mês e chave idempotente de requisição |
| Validade próxima e estoque mínimo | Painel de alertas; bloqueio de vencidos; FEFO |
| Tabela de preços temporizada | Vigências com data e preço congelado em cada item entregue |
| Relatórios de entrada, saída e consumo familiar | Balanços por mês, histórico, comprovantes e CSV |
| Migração das planilhas | Importação CSV de famílias com validação e rollback integral |
| Telas desktop e mobile | Interface responsiva única, com formulários nativos |
| Autenticação e segurança | Argon2id, sessão protegida, CSRF, perfis e auditoria |
| Modelagem do banco / DER / SQL | UML gerado da migração SQLite, PlantUML editável e SQL |
| Hospedagem solicitada pelo usuário | Docker em servidor próprio, acesso HTTPS Cloudflare |
| Documentação e continuidade | Código-fonte, manual, deploy e backup documentados |

## Decisões que precisam ser validadas na homologação

- A coordenação define manualmente o crédito e registra os critérios. Não foi inventada uma fórmula de elegibilidade socioeconômica.
- Cota mensal sem acumulação e uma retirada por mês elegível. O documento pede trava contra duplicidade, mas não especifica entregas parciais.
- Quantidades inteiras por embalagem. Produtos a granel devem ser cadastrados como embalagens padronizadas.
- Lote pode ser entregue até a data de validade inclusive. Alterar essa regra se a instituição exigir margem adicional.
- Não se apagam entregas, preços ou movimentos de estoque pela interface. Eventuais correções posteriores precisam de um procedimento de estorno validado pela coordenação antes de serem acrescentadas.
- Não há funcionamento offline. O sistema responde com formulários leves, mas precisa de conexão com o servidor.
- Sem planilhas reais anexadas, não houve migração de dados pessoais. Não há integração automática com Excel.
- A versão inicial usa um Quick Tunnel; um endereço institucional fixo requer um túnel nomeado e domínio da conta Cloudflare do responsável.

Treinamento, pesquisa de satisfação, medição presencial do tempo de atendimento e homologação com Marley Mota são atividades da intervenção, não resultados simulados pelo software.
