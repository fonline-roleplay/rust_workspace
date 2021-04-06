use actix_web::{error::BlockingError, web};

pub async fn blocking<F, R, E>(f: F) -> Result<R, E>
where
    F: FnOnce() -> Result<R, E> + Send + 'static,
    E: Send + 'static + From<BlockingError>,
    R: Send + 'static,
{
    web::block(f).await.unwrap_or_else(|err| Err(err.into()))
}
