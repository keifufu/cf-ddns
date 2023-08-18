# cf-ddns

A simple dns updater for cloudflare.  
It will update specified type A records with the current ipv4.

## Getting Started

### With Docker

Using docker compose:

```yaml
version: "3"
services:
  cf-ddns:
    image: keifufu/cf-ddns:latest
    container_name: cf-ddns
    restart: unless-stopped
    volumes:
      - ./config:/app/config
```

or docker run:

```
docker run -v ./config:/app/config keifufu/cf-ddns:latest
```
