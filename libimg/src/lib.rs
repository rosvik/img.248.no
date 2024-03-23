use std::error::Error;
use std::io::ErrorKind;

pub async fn fetch_image(url: String) -> Result<image::DynamicImage, Box<dyn Error>> {
    let image_response = match reqwest::get(url).await {
        Ok(r) => r,
        Err(e) => return Err(Box::new(e)),
    };
    let status_code = image_response.status();
    if !status_code.is_success() {
        return Err(Box::new(std::io::Error::new(
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
