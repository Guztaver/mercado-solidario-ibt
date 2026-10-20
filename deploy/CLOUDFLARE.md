# Endereço Cloudflare

O deploy inicial usa um Quick Tunnel, que fornece HTTPS sem exigir domínio ou token da sua conta. Esse endereço pode mudar após recriar o serviço e deve ser usado para apresentação e homologação.

Para um endereço estável de uso institucional:

1. Na sua conta Cloudflare, crie um túnel nomeado para este projeto.
2. Configure o hostname escolhido com destino HTTP `http://app:3100`.
3. Coloque o token do túnel em `.env` como `TUNNEL_TOKEN=...` sem incluir o arquivo no repositório.
4. No serviço `tunnel` do Compose, substitua o `command` por `tunnel --no-autoupdate run` e acrescente:

```yaml
    environment:
      TUNNEL_TOKEN: ${TUNNEL_TOKEN}
```

5. Recrie apenas o túnel: `docker compose up -d tunnel`.
6. Teste login, `/healthz` e atendimento pelo hostname escolhido.

Preserve `COOKIE_SECURE=true` no acesso HTTPS. Não publique a porta 13100 em todas as interfaces. Não é necessário compartilhar o token no chat.

Documentação: https://developers.cloudflare.com/tunnel/get-started/
