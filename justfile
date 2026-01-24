set unstable

cargo_install := which("cargo-binstall") || "cargo install"

help:
  just -l -u --list-submodules

# setup project
setup:
  rustup target add wasm32-unknown-unknown
  {{ cargo_install }} cargo-leptos wasm-bindgen-cli
  cargo update
  just r2 download-weights

# build with leptos for cloudflare
build-leptos:
  cargo leptos build --release

# serve in a dev environment with cloudflare
dev:
  npx wrangler dev --ip 0.0.0.0 --port 8786

# deploy to cloudflare
deploy:
  npx wrangler deploy

# serve in a dev environment without cloudflare
dev-nocloudflare:
  cargo leptos watch --release --split

# check compilation
check features="ssr,cloudflare":
  cargo check --features {{ features }} --target wasm32-unknown-unknown

# run wasm tests with firefox
test-wasm:
  wasm-pack test --headless --firefox

_claude *args:
    claude {{ args }}

claude *args:
    just -E .env-claude _claude {{ args }}

mod r2 "assets"
