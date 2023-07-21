use axum::{
    body::Bytes,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use http::{header::CONTENT_TYPE, HeaderValue};
use serde::Deserialize;
use std::{io::Cursor, net::SocketAddr};

const BAD_REQUEST: StatusCode = StatusCode::BAD_REQUEST;
const OK: StatusCode = StatusCode::OK;
const INTERNAL_SERVER_ERROR: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;

const PLAINTEXT: HeaderValue = HeaderValue::from_static("text/plain");

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
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, PLAINTEXT);

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
        return (BAD_REQUEST, headers, "Invalid file extension".into());
    }

    let image_response = match reqwest::get(query.url.clone()).await {
        Ok(r) => r,
        Err(e) => {
            return (
                BAD_REQUEST,
                headers,
                format!("Failed to fetch image: {}", e).into(),
            )
        }
    };
    let image_bytes = match image_response.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return (
                BAD_REQUEST,
                headers,
                format!("Failed to parse image: {}", e).into(),
            )
        }
    };
    let image = match image::load_from_memory(&image_bytes) {
        Ok(i) => i,
        Err(e) => {
            return (
                INTERNAL_SERVER_ERROR,
                headers,
                format!("Failed to load image from memory: {}", e).into(),
            )
        }
    };

    let (width, height) = get_size(query.w, query.h, &image);

    println!(
        "Resizing image '{}' to {}x{} (source {}x{})",
        query.url,
        width,
        height,
        image.width(),
        image.height()
    );

    let resized = image.resize_exact(
        width,
        height,
        image::imageops::FilterType::Lanczos3, // https://stackoverflow.com/a/6171860
    );
    let mut buffer = Cursor::new(Vec::new());
    match resized.write_to(&mut buffer, image_output_format) {
        Ok(_) => (),
        Err(e) => {
            return (
                INTERNAL_SERVER_ERROR,
                headers,
                format!("Failed to write image to buffer: {}", e).into(),
            )
        }
    }

    let mut image_headers = HeaderMap::new();
    image_headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static(header_value),
    );
    (OK, image_headers, Bytes::from(buffer.into_inner()))
}

fn get_size(w: Option<u32>, h: Option<u32>, image: &image::DynamicImage) -> (u32, u32) {
    let width: u32;
    let height: u32;
    let aspect_ratio: f32 = image.height() as f32 / image.width() as f32;
    match (w, h) {
        (None, None) => {
            width = image.width();
            height = image.height();
        }
        (Some(w), Some(h)) => {
            if w == 0 && h == 0 {
                return get_size(None, None, image);
            } else if w == 0 {
                return get_size(None, Some(h), image);
            } else if h == 0 {
                return get_size(Some(w), None, image);
            }
            width = w;
            height = h;
        }
        (Some(w), None) => {
            if w > image.width() {
                return get_size(None, None, image);
            }
            width = w;
            height = (w as f32 * aspect_ratio) as u32;
        }
        (None, Some(h)) => {
            if h > image.height() {
                return get_size(None, None, image);
            }
            width = (h as f32 / aspect_ratio) as u32;
            height = h;
        }
    }
    (width, height)
}
