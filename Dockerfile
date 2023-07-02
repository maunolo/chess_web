FROM rustlang/rust:nightly-slim

# Args
ARG NODE_ENV=production

# # Install deps
RUN apt-get -y update \
    && apt-get -y install curl pkg-config libssl-dev

# Install node
RUN curl -sL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && npm i -g yarn

# Leptos dependencies
RUN rustup target add wasm32-unknown-unknown \
    && cargo install --git https://github.com/leptos-rs/cargo-leptos --locked cargo-leptos 

# Create app directory
WORKDIR /usr/src/app

# Bundle app source
COPY . .

# Compile Project
RUN cargo leptos build --release

# Compile CSS
RUN yarn install && yarn build:styles

# Env variables
ENV NODE_ENV=${NODE_ENV}
ENV PORT=3000
ENV JWT_SECRET=secret
ENV LEPTOS_OUTPUT_NAME=chess_web
ENV LEPTOS_SITE_ROOT=target/site
ENV LEPTOS_SITE_PKG_DIR=pkg
ENV LEPTOS_SITE_ADDR=127.0.0.1:3000
ENV LEPTOS_LIB_DIR=.
ENV LEPTOS_BIN_DIR=.

# Start
CMD ["./target/server/release/chess_web"]