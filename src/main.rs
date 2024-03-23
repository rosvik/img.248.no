use axum::{
    body::Bytes,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use base64::Engine;
use http::HeaderValue;
use serde::{de, Deserialize, Deserializer};
use std::{fmt, io::Cursor, net::SocketAddr, str::FromStr};

const BAD_REQUEST: StatusCode = StatusCode::BAD_REQUEST;
const OK: StatusCode = StatusCode::OK;
const INTERNAL_SERVER_ERROR: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;

const BASE_64: base64::engine::GeneralPurpose = base64::engine::GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    base64::engine::general_purpose::NO_PAD,
);

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/:filename", get(img_resize));

    let addr = SocketAddr::from(([127, 0, 0, 1], 2338));
    println!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../templates/index.html"))
}

#[derive(Deserialize)]
struct ImgResizeParameters {
    url: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    w: Option<u32>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    h: Option<u32>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    quality: Option<u8>,
    #[serde(default, deserialize_with = "string_as_bool")]
    base64: bool,
}
async fn img_resize(
    Path(filename): Path<String>,
    Query(query): Query<ImgResizeParameters>,
) -> impl IntoResponse {
    let image_output_format: image::ImageOutputFormat;
    let header_value: &'static str;
    if filename.ends_with(".jpg") {
        header_value = "image/jpeg";
        let quality = query.quality.unwrap_or(100).clamp(0, 100);
        image_output_format = image::ImageOutputFormat::Jpeg(quality);
    } else if filename.ends_with(".png") {
        header_value = "image/png";
        image_output_format = image::ImageOutputFormat::Png;
    } else if filename.ends_with(".gif") {
        header_value = "image/gif";
        image_output_format = image::ImageOutputFormat::Gif;
    } else {
        return (
            BAD_REQUEST,
            http_headers("text/plain"),
            "Invalid file extension".into(),
        );
    }

    let image = match libimg::fetch_image(query.url.clone()).await {
        Ok(i) => i,
        Err(e) => {
            return (
                BAD_REQUEST,
                http_headers("text/plain"),
                format!("Failed to fetch image: {}", e).into(),
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
                http_headers("text/plain"),
                format!("Failed to write image to buffer: {}", e).into(),
            )
        }
    }

    if query.base64 {
        let base64_data = BASE_64.encode(buffer.get_ref());
        let prefix = format!("data:{header_value};base64,");
        return (
            OK,
            http_headers("text/plain"),
            format!("{}{}", prefix, base64_data).into(),
        );
    }

    (
        OK,
        http_headers(header_value),
        Bytes::from(buffer.into_inner()),
    )
}

fn http_headers(value: &'static str) -> HeaderMap {
    let mut image_headers = HeaderMap::new();
    image_headers.insert(http::header::CONTENT_TYPE, HeaderValue::from_static(value));
    image_headers
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

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}
fn string_as_bool<'de, D>(de: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = String::deserialize(de).unwrap_or("off".to_string());
    match opt.as_str() {
        "on" | "true" | "" => Ok(true),
        "off" | "false" => Ok(false),
        _ => Ok(false),
    }
}
