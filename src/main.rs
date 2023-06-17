mod app;
mod components;
mod entities;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        mod server;

        use actix_files::Files;
        use leptos::*;
        use crate::app::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use actix_web::{
            middleware, App, HttpServer, get, web, Error, HttpRequest, HttpResponse, Responder
        };
        use actix_web_actors::ws;
        use server::{
            middlewares::cache_control::CacheControlInterceptor,
        };
        use actix::Addr;
        use actix::Actor;
        use std::time::Instant;

        use std::env;
        use std::net::SocketAddr;
        use std::sync::{atomic::AtomicUsize, Arc};

        #[get("/style.css")]
        async fn css() -> impl Responder {
            actix_files::NamedFile::open_async("./target/site/pkg/chess_web.css").await
        }

        /// Entry point for our websocket route
        async fn chess_route(
            req: HttpRequest,
            stream: web::Payload,
            srv: web::Data<Addr<server::chess_server::ChessServer>>,
        ) -> Result<HttpResponse, Error> {
            ws::start(
                server::websockets::session::WsChessSession {
                    id: 0,
                    hb: Instant::now(),
                    room_name: "main".to_owned(),
                    name: None,
                    addr: srv.get_ref().clone(),
                },
                &req,
                stream,
            )
        }

        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            env_logger::init();

            // Setting this to None means we'll be using cargo-leptos and its env vars.
            let conf = get_configuration(Some("Cargo.toml")).await.unwrap();

            let port = env::var("PORT").unwrap_or("3100".to_string()).parse::<u16>().unwrap_or(3100);
            let addr = SocketAddr::from(([0, 0, 0, 0], port));

            // Generate the list of routes in your Leptos App
            let routes = generate_route_list(|cx| view! { cx, <App/> });

            // set up applications state
            // keep a count of the number of visitors
            let app_state = Arc::new(AtomicUsize::new(0));

            // start chat server actor
            let server = server::chess_server::ChessServer::new(app_state.clone()).start();

            log::info!("starting HTTP server at http://0.0.0.0:{}", port);

            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;
                let routes = &routes;
                App::new()
                    .app_data(web::Data::from(app_state.clone()))
                    .app_data(web::Data::new(server.clone()))
                    .wrap(middleware::Logger::default())
                    .wrap(CacheControlInterceptor)
                    // websocket route
                    .route("/ws", web::get().to(chess_route))
                    .service(css)
                    .leptos_routes(leptos_options.to_owned(), routes.to_owned(), |cx| view! { cx, <App/> })
                    .service(Files::new("/", site_root).show_files_listing())
                    .wrap(middleware::Compress::default())
            })
            .bind(&addr)?
            .run()
            .await
        }
    }
    else {
        mod handlers;
        mod utils;
        mod client;
        pub fn main() {}
    }
}
