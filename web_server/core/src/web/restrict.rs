use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    error::InternalError,
    HttpRequest, HttpResponse,
};
use futures::{
    future::{self as fut, LocalBoxFuture},
    FutureExt,
};
use std::{future::Future, rc::Rc};

pub enum Restrict {
    Allow,
    Response(ServiceResponse),
    Deny(String),
}
impl Restrict {
    pub fn response(req: HttpRequest, res: HttpResponse) -> Self {
        Self::Response(ServiceResponse::new(req, res))
    }
}
/*
pub fn wrap<
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error>,
    F: 'static + Future<Output = Result<Restrict, actix_web::Error>>,
>(
    rule: fn(HttpRequest) -> F,
) -> impl 'static
       + Clone
       + Fn(ServiceRequest, &S) -> LocalBoxFuture<'static, Result<ServiceResponse, actix_web::Error>>
where
    S::Future: 'static,
{
    move |req: ServiceRequest, srv: &S| {
        let (http_req, payload) = req.into_parts();
        let req = ServiceRequest::from_parts(http_req.clone(), payload);
        let rule = rule(http_req);
        let res = srv.call(req);
        Box::pin(async move {
            match rule.await {
                Ok(Restrict::Allow) => res.await,
                Ok(Restrict::Response(res)) => Ok(res),
                Ok(Restrict::Deny(msg)) => {
                    Err(InternalError::new(msg, actix_web::http::StatusCode::FORBIDDEN).into())
                }
                Err(err) => Err(err),
            }
        })
    }
}
*/

pub fn restrict<F>(rule: fn(HttpRequest) -> F) -> RestrictWrapper<F>
where
    F: 'static + Future<Output = Result<Restrict, actix_web::Error>>,
{
    RestrictWrapper { rule }
}

pub struct RestrictWrapper<F> {
    rule: fn(HttpRequest) -> F,
}

impl<S, F> Transform<S, ServiceRequest> for RestrictWrapper<F>
where
    S: 'static + Service<ServiceRequest, Response = ServiceResponse>,
    S::Future: 'static,
    // TODO: Remove actix_web::Error bound
    S::Error: 'static + From<InternalError<String>> + From<actix_web::Error>,
    F: 'static + Future<Output = Result<Restrict, actix_web::Error>>,
{
    type Response = ServiceResponse;
    type Error = S::Error;
    type InitError = ();
    type Transform = RestrictMiddleware<S, F>;
    type Future = fut::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        fut::ok(RestrictMiddleware {
            service: Rc::new(service),
            rule: self.rule.clone(),
        })
    }
}

/// Cookie based session middleware.
pub struct RestrictMiddleware<S, F> {
    service: Rc<S>,
    rule: fn(HttpRequest) -> F,
}

impl<S, F> Service<ServiceRequest> for RestrictMiddleware<S, F>
where
    S: 'static + Service<ServiceRequest, Response = ServiceResponse>,
    S::Future: 'static,
    // TODO: Remove actix_web::Error bound
    S::Error: 'static + From<InternalError<String>> + From<actix_web::Error>,
    F: 'static + Future<Output = Result<Restrict, actix_web::Error>>,
{
    type Response = ServiceResponse;
    type Error = S::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let rule = self.rule.clone();
        async move {
            let (http_req, payload) = req.into_parts();
            //let req = ServiceRequest::from_parts(http_req.clone(), payload);
            match (rule)(http_req.clone()).await {
                Ok(Restrict::Allow) => {
                    let req = ServiceRequest::from_parts(http_req, payload);
                    service.call(req).await
                }
                Ok(Restrict::Response(res)) => Ok(res),
                Ok(Restrict::Deny(msg)) => {
                    Err(InternalError::new(msg, actix_web::http::StatusCode::FORBIDDEN).into())
                }
                Err(err) => Err(err.into()),
            }
        }
        .boxed_local()
    }
}
