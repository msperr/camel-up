//! Development server for the frontend cockpit.

use camel_up::api::app;

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("could not bind 127.0.0.1:3000");

    println!("Camel Up API listening on http://127.0.0.1:3000");

    axum::serve(listener, app())
        .await
        .expect("API server failed");
}
