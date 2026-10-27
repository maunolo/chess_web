# Chessboard Website

Available at: [chess.maunolo.dev](https://chess.maunolo.dev)

![image](https://github.com/maunolo/chess_web/assets/44609720/39a3c9b7-9089-4564-b8ff-8b13c5b5416c)

# Playing

- **Free board** at `/`: anyone moves any piece, captured pieces go to the trash and can be put back. Options turns on rule validation.
- **Games**: Menu → New game picks a color and a clock (none, or 1+0 up to 30+0), then shares a `/g/<id>` link. Whoever opens it first takes the other seat, everyone after that watches. The server enforces turns and the clock, and ends the game on checkmate, stalemate, time, resignation, agreed draw, threefold repetition, the fifty-move rule or insufficient material. Rematches swap colors.
- **Moving**: drag a piece, or tap it and then tap its square. With validation on, the legal moves of the selected piece are marked.
- **Language**: English, or Brazilian Portuguese when the browser asks for Portuguese (`Accept-Language`). The strings live in `src/i18n.rs`; server notifications are sent as keys and translated on the page.

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

1. Rust stable 1.88 or newer (required by Leptos 0.8), the Docker image builds with 1.99
2. `cargo install --locked cargo-leptos` - version 0.3 or newer, it downloads the `wasm-bindgen-cli` matching `Cargo.lock` (don't install `wasm-bindgen-cli` globally, it would take precedence)
3. `rustup target add wasm32-unknown-unknown` - add the ability to compile Rust to WebAssembly
4. `npm install` - Node.js 20 or newer, for the CSS build (Tailwind + PostCSS)

### Running

```sh
npm run dev
```

This builds the CSS, then runs the PostCSS watcher and `cargo leptos watch` (the actix server and the wasm package, with live reload, CSS included). Open browser on [http://localhost:3100/](http://localhost:3100/)

The server exits at startup without `JWT_SECRET`; `npm run dev` defaults it to `dev_secret` and `RUST_LOG` to `info`, override them in the environment. `APP_TITLE` sets the page title at compile time and defaults to `Chess`.

For a release build, `npm run build`, then run `./target/release/chess_web` with `JWT_SECRET` set (`npm start` does both).

### Tests

```sh
cargo test --features ssr
```
