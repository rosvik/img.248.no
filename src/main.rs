mod deserializers;
mod generate_image;

use axum::{response::Html, routing::get, Router, Server};
use generate_image::generate_image;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/:filename", get(generate_image));

    let addr = SocketAddr::from(([127, 0, 0, 1], 2338));
    println!("Listening on http://{}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../templates/index.html"))
}
