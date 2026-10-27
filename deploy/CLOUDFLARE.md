# Túnel Cloudflare permanente

O Compose padrão usa um Quick Tunnel para instalações locais e demonstrações. O endereço `trycloudflare.com` pode mudar quando o túnel é recriado.

Para configurar um domínio próprio:

1. Crie um túnel remoto em Cloudflare Zero Trust → **Networks → Tunnels** e selecione Docker.
2. Adicione um **Public Hostname** para o domínio escolhido, com serviço HTTP `app:3100`. Isso cria a rota pública e o registro DNS.
3. Salve o token do túnel em `secrets/cloudflare-tunnel.env`, sem o adicionar ao Git:

   ```dotenv
   TUNNEL_TOKEN=cole-o-token-localmente
   ```

4. Restrinja o arquivo a `600` e a pasta `secrets/` a `700`.
5. Ative a configuração nomeada:

   ```sh
   docker compose -f compose.yaml -f deploy/compose.permanent.yaml up -d tunnel
   ```

6. Verifique `https://seu-dominio/healthz` e a tela de entrada.

No servidor institucional, `compose.override.yaml` já mantém o túnel nomeado ativo quando o Compose é executado sem opções adicionais. Preserve `COOKIE_SECURE=true`, não exponha a porta 13100 publicamente e nunca compartilhe o token em chats ou no repositório.

Documentação: https://developers.cloudflare.com/tunnel/get-started/
