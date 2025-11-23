help:
  just -l -u

dev:
  npx wrangler dev --ip 0.0.0.0 --port 8786

deploy:
  npx wrangler deploy

_claude *args:
    claude {{ args }}

claude *args:
    just -E .env-claude _claude {{ args }}
