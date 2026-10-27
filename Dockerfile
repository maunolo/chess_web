FROM rust:1.99-alpine AS builder

RUN apk update && \
    apk add --no-cache bash curl npm libc-dev binaryen
    # protoc openssl-dev protobuf-dev gcc git g++ libc-dev make binaryen

# Install Cargo Leptos
# (it downloads the wasm-bindgen-cli matching Cargo.lock during the build)
RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/download/v0.3.11/cargo-leptos-installer.sh | sh

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /work
COPY . .

# Args
ARG NODE_ENV=production
ARG APP_TITLE=Chess

# Compile CSS, cargo-leptos then bundles it into the site
RUN npm ci && npm run build:styles

# Build the project
RUN cargo leptos build --release -vv

# The server binary is statically linked against musl, it only needs the site files
FROM alpine:3.24 AS runner

WORKDIR /app

COPY --from=builder /work/target/release/chess_web /app/
COPY --from=builder /work/target/site /app/site
COPY --from=builder /work/Cargo.toml /app/

ENV PORT=3100
EXPOSE ${PORT}
ENV LEPTOS_SITE_ROOT=./site
# JWT_SECRET has no default: the server refuses to start without it

CMD ["/app/chess_web"]
