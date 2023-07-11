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

1. `cargo install --locked cargo-leptos`
2. `rustup toolchain install nightly --allow-downgrade` - make sure you have Rust nightly
3. `rustup default nightly` - setup nightly as default, or you can use rust-toolchain file later on 
4. `rustup target add wasm32-unknown-unknown` - add the ability to compile Rust to WebAssembly
5. `yarn install` if you don't have yarn installed you can find how to do it [here](https://classic.yarnpkg.com/lang/en/docs/install)

### Running

For running the actix server and compiling the wasm package run:

```sh
JWT_SECRET=your_secret RUST_LOG=debug cargo leptos watch
```

and for compiling the css run (needs to run after the server is running):

```sh
yarn watch:styles
```

Open browser on [http://localhost:3100/](http://localhost:3100/)
