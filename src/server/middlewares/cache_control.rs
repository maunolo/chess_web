use actix_service::{Service, Transform};
use actix_web::{
    body::MessageBody,
    dev::ServiceResponse,
    http::header::{HeaderName, HeaderValue},
    Error,
};
use futures::{
    future::{ok, Ready},
    Future,
};

use std::pin::Pin;
use std::task::{Context, Poll};

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
            if path.starts_with("/static") {
                let headers = res.headers_mut();
                headers.append(
                    HeaderName::from_lowercase(b"cache-control").unwrap(),
                    HeaderValue::from_str("max-age=3600").unwrap(),
                );
            }
            return Ok(res);
        })
    }
}
