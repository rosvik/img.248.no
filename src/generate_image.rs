use crate::deserializers::{empty_string_as_none, string_as_bool};
use axum::{
    body::Bytes,
    extract::{Path, Query},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use libimg::ResizeMode;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ImgResizeParameters {
    url: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    w: Option<u32>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    h: Option<u32>,
    mode: Option<ResizeMode>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    quality: Option<u8>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    sampling: Option<String>,
    #[serde(default, deserialize_with = "string_as_bool")]
    base64: bool,
}
pub async fn generate_image(
    Path(filename): Path<String>,
    Query(query): Query<ImgResizeParameters>,
) -> impl IntoResponse {
    let image_output_format: libimg::Format;
    let content_type: &'static str;
    if filename.ends_with(".jpg") {
        content_type = "image/jpeg";
        let quality = query.quality.unwrap_or(100).clamp(0, 100);
        image_output_format = libimg::format::Jpeg(quality);
    } else if filename.ends_with(".png") {
        content_type = "image/png";
        image_output_format = libimg::format::Png;
    } else if filename.ends_with(".gif") {
        content_type = "image/gif";
        image_output_format = libimg::format::Gif;
    } else {
        return (
            StatusCode::BAD_REQUEST,
            http_headers("text/plain"),
            "Invalid file extension".into(),
        );
    }

    let image = match libimg::fetch_image(query.url.clone()).await {
        Ok(i) => i,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                http_headers("text/plain"),
                format!("Failed to fetch image: {}", e).into(),
            )
        }
    };

    let (width, height) = get_size(query.w, query.h, &image);
    let resize_mode = query.mode.unwrap_or(ResizeMode::Crop);
    let sampling_filter = get_samling_filter(&query.sampling);

    println!(
        "Resizing image '{}' to {}x{} (source {}x{})",
        query.url,
        width,
        height,
        image.width(),
        image.height()
    );

    let resized = libimg::resize_image(image, width, height, sampling_filter, resize_mode);
    let buffer = match libimg::to_buffer(resized, image_output_format) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                http_headers("text/plain"),
                format!("Failed to resize image: {}", e).into(),
            )
        }
    };

    if query.base64 {
        let b64_data = libimg::to_base64(&buffer, content_type);
        return (StatusCode::OK, http_headers("text/plain"), b64_data.into());
    }

    (
        StatusCode::OK,
        http_headers(content_type),
        Bytes::from(buffer.into_inner()),
    )
}

fn http_headers(value: &'static str) -> HeaderMap {
    let mut image_headers = HeaderMap::new();
    image_headers.insert(http::header::CONTENT_TYPE, HeaderValue::from_static(value));
    image_headers
}

fn get_size(w: Option<u32>, h: Option<u32>, image: &libimg::Image) -> (u32, u32) {
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

fn get_samling_filter(query: &Option<String>) -> Option<libimg::SamplingFilter> {
    match query {
        Some(f) => match f.to_lowercase().as_str() {
            "nearest" => Some(libimg::sampling_filter::Nearest),
            "linear" => Some(libimg::sampling_filter::Linear),
            "cubic" => Some(libimg::sampling_filter::Cubic),
            "gaussian" => Some(libimg::sampling_filter::Gaussian),
            "lanczos" => Some(libimg::sampling_filter::Lanczos),

            // Lanczos gives the best results, at least for downsampling.
            // https://stackoverflow.com/a/6171860
            // TODO: Are there better options for upsampling?
            "best" => Some(libimg::sampling_filter::Lanczos),

            _ => None,
        },
        None => None,
    }
}
