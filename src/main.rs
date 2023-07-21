use axum::{
    body::Bytes,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use http::HeaderValue;
use serde::Deserialize;
use std::{io::Cursor, net::SocketAddr};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/:filename", get(img_resize));

    let addr = SocketAddr::from(([127, 0, 0, 1], 2338));
    println!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> impl IntoResponse {
    "Usage example: /image-name.jpg?url=https://example.com/image.png&w=100\n"
}

#[derive(Deserialize)]
struct ImgResizeParameters {
    url: String,
    w: Option<u32>,
    h: Option<u32>,
}
async fn img_resize(
    Path(filename): Path<String>,
    Query(query): Query<ImgResizeParameters>,
) -> impl IntoResponse {
    println!("filename: {}", filename);

    let mut h = HeaderMap::new();
    h.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain"),
    );

    let image_output_format: image::ImageOutputFormat;
    let header_value: &'static str;

    if filename.ends_with(".jpg") {
        header_value = "image/jpeg";
        image_output_format = image::ImageOutputFormat::Jpeg(100);
    } else if filename.ends_with(".png") {
        header_value = "image/png";
        image_output_format = image::ImageOutputFormat::Png;
    } else if filename.ends_with(".gif") {
        header_value = "image/gif";
        image_output_format = image::ImageOutputFormat::Gif;
    } else {
        return (StatusCode::BAD_REQUEST, h, "Invalid file extension".into());
    }

    let img_bytes = reqwest::get(query.url)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let image = image::load_from_memory(&img_bytes).unwrap();
    let aspect_ratio: f32 = image.height() as f32 / image.width() as f32;

    let width: u32;
    let height: u32;
    match (query.w, query.h) {
        (Some(w), Some(h)) => {
            width = w;
            height = h;
        }
        (Some(w), None) => {
            width = w;
            height = (w as f32 * aspect_ratio) as u32;
        }
        (None, Some(h)) => {
            width = (h as f32 / aspect_ratio) as u32;
            height = h;
        }
        (None, None) => {
            width = image.width();
            height = image.height();
        }
    }

    println!(
        "Resizing image to {}x{} (aspect ratio: {})",
        width, height, aspect_ratio
    );

    let resized = image.resize_exact(
        width,
        height,
        // https://stackoverflow.com/a/6171860
        image::imageops::FilterType::Lanczos3,
    );
    let mut buffer = Cursor::new(Vec::new());
    resized
        .write_to(&mut buffer, image_output_format)
        .expect("Failed to write image to buffer");

    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static(header_value),
    );

    (StatusCode::OK, headers, Bytes::from(buffer.into_inner()))
}
