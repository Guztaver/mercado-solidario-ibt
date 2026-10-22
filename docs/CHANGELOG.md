# Alterações

## 1.0.1 — 22/10/2026

Corrigido o erro “Sessão do formulário expirada” que ocorria antes do primeiro login. A requisição automática a `/favicon.ico` era tratada como uma página protegida e redirecionada a `/login`. Esse segundo formulário substituía o cookie do primeiro, causando a rejeição do envio.

- Ícone SVG servido em rotas próprias, sem redirecionamento e sem alterar cookies de login.
- Endereços inexistentes respondem com 404 antes da autenticação.
- Uma nova página de login reutiliza o código de segurança válido, preservando outros formulários abertos.
- Um código ausente ou incorreto continua sendo rejeitado, mas agora a resposta apresenta uma nova tela utilizável.
- Leitura de cookies aceita múltiplos cabeçalhos Cookie.
- Cinco verificações de regressão HTTP adicionadas ao fluxo completo, totalizando 35 verificações.

As senhas, sessões autenticadas e regras de autorização foram preservadas.
