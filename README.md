# Chessboard Website

Available at: [chess.maunolo.dev](https://chess.maunolo.dev)

![image](https://github.com/maunolo/chess_web/assets/44609720/39a3c9b7-9089-4564-b8ff-8b13c5b5416c)

# Development

## Docker

##### docker-compose.yml

```yml
services:
    chess_web:
        build:
            context: .
        ports:
            - 3100:3100
        environment:
            PORT: 3100
            JWT_SECRET: your_secret
            RUST_LOG: debug
```

## Local

### Dependencies

1. Rust stable 1.85 or newer (the crate uses the 2024 edition), the Docker image builds with 1.99
2. `cargo install --locked cargo-leptos`
3. `rustup target add wasm32-unknown-unknown` - add the ability to compile Rust to WebAssembly
4. `yarn install` if you don't have yarn installed you can find how to do it [here](https://classic.yarnpkg.com/lang/en/docs/install)

### Running

For running the actix server and compiling the wasm package run:

```sh
JWT_SECRET=your_secret RUST_LOG=debug cargo leptos watch
```

`JWT_SECRET` is required, the server exits at startup without it. `APP_TITLE` sets the page title at compile time and defaults to `Chess`.

### Tests

```sh
cargo test --features ssr
```

and for compiling the css run (needs to run after the server is running):

```sh
yarn watch:styles
```

Open browser on [http://localhost:3100/](http://localhost:3100/)
