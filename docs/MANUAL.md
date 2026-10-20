# Manual da equipe — Mercado Solidário IBT

## Preparar o primeiro atendimento

1. Entre pelo link HTTPS com o acesso recebido. Em **Equipe & acesso**, troque sua senha e crie acessos individuais para os voluntários.
2. Em **Ciclos**, crie o ciclo e informe o primeiro mês. Grupo A: meses 1, 3 e 5. Grupo B: meses 2, 4 e 6.
3. Em **Famílias & créditos**, cadastre referência interna, responsável, integrantes, ciclo, grupo, crédito mensal e os critérios aprovados. Cada grupo comporta 50 famílias.
4. Em **Estoque**, cadastre os produtos, embalagens, estoque mínimo e preço social em créditos.
5. Em **Doações**, registre os lotes recebidos. Cada lançamento corresponde a um produto, um lote e uma validade.

Não registre CPF, documentos ou informações sensíveis nos campos de observação. Use referências internas, por exemplo `IBT-001`.

## Registrar uma doação

Informe produto, origem, quantidade de embalagens, valor total estimado em reais, lote, recebimento e validade. Para doação sem identificação, use “Doação anônima”. Valores podem ser escritos com vírgula: `25,50`. Se não souber o valor estimado, informe `0,00`.

O recebimento não pode ser futuro e a validade não pode ser anterior a hoje nem ao recebimento. Lotes com validades diferentes devem ser lançados separadamente.

## Atender uma família

1. Abra **Atendimento** e selecione a família. A lista mostra apenas inscrições ativas, no mês correto e ainda sem retirada.
2. Informe a quantidade de cada produto. Confira o total e o saldo apresentados.
3. Confirme a entrega. O sistema verifica novamente créditos, validade, estoque e retirada duplicada antes de salvar.
4. Abra o comprovante na lista de entregas. Ele pode ser impresso e permite consultar os lotes utilizados.

A família faz uma retirada por mês elegível. A cota não se acumula. Separar os produtos e conferir as quantidades antes da confirmação evita correções posteriores. O sistema escolhe os lotes com vencimento mais próximo; separe fisicamente esses mesmos lotes.

Se a conexão cair após confirmar, atualize a página e confira a lista de entregas antes de tentar novamente. O sistema impede duplicidades; a atualização após um envio não repete a entrega.

## Consultar famílias e renovar ciclos

Use a busca por nome ou referência em **Famílias & créditos**. **Consultar** abre todo o histórico de consumo. Para um novo semestre, use **Inscrever uma família em um novo ciclo**: não cadastre a mesma família outra vez.

A coordenação pode ajustar créditos e suspender inscrições em **Ajustar crédito ou suspender uma inscrição**. O sistema preserva os dados anteriores no histórico de auditoria e impede reduzir a cota abaixo de uma entrega já realizada.

## Importar uma planilha

Exporte ou organize os dados como CSV com ponto e vírgula e o seguinte cabeçalho exato:

```csv
nome;referencia;membros;grupo;credito;criterios
Família Exemplo;IBT-001;4;A;100,00;Critério aprovado pela coordenação
```

Escolha o ciclo e cole o conteúdo em **Importar famílias de uma planilha**. Até 100 linhas podem ser importadas por vez. Se houver referência duplicada ou dado inválido, nenhuma família do arquivo será importada. Corrija e repita.

O exemplo acima é fictício e não vem cadastrado no sistema.

## Estoque, validade e perdas

O painel mostra os produtos no mínimo e os lotes que vencem em até 30 dias. Lotes vencidos ficam bloqueados para entrega. A coordenação registra o descarte com quantidade e justificativa; o estoque nunca é alterado sem um movimento.

Para preço novo, registre uma nova vigência. Os comprovantes antigos preservam o preço praticado na entrega. Uma vigência já cadastrada para o mesmo dia não pode ser sobrescrita.

## Relatórios

Escolha o mês em **Relatórios**. Confira recebimentos, entregas, descartes e consumo por família. O saldo físico mostrado é a posição atual, incluindo vencidos. A conferência automática compara os saldos do banco com os movimentos; a contagem física dos produtos deve ser feita pela equipe.

**Exportar entregas CSV** baixa os detalhes de produtos e créditos do mês. Campos de crédito no CSV estão em centésimos: `1250` representa `12,50`. O arquivo não contém senhas.

## Acessos e cuidados diários

Use um acesso por pessoa. Voluntários registram a operação; administradores também gerenciam ciclos, créditos, preços, descartes e acessos. A sessão dura oito horas. Cinco erros de senha bloqueiam novas tentativas para aquela conta por quinze minutos.

Ao concluir, clique no ícone de saída na barra lateral. Em telas pequenas, o menu fica na faixa superior e pode ser rolado horizontalmente. Para sair pelo celular, acesse **Equipe & acesso** e use o botão de saída.

Mantenha backup fora do servidor. O procedimento técnico está no README. Para dúvidas de operação e concessão de crédito, procure a coordenação do Mercado Solidário.
