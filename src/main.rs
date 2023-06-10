mod app;
mod components;
mod entities;
use actix_web::Responder;
use cfg_if::cfg_if;
use server::middlewares::cache_control::CacheControlInterceptor;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        mod server;

        use actix_files::Files;
        use leptos::*;
        use crate::app::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use actix_web::{
            middleware, App, HttpServer, get
        };

        use std::env;
        use std::net::SocketAddr;

        #[get("/style.css")]
        async fn css() -> impl Responder {
            actix_files::NamedFile::open_async("./target/site/pkg/chess_web.css").await
        }

        #[actix_web::main]
        async fn main() -> std::io::Result<()> {

            // Setting this to None means we'll be using cargo-leptos and its env vars.
            let conf = get_configuration(Some("Cargo.toml")).await.unwrap();

            let port = env::var("PORT").unwrap_or("3100".to_string()).parse::<u16>().unwrap_or(3100);
            let addr = SocketAddr::from(([0, 0, 0, 0], port));

            // Generate the list of routes in your Leptos App
            let routes = generate_route_list(|cx| view! { cx, <App/> });

            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;
                let routes = &routes;
                App::new()
                    .wrap(middleware::Logger::default())
                    .wrap(CacheControlInterceptor)
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
        pub fn main() {}
    }
}