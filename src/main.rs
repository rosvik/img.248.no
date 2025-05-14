mod deserializers;
mod generate_image;

use axum::{response::Html, routing::get, Router};
use generate_image::generate_image;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/{filename}", get(generate_image));

    let addr = SocketAddr::from(([0, 0, 0, 0], 2338));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap()
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../templates/index.html"))
}
