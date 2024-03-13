FROM rustlang/rust:nightly-alpine as builder

RUN apk update && \
    apk add --no-cache bash curl npm libc-dev binaryen
    # protoc openssl-dev protobuf-dev gcc git g++ libc-dev make binaryen

# Install Cargo Leptos
RUN curl --proto '=https' --tlsv1.2 -LsSf https://leptos-rs.artifacts.axodotdev.host/cargo-leptos/v0.2.16/cargo-leptos-installer.sh | sh

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /work
COPY . .

# Args
ARG NODE_ENV=production
ARG APP_TITLE=Chess

# Build the project
RUN cargo leptos build --release -vv

# Compile CSS
RUN npm install && npm run build:styles

FROM rustlang/rust:nightly-alpine as runner

WORKDIR /app

COPY --from=builder /work/target/release/chess_web /app/
COPY --from=builder /work/target/site /app/site
COPY --from=builder /work/Cargo.toml /app/

EXPOSE $PORT
ENV LEPTOS_SITE_ROOT=./site
ENV JWT_SECRET=secret

CMD ["/app/chess_web"]
