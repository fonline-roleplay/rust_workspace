use actix_web::{web, HttpResponse};
use fo_data::Converter;

pub async fn get(
    path: web::Path<String>,
    data: web::Data<super::AppState>,
) -> actix_web::Result<HttpResponse> {
    let image = web::block(move || {
        println!("data::get incoming... {:?}", path.as_ref());
        data.fo_data.get_png(&path)
    })
    .await?
    .map_err(|err| super::internal_error(err))?;

    use fo_data::DataType;
    let content_type = match image.data_type {
        DataType::Png => "image/png",
        //DataType::Gif => "image/gif",
        _ => unimplemented!(),
    };
    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .append_header(("Cache-Control", "max-age=10000000"))
        .append_header((
            "Access-Control-Expose-Headers",
            "x-image-offset-x, x-image-offset-y",
        ))
        .append_header(("x-image-offset-x", format!("{}", image.offset.0)))
        .append_header(("x-image-offset-y", format!("{}", image.offset.1)))
        .body(image.data))
}
