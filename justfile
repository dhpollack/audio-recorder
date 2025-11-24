help:
  just -l -u

build-leptos:
  cargo leptos build --release

# Build the web worker
build-worker:
  mkdir -p target/site/pkg
  cargo build --bin audio_worker --target wasm32-unknown-unknown
  wasm-bindgen --target no-modules --out-dir target/site/pkg --out-name audio_worker --no-typescript target/wasm32-unknown-unknown/debug/audio_worker.wasm

dev:
  npx wrangler dev --ip 0.0.0.0 --port 8786

deploy:
  npx wrangler deploy

_claude *args:
    claude {{ args }}

claude *args:
    just -E .env-claude _claude {{ args }}
