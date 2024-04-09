use serde::{Deserialize, Serialize};
use std::io::{Cursor, Error, ErrorKind};

pub use image::imageops::FilterType as SamplingFilter;
pub use image::DynamicImage as Image;
pub use image::ImageOutputFormat as Format;

pub mod format {
    pub use image::ImageOutputFormat::Gif;
    pub use image::ImageOutputFormat::Jpeg;
    pub use image::ImageOutputFormat::Png;
}
pub mod sampling_filter {
    pub use image::imageops::CatmullRom as Cubic;
    pub use image::imageops::Gaussian;
    pub use image::imageops::Lanczos3 as Lanczos;
    pub use image::imageops::Nearest;
    pub use image::imageops::Triangle as Linear;
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResizeMode {
    Fit,
    Crop,
    Stretch,
    Cover,
}

pub async fn fetch_image(url: &String) -> Result<image::DynamicImage, Box<dyn std::error::Error>> {
    let image_response = match reqwest::get(url).await {
        Ok(r) => r,
        Err(e) => return Err(Box::new(e)),
    };
    let status_code = image_response.status();
    if !status_code.is_success() {
        return Err(Box::new(Error::new(
            ErrorKind::Other,
            format!("Status code from source: {}", status_code),
        )));
    }
    let image_bytes = match image_response.bytes().await {
        Ok(b) => b,
        Err(e) => return Err(Box::new(e)),
    };
    let image = match image::load_from_memory(&image_bytes) {
        Ok(i) => i,
        Err(e) => return Err(Box::new(e)),
    };

    Ok(image)
}

pub fn resize_image(
    image: image::DynamicImage,
    width: u32,
    height: u32,
    filter: Option<image::imageops::FilterType>,
    mode: ResizeMode,
) -> image::DynamicImage {
    // Lanczos gives the best results (https://stackoverflow.com/a/6171860), at
    // least for downsampling, while being quite slow. Triangle gives ok enough
    // results, but it is a lot faster.
    // https://docs.rs/image/latest/image/imageops/enum.FilterType.html
    let filter_type = filter.unwrap_or(image::imageops::FilterType::Triangle);

    match mode {
        ResizeMode::Fit => image.resize(width, height, filter_type),
        ResizeMode::Crop => image.resize_to_fill(width, height, filter_type),
        ResizeMode::Stretch => image.resize_exact(width, height, filter_type),
        ResizeMode::Cover => {
            let (image_width, image_height) = (image.width(), image.height());
            let (new_width, new_height) =
                if width as f32 / height as f32 > image_width as f32 / image_height as f32 {
                    (width, (width * image_height) / image_width)
                } else {
                    ((height * image_width) / image_height, height)
                };
            image.resize(new_width, new_height, filter_type)
        }
    }
}

pub fn to_buffer(
    image: image::DynamicImage,
    format: image::ImageOutputFormat,
) -> Result<Cursor<Vec<u8>>, Box<dyn std::error::Error>> {
    let mut buffer = Cursor::new(Vec::new());
    match image.write_to(&mut buffer, format) {
        Ok(_) => (),
        Err(e) => return Err(Box::new(e)),
    }
    Ok(buffer)
}

use base64::Engine as _;
const B64_ENGINE: base64::engine::GeneralPurpose = base64::engine::GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    base64::engine::general_purpose::NO_PAD,
);
pub fn to_base64(buffer: &Cursor<Vec<u8>>, content_type: &str) -> String {
    let base64_data = B64_ENGINE.encode(buffer.get_ref());
    let prefix = format!("data:{};base64,", content_type);
    format!("{}{}", prefix, base64_data)
}
