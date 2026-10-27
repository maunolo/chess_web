use actix_service::{Service, Transform};
use actix_web::{
    body::MessageBody,
    dev::ServiceResponse,
    http::header::{HeaderValue, CACHE_CONTROL},
    Error,
};
use futures::{
    future::{ok, Ready},
    Future,
};

use std::pin::Pin;
use std::task::{Context, Poll};

/// `Cache-Control` by path prefix. The JS, WASM and CSS in `/pkg/` keep their
/// URLs across deploys, so browsers revalidate them (the file handler answers
/// 304 while a file is unchanged) and never run a bundle older than the page.
const CACHE_RULES: [(&str, &str); 3] = [
    ("/pkg/", "no-cache"),
    ("/static/", "max-age=3600"),
    ("/favicon.ico", "max-age=3600"),
];

pub struct CacheControlInterceptor;

impl<S, B, Req> Transform<S, Req> for CacheControlInterceptor
where
    S: Service<Req, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CacheControlInterceptorMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CacheControlInterceptorMiddleware { service })
    }
}

pub struct CacheControlInterceptorMiddleware<S> {
    service: S,
}

impl<S, B, Req> Service<Req> for CacheControlInterceptorMiddleware<S>
where
    S: Service<Req, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: Req) -> Self::Future {
        let fut = self.service.call(req);
        Box::pin(async move {
            let mut res = fut.await?;
            let path = res.request().path();
            let cache_control = CACHE_RULES
                .iter()
                .find(|(prefix, _)| path.starts_with(prefix))
                .map(|(_, value)| *value);

            // An error isn't cached, so a file missing for a moment is fetched again
            if let (Some(value), true) = (cache_control, res.status().is_success()) {
                res.headers_mut()
                    .insert(CACHE_CONTROL, HeaderValue::from_static(value));
            }
            Ok(res)
        })
    }
}
