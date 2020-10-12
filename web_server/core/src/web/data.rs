use actix_web::{error::BlockingError, web, HttpRequest, HttpResponse, Responder};
use fo_data::Converter;
use futures::{
    future::{err as fut_err, ok as fut_ok, Either},
    Future, FutureExt, TryFutureExt,
};

pub fn get(
    path: web::Path<String>,
    data: web::Data<super::AppState>,
) -> impl Future<Output = actix_web::Result<HttpResponse>> {
    web::block(move || {
        println!("data::get incoming... {:?}", path.as_ref());
        data.fo_data
            .get_png(&path)
            .map_err(|err| format!("FoData::get_image error: {:?}", err))
    })
    .err_into()
    .and_then(|image| {
        use fo_data::DataType;
        let content_type = match image.data_type {
            DataType::Png => "image/png",
            //DataType::Gif => "image/gif",
            _ => unimplemented!(),
        };
        HttpResponse::Ok()
            .content_type(content_type)
            .header("Cache-Control", "max-age=10000000")
            .header(
                "Access-Control-Expose-Headers",
                "x-image-offset-x, x-image-offset-y",
            )
            .header("x-image-offset-x", format!("{}", image.offset.0))
            .header("x-image-offset-y", format!("{}", image.offset.1))
            .body(image.data)
    })
}
