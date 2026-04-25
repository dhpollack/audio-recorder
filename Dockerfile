FROM rust:1.92.0-trixie AS builder

RUN apt-get update -y \
  && apt-get install -y --no-install-recommends clang

# Install cargo-leptos
#RUN cargo binstall cargo-leptos -y
RUN cargo install cargo-leptos

RUN rustup target add wasm32-unknown-unknown

RUN mkdir -p /app
WORKDIR /app
COPY . .

RUN cargo leptos build --release

FROM debian:trixie-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/audio-recorder /app/

COPY --from=builder /app/target/site /app/site

COPY --from=builder /app/Cargo.toml /app/

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"
EXPOSE 8080

# -- NB: update binary name from "leptos_start" to match your app name in Cargo.toml --
# Run the server
CMD ["/app/audio-recorder"]

