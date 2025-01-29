mod entropy_reset;
mod code_runner;

use axum::{
    routing::post,
    Router,
};
use std::net::SocketAddr;
use tokio::signal;

#[tokio::main]
async fn main() {
    // Build our application with routes
    let app = Router::new()
        .route("/run", post(code_runner::run_handler))
        .route("/reset_entropy", post(entropy_reset::handle_reset_entropy));

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 4000));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    println!("Received shutdown signal...");
}
