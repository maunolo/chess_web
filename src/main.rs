mod app;
mod components;
mod entities;
mod handlers;
mod utils;

use cfg_if::cfg_if;

use crate::server::chess_server;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        mod server;

        use actix_files::Files;
        use leptos::*;
        use crate::app::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use actix_web::{
            middleware, App, HttpServer, get, post, error, web, Error, HttpRequest, HttpResponse, Responder
        };
        use actix_web_actors::ws;
        use server::{
            middlewares::cache_control::CacheControlInterceptor,
            websockets::session::WsChessSession,
            chess_server::ChessServer,
        };
        use actix::Addr;
        use actix::Actor;

        use std::env;
        use std::net::SocketAddr;
        use std::sync::{atomic::AtomicUsize, Arc};
        use std::path::Path;
        use std::time::{SystemTime, UNIX_EPOCH};

        use futures::StreamExt;
        use serde::{Deserialize, Serialize};

        use utils::SessionPayload;

        #[derive(Serialize, Deserialize)]
        struct PostSessionPayload {
            username: String
        }

        const MAX_SIZE: usize = 262_144; // max payload size is 256k

        #[post("/sessions")]
        async fn create_session(req: HttpRequest, mut payload: web::Payload, srv: web::Data<Addr<ChessServer>>) -> impl Responder {
            // payload is a stream of Bytes objects
            let mut body = web::BytesMut::new();
            while let Some(chunk) = payload.next().await {
                let chunk = chunk?;
                // limit max size of in-memory payload
                if (body.len() + chunk.len()) > MAX_SIZE {
                    return Err(error::ErrorBadRequest("overflow"));
                }
                body.extend_from_slice(&chunk);
            }

            // payload is loaded, now we can deserialize serde-json
            let payload: PostSessionPayload = serde_json::from_slice::<PostSessionPayload>(&body)?;

            let session_payload = if let Some(session_cookie) = req.cookie("session_token") {
                let Ok(token) = utils::jwt::verified_decode::<SessionPayload>(&session_cookie.value().to_string()) else {
                    return Ok(HttpResponse::Unauthorized().finish());
                };

                let chess_server_addr = srv.get_ref().clone();

                chess_server_addr.do_send(
                    chess_server::UserSync { id: token.claims().sub.clone(), name: payload.username.clone() }
                );

                SessionPayload {
                    sub: token.claims().sub.clone(),
                    name: payload.username,
                    iat: token.claims().iat,
                }
            } else {
                let iat = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
                SessionPayload {
                    sub: uuid::Uuid::new_v4().to_string(),
                    name: payload.username,
                    iat: iat,
                }
            };


            let session_token = utils::jwt::encode(session_payload).expect("Failed to encode JWT");

            let session_cookie = actix_web::cookie::Cookie::build("session_token", session_token).path("/").finish();

            Ok(HttpResponse::Ok().cookie(session_cookie).finish()) // <- send response
        }

        #[get("/style.css")]
        async fn css() -> impl Responder {
            let site_path = env::var("LEPTOS_SITE_ROOT").unwrap_or("./target/site".to_string());
            let css_path = Path::new(&site_path).join("pkg/chess_web.css");
            actix_files::NamedFile::open_async(css_path).await
        }

        /// Entry point for our websocket route
        async fn chess_route(
            req: HttpRequest,
            stream: web::Payload,
            srv: web::Data<Addr<ChessServer>>,
        ) -> Result<HttpResponse, Error> {
            let Some(session_cookie) = req.cookie("session_token") else {
                return Ok(HttpResponse::Unauthorized().finish());
            };
            let Ok(token) = utils::jwt::verified_decode::<SessionPayload>(&session_cookie.value().to_string()) else {
                return Ok(HttpResponse::Unauthorized().finish());
            };

            let username = token.claims().name.clone();
            let id = token.claims().sub.clone();

            ws::start(
                WsChessSession::new(srv.get_ref().clone(), id, username),
                &req,
                stream,
            )
        }

        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            env_logger::init();

            // Setting this to None means we'll be using cargo-leptos and its env vars.
            let conf = get_configuration(None).await.unwrap();

            let port = env::var("PORT").unwrap_or("3100".to_string()).parse::<u16>().unwrap_or(3100);
            let addr = SocketAddr::from(([0, 0, 0, 0], port));

            // Generate the list of routes in your Leptos App
            let routes = generate_route_list(|| view! { <App/> });

            // set up applications state
            // keep a count of the number of visitors
            let app_state = Arc::new(AtomicUsize::new(0));

            // start chat server actor
            let server = ChessServer::new(app_state.clone()).start();

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
                    .service(create_session)
                    .service(css)
                    .leptos_routes(leptos_options.to_owned(), routes.to_owned(), || view! { <App/> })
                    .service(Files::new("/", site_root).show_files_listing())
                    .wrap(middleware::Compress::default())
            })
            .workers(2)
            .bind(&addr)?
            .run()
            .await
        }
    }
    else {
        mod client;
        pub fn main() {}
    }
}
