use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use actix_files::Files;
        use leptos::config::get_configuration;
        use chess_web::app::{shell, App as ChessApp};
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use actix_web::{
            cookie::{Cookie, SameSite},
            middleware, App, HttpServer, get, post, error, web, Error, HttpRequest, HttpResponse
        };
        use chess_web::server::{
            middlewares::cache_control::CacheControlInterceptor,
            websockets::session,
            chess_server::{self, ChessServer},
        };
        use actix::Addr;
        use actix::Actor;

        use std::env;
        use std::net::SocketAddr;
        use std::time::{SystemTime, UNIX_EPOCH};

        use serde::{Deserialize, Serialize};

        use chess_web::entities::protocol::ClientMsg;
        use chess_web::utils::{self, sanitize::sanitize_username, SessionPayload};

        #[derive(Serialize, Deserialize)]
        struct PostSessionPayload {
            username: String
        }

        /// Creates the session cookie, or renames the user of an existing one
        // `web::Bytes` caps the body at 256k. The client doesn't send a JSON
        // content type, so the body is parsed by hand instead of `web::Json`.
        #[post("/sessions")]
        async fn create_session(req: HttpRequest, body: web::Bytes, srv: web::Data<Addr<ChessServer>>) -> Result<HttpResponse, Error> {
            let payload: PostSessionPayload = serde_json::from_slice(&body)?;
            let Some(username) = sanitize_username(&payload.username) else {
                return Err(error::ErrorBadRequest("username is required"));
            };

            // A token that doesn't verify (e.g. the secret changed) gets a new
            // identity instead of locking the browser out for good
            let token = req
                .cookie("session_token")
                .and_then(|cookie| utils::jwt::verified_decode::<SessionPayload>(cookie.value()).ok());

            let session_payload = if let Some(token) = token {
                srv.get_ref().do_send(chess_server::ClientRequest {
                    id: token.claims().sub.clone(),
                    msg: ClientMsg::Username { name: username.clone() },
                });

                SessionPayload {
                    sub: token.claims().sub.clone(),
                    name: username,
                    iat: token.claims().iat,
                }
            } else {
                let iat = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
                SessionPayload {
                    sub: uuid::Uuid::new_v4().to_string(),
                    name: username,
                    iat,
                }
            };

            let session_token = utils::jwt::encode(session_payload).map_err(error::ErrorInternalServerError)?;

            let session_cookie = Cookie::build("session_token", session_token)
                .path("/")
                .same_site(SameSite::Lax)
                .finish();

            Ok(HttpResponse::Ok().cookie(session_cookie).finish()) // <- send response
        }

        /// The user of the session cookie, 401 when there is none or it doesn't verify
        #[get("/sessions")]
        async fn current_session(req: HttpRequest) -> HttpResponse {
            let token = req
                .cookie("session_token")
                .and_then(|cookie| utils::jwt::verified_decode::<SessionPayload>(cookie.value()).ok());

            match token {
                Some(token) => HttpResponse::Ok().json(serde_json::json!({
                    "id": token.claims().sub,
                    "name": token.claims().name,
                })),
                None => HttpResponse::Unauthorized().finish(),
            }
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
            let Ok(token) = utils::jwt::verified_decode::<SessionPayload>(session_cookie.value()) else {
                return Ok(HttpResponse::Unauthorized().finish());
            };

            // Tokens issued before names were sanitized may still carry a bad one
            let username = sanitize_username(&token.claims().name).unwrap_or("Anonymous".to_string());
            let id = token.claims().sub.clone();

            let (response, ws_session, msg_stream) = actix_ws::handle(&req, stream)?;
            actix_web::rt::spawn(session::run(srv.get_ref().clone(), id, username, ws_session, msg_stream));

            Ok(response)
        }

        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            env_logger::init();

            if env::var("JWT_SECRET").map_or(true, |secret| secret.is_empty()) {
                log::error!("JWT_SECRET must be set");
                std::process::exit(1);
            }

            // Setting this to None means we'll be using cargo-leptos and its env vars.
            let conf = get_configuration(None).expect("Failed to read the Leptos configuration");

            let port = env::var("PORT").unwrap_or("3100".to_string()).parse::<u16>().unwrap_or(3100);
            let addr = SocketAddr::from(([0, 0, 0, 0], port));

            // Generate the list of routes in your Leptos App
            let routes = generate_route_list(ChessApp);

            // start chat server actor
            let server = ChessServer::new().start();

            log::info!("starting HTTP server at http://0.0.0.0:{}", port);

            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = leptos_options.site_root.to_string();
                App::new()
                    .app_data(web::Data::new(server.clone()))
                    .app_data(web::Data::new(leptos_options.to_owned()))
                    .wrap(middleware::Logger::default())
                    .wrap(CacheControlInterceptor)
                    // websocket route
                    .route("/ws", web::get().to(chess_route))
                    .service(create_session)
                    .service(current_session)
                    .leptos_routes(routes.to_owned(), {
                        let leptos_options = leptos_options.to_owned();
                        move || shell(leptos_options.clone())
                    })
                    .service(Files::new("/", site_root))
                    .wrap(middleware::Compress::default())
            })
            .workers(2)
            .bind(&addr)?
            .run()
            .await
        }
    }
    else {
        pub fn main() {}
    }
}
